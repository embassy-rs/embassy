//! A synchronization primitive for passing the latest value to a task.
use core::future::{poll_fn, Future};
use core::task::{Context, Poll, Waker};

use scoped_mutex::{BlockingMutex, RawMutex};

/// Single-slot signaling primitive.
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
    state: BlockingMutex<M, State<T>>,
}

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
            state: BlockingMutex::new(State::None),
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
        self.state.lock(|state| {
            let old = core::mem::replace(state, State::Signaled(val));
            if let State::Waiting(waker) = old {
                waker.wake();
            }
        })
    }

    /// Remove the queued value in this `Signal`, if any.
    pub fn reset(&self) {
        self.state.lock(|state| *state = State::None);
    }

    fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<T> {
        self.state.lock(|state| {
            let old = core::mem::replace(state, State::None);
            match old {
                State::None => {
                    *state = State::Waiting(cx.waker().clone());
                    Poll::Pending
                }
                State::Waiting(w) if w.will_wake(cx.waker()) => {
                    *state = State::Waiting(w);
                    Poll::Pending
                }
                State::Waiting(w) => {
                    *state = State::Waiting(cx.waker().clone());
                    w.wake();
                    Poll::Pending
                }
                State::Signaled(res) => Poll::Ready(res),
            }
        })
    }

    /// Future that completes when this Signal has been signaled.
    pub fn wait(&self) -> impl Future<Output = T> + '_ {
        poll_fn(move |cx| self.poll_wait(cx))
    }

    /// non-blocking method to try and take the signal value.
    pub fn try_take(&self) -> Option<T> {
        self.state.lock(|state| {
            let old = core::mem::replace(state, State::None);
            match old {
                State::Signaled(res) => Some(res),
                ostate => {
                    *state = ostate;
                    None
                }
            }
        })
    }

    /// non-blocking method to check whether this signal has been signaled. This does not clear the signal.  
    pub fn signaled(&self) -> bool {
        self.state.lock(|state| {
            let res = matches!(state, State::Signaled(_));
            res
        })
    }
}
