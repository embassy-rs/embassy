//! This module models the FlexCAN mailbox for Classic CAN (not FD). 
//! 
//! TX/outgoing messages are handled in the FlexCAN message buffer, which uses the memory area 80h - 27Fh (see page 1545 of the datasheet). This memory area is 512 bytes in total.
//! Each message consists of the CS Register (4 bytes), the Id Register (4 bytes), and the 8-byte message payload. So, each message buffer is 16 bytes in total.
//! This means that the message buffer can hold a total of 512 / 16 = 32 messages.
//! 
//! RX/incoming messages are handeled by the chip's Enhanced RX FIFO (see page 1556 of the datasheet).
//! This FIFO can store 12 messages, which are filled automatically by the hardware as they come in.
//! Messages can be dequeued from this FIFO by reading the 2000h - 2048h memory area as a message buffer, and then setting the erfda flag (to tell the hardware that the memory area is ready to be filled with the next message from the FIFO).

use cortex_m::register::control::Control;
use nxp_pac::can as pac;
use crate::flexcan::can::Info;

/// The "raw" data structure of a FlexCAN message described in the datasheet.
/// For Classic CAN, this is the CS Register (4 bytes), the Id Register (4 bytes), and the 8-byte message payload.
/// This structure is used for both TX and RX messages. As mentioned, TX messages live in the FlexCAN message buffer, while RX messages live in the Enhanced RX FIFO.
struct Message {
    pub cs: pac::Cs,
    pub id: pac::Id,
    pub payload: [u8; 8],
}

/// Possible errors that may occur during mailbox operations.
pub(in crate::flexcan) enum MailboxError {
    /// During a mailbox operation, hardware failed to respond within a reasonable timeframe.
    Timeout,

    /// When trying to read the `CODE` field of a TX message, no known `TxCode` variant matched.
    UnknownTxCode,
}

pub(in crate::flexcan) mod tx {
    use super::Message;
    use super::pac;
    use crate::flexcan::can::Info;
    use crate::flexcan::frame::{Frame, Id};
    use crate::flexcan::mailbox::MailboxError;
    use core::sync::atomic::Ordering;
    use core::convert::Infallible;

    /// Represents the message buffer memory area (80h - 27Fh), which this HAL uses for dispatching TX messages.
    pub(in crate::flexcan) mod buffer {
        use super::Message;
        use super::TxMessage;
        use super::Info;
        use super::TxCode;

        /// Writes a `TxMessage` into one of the 32 message buffers.
        /// * `info` - The type-erased instance handle.
        /// * `message` - The TxMessage to write.
        /// * `n` - The message buffer element to write (0 through 31).
        pub fn write(info: &Info, message: &TxMessage, n: usize) {
            // Write in the payload
            let [b0, b1, b2, b3, b4, b5, b6, b7] = message.inner.payload;
            let word0 = u32::from_be_bytes([b0, b1, b2, b3]);
            let word1 = u32::from_be_bytes([b4, b5, b6, b7]);
            info.regs.word0(n).write(|w| { *w = word0 });
            info.regs.word1(n).write(|w| { *w = word1 });

            info.regs.id(n).write(|w| { w.0 = message.inner.id.0 });
            info.regs.cs(n).write(|w| { w.0 = message.inner.cs.0 }); // Need to write in CS last because this is when we update CODE (which could trigger a TX dispatch)
        }

        /// Reads one of the 32 message buffers into a `TxMessage`.
        /// * `info` - The type-erased instance handle.
        /// * `n` - The message buffer element to read (0 through 31).
        pub fn read(info: &Info, n: usize) -> TxMessage {
            let cs = info.regs.cs(n).read();
            let id = info.regs.id(n).read();

            // Read out the payload
            let word0 = info.regs.word0(n).read();
            let word1 = info.regs.word1(n).read();
            let [b0, b1, b2, b3] = word0.to_be_bytes();
            let [b4, b5, b6, b7] = word1.to_be_bytes();
            let payload = [b0, b1, b2, b3, b4, b5, b6, b7];

            TxMessage { inner: Message { cs, id, payload } }
        }

        /// Sets a buffer to its `INACTIVE` state. Only the CS register is affected.
        /// * `info` - The type-erased instance handle.
        /// * `n` - The buffer to reset (0 through 31).
        pub fn set_inactive(info: &Info, n: usize) {
            info.regs.cs(n).write(|w| w.set_code(TxCode::INACTIVE));
        }
    }

    /// Sets up the the FlexCAN mailbox for TX.
    /// This function resets the TX message buffers and our state tracking for what buffers are available.
    pub(in crate::flexcan) fn setup(info: &Info) -> Result<(), MailboxError> {
        use core::sync::atomic::Ordering;
        use embassy_time::{Duration, Instant};

        let mut control = crate::flexcan::control::Control::new(info);

        // Make sure we're frozen before continuing.
        const FREEZE_TIMEOUT: u64 = 10; // ms
        match control.freeze(Some(Duration::from_millis(FREEZE_TIMEOUT))) {
            Ok(_) => (),
            Err(_) => { return Err(MailboxError::Timeout); }
        }

        // Disable all 32 interrupts via the IMASK1 register
        const IMASK1_DISABLED: u32 = 0x0000_0000;
        info.regs.imask1().write(|w| w.0 = IMASK1_DISABLED);

        // Clear all IFLAG1 bits (this register is "write 1 to clear", so writing all 1s will clear the whole register)
        const IFLAG1_INIT: u32 = 0xFFFF_FFFF;
        info.regs.iflag1().write(|w| w.0 = IFLAG1_INIT);

        // Make sure IFLAG1 register actually clears before moving forward
        const IFLAG1_TIMEOUT: u64 = 10; // Timeout for IFLAG1 readback, in ms.
        const IFLAG1_CLEARED: u32 = 0x0000_0000; // IFLAG1 register with all bits cleared
        let deadline = Instant::now() + Duration::from_millis(IFLAG1_TIMEOUT);
        while info.regs.iflag1().read().0 != IFLAG1_CLEARED {
            if Instant::now() >= deadline {
                return Err(MailboxError::Timeout);
            }
        }

        // Initialize tx_available so all bits are set, indicating that all 32 message buffers are available for use.
        const TX_AVAILABLE_INIT: u32 = 0xFFFF_FFFF;
        info.tx_available.store(TX_AVAILABLE_INIT, Ordering::SeqCst);

        // Initialize tx_remote so all bits are cleared, indicating that no message buffers were last used to send REMOTE frames (which is true because we just initialized and haven't sent ~any~ messages in this session yet).
        const TX_REMOTE_INIT: u32 = 0x0000_0000;
        info.tx_remote.store(TX_REMOTE_INIT, Ordering::SeqCst);

        // Set all 32 TX message buffers to INACTIVE
        for i in 0..32 {
            buffer::set_inactive(info, i);
        }

        // Re-enable interrupts. Set all 32 IMASK1 bits, since we want all 32 message buffers to have an interrupt in IFLAG1
        const IMASK1_INIT: u32 = 0xFFFF_FFFF;
        info.regs.imask1().write(|w| w.0 = IMASK1_INIT);

        Ok(())
    }

    /// Represents the possible values of the `CODE` field inside a TX message.
    /// See pages 1546 - 1548 of the datasheet.
    #[repr(u8)]
    pub(in crate::flexcan) enum TxCode {
        /// TX: INACTIVE - Message buffer is not active.
        TxInactive = 0b1000,

        /// TX: ABORT - Message buffer is aborted.
        TxAbort = 0b1001,

        /// TX: DATA - Message buffer is a TX data frame (either normal or RTR) ready to be transmitted.
        TxReady = 0b1100,

        /// TX: TANSWER - Message buffer is a Transmit Response frame from an incoming Remote Request frame.
        TxTanswer = 0b1110,
    }

    impl TxCode {
        pub(in crate::flexcan) const INACTIVE: u8 = Self::TxInactive as u8;
        pub(in crate::flexcan) const ABORT:    u8 = Self::TxAbort as u8;
        pub(in crate::flexcan) const READY:    u8 = Self::TxReady as u8;
        pub(in crate::flexcan) const TANSWER:  u8 = Self::TxTanswer as u8;
    }

    pub(in crate::flexcan) struct TxMessage{inner: Message}
    impl TxMessage {
        /// Gets the current reading of this message's `CODE` field.
        const fn code(&self) -> Result<TxCode, MailboxError> {
            let code: u8 = self.inner.cs.code();
            match code {
                TxCode::INACTIVE => Ok(TxCode::TxInactive),
                TxCode::ABORT => Ok(TxCode::TxAbort),
                TxCode::READY => Ok(TxCode::TxReady),
                TxCode::TANSWER => Ok(TxCode::TxTanswer),
                _ => Err(MailboxError::UnknownTxCode)
            }
        }

        /// Sets this message's `CODE` field.
        const fn set_code(&mut self, code: TxCode) {
            match code {
                TxCode::TxInactive => self.inner.cs.set_code(TxCode::INACTIVE),
                TxCode::TxAbort =>    self.inner.cs.set_code(TxCode::ABORT),
                TxCode::TxReady =>    self.inner.cs.set_code(TxCode::READY),
                TxCode::TxTanswer =>  self.inner.cs.set_code(TxCode::TANSWER),
            }
        }
    }

    // Converts a generic `Frame` into a hardware-specific `TxMessage`.
    // Lets you do `let frame: TxMessage = frame.into()` (where `frame` starts as a `Frame`)
    impl From<Frame> for TxMessage {
        fn from(frame: Frame) -> Self {
            use embedded_can::Frame;

            let mut message = TxMessage { inner: Message { cs: pac::Cs(0), id: pac::Id(0), payload: frame.data } };

            message.inner.cs.set_edl(false);
            message.inner.cs.set_rtr(frame.is_remote_frame());
            message.inner.cs.set_dlc(frame.dlc() as u8);

            match frame.id() {
                Id::Standard(sid)  => { 
                    message.inner.cs.set_ide(false);
                    message.inner.id.set_std(sid.as_raw());
                }

                Id::Extended(eid)  => { 
                    message.inner.cs.set_ide(true);
                    message.inner.cs.set_srr(true);
                    message.inner.id.set_std(eid.standard_id().as_raw());
                    message.inner.id.set_ext(eid.as_raw());
                }
            }

            message.set_code(TxCode::TxReady);

            message
        }
    }

    /// Finds an available space in the message buffer, 
    pub(in crate::flexcan) fn dispatch(info: &Info, message: &TxMessage) -> nb::Result<(), Infallible> {
        // This loop exists to prevent races to claim a buffer if multiple
        // senders call dispatch() at the same time. In practice though,
        // this loop will never run more than once unless there's multiple
        // executors being used, since dispatch() isn't async
        loop {
            let available = info.tx_available.load(Ordering::Acquire);
            if available == 0 {
                return Err(nb::Error::WouldBlock); // No buffers free.
            }
            let n = available.trailing_zeros();
            let mask = 1u32 << n;

            // Try to claim the buffer by clearning it's bit
            // fetch_and returns the previous value, so if our bit was still set, we won
            if info.tx_available.fetch_and(!mask, Ordering::AcqRel) & mask != 0 {
                // If this is a REMOTE (RTR = 1) frame, flag the buffer so the ISR knows to
                // force it back to INACTIVE after transmission (the hardware otherwise
                // auto-flips it to RX-EMPTY). Must be recorded before the write below, since
                // that write triggers the transmission whose completion fires the ISR.
                if message.inner.cs.rtr() {
                    info.tx_remote.fetch_or(mask, Ordering::Release);
                }
                buffer::write(info, message, n as usize);
                return Ok(());
            }
            // Another sender claimed the buffer first, so loop and try a different buffer.
        }
    }

    
}