use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

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

impl<T: Send> Signal<T> {
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(State::None),
        }
    }

    pub fn signal(&self, val: T) {
        unsafe {
            cortex_m::interrupt::free(|_| {
                let state = &mut *self.state.get();
                match mem::replace(state, State::Signaled(val)) {
                    State::Waiting(waker) => waker.wake(),
                    _ => {}
                }
            })
        }
    }

    pub fn wait<'a>(&'a self) -> impl Future<Output = T> + 'a {
        WaitFuture { signal: self }
    }
}

struct WaitFuture<'a, T> {
    signal: &'a Signal<T>,
}

impl<'a, T: Send> Future for WaitFuture<'a, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        unsafe {
            cortex_m::interrupt::free(|_| {
                let state = &mut *self.signal.state.get();
                match state {
                    State::None => {
                        *state = State::Waiting(cx.waker().clone());
                        Poll::Pending
                    }
                    State::Waiting(w) if w.will_wake(cx.waker()) => Poll::Pending,
                    State::Waiting(_) => depanic!("waker overflow"),
                    State::Signaled(_) => match mem::replace(state, State::None) {
                        State::Signaled(res) => Poll::Ready(res),
                        _ => unreachable!(),
                    },
                }
            })
        }
    }
}
