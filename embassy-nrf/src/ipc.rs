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
pub struct Ipc<'d> {
    /// Event 0
    pub event0: Event<'d>,
    /// Event 1
    pub event1: Event<'d>,
    /// Event 2
    pub event2: Event<'d>,
    /// Event 3
    pub event3: Event<'d>,
    /// Event 4
    pub event4: Event<'d>,
    /// Event 5
    pub event5: Event<'d>,
    /// Event 6
    pub event6: Event<'d>,
    /// Event 7
    pub event7: Event<'d>,
    /// Event 8
    pub event8: Event<'d>,
    /// Event 9
    pub event9: Event<'d>,
    /// Event 10
    pub event10: Event<'d>,
    /// Event 11
    pub event11: Event<'d>,
    /// Event 12
    pub event12: Event<'d>,
    /// Event 13
    pub event13: Event<'d>,
    /// Event 14
    pub event14: Event<'d>,
    /// Event 15
    pub event15: Event<'d>,
}

impl<'d> Ipc<'d> {
    /// Create a new IPC driver.
    pub fn new<T: Instance>(
        _p: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let r = T::regs();
        let state = T::state();
        #[rustfmt::skip]
        let result = Self { // attributes on expressions are experimental
            event0: Event { number: EventNumber::Event0, r, state, _phantom: PhantomData },
            event1: Event { number: EventNumber::Event1, r, state, _phantom: PhantomData },
            event2: Event { number: EventNumber::Event2, r, state, _phantom: PhantomData },
            event3: Event { number: EventNumber::Event3, r, state, _phantom: PhantomData },
            event4: Event { number: EventNumber::Event4, r, state, _phantom: PhantomData },
            event5: Event { number: EventNumber::Event5, r, state, _phantom: PhantomData },
            event6: Event { number: EventNumber::Event6, r, state, _phantom: PhantomData },
            event7: Event { number: EventNumber::Event7, r, state, _phantom: PhantomData },
            event8: Event { number: EventNumber::Event8, r, state, _phantom: PhantomData },
            event9: Event { number: EventNumber::Event9, r, state, _phantom: PhantomData },
            event10: Event { number: EventNumber::Event10, r, state, _phantom: PhantomData },
            event11: Event { number: EventNumber::Event11, r, state, _phantom: PhantomData },
            event12: Event { number: EventNumber::Event12, r, state, _phantom: PhantomData },
            event13: Event { number: EventNumber::Event13, r, state, _phantom: PhantomData },
            event14: Event { number: EventNumber::Event14, r, state, _phantom: PhantomData },
            event15: Event { number: EventNumber::Event15, r, state, _phantom: PhantomData },
        };
        result
    }
}

/// IPC event
pub struct Event<'d> {
    number: EventNumber,
    r: pac::ipc::Ipc,
    state: &'static State,
    _phantom: PhantomData<&'d ()>,
}

impl<'d> Event<'d> {
    /// Trigger the event.
    pub fn trigger(&self) {
        let nr = self.number;
        self.r.tasks_send(nr as usize).write_value(1);
    }

    /// Wait for the event to be triggered.
    pub async fn wait(&mut self) {
        let nr = self.number as usize;
        self.r.intenset().write(|w| w.0 = 1 << nr);
        poll_fn(|cx| {
            self.state.wakers[nr].register(cx.waker());

            if self.r.events_receive(nr).read() == 1 {
                self.r.events_receive(nr).write_value(0x00);
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
    pub fn trigger_handle(&self) -> EventTrigger<'d> {
        EventTrigger {
            number: self.number,
            r: self.r,
            _phantom: PhantomData,
        }
    }

    /// Configure the channels the event will broadcast to
    pub fn configure_trigger<I: IntoIterator<Item = IpcChannel>>(&mut self, channels: I) {
        self.r.send_cnf(self.number as usize).write(|w| {
            for channel in channels {
                w.0 |= channel.mask();
            }
        })
    }

    /// Configure the channels the event will listen on
    pub fn configure_wait<I: IntoIterator<Item = IpcChannel>>(&mut self, channels: I) {
        self.r.receive_cnf(self.number as usize).write(|w| {
            for channel in channels {
                w.0 |= channel.mask();
            }
        });
    }

    /// Get the task for the IPC event to use with PPI.
    pub fn task(&self) -> ppi::Task<'d> {
        let nr = self.number as usize;
        ppi::Task::from_reg(self.r.tasks_send(nr))
    }

    /// Get the event for the IPC event to use with PPI.
    pub fn event(&self) -> ppi::Event<'d> {
        let nr = self.number as usize;
        ppi::Event::from_reg(self.r.events_receive(nr))
    }

    /// Reborrow into a "child" Event.
    ///
    /// `self` will stay borrowed until the child Event is dropped.
    pub fn reborrow(&mut self) -> Event<'_> {
        Event {
            number: self.number,
            r: self.r,
            state: self.state,
            _phantom: PhantomData,
        }
    }

    /// Steal an IPC event by number.
    ///
    /// # Safety
    ///
    /// The event number must not be in use by another [`Event`].
    pub unsafe fn steal<T: Instance>(number: EventNumber) -> Self {
        Self {
            number,
            r: T::regs(),
            state: T::state(),
            _phantom: PhantomData,
        }
    }
}

/// A handle that can trigger an IPC event.
///
/// This `struct` is returned by [`Event::trigger_handle`].
pub struct EventTrigger<'d> {
    number: EventNumber,
    r: pac::ipc::Ipc,
    _phantom: PhantomData<&'d ()>,
}

impl EventTrigger<'_> {
    /// Trigger the event.
    pub fn trigger(&self) {
        let nr = self.number;
        self.r.tasks_send(nr as usize).write_value(1);
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
