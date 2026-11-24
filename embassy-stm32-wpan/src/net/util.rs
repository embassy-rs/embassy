//! Utils for the driver

use core::cell::RefCell;
use core::future::poll_fn;
use core::pin::Pin;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;

use crate::net::iface::ControllerToHostPacketBox;

pub struct ZeroCopyPubSub<B: ControllerToHostPacketBox> {
    event: RefCell<Option<Signal<NoopRawMutex, B>>>,
}

impl<B: ControllerToHostPacketBox> ZeroCopyPubSub<B> {
    pub const fn new() -> Self {
        Self {
            event: RefCell::new(None),
        }
    }

    pub fn publish(&self, event: B) {
        if let Some(signal) = self.event.borrow_mut().as_ref() {
            signal.signal(event);
        }
    }

    pub fn subscribe<'a>(&'a self) -> Subscriber<'a, B> {
        Subscriber::new(&self.event)
    }
}

pub struct Subscriber<'a, B: ControllerToHostPacketBox> {
    event: &'a RefCell<Option<Signal<NoopRawMutex, B>>>,
}

impl<'a, B: ControllerToHostPacketBox> Subscriber<'a, B> {
    fn new(event: &'a RefCell<Option<Signal<NoopRawMutex, B>>>) -> Self {
        if event.borrow_mut().replace(Signal::new()).is_some() {
            panic!("ZeroCopyPubSub cannot have multiple subscribers ")
        }

        Self { event }
    }

    pub async fn wait(&self) -> B {
        poll_fn(|cx| Pin::new(&mut self.event.borrow_mut().as_ref().unwrap().wait()).poll(cx)).await
    }
}

impl<'a, B: ControllerToHostPacketBox> Drop for Subscriber<'a, B> {
    fn drop(&mut self) {
        self.event.borrow_mut().take();
    }
}
