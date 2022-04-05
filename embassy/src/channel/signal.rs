use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::task::{Context, Poll, Waker};

/// Synchronization primitive. Allows creating awaitable signals that may be passed between tasks.
/// For a simple use-case where the receiver is only ever interested in the latest value of
/// something, Signals work well. For more advanced use cases, you might want to use [`Channel`](crate::channel::channel::Channel) instead..
///
/// Signals are generally declared as being a static const and then borrowed as required.
///
/// ```
/// use embassy::channel::signal::Signal;
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
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(State::None),
        }
    }
}

impl<T: Send> Signal<T> {
    /// Mark this Signal as completed.
    pub fn signal(&self, val: T) {
        critical_section::with(|_| unsafe {
            let state = &mut *self.state.get();
            if let State::Waiting(waker) = mem::replace(state, State::Signaled(val)) {
                waker.wake();
            }
        })
    }

    pub fn reset(&self) {
        critical_section::with(|_| unsafe {
            let state = &mut *self.state.get();
            *state = State::None
        })
    }

    pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<T> {
        critical_section::with(|_| unsafe {
            let state = &mut *self.state.get();
            match state {
                State::None => {
                    *state = State::Waiting(cx.waker().clone());
                    Poll::Pending
                }
                State::Waiting(w) if w.will_wake(cx.waker()) => Poll::Pending,
                State::Waiting(_) => panic!("waker overflow"),
                State::Signaled(_) => match mem::replace(state, State::None) {
                    State::Signaled(res) => Poll::Ready(res),
                    _ => unreachable!(),
                },
            }
        })
    }

    /// Future that completes when this Signal has been signaled.
    pub fn wait(&self) -> impl Future<Output = T> + '_ {
        futures::future::poll_fn(move |cx| self.poll_wait(cx))
    }

    /// non-blocking method to check whether this signal has been signaled.
    pub fn signaled(&self) -> bool {
        critical_section::with(|_| matches!(unsafe { &*self.state.get() }, State::Signaled(_)))
    }
}
