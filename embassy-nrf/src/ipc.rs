//! InterProcessor Communication (IPC)

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, ppi};

const EVENT_COUNT: usize = 16;

/// IPC Event
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventNumber {
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

const EVENTS: [EventNumber; EVENT_COUNT] = [
    EventNumber::Event0,
    EventNumber::Event1,
    EventNumber::Event2,
    EventNumber::Event3,
    EventNumber::Event4,
    EventNumber::Event5,
    EventNumber::Event6,
    EventNumber::Event7,
    EventNumber::Event8,
    EventNumber::Event9,
    EventNumber::Event10,
    EventNumber::Event11,
    EventNumber::Event12,
    EventNumber::Event13,
    EventNumber::Event14,
    EventNumber::Event15,
];

/// IPC Channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl IpcChannel {
    fn mask(self) -> u32 {
        1 << (self as u32)
    }
}

/// Interrupt Handler
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();

        // Check if an event was generated, and if it was, trigger the corresponding waker
        for event in EVENTS {
            if regs.events_receive(event as usize).read() & 0x01 == 0x01 {
                regs.intenclr().write(|w| w.0 = 0x01 << event as u32);
                T::state().wakers[event as usize].wake();
            }
        }
    }
}

/// IPC driver
#[non_exhaustive]
pub struct Ipc<'d, T: Instance> {
    /// Event 0
    pub event0: Event<'d, T>,
    /// Event 1
    pub event1: Event<'d, T>,
    /// Event 2
    pub event2: Event<'d, T>,
    /// Event 3
    pub event3: Event<'d, T>,
    /// Event 4
    pub event4: Event<'d, T>,
    /// Event 5
    pub event5: Event<'d, T>,
    /// Event 6
    pub event6: Event<'d, T>,
    /// Event 7
    pub event7: Event<'d, T>,
    /// Event 8
    pub event8: Event<'d, T>,
    /// Event 9
    pub event9: Event<'d, T>,
    /// Event 10
    pub event10: Event<'d, T>,
    /// Event 11
    pub event11: Event<'d, T>,
    /// Event 12
    pub event12: Event<'d, T>,
    /// Event 13
    pub event13: Event<'d, T>,
    /// Event 14
    pub event14: Event<'d, T>,
    /// Event 15
    pub event15: Event<'d, T>,
}

impl<'d, T: Instance> Ipc<'d, T> {
    /// Create a new IPC driver.
    pub fn new(
        _p: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let _phantom = PhantomData;
        #[rustfmt::skip]
        let r = Self { // attributes on expressions are experimental
            event0: Event { number: EventNumber::Event0, _phantom },
            event1: Event { number: EventNumber::Event1, _phantom },
            event2: Event { number: EventNumber::Event2, _phantom },
            event3: Event { number: EventNumber::Event3, _phantom },
            event4: Event { number: EventNumber::Event4, _phantom },
            event5: Event { number: EventNumber::Event5, _phantom },
            event6: Event { number: EventNumber::Event6, _phantom },
            event7: Event { number: EventNumber::Event7, _phantom },
            event8: Event { number: EventNumber::Event8, _phantom },
            event9: Event { number: EventNumber::Event9, _phantom },
            event10: Event { number: EventNumber::Event10, _phantom },
            event11: Event { number: EventNumber::Event11, _phantom },
            event12: Event { number: EventNumber::Event12, _phantom },
            event13: Event { number: EventNumber::Event13, _phantom },
            event14: Event { number: EventNumber::Event14, _phantom },
            event15: Event { number: EventNumber::Event15, _phantom },
        };
        r
    }
}

/// IPC event
pub struct Event<'d, T: Instance> {
    number: EventNumber,
    _phantom: PhantomData<&'d T>,
}

impl<'d, T: Instance> Event<'d, T> {
    /// Trigger the event.
    pub fn trigger(&self) {
        let nr = self.number;
        T::regs().tasks_send(nr as usize).write_value(1);
    }

    /// Wait for the event to be triggered.
    pub async fn wait(&mut self) {
        let regs = T::regs();
        let nr = self.number as usize;
        regs.intenset().write(|w| w.0 = 1 << nr);
        poll_fn(|cx| {
            T::state().wakers[nr].register(cx.waker());

            if regs.events_receive(nr).read() == 1 {
                regs.events_receive(nr).write_value(0x00);
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Returns the [`EventNumber`] of the event.
    pub fn number(&self) -> EventNumber {
        self.number
    }

    /// Create a handle that can trigger the event.
    pub fn trigger_handle(&self) -> EventTrigger<'d, T> {
        EventTrigger {
            number: self.number,
            _phantom: PhantomData,
        }
    }

    /// Configure the channels the event will broadcast to
    pub fn configure_trigger<I: IntoIterator<Item = IpcChannel>>(&mut self, channels: I) {
        T::regs().send_cnf(self.number as usize).write(|w| {
            for channel in channels {
                w.0 |= channel.mask();
            }
        })
    }

    /// Configure the channels the event will listen on
    pub fn configure_wait<I: IntoIterator<Item = IpcChannel>>(&mut self, channels: I) {
        T::regs().receive_cnf(self.number as usize).write(|w| {
            for channel in channels {
                w.0 |= channel.mask();
            }
        });
    }

    /// Get the task for the IPC event to use with PPI.
    pub fn task(&self) -> ppi::Task<'d> {
        let nr = self.number as usize;
        let regs = T::regs();
        ppi::Task::from_reg(regs.tasks_send(nr))
    }

    /// Get the event for the IPC event to use with PPI.
    pub fn event(&self) -> ppi::Event<'d> {
        let nr = self.number as usize;
        let regs = T::regs();
        ppi::Event::from_reg(regs.events_receive(nr))
    }

    /// Reborrow into a "child" Event.
    ///
    /// `self` will stay borrowed until the child Event is dropped.
    pub fn reborrow(&mut self) -> Event<'_, T> {
        Self { ..*self }
    }

    /// Steal an IPC event by number.
    ///
    /// # Safety
    ///
    /// The event number must not be in use by another [`Event`].
    pub unsafe fn steal(number: EventNumber) -> Self {
        Self {
            number,
            _phantom: PhantomData,
        }
    }
}

/// A handle that can trigger an IPC event.
///
/// This `struct` is returned by [`Event::trigger_handle`].
#[derive(Debug, Copy, Clone)]
pub struct EventTrigger<'d, T: Instance> {
    number: EventNumber,
    _phantom: PhantomData<&'d T>,
}

impl<T: Instance> EventTrigger<'_, T> {
    /// Trigger the event.
    pub fn trigger(&self) {
        let nr = self.number;
        T::regs().tasks_send(nr as usize).write_value(1);
    }

    /// Returns the [`EventNumber`] of the event.
    pub fn number(&self) -> EventNumber {
        self.number
    }
}

pub(crate) struct State {
    wakers: [AtomicWaker; EVENT_COUNT],
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            wakers: [const { AtomicWaker::new() }; EVENT_COUNT],
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::ipc::Ipc;
    fn state() -> &'static State;
}

/// IPC peripheral instance.
#[allow(private_bounds)]
pub trait Instance: PeripheralType + SealedInstance + 'static + Send {
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
