//! This module holds everything specific to the Blocking flavor of the driver.
//!
//! This blocking mode doesn't use any interrupts at all (sad). It just directly polls the registers.

use core::sync::atomic::Ordering;

use embassy_hal_internal::Peri;

use super::mailbox::tx;
use super::{
    BusErrorMode, FlexCan, FlexCanConfig, FlexCanRx, FlexCanTx, InitError, Instance, Mode, ReceiveError, SendError,
    mailbox, sealed,
};
use crate::flexcan::classic::frame::Frame;
use crate::flexcan::{RxPin, TxPin};

/// Blocking driver mode. Use `FlexCan::new_blocking()` to construct a driver in
/// this mode.
///
/// This mode doesn't use any interrupts. The `blocking_send()`/`blocking_receive()` functions just do a blocking poll
/// the hardware until they can make progress. For most cases, you should probably just use the
/// `Async` mode, unless you specifically need Blocking functionality and are okay with accepting the risks.
#[doc = docs::doc_blocking_example!()]
pub struct Blocking;

impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Functions for `FlexCan` that are specific to `Blocking` mode.
impl<'d> FlexCan<'d, Blocking> {
    /// Constructs a new `Blocking` FlexCAN driver instance, in Classic mode.
    ///
    /// This mode doesn't use any interrupts. The `blocking_send()`/`blocking_receive()` functions just do a blocking poll
    /// the hardware until they can make progress. For most cases, you should probably just use the
    /// `Async` mode, unless you specifically need Blocking functionality and are okay with accepting the risks.
    #[doc = docs::doc_blocking_example!()]
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        config: FlexCanConfig<'_>,
    ) -> Result<Self, InitError> {
        let (info, rx_pin, tx_pin, wake_guard) = super::init::<T>(peri, rx, tx, &config)?;

        info.control.unfreeze();

        let tx = FlexCanTx::new(info, tx_pin, wake_guard.clone(), Blocking);
        let rx = FlexCanRx::new(info, rx_pin, wake_guard, Blocking);
        Ok(Self { tx, rx })
    }

    #[doc = docs::doc_blocking_send!()]
    pub fn blocking_send(&mut self, frame: &Frame) {
        self.tx.blocking_send(frame)
    }
    #[doc = docs::doc_try_send!()]
    pub fn try_send(&mut self, frame: &Frame) -> Result<(), SendError> {
        self.tx.try_send(frame)
    }
    #[doc = docs::doc_blocking_receive!()]
    pub fn blocking_receive(&self) -> Frame {
        self.rx.blocking_receive()
    }
    #[doc = docs::doc_try_receive!()]
    pub fn try_receive(&self) -> Result<Frame, ReceiveError> {
        self.rx.try_receive()
    }
}

/// Functions for `FlexCanTx` that are specific to `Blocking` mode.
impl<'d> FlexCanTx<'d, Blocking> {
    #[doc = docs::doc_blocking_send!()]
    pub fn blocking_send(&mut self, frame: &Frame) {
        let message = tx::TxMessage::from(frame);
        mailbox::tx::reclaim_completed(self.info);
        if tx::dispatch(self.info, &message).is_ok() {
            return;
        }

        // If the mailbox is full, we need to loop
        self.info.tx_mailbox_full_count.fetch_add(1, Ordering::Acquire);
        loop {
            mailbox::tx::reclaim_completed(self.info);
            match tx::dispatch(self.info, &message) {
                Ok(()) => return,
                Err(nb::Error::WouldBlock) => core::hint::spin_loop(),
                Err(nb::Error::Other(e)) => match e {},
            }
        }
    }
    #[doc = docs::doc_try_send!()]
    pub fn try_send(&mut self, frame: &Frame) -> Result<(), SendError> {
        if self.error_mode() == BusErrorMode::BusOff {
            return Err(SendError::BusOff);
        }
        mailbox::tx::reclaim_completed(self.info);
        let message = tx::TxMessage::from(frame);
        match tx::dispatch(self.info, &message) {
            Ok(()) => Ok(()),
            Err(nb::Error::WouldBlock) => {
                self.info.tx_mailbox_full_count.fetch_add(1, Ordering::Acquire);
                Err(SendError::TxMailboxFull)
            }
            Err(nb::Error::Other(e)) => match e {},
        }
    }
}

/// Functions for `FlexCanRx` that are specific to `Blocking` mode.
impl<'d> FlexCanRx<'d, Blocking> {
    #[doc = docs::doc_blocking_receive!()]
    pub fn blocking_receive(&self) -> Frame {
        loop {
            if let Some(frame) = self.poll_fifo() {
                return frame;
            }
            core::hint::spin_loop();
        }
    }
    #[doc = docs::doc_try_receive!()]
    pub fn try_receive(&self) -> Result<Frame, ReceiveError> {
        self.poll_fifo().ok_or(ReceiveError::NoMessages)
    }

    /// Helper to pop a frame from the hardware RX FIFO if one is available.
    fn poll_fifo(&self) -> Option<Frame> {
        let message = mailbox::rx::fifo::get(self.info)?;
        message.try_into().ok()
    }
}

/// Shared rustdocs that are re-used multiple places.
pub(in crate::flexcan::classic) mod docs {
    macro_rules! doc_blocking_send {
        () => {
            concat!(
                "Sends a CAN message.\n",
                "\n",
                "If all TX buffers are full, this blocks indefinietely until one\n",
                "frees up. If called during a BusOff event, this will block until the\n",
                "BusOff event recovers. By default, FlexCAN will recover from BusOff automatically.\n",
                "\n",
                "Note: If you need to avoid blocking, see `try_send()` (or, just use the `Async` mode).",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_blocking_send;

    macro_rules! doc_try_send {
        () => {
            concat!("Attempts to send a CAN message.",)
        };
    }
    pub(in crate::flexcan::classic) use doc_try_send;

    macro_rules! doc_blocking_receive {
        () => {
            concat!(
                "Receives a CAN message.\n",
                "\n",
                "If the hardware RX FIFO is empty, this will block until a frame arrives. If you need to\n",
                "avoid blocking, see `try_receive()` (or, just use the `Async` mode).",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_blocking_receive;

    macro_rules! doc_try_receive {
        () => {
            concat!(
                "Like `receive()`, but returns immediately with `ReceiveError::NoMessages` if the RX FIFO is empty.",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_try_receive;

    macro_rules! doc_blocking_example {
        () => { concat!(
            "<details>\n\n",
            "<summary><h4>Blocking Example</h4></summary>\n\n",
            "Here's a short example program that demonstrates how to set up a FlexCAN peripheral in Blocking mode for Classic CAN using this HAL:\n",
            "```rust,no_run\n",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/mcxa2xx/src/bin/flexcan-classic-blocking.rs")),
            "\n```\n",
            "</details>",
        ) };
    }
    pub(in crate::flexcan::classic) use doc_blocking_example;
}
