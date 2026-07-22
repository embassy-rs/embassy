//! PPIB (PPI bridge).
//!
//! On parts with multiple power domains (e.g. nRF54L) DPPI channels are local to
//! a domain; a PPIB forwards a signal across domains through a fixed pair of PPIB
//! instances. A cross-domain link is two ordinary [`Ppi`](crate::ppi::Ppi) hops
//! (one per domain) joined by a [`Ppib`]: connect the source event to the
//! bridge's [`task`](Ppib::task), and the bridge's [`event`](Ppib::event) to the
//! destination task.
//!
//! Hardwired pairs (nRF54L): PPIB00<->PPIB10, PPIB01<->PPIB20, PPIB11<->PPIB21,
//! PPIB22<->PPIB30.

use core::ptr::NonNull;

use embassy_hal_internal::{Peri, PeripheralType};

use crate::pac;
use crate::ppi::{Event, Task};

pub(crate) trait SealedBridgeChannel {
    fn regs(&self) -> pac::ppib::Ppib;
    fn number(&self) -> usize;
}

/// One half of a hardwired bridge. [`Paired`](Self::Paired) is the channel it is
/// physically connected to in the other domain (same index, paired instance).
#[allow(private_bounds)]
pub trait BridgeChannel: SealedBridgeChannel + PeripheralType + Sized + 'static {
    /// The hardwired counterpart in the other domain. Symmetric:
    /// `<Self::Paired as BridgeChannel>::Paired` is `Self`.
    type Paired: BridgeChannel;
}

/// A cross-domain PPI bridge: a source channel and its hardwired sink channel in
/// the other domain.
pub struct Ppib<'d> {
    task: Task<'d>,
    event: Event<'d>,
}

impl<'d> Ppib<'d> {
    /// Bridge from the `source` channel to the `sink` channel.
    pub fn new<S: BridgeChannel>(source: Peri<'d, S>, sink: Peri<'d, S::Paired>) -> Self {
        let task = source.regs().tasks_send(source.number()).as_ptr();
        let event = sink.regs().events_receive(sink.number()).as_ptr();
        Self {
            task: unsafe { Task::new_unchecked(NonNull::new_unchecked(task)) },
            event: unsafe { Event::new_unchecked(NonNull::new_unchecked(event)) },
        }
    }

    /// The source-domain SEND task.
    pub fn task(&self) -> Task<'d> {
        self.task
    }

    /// The destination-domain RECEIVE event.
    pub fn event(&self) -> Event<'d> {
        self.event
    }
}

macro_rules! impl_ppib_channel {
    ($type:ident, $inst:ident, $number:expr, $paired:ident) => {
        impl crate::ppib::SealedBridgeChannel for crate::peripherals::$type {
            fn regs(&self) -> crate::pac::ppib::Ppib {
                crate::pac::$inst
            }
            fn number(&self) -> usize {
                $number
            }
        }
        impl crate::ppib::BridgeChannel for crate::peripherals::$type {
            type Paired = crate::peripherals::$paired;
        }
    };
}
pub(crate) use impl_ppib_channel;
