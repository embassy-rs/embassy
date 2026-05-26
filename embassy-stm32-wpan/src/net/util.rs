//! Utils for the driver

use core::cell::{RefCell, UnsafeCell};
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_sync::waitqueue::AtomicWaker;
use futures_util::FutureExt;

use crate::net::iface::{ControllerToHostPacket, ControllerToHostPacketBox};

const ALLOC_SIZE: usize = 3;

/// Manages packet allocation for the buffer
pub struct Allocator<'a> {
    buf: [UnsafeCell<[u8; 255]>; ALLOC_SIZE],
    taken: [AtomicBool; ALLOC_SIZE],
    waker: AtomicWaker,
    _lifetime: PhantomData<&'a ()>,
}

pub struct RefBox<'a, B: ControllerToHostPacketBox<'a>> {
    pkt: B,
    taken: &'a AtomicBool,
    waker: &'a AtomicWaker,
}

impl<'a, B: ControllerToHostPacketBox<'a>> RefBox<'a, B> {
    pub fn packet<'b>(&'b self) -> ControllerToHostPacket<'b> {
        self.pkt.packet()
    }
}

impl<'a, B: ControllerToHostPacketBox<'a>> Drop for RefBox<'a, B> {
    fn drop(&mut self) {
        self.taken.store(false, Ordering::Release);
        self.waker.wake();
    }
}

impl<'a> Allocator<'a> {
    pub const fn new() -> Self {
        Self {
            buf: [const { UnsafeCell::new([0u8; 255]) }; 3],
            taken: [const { AtomicBool::new(false) }; 3],
            waker: AtomicWaker::new(),
            _lifetime: PhantomData,
        }
    }

    pub async fn make_packet<B, E>(
        &'a self,
        f: impl AsyncFn(&'a mut [u8; 255]) -> Result<B, E>,
    ) -> Result<RefBox<'a, B>, E>
    where
        B: ControllerToHostPacketBox<'a>,
    {
        let (buf, taken) = poll_fn(|cx| {
            self.waker.register(cx.waker());

            for (buf, taken) in self.buf.iter().zip(self.taken.iter()) {
                if !taken.load(Ordering::Acquire) {
                    return Poll::Ready((unsafe { &mut *buf.get() }, taken));
                }
            }

            Poll::Pending
        })
        .await;

        let pkt = f(buf).await?;

        taken.store(true, Ordering::Release);

        Ok(RefBox {
            pkt,
            taken: taken,
            waker: &self.waker,
        })
    }
}

pub struct ZeroCopyPubSub<'a, B: ControllerToHostPacketBox<'a>> {
    event: blocking_mutex::Mutex<NoopRawMutex, RefCell<Option<Signal<NoopRawMutex, RefBox<'a, B>>>>>,
}

impl<'a, B: ControllerToHostPacketBox<'a>> ZeroCopyPubSub<'a, B> {
    pub const fn new() -> Self {
        Self {
            event: blocking_mutex::Mutex::const_new(NoopRawMutex::new(), RefCell::new(None)),
        }
    }

    pub fn publish(&self, event: RefBox<'a, B>) {
        if let Some(signal) = self.event.borrow().borrow_mut().as_ref() {
            signal.signal(event);
        }
    }

    pub fn subscribe(&'a self) -> Subscriber<'a, B> {
        Subscriber::new(&self.event)
    }
}

pub struct Subscriber<'a, B: ControllerToHostPacketBox<'a>> {
    event: &'a blocking_mutex::Mutex<NoopRawMutex, RefCell<Option<Signal<NoopRawMutex, RefBox<'a, B>>>>>,
}

impl<'a, B: ControllerToHostPacketBox<'a>> Subscriber<'a, B> {
    fn new(
        event: &'a blocking_mutex::Mutex<NoopRawMutex, RefCell<Option<Signal<NoopRawMutex, RefBox<'a, B>>>>>,
    ) -> Self {
        if event.borrow().borrow_mut().replace(Signal::new()).is_some() {
            panic!("ZeroCopyPubSub cannot have multiple subscribers ")
        }

        Self { event }
    }

    pub async fn wait(&self) -> RefBox<'a, B> {
        poll_fn(|cx| self.event.borrow().borrow_mut().as_ref().unwrap().wait().poll_unpin(cx)).await
    }
}

impl<'a, B: ControllerToHostPacketBox<'a>> Drop for Subscriber<'a, B> {
    fn drop(&mut self) {
        self.event.borrow().borrow_mut().take();
    }
}
