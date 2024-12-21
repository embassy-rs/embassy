//! InterProcessor Communication (IPC)

#![macro_use]

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::peripherals::IPC;
use crate::{interrupt, pac};

/// IPC Event
#[derive(Debug, Clone, Copy)]
pub enum IpcEvent {
    /// IPC Event 0
    Event0 = 0,
    /// IPC Event 1
    Event1 = 1,
    /// IPC Event 2
    Event2 = 2,
    /// IPC Event 3
    Event3 = 3,
    /// IPC Event 4
    Event4 = 4,
    /// IPC Event 5
    Event5 = 5,
    /// IPC Event 6
    Event6 = 6,
    /// IPC Event 7
    Event7 = 7,
    /// IPC Event 8
    Event8 = 8,
    /// IPC Event 9
    Event9 = 9,
    /// IPC Event 10
    Event10 = 10,
    /// IPC Event 11
    Event11 = 11,
    /// IPC Event 12
    Event12 = 12,
    /// IPC Event 13
    Event13 = 13,
    /// IPC Event 14
    Event14 = 14,
    /// IPC Event 15
    Event15 = 15,
}

const EVENTS: [IpcEvent; 16] = [
    IpcEvent::Event0,
    IpcEvent::Event1,
    IpcEvent::Event2,
    IpcEvent::Event3,
    IpcEvent::Event4,
    IpcEvent::Event5,
    IpcEvent::Event6,
    IpcEvent::Event7,
    IpcEvent::Event8,
    IpcEvent::Event9,
    IpcEvent::Event10,
    IpcEvent::Event11,
    IpcEvent::Event12,
    IpcEvent::Event13,
    IpcEvent::Event14,
    IpcEvent::Event15,
];

/// IPC Channel
#[derive(Debug, Clone, Copy)]
pub enum IpcChannel {
    /// IPC Channel 0
    Channel0,
    /// IPC Channel 1
    Channel1,
    /// IPC Channel 2
    Channel2,
    /// IPC Channel 3
    Channel3,
    /// IPC Channel 4
    Channel4,
    /// IPC Channel 5
    Channel5,
    /// IPC Channel 6
    Channel6,
    /// IPC Channel 7
    Channel7,
    /// IPC Channel 8
    Channel8,
    /// IPC Channel 9
    Channel9,
    /// IPC Channel 10
    Channel10,
    /// IPC Channel 11
    Channel11,
    /// IPC Channel 12
    Channel12,
    /// IPC Channel 13
    Channel13,
    /// IPC Channel 14
    Channel14,
    /// IPC Channel 15
    Channel15,
}

/// Interrupt Handler
pub struct InterruptHandler {}

impl interrupt::typelevel::Handler<interrupt::typelevel::IPC> for InterruptHandler {
    unsafe fn on_interrupt() {
        let regs = IPC::regs();

        // Check if an event was generated, and if it was, trigger the corresponding waker
        for event in EVENTS {
            if regs.events_receive(event as usize).read() & 0x01 == 0x01 {
                // Event is set. Reset and wake waker
                regs.events_receive(event as usize).write_value(0);
                IPC::state().waker_for(event);
            }

            // Ensure the state is actually cleared
            //  Ref: nRF5340 PS v1.5 7.1.9.1 p.153
            compiler_fence(Ordering::SeqCst);
            while regs.events_receive(event as usize).read() & 0x01 != 0x00 {}
        }
    }
}

/// IPC driver
pub struct Ipc<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> From<PeripheralRef<'d, T>> for Ipc<'d, T> {
    fn from(value: PeripheralRef<'d, T>) -> Self {
        Self { _peri: value }
    }
}

impl<'d, T: Instance> Ipc<'d, T> {
    /// Create IPC driver
    pub fn new(ipc: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(ipc);

        Self { _peri: ipc }
    }

    /// Duplicates the peripheral singleton
    ///
    /// # Safety
    ///
    /// Ensure manually that only one peripheral is in use at one time
    pub unsafe fn clone_unchecked(&self) -> Self {
        Self {
            _peri: self._peri.clone_unchecked(),
        }
    }

    /// Configures the sending of events
    ///
    /// Events can be configured to broadcast on one or multiple IPC channels.
    pub fn configure_send_event<I: IntoIterator<Item = IpcChannel>>(&self, ev: IpcEvent, channels: I) {
        let regs = T::regs();

        regs.send_cnf(ev as usize).write(|w| {
            for channel in channels {
                match channel {
                    IpcChannel::Channel0 => w.set_chen0(true),
                    IpcChannel::Channel1 => w.set_chen1(true),
                    IpcChannel::Channel2 => w.set_chen2(true),
                    IpcChannel::Channel3 => w.set_chen3(true),
                    IpcChannel::Channel4 => w.set_chen4(true),
                    IpcChannel::Channel5 => w.set_chen5(true),
                    IpcChannel::Channel6 => w.set_chen6(true),
                    IpcChannel::Channel7 => w.set_chen7(true),
                    IpcChannel::Channel8 => w.set_chen8(true),
                    IpcChannel::Channel9 => w.set_chen9(true),
                    IpcChannel::Channel10 => w.set_chen10(true),
                    IpcChannel::Channel11 => w.set_chen11(true),
                    IpcChannel::Channel12 => w.set_chen12(true),
                    IpcChannel::Channel13 => w.set_chen13(true),
                    IpcChannel::Channel14 => w.set_chen14(true),
                    IpcChannel::Channel15 => w.set_chen15(true),
                }
            }
        })
    }

    /// Configures the receiving of events
    ///
    /// Events can be configured to be received by one or multiple IPC channels.
    pub fn configure_receive_event<I: IntoIterator<Item = IpcChannel>>(&self, ev: IpcEvent, channels: I) {
        let regs = T::regs();

        regs.receive_cnf(ev as usize).write(|w| {
            for channel in channels {
                match channel {
                    IpcChannel::Channel0 => w.set_chen0(true),
                    IpcChannel::Channel1 => w.set_chen1(true),
                    IpcChannel::Channel2 => w.set_chen2(true),
                    IpcChannel::Channel3 => w.set_chen3(true),
                    IpcChannel::Channel4 => w.set_chen4(true),
                    IpcChannel::Channel5 => w.set_chen5(true),
                    IpcChannel::Channel6 => w.set_chen6(true),
                    IpcChannel::Channel7 => w.set_chen7(true),
                    IpcChannel::Channel8 => w.set_chen8(true),
                    IpcChannel::Channel9 => w.set_chen9(true),
                    IpcChannel::Channel10 => w.set_chen10(true),
                    IpcChannel::Channel11 => w.set_chen11(true),
                    IpcChannel::Channel12 => w.set_chen12(true),
                    IpcChannel::Channel13 => w.set_chen13(true),
                    IpcChannel::Channel14 => w.set_chen14(true),
                    IpcChannel::Channel15 => w.set_chen15(true),
                }
            }
        });
    }

    /// Triggers an event
    pub fn trigger_event(&self, ev: IpcEvent) {
        let regs = T::regs();

        regs.tasks_send(ev as usize).write_value(0x01);
    }

    /// Wait for event to be triggered
    pub async fn wait_for_event(&self, ev: IpcEvent) {
        let regs = T::regs();

        // Enable interrupt
        match ev {
            IpcEvent::Event0 => {
                regs.inten().modify(|m| m.set_receive0(true));
            }
            IpcEvent::Event1 => {
                regs.inten().modify(|m| m.set_receive1(true));
            }
            IpcEvent::Event2 => {
                regs.inten().modify(|m| m.set_receive2(true));
            }
            IpcEvent::Event3 => {
                regs.inten().modify(|m| m.set_receive3(true));
            }
            IpcEvent::Event4 => {
                regs.inten().modify(|m| m.set_receive4(true));
            }
            IpcEvent::Event5 => {
                regs.inten().modify(|m| m.set_receive5(true));
            }
            IpcEvent::Event6 => {
                regs.inten().modify(|m| m.set_receive6(true));
            }
            IpcEvent::Event7 => {
                regs.inten().modify(|m| m.set_receive7(true));
            }
            IpcEvent::Event8 => {
                regs.inten().modify(|m| m.set_receive8(true));
            }
            IpcEvent::Event9 => {
                regs.inten().modify(|m| m.set_receive9(true));
            }
            IpcEvent::Event10 => {
                regs.inten().modify(|m| m.set_receive10(true));
            }
            IpcEvent::Event11 => {
                regs.inten().modify(|m| m.set_receive11(true));
            }
            IpcEvent::Event12 => {
                regs.inten().modify(|m| m.set_receive12(true));
            }
            IpcEvent::Event13 => {
                regs.inten().modify(|m| m.set_receive13(true));
            }
            IpcEvent::Event14 => {
                regs.inten().modify(|m| m.set_receive14(true));
            }
            IpcEvent::Event15 => {
                regs.inten().modify(|m| m.set_receive15(true));
            }
        };

        poll_fn(|cx| {
            IPC::state().waker_for(ev).register(cx.waker());

            if regs.events_receive(ev as usize).read() & 0x01 == 0x01 {
                regs.events_receive(ev as usize).write_value(0x00);

                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}

pub(crate) struct State {
    wakers: [AtomicWaker; 16],
}

impl State {
    pub(crate) const fn new() -> Self {
        const WAKER: AtomicWaker = AtomicWaker::new();

        Self { wakers: [WAKER; 16] }
    }

    const fn waker_for(&self, ev: IpcEvent) -> &AtomicWaker {
        match ev {
            IpcEvent::Event0 => &self.wakers[0],
            IpcEvent::Event1 => &self.wakers[1],
            IpcEvent::Event2 => &self.wakers[2],
            IpcEvent::Event3 => &self.wakers[3],
            IpcEvent::Event4 => &self.wakers[4],
            IpcEvent::Event5 => &self.wakers[5],
            IpcEvent::Event6 => &self.wakers[6],
            IpcEvent::Event7 => &self.wakers[7],
            IpcEvent::Event8 => &self.wakers[8],
            IpcEvent::Event9 => &self.wakers[9],
            IpcEvent::Event10 => &self.wakers[10],
            IpcEvent::Event11 => &self.wakers[11],
            IpcEvent::Event12 => &self.wakers[12],
            IpcEvent::Event13 => &self.wakers[13],
            IpcEvent::Event14 => &self.wakers[14],
            IpcEvent::Event15 => &self.wakers[15],
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::ipc::Ipc;
    fn state() -> &'static State;
}

/// IPC peripheral instance.
#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + SealedInstance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_ipc {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::ipc::SealedInstance for peripherals::$type {
            fn regs() -> pac::ipc::Ipc {
                pac::$pac_type
            }

            fn state() -> &'static crate::ipc::State {
                static STATE: crate::ipc::State = crate::ipc::State::new();
                &STATE
            }
        }
        impl crate::ipc::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
