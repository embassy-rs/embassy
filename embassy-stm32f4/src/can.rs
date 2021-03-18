//! Async low power Serial.
//!
//! The peripheral is autmatically enabled and disabled as required to save power.
//! Lowest power consumption can only be guaranteed if the send receive futures
//! are dropped correctly (e.g. not using `mem::forget()`).

use bxcan;
use bxcan::Interrupts;
use core::future::Future;
use embassy::interrupt::Interrupt;
use embassy::util::InterruptFuture;
use nb;
use nb::block;

use crate::interrupt;

/// Interface to the Serial peripheral
pub struct Can<T: Instance> {
    can: bxcan::Can<T>,
    tx_int: T::TInterrupt,
    rx_int: T::RInterrupt,
}

impl<T: Instance> Can<T> {
    pub fn new(mut can: bxcan::Can<T>, tx_int: T::TInterrupt, rx_int: T::RInterrupt) -> Self {
        // Sync to the bus and start normal operation.
        can.enable_interrupts(
            Interrupts::TRANSMIT_MAILBOX_EMPTY | Interrupts::FIFO0_MESSAGE_PENDING,
        );
        block!(can.enable()).unwrap();

        Can {
            can: can,
            tx_int: tx_int,
            rx_int: rx_int,
        }
    }

    /// Sends can frame.
    ///
    /// This method async-blocks until the frame is transmitted.
    pub fn transmit<'a>(&'a mut self, frame: &'a bxcan::Frame) -> impl Future<Output = ()> + 'a {
        async move {
            let fut = InterruptFuture::new(&mut self.tx_int);
            self.can.transmit(frame);

            fut.await;
        }
    }

    /// Receive can frame.
    ///
    /// This method async-blocks until the frame is received.
    pub fn receive<'a>(&'a mut self) -> impl Future<Output = bxcan::Frame> + 'a {
        async move {
            let mut frame: Option<bxcan::Frame>;

            loop {
                let fut = InterruptFuture::new(&mut self.rx_int);
                frame = match self.can.receive() {
                    Ok(frame) => Some(frame),
                    Err(nb::Error::WouldBlock) => None,
                    Err(nb::Error::Other(_)) => None, // Ignore overrun errors.
                };
                if frame.is_some() {
                    break;
                }
                fut.await;
            }

            frame.unwrap()
        }
    }
}

mod private {
    pub trait Sealed {}
}

pub trait Instance: bxcan::Instance + private::Sealed {
    type TInterrupt: Interrupt;
    type RInterrupt: Interrupt;
}

macro_rules! can {
    ($($can:ident => ($tint:ident, $rint:ident),)+) => {
        $(
            impl private::Sealed for crate::hal::can::Can<crate::pac::$can> {}
            impl Instance for crate::hal::can::Can<crate::pac::$can> {
                type TInterrupt = interrupt::$tint;
                type RInterrupt = interrupt::$rint;
            }
        )+
    }
}

#[cfg(any(feature = "stm32f405",))]
can! {
    CAN1 => (CAN1_TX, CAN1_RX0),
    CAN2 => (CAN2_TX, CAN2_RX0),
}
