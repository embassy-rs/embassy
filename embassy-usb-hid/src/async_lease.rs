use core::cell::Cell;
use core::future::Future;
use core::task::{Poll, Waker};

enum AsyncLeaseState {
    Empty,
    Waiting(*mut u8, usize, Waker),
    Done(usize),
}

impl Default for AsyncLeaseState {
    fn default() -> Self {
        AsyncLeaseState::Empty
    }
}

#[derive(Default)]
pub struct AsyncLease {
    state: Cell<AsyncLeaseState>,
}

pub struct AsyncLeaseFuture<'a> {
    buf: &'a mut [u8],
    state: &'a Cell<AsyncLeaseState>,
}

impl<'a> Drop for AsyncLeaseFuture<'a> {
    fn drop(&mut self) {
        self.state.set(AsyncLeaseState::Empty);
    }
}

impl<'a> Future for AsyncLeaseFuture<'a> {
    type Output = usize;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.state.take() {
            AsyncLeaseState::Done(len) => Poll::Ready(len),
            state => {
                if let AsyncLeaseState::Waiting(ptr, _, _) = state {
                    assert_eq!(
                        ptr,
                        self.buf.as_mut_ptr(),
                        "lend() called on a busy AsyncLease."
                    );
                }

                self.state.set(AsyncLeaseState::Waiting(
                    self.buf.as_mut_ptr(),
                    self.buf.len(),
                    cx.waker().clone(),
                ));
                Poll::Pending
            }
        }
    }
}

pub struct AsyncLeaseNotReady {}

impl AsyncLease {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn try_borrow_mut<F: FnOnce(&mut [u8]) -> usize>(
        &self,
        f: F,
    ) -> Result<(), AsyncLeaseNotReady> {
        if let AsyncLeaseState::Waiting(data, len, waker) = self.state.take() {
            let buf = unsafe { core::slice::from_raw_parts_mut(data, len) };
            let len = f(buf);
            self.state.set(AsyncLeaseState::Done(len));
            waker.wake();
            Ok(())
        } else {
            Err(AsyncLeaseNotReady {})
        }
    }

    pub fn lend<'a>(&'a self, buf: &'a mut [u8]) -> AsyncLeaseFuture<'a> {
        AsyncLeaseFuture {
            buf,
            state: &self.state,
        }
    }
}
