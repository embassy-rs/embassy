use core::cell::{Cell, Ref, RefCell};
use core::future::{Future, poll_fn};
use core::task::{Poll, Waker};

use embassy_sync::waitqueue::WakerRegistration;

use crate::consts::Ioctl;
use crate::fmt::Bytes;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IoctlType {
    Get = 0,
    Set = 2,
}

/// Largest ioctl request: a `clmload` chunk plus its 20 bytes of headers.
pub(crate) const MAX_IOCTL_LEN: usize = 8 + 12 + 1024;

#[derive(Clone, Copy)]
pub struct PendingIoctl {
    /// Length of the request in [`IoctlState::request`].
    pub len: usize,
    pub kind: IoctlType,
    pub cmd: Ioctl,
    pub iface: u32,
}

#[derive(Clone, Copy)]
enum IoctlStateInner {
    Pending(PendingIoctl),
    Sent,
    Done { resp_len: usize },
}

struct Wakers {
    control: WakerRegistration,
    runner: WakerRegistration,
}

impl Wakers {
    const fn new() -> Self {
        Self {
            control: WakerRegistration::new(),
            runner: WakerRegistration::new(),
        }
    }
}

pub struct IoctlState {
    state: Cell<IoctlStateInner>,
    /// Request data while an ioctl is pending or sent, response data once done.
    buf: RefCell<[u8; MAX_IOCTL_LEN]>,
    wakers: RefCell<Wakers>,
}

impl IoctlState {
    pub const fn new() -> Self {
        Self {
            state: Cell::new(IoctlStateInner::Done { resp_len: 0 }),
            buf: RefCell::new([0; MAX_IOCTL_LEN]),
            wakers: RefCell::new(Wakers::new()),
        }
    }

    fn wake_control(&self) {
        self.wakers.borrow_mut().control.wake();
    }

    fn register_control(&self, waker: &Waker) {
        self.wakers.borrow_mut().control.register(waker);
    }

    fn wake_runner(&self) {
        self.wakers.borrow_mut().runner.wake();
    }

    fn register_runner(&self, waker: &Waker) {
        self.wakers.borrow_mut().runner.register(waker);
    }

    pub fn wait_complete(&self) -> impl Future<Output = usize> + '_ {
        poll_fn(|cx| {
            if let IoctlStateInner::Done { resp_len } = self.state.get() {
                Poll::Ready(resp_len)
            } else {
                self.register_control(cx.waker());
                Poll::Pending
            }
        })
    }

    pub fn wait_pending(&self) -> impl Future<Output = PendingIoctl> + '_ {
        poll_fn(|cx| {
            if let IoctlStateInner::Pending(pending) = self.state.get() {
                self.state.set(IoctlStateInner::Sent);
                Poll::Ready(pending)
            } else {
                self.register_runner(cx.waker());
                Poll::Pending
            }
        })
    }

    pub fn cancel_ioctl(&self) {
        self.state.set(IoctlStateInner::Done { resp_len: 0 });
    }

    /// Whether no ioctl is submitted or in flight right now.
    pub fn is_idle(&self) -> bool {
        matches!(self.state.get(), IoctlStateInner::Done { .. })
    }

    /// The ioctl buffer; the pending request is its first [`PendingIoctl::len`] bytes.
    pub fn request(&self) -> Ref<'_, [u8]> {
        Ref::map(self.buf.borrow(), |buf| &buf[..])
    }

    pub async fn do_ioctl(&self, kind: IoctlType, cmd: Ioctl, iface: u32, buf: &mut [u8]) -> usize {
        assert!(buf.len() <= MAX_IOCTL_LEN, "ioctl request too large");
        self.buf.borrow_mut()[..buf.len()].copy_from_slice(buf);
        self.state.set(IoctlStateInner::Pending(PendingIoctl {
            len: buf.len(),
            kind,
            cmd,
            iface,
        }));
        self.wake_runner();
        let resp_len = self.wait_complete().await.min(buf.len());
        buf[..resp_len].copy_from_slice(&self.buf.borrow()[..resp_len]);
        resp_len
    }

    pub fn ioctl_done(&self, response: &[u8]) {
        if let IoctlStateInner::Sent = self.state.get() {
            trace!("IOCTL Response: {:02x}", Bytes(response));

            let resp_len = response.len().min(MAX_IOCTL_LEN);
            self.buf.borrow_mut()[..resp_len].copy_from_slice(&response[..resp_len]);

            self.state.set(IoctlStateInner::Done { resp_len });
            self.wake_control();
        } else {
            warn!("IOCTL Response but no pending Ioctl");
        }
    }
}

#[cfg(test)]
mod tests {
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};

    use super::*;

    #[test]
    fn request_and_response_go_through_the_owned_buffer() {
        let state = IoctlState::new();
        let mut cx = Context::from_waker(Waker::noop());

        let mut buf = [1, 2, 3, 4];
        {
            let mut ioctl = pin!(state.do_ioctl(IoctlType::Get, Ioctl::GetRssi, 0, &mut buf));
            assert!(ioctl.as_mut().poll(&mut cx).is_pending());

            let Poll::Ready(pending) = pin!(state.wait_pending()).as_mut().poll(&mut cx) else {
                panic!("expected a pending ioctl");
            };
            assert_eq!(pending.len, 4);
            assert_eq!(&state.request()[..pending.len], &[1, 2, 3, 4]);

            // A response longer than the caller's buffer is truncated to it.
            state.ioctl_done(&[5, 6, 7, 8, 9, 10]);
            assert_eq!(ioctl.as_mut().poll(&mut cx), Poll::Ready(4));
        }
        assert_eq!(buf, [5, 6, 7, 8]);
    }
}
