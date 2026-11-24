//! A synchronization primitive for passing the latest value to a task.
use core::convert::Infallible;
use core::future::{Future, poll_fn};
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use crate::blocking_mutex::Mutex;
use crate::blocking_mutex::raw::RawMutex;

/// Single-slot signaling primitive for a _single_ consumer.
///
/// This is similar to a [`Channel`](crate::channel::Channel) with a buffer size of 1, except
/// "sending" to it (calling [`Signal::signal`]) when full will overwrite the previous value instead
/// of waiting for the receiver to pop the previous value.
///
/// It is useful for sending data between tasks when the receiver only cares about
/// the latest data, and therefore it's fine to "lose" messages. This is often the case for "state"
/// updates.
///
/// For more advanced use cases, you might want to use [`Channel`](crate::channel::Channel) instead.
/// For multiple consumers, use [`Watch`](crate::watch::Watch) instead.
///
/// Signals are generally declared as `static`s and then borrowed as required.
///
/// ```
/// use embassy_sync::signal::Signal;
/// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
///
/// enum SomeCommand {
///   On,
///   Off,
/// }
///
/// static SOME_SIGNAL: Signal<CriticalSectionRawMutex, SomeCommand> = Signal::new();
/// ```
pub struct Signal<M, T>
where
    M: RawMutex,
{
    state: Mutex<M, State<T>>,
}

#[derive(Debug)]
enum State<T> {
    None,
    Waiting(Waker),
    Signaled(T),
}

impl<M, T> Signal<M, T>
where
    M: RawMutex,
{
    /// Create a new `Signal`.
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(State::None),
        }
    }
}

impl<M, T> Default for Signal<M, T>
where
    M: RawMutex,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<M, T> Signal<M, T>
where
    M: RawMutex,
{
    /// Mark this Signal as signaled.
    pub fn signal(&self, val: T) {
        // Wake the waiter and drop the previous value after the lock is released.
        let state = self.state.lock(|state| mem::replace(state, State::Signaled(val)));
        if let State::Waiting(waker) = state {
            waker.wake();
        }
    }

    /// Remove the queued value in this `Signal`, if any.
    pub fn reset(&self) {
        self.try_take();
    }

    /// Poll for state changes of this Signal.
    pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<T> {
        let (poll, evicted) = self.state.lock(|state| match mem::replace(state, State::None) {
            State::None => {
                *state = State::Waiting(cx.waker().clone());
                (Poll::Pending, None)
            }
            State::Waiting(w) if w.will_wake(cx.waker()) => {
                *state = State::Waiting(w);
                (Poll::Pending, None)
            }
            State::Waiting(w) => {
                *state = State::Waiting(cx.waker().clone());
                (Poll::Pending, Some(w))
            }
            State::Signaled(res) => (Poll::Ready(res), None),
        });
        // Wake the previous waiter after the lock is released.
        if let Some(w) = evicted {
            w.wake();
        }
        poll
    }

    /// Future that completes when this Signal has been signaled, taking the value out of the signal.
    ///
    /// The returned Future is cancel-safe. No value will be lost even if it isn't polled to completion.
    pub fn wait(&self) -> impl Future<Output = T> + '_ {
        poll_fn(move |cx| self.poll_wait(cx))
    }

    /// non-blocking method to try and take the signal value.
    pub fn try_take(&self) -> Option<T> {
        self.state.lock(|state| match mem::replace(state, State::None) {
            State::Signaled(res) => Some(res),
            other => {
                *state = other;
                None
            }
        })
    }

    /// non-blocking method to check whether this signal has been signaled. This does not clear the signal.
    pub fn signaled(&self) -> bool {
        self.state.lock(|state| matches!(state, State::Signaled(_)))
    }
}

macro_rules! impl_stream_sink {
    ($($ref:tt)?) => {
        impl<M, T> futures_core::Stream for $($ref)? Signal<M, T>
        where
            M: RawMutex,
        {
            type Item = T;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                self.poll_wait(cx).map(Some)
            }
        }

        impl<M, T> futures_sink::Sink<T> for $($ref)? Signal<M, T>
        where
            M: RawMutex,
        {
            type Error = Infallible;

            fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
                self.signal(item);
                Ok(())
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
        }
    };
}

impl_stream_sink!();
impl_stream_sink!(&);
