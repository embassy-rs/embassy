use core::cell::RefCell;
use core::future::{Future, poll_fn};
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
    state: ControlState,
    last_link_event: Option<LinkState>,
    reset_request: ResetRequest,
    control_waker: WakerRegistration,
    runner_waker: WakerRegistration,
    runner_reset_waker: WakerRegistration,
}

#[derive(Clone, Copy)]
pub(crate) enum ControlState {
    Init,
    Reboot,
    Ready,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LinkState {
    Down,
    Up,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ResetRequest {
    None,
    Requested,
    Performed,
}

impl Shared {
    pub fn new() -> Self {
        Self(RefCell::new(SharedInner {
            ioctl: IoctlState::Done { resp_len: 0 },
            state: ControlState::Init,
            last_link_event: None,
            reset_request: ResetRequest::None,
            control_waker: WakerRegistration::new(),
            runner_waker: WakerRegistration::new(),
            runner_reset_waker: WakerRegistration::new(),
        }))
    }

    pub async fn issue_reset_request(&self) {
        if self.0.borrow_mut().reset_request == ResetRequest::None {
            self.0.borrow_mut().reset_request = ResetRequest::Requested;
            self.0.borrow_mut().runner_reset_waker.wake();

            poll_fn(|cx| {
                let mut this = self.0.borrow_mut();
                if this.reset_request == ResetRequest::Performed {
                    this.reset_request = ResetRequest::None;
                    Poll::Ready(())
                } else {
                    this.control_waker.register(cx.waker());
                    Poll::Pending
                }
            })
            .await;
        }
    }

    pub fn reset_request_done(&self) {
        let mut this = self.0.borrow_mut();
        this.ioctl = IoctlState::Done { resp_len: 0 };
        this.state = ControlState::Init;
        this.reset_request = ResetRequest::Performed;
        this.control_waker.wake();
    }

    pub fn wait_for_reset_request(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| {
            let mut this = self.0.borrow_mut();
            if this.reset_request == ResetRequest::Requested {
                Poll::Ready(())
            } else {
                this.runner_reset_waker.register(cx.waker());
                Poll::Pending
            }
        })
    }

    pub fn ioctl_wait_complete(&self) -> impl Future<Output = usize> + '_ {
        poll_fn(|cx| {
            let mut this = self.0.borrow_mut();
            if let IoctlState::Done { resp_len } = this.ioctl {
                Poll::Ready(resp_len)
            } else {
                this.control_waker.register(cx.waker());
                Poll::Pending
            }
        })
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

    // ota
    #[allow(unused)]
    pub fn ota_done(&self) {
        let mut this = self.0.borrow_mut();
        this.state = ControlState::Reboot;
    }

    // // // // // // // // // // // // // // // // // // // //
    //
    // check if ota is in progress
    pub(crate) fn state(&self) -> ControlState {
        let this = self.0.borrow();
        this.state
    }

    pub fn init_done(&self) {
        let mut this = self.0.borrow_mut();
        this.state = ControlState::Ready;
        this.control_waker.wake();
    }

    pub fn init_wait(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| {
            let mut this = self.0.borrow_mut();
            if let ControlState::Ready = this.state {
                Poll::Ready(())
            } else {
                this.control_waker.register(cx.waker());
                Poll::Pending
            }
        })
    }

    pub(crate) fn link_up_done(&self) {
        let mut this = self.0.borrow_mut();
        this.last_link_event = Some(LinkState::Up);
        this.control_waker.wake();
    }

    pub(crate) fn link_down_done(&self) {
        let mut this = self.0.borrow_mut();
        this.last_link_event = Some(LinkState::Down);
        this.control_waker.wake();
    }

    pub(crate) fn link_event_wait(&self) -> impl Future<Output = LinkState> + '_ {
        poll_fn(|cx| {
            let mut this = self.0.borrow_mut();
            if let Some(s) = this.last_link_event {
                this.last_link_event = None;
                Poll::Ready(s)
            } else {
                this.control_waker.register(cx.waker());
                Poll::Pending
            }
        })
    }
}
