use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::ptr;
use core::task::{Context, Poll, Waker};
use cortex_m::peripheral::NVIC;
use cortex_m::peripheral::{scb, SCB};

use crate::executor;
use crate::fmt::panic;
use crate::interrupt::{Interrupt, InterruptExt};

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
        cortex_m::interrupt::free(|_| unsafe {
            let state = &mut *self.state.get();
            if let State::Waiting(waker) = mem::replace(state, State::Signaled(val)) {
                waker.wake();
            }
        })
    }

    pub fn reset(&self) {
        cortex_m::interrupt::free(|_| unsafe {
            let state = &mut *self.state.get();
            *state = State::None
        })
    }

    pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<T> {
        cortex_m::interrupt::free(|_| unsafe {
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

    pub fn wait(&self) -> impl Future<Output = T> + '_ {
        futures::future::poll_fn(move |cx| self.poll_wait(cx))
    }

    pub fn signaled(&self) -> bool {
        cortex_m::interrupt::free(|_| matches!(unsafe { &*self.state.get() }, State::Signaled(_)))
    }
}

struct NrWrap(u8);
unsafe impl cortex_m::interrupt::Nr for NrWrap {
    fn nr(&self) -> u8 {
        self.0
    }
}

pub struct InterruptFuture<'a, I: Interrupt> {
    interrupt: &'a mut I,
}

impl<'a, I: Interrupt> Drop for InterruptFuture<'a, I> {
    fn drop(&mut self) {
        self.interrupt.disable();
        self.interrupt.remove_handler();
    }
}

impl<'a, I: Interrupt> InterruptFuture<'a, I> {
    pub fn new(interrupt: &'a mut I) -> Self {
        interrupt.disable();
        interrupt.set_handler(Self::interrupt_handler);
        interrupt.set_handler_context(ptr::null_mut());
        interrupt.unpend();
        interrupt.enable();

        Self { interrupt }
    }

    unsafe fn interrupt_handler(ctx: *mut ()) {
        let irq = match SCB::vect_active() {
            scb::VectActive::Interrupt { irqn } => irqn,
            _ => unreachable!(),
        };

        if !ctx.is_null() {
            executor::raw::wake_task(ptr::NonNull::new_unchecked(ctx as _));
        }

        NVIC::mask(crate::interrupt::NrWrap(irq as u16));
    }
}

impl<'a, I: Interrupt> Unpin for InterruptFuture<'a, I> {}

impl<'a, I: Interrupt> Future for InterruptFuture<'a, I> {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let s = unsafe { self.get_unchecked_mut() };
        s.interrupt.set_handler(Self::interrupt_handler);
        s.interrupt.set_handler_context(unsafe {
            executor::raw::task_from_waker(&cx.waker()).cast().as_ptr()
        });
        if s.interrupt.is_enabled() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
