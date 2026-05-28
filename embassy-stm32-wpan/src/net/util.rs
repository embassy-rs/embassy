//! Utils for the driver

use core::cell::RefCell;
use core::future::poll_fn;

use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use futures_util::FutureExt;

use crate::net::iface::ControllerToHostPacketBox;

pub struct ZeroCopyPubSub<B: ControllerToHostPacketBox> {
    event: blocking_mutex::Mutex<NoopRawMutex, RefCell<Option<Signal<NoopRawMutex, B>>>>,
}

impl<B: ControllerToHostPacketBox> ZeroCopyPubSub<B> {
    pub const fn new() -> Self {
        Self {
            event: blocking_mutex::Mutex::const_new(NoopRawMutex::new(), RefCell::new(None)),
        }
    }

    pub fn publish(&self, event: B) {
        if let Some(signal) = self.event.borrow().borrow_mut().as_ref() {
            signal.signal(event);
        }
    }

    pub fn subscribe<'a>(&'a self) -> Subscriber<'a, B> {
        Subscriber::new(&self.event)
    }
}

pub struct Subscriber<'a, B: ControllerToHostPacketBox> {
    event: &'a blocking_mutex::Mutex<NoopRawMutex, RefCell<Option<Signal<NoopRawMutex, B>>>>,
}

impl<'a, B: ControllerToHostPacketBox> Subscriber<'a, B> {
    fn new(event: &'a blocking_mutex::Mutex<NoopRawMutex, RefCell<Option<Signal<NoopRawMutex, B>>>>) -> Self {
        if event.borrow().borrow_mut().replace(Signal::new()).is_some() {
            panic!("ZeroCopyPubSub cannot have multiple subscribers ")
        }

        Self { event }
    }

    pub async fn wait(&self) -> B {
        poll_fn(|cx| self.event.borrow().borrow_mut().as_ref().unwrap().wait().poll_unpin(cx)).await
    }
}

impl<'a, B: ControllerToHostPacketBox> Drop for Subscriber<'a, B> {
    fn drop(&mut self) {
        self.event.borrow().borrow_mut().take();
    }
}
