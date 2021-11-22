use crate::blocking_mutex::kind::MutexKind;
use crate::blocking_mutex::Mutex;
use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::task::{Context, Poll, Waker};

/// Synchronization primitive. Allows creating awaitable signals that may be passed between tasks.
/// For a simple use-case where the receiver is only ever interested in the latest value of
/// something, Signals work well. For more advanced use cases, please consider [crate::channel::mpsc].
///
/// Signals are generally declared as being a static const and then borrowed as required.
///
/// ```
/// use embassy::blocking_mutex::kind::Noop;
/// use embassy::channel::signal::Signal;
/// use embassy::util::Forever;
///
/// enum SomeCommand {
///   On,
///   Off,
/// }
///
/// static SOME_SIGNAL: Forever<Signal<Noop, SomeCommand>> = Forever::new();
///
/// // Then, during initialization...
/// let some_signal = SOME_SIGNAL.put(Signal::new());
/// ```
pub struct Signal<M, T>
where
    M: MutexKind,
{
    state: M::Mutex<UnsafeCell<State<T>>>,
}

enum State<T> {
    None,
    Waiting(Waker),
    Signaled(T),
}

unsafe impl<M, T> Send for Signal<M, T>
where
    M: MutexKind,
    T: Send,
{
}
unsafe impl<M, T> Sync for Signal<M, T>
where
    M: MutexKind,
    T: Send,
{
}

impl<M, T> Signal<M, T>
where
    M: MutexKind,
    T: Send,
{
    pub fn new() -> Self {
        Self {
            state: M::Mutex::new(UnsafeCell::new(State::None)),
        }
    }

    /// Mark this Signal as completed.
    pub fn signal(&self, val: T) {
        self.state.lock(|cell| unsafe {
            let state = &mut *cell.get();
            if let State::Waiting(waker) = mem::replace(state, State::Signaled(val)) {
                waker.wake();
            }
        })
    }

    pub fn reset(&self) {
        self.state.lock(|cell| unsafe {
            let state = &mut *cell.get();
            *state = State::None
        })
    }

    pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<T> {
        self.state.lock(|cell| unsafe {
            let state = &mut *cell.get();
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
        self.state.lock(|cell| unsafe {
            let state = &*cell.get();
            matches!(state, State::Signaled(_))
        })
    }
}
