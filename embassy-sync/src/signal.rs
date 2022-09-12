//! A synchronization primitive for passing the latest value to a task.
use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::task::{Context, Poll, Waker};

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
///
/// enum SomeCommand {
///   On,
///   Off,
/// }
///
/// static SOME_SIGNAL: Signal<SomeCommand> = Signal::new();
/// ```
pub struct Signal<T> {
    state: UnsafeCell<State<T>>,
}

enum State<T> {
    None,
    Waiting(Waker),
    Signaled(T),
}

unsafe impl<T: Send> Send for Signal<T> {}
unsafe impl<T: Send> Sync for Signal<T> {}

impl<T> Signal<T> {
    /// Create a new `Signal`.
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(State::None),
        }
    }
}

impl<T: Send> Signal<T> {
    /// Mark this Signal as signaled.
    pub fn signal(&self, val: T) {
        critical_section::with(|_| unsafe {
            let state = &mut *self.state.get();
            if let State::Waiting(waker) = mem::replace(state, State::Signaled(val)) {
                waker.wake();
            }
        })
    }

    /// Remove the queued value in this `Signal`, if any.
    pub fn reset(&self) {
        critical_section::with(|_| unsafe {
            let state = &mut *self.state.get();
            *state = State::None
        })
    }

    /// Manually poll the Signal future.
    pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<T> {
        critical_section::with(|_| unsafe {
            let state = &mut *self.state.get();
            match state {
                State::None => {
                    *state = State::Waiting(cx.waker().clone());
                    Poll::Pending
                }
                State::Waiting(w) if w.will_wake(cx.waker()) => Poll::Pending,
                State::Waiting(w) => {
                    let w = mem::replace(w, cx.waker().clone());
                    w.wake();
                    Poll::Pending
                }
                State::Signaled(_) => match mem::replace(state, State::None) {
                    State::Signaled(res) => Poll::Ready(res),
                    _ => unreachable!(),
                },
            }
        })
    }

    /// Future that completes when this Signal has been signaled.
    pub fn wait(&self) -> impl Future<Output = T> + '_ {
        futures_util::future::poll_fn(move |cx| self.poll_wait(cx))
    }

    /// non-blocking method to check whether this signal has been signaled.
    pub fn signaled(&self) -> bool {
        critical_section::with(|_| matches!(unsafe { &*self.state.get() }, State::Signaled(_)))
    }
}
