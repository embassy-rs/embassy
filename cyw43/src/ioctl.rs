use core::cell::{Cell, RefCell};
use core::future::{poll_fn, Future};
use core::task::{Poll, Waker};

use embassy_sync::waitqueue::WakerRegistration;

use crate::consts::Ioctl;
use crate::fmt::Bytes;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IoctlType {
    Get = 0,
    Set = 2,
}

#[derive(Clone, Copy)]
pub struct PendingIoctl {
    pub buf: *mut [u8],
    pub kind: IoctlType,
    pub cmd: Ioctl,
    pub iface: u32,
}

#[derive(Clone, Copy)]
enum IoctlStateInner {
    Pending(PendingIoctl),
    Sent { buf: *mut [u8] },
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
    wakers: RefCell<Wakers>,
}

impl IoctlState {
    pub const fn new() -> Self {
        Self {
            state: Cell::new(IoctlStateInner::Done { resp_len: 0 }),
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
                self.state.set(IoctlStateInner::Sent { buf: pending.buf });
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

    pub async fn do_ioctl(&self, kind: IoctlType, cmd: Ioctl, iface: u32, buf: &mut [u8]) -> usize {
        self.state
            .set(IoctlStateInner::Pending(PendingIoctl { buf, kind, cmd, iface }));
        self.wake_runner();
        self.wait_complete().await
    }

    pub fn ioctl_done(&self, response: &[u8]) {
        if let IoctlStateInner::Sent { buf } = self.state.get() {
            trace!("IOCTL Response: {:02x}", Bytes(response));

            // TODO fix this
            (unsafe { &mut *buf }[..response.len()]).copy_from_slice(response);

            self.state.set(IoctlStateInner::Done {
                resp_len: response.len(),
            });
            self.wake_control();
        } else {
            warn!("IOCTL Response but no pending Ioctl");
        }
    }
}
