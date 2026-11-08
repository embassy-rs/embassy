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
