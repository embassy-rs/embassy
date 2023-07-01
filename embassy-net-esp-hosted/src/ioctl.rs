use core::cell::RefCell;
use core::future::poll_fn;
use core::task::Poll;

use embassy_sync::waitqueue::WakerRegistration;

use crate::fmt::Bytes;

#[derive(Clone, Copy)]
pub struct PendingIoctl {
    pub buf: *mut [u8],
    pub req_len: usize,
}

#[derive(Clone, Copy)]
enum IoctlState {
    Pending(PendingIoctl),
    Sent { buf: *mut [u8] },
    Done { resp_len: usize },
}

pub struct Shared(RefCell<SharedInner>);

struct SharedInner {
    ioctl: IoctlState,
    is_init: bool,
    control_waker: WakerRegistration,
    runner_waker: WakerRegistration,
}

impl Shared {
    pub fn new() -> Self {
        Self(RefCell::new(SharedInner {
            ioctl: IoctlState::Done { resp_len: 0 },
            is_init: false,
            control_waker: WakerRegistration::new(),
            runner_waker: WakerRegistration::new(),
        }))
    }

    pub async fn ioctl_wait_complete(&self) -> usize {
        poll_fn(|cx| {
            let mut this = self.0.borrow_mut();
            if let IoctlState::Done { resp_len } = this.ioctl {
                Poll::Ready(resp_len)
            } else {
                this.control_waker.register(cx.waker());
                Poll::Pending
            }
        })
        .await
    }

    pub async fn ioctl_wait_pending(&self) -> PendingIoctl {
        let pending = poll_fn(|cx| {
            let mut this = self.0.borrow_mut();
            if let IoctlState::Pending(pending) = this.ioctl {
                Poll::Ready(pending)
            } else {
                this.runner_waker.register(cx.waker());
                Poll::Pending
            }
        })
        .await;

        self.0.borrow_mut().ioctl = IoctlState::Sent { buf: pending.buf };
        pending
    }

    pub fn ioctl_cancel(&self) {
        self.0.borrow_mut().ioctl = IoctlState::Done { resp_len: 0 };
    }

    pub async fn ioctl(&self, buf: &mut [u8], req_len: usize) -> usize {
        trace!("ioctl req bytes: {:02x}", Bytes(&buf[..req_len]));

        {
            let mut this = self.0.borrow_mut();
            this.ioctl = IoctlState::Pending(PendingIoctl { buf, req_len });
            this.runner_waker.wake();
        }

        self.ioctl_wait_complete().await
    }

    pub fn ioctl_done(&self, response: &[u8]) {
        let mut this = self.0.borrow_mut();
        if let IoctlState::Sent { buf } = this.ioctl {
            trace!("ioctl resp bytes: {:02x}", Bytes(response));

            // TODO fix this
            (unsafe { &mut *buf }[..response.len()]).copy_from_slice(response);

            this.ioctl = IoctlState::Done {
                resp_len: response.len(),
            };
            this.control_waker.wake();
        } else {
            warn!("IOCTL Response but no pending Ioctl");
        }
    }

    // // // // // // // // // // // // // // // // // // // //

    pub fn init_done(&self) {
        let mut this = self.0.borrow_mut();
        this.is_init = true;
        this.control_waker.wake();
    }

    pub async fn init_wait(&self) {
        poll_fn(|cx| {
            let mut this = self.0.borrow_mut();
            if this.is_init {
                Poll::Ready(())
            } else {
                this.control_waker.register(cx.waker());
                Poll::Pending
            }
        })
        .await
    }
}
