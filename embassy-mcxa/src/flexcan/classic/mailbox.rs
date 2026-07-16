//! This module models the FlexCAN mailbox for Classic CAN (not FD).
//!
//! TX/outgoing messages are handled in the FlexCAN message buffer, which uses the memory area 80h - 27Fh (see page 1545 of the datasheet). This memory area is 512 bytes in total.
//! Each message consists of the CS Register (4 bytes), the Id Register (4 bytes), and the 8-byte message payload. So, each message buffer is 16 bytes in total.
//! This means that the message buffer can hold a total of 512 / 16 = 32 messages.
//!
//! RX/incoming messages are handeled by the chip's Enhanced RX FIFO (see page 1556 of the datasheet).
//! This FIFO can store 12 messages, which are filled automatically by the hardware as they come in.
//! Messages can be dequeued from this FIFO by reading the 2000h - 2047h memory area as a message buffer, and then setting the erfda flag (to tell the hardware that the memory area is ready to be filled with the next message from the FIFO).

use nxp_pac::can as pac;

/// The "raw" data structure of a FlexCAN message described in the datasheet.
/// For Classic CAN, this is the CS Register (4 bytes), the Id Register (4 bytes), and the 8-byte message payload.
/// This structure is used for both TX and RX messages. As mentioned, TX messages live in the FlexCAN message buffer, while RX messages live in the Enhanced RX FIFO.
struct Message {
    pub cs: pac::Cs,
    pub id: pac::Id,
    pub payload: [u8; 8],
}

/// Possible errors that may occur during mailbox operations.
#[allow(dead_code)]
pub(in crate::flexcan) enum MailboxError {
    /// During a mailbox operation, hardware failed to respond within a reasonable timeframe.
    Timeout,

    /// When trying to read the `CODE` field of a TX message, no known `TxCode` variant matched.
    UnknownTxCode,
}

/// The TX subsystem (for transmitting messages).
pub(in crate::flexcan) mod tx {
    use core::convert::Infallible;
    use core::sync::atomic::Ordering;

    use super::{MailboxError, Message, pac};
    use crate::flexcan::classic::Info;
    use crate::flexcan::classic::frame::{Frame, Id};

    /// Represents the message buffer memory area (80h - 27Fh), which this HAL uses for dispatching TX messages.
    pub(in crate::flexcan) mod buffer {
        use super::{Info, Message, TxCode, TxMessage};

        /// Writes a `TxMessage` into one of the 32 message buffers.
        /// * `info` - The type-erased instance handle.
        /// * `message` - The TxMessage to write.
        /// * `n` - The message buffer element to write (0 through 31).
        pub fn write(info: &Info, message: &TxMessage, n: usize) {
            // Write in the payload
            let [b0, b1, b2, b3, b4, b5, b6, b7] = message.inner.payload;
            let word0 = u32::from_be_bytes([b0, b1, b2, b3]);
            let word1 = u32::from_be_bytes([b4, b5, b6, b7]);
            info.control.regs().word0(n).write(|w| *w = word0);
            info.control.regs().word1(n).write(|w| *w = word1);

            info.control.regs().id(n).write(|w| w.0 = message.inner.id.0);
            info.control.regs().cs(n).write(|w| w.0 = message.inner.cs.0); // Need to write in CS last because this is when we update CODE (which could trigger a TX dispatch)
        }

        /// Reads one of the 32 message buffers into a `TxMessage`.
        /// * `info` - The type-erased instance handle.
        /// * `n` - The message buffer element to read (0 through 31).
        #[allow(dead_code)]
        pub fn read(info: &Info, n: usize) -> TxMessage {
            let cs = info.control.regs().cs(n).read();
            let id = info.control.regs().id(n).read();

            // Read out the payload
            let word0 = info.control.regs().word0(n).read();
            let word1 = info.control.regs().word1(n).read();
            let [b0, b1, b2, b3] = word0.to_be_bytes();
            let [b4, b5, b6, b7] = word1.to_be_bytes();
            let payload = [b0, b1, b2, b3, b4, b5, b6, b7];

            TxMessage {
                inner: Message { cs, id, payload },
            }
        }

        /// Sets a buffer to its `INACTIVE` state. Only the CS register is affected.
        /// * `info` - The type-erased instance handle.
        /// * `n` - The buffer to reset (0 through 31).
        pub fn set_inactive(info: &Info, n: usize) {
            info.control.regs().cs(n).write(|w| w.set_code(TxCode::INACTIVE));
        }
    }

    /// Sets up the TX subsystem.
    /// This function resets the TX message buffers and our state tracking for what buffers are available.
    pub(in crate::flexcan) fn setup(info: &Info) -> Result<(), MailboxError> {
        use core::sync::atomic::Ordering;

        use embassy_time::{Duration, Instant};

        // Make sure we're frozen before continuing.
        const FREEZE_TIMEOUT: u64 = 10; // ms
        match info.control.freeze(Some(Duration::from_millis(FREEZE_TIMEOUT))) {
            Ok(_) => (),
            Err(_) => {
                return Err(MailboxError::Timeout);
            }
        }

        // Disable all 32 interrupts via the IMASK1 register
        const IMASK1_DISABLED: u32 = 0x0000_0000;
        info.control.regs().imask1().write(|w| w.0 = IMASK1_DISABLED);

        // Clear all IFLAG1 bits (this register is "write 1 to clear", so writing all 1s will clear the whole register)
        const IFLAG1_INIT: u32 = 0xFFFF_FFFF;
        info.control.regs().iflag1().write(|w| w.0 = IFLAG1_INIT);

        // Make sure IFLAG1 register actually clears before moving forward
        const IFLAG1_TIMEOUT: u64 = 10; // Timeout for IFLAG1 readback, in ms.
        const IFLAG1_CLEARED: u32 = 0x0000_0000; // IFLAG1 register with all bits cleared
        let deadline = Instant::now() + Duration::from_millis(IFLAG1_TIMEOUT);
        while info.control.regs().iflag1().read().0 != IFLAG1_CLEARED {
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
        info.control.regs().imask1().write(|w| w.0 = IMASK1_INIT);

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
        pub(in crate::flexcan) const ABORT: u8 = Self::TxAbort as u8;
        pub(in crate::flexcan) const READY: u8 = Self::TxReady as u8;
        pub(in crate::flexcan) const TANSWER: u8 = Self::TxTanswer as u8;
    }

    /// Struct representing a TX message.
    ///
    /// This is just a thin wrapper around the `Message` type that is generic to both TX and RX messages.
    /// The reason we need explicit `TxMessage` and `RxMessage` structs is because the CODE field
    /// inside `Message` has different meanings depending on TX or RX.
    pub(in crate::flexcan) struct TxMessage {
        inner: Message,
    }
    impl TxMessage {
        /// Gets the current reading of this message's `CODE` field.
        #[allow(dead_code)]
        const fn code(&self) -> Result<TxCode, MailboxError> {
            let code: u8 = self.inner.cs.code();
            match code {
                TxCode::INACTIVE => Ok(TxCode::TxInactive),
                TxCode::ABORT => Ok(TxCode::TxAbort),
                TxCode::READY => Ok(TxCode::TxReady),
                TxCode::TANSWER => Ok(TxCode::TxTanswer),
                _ => Err(MailboxError::UnknownTxCode),
            }
        }

        /// Sets this message's `CODE` field.
        const fn set_code(&mut self, code: TxCode) {
            match code {
                TxCode::TxInactive => self.inner.cs.set_code(TxCode::INACTIVE),
                TxCode::TxAbort => self.inner.cs.set_code(TxCode::ABORT),
                TxCode::TxReady => self.inner.cs.set_code(TxCode::READY),
                TxCode::TxTanswer => self.inner.cs.set_code(TxCode::TANSWER),
            }
        }
    }

    // Converts a generic `Frame` into a hardware-specific `TxMessage`.
    // Lets you do `let frame: TxMessage = (&frame).into()` (where `frame` starts as a `Frame`)
    impl From<&Frame> for TxMessage {
        fn from(frame: &Frame) -> Self {
            let mut message = TxMessage {
                inner: Message {
                    cs: pac::Cs(0),
                    id: pac::Id(0),
                    payload: frame.data,
                },
            };

            message.inner.cs.set_edl(false);
            message.inner.cs.set_rtr(frame.is_remote_frame());
            message.inner.cs.set_dlc(frame.dlc() as u8);

            match frame.id() {
                Id::Standard(sid) => {
                    message.inner.cs.set_ide(false);
                    message.inner.id.set_std(sid.as_raw());
                }

                Id::Extended(eid) => {
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

    /// Finds an available space in the message buffer, and then puts the message in there.
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

    /// Reclaims any TX message buffers that have finished transmitting, making them available for reuse.
    pub(in crate::flexcan) fn reclaim_completed(info: &Info) -> bool {
        let can = info.control.regs();

        // Check what TX buffers have fired.
        let tx_flags = can.iflag1().read().0;
        let tx_enabled = can.imask1().read().0;
        let tx_fired = tx_flags & tx_enabled; // Any TX buffers that have just fired and need to be reset are marked as `1` here.

        if tx_fired == 0 {
            return false;
        }

        // For more context about this following block, see the comment above `tx_remote`. TLDR: This block of code
        // is only relavent when we transmit REMOTE frames.
        let remote_fired = tx_fired & info.tx_remote.load(Ordering::Relaxed);
        if remote_fired != 0 {
            let mut bits = remote_fired;
            while bits != 0 {
                let n = bits.trailing_zeros() as usize;
                buffer::set_inactive(info, n); // INACTIVE
                bits &= bits - 1; // Clear the lowest set bit.
            }
            // Clear the remote markings before the buffers are advertised as available, so
            // that `dispatch()` never observes a free buffer that is still flagged remote.
            info.tx_remote.fetch_and(!remote_fired, Ordering::Relaxed);
        }

        // Actually clear the interrupt flag
        can.iflag1().write(|w| w.0 = tx_fired); // IFLAG1 is a "write 1 to clear" register. So, doing this basically just acknowledges that these interrupts fired, and clears them back to zero (so they can fire again in the future).
        let _ = can.iflag1().read(); // read back from the register so we make sure the write finished before we return
        info.tx_available.fetch_or(tx_fired, Ordering::Release); // Update the `tx_available` tracker accordingly.

        true
    }
}

/// The RX subsystem (for recieving messages)
pub(in crate::flexcan) mod rx {
    use super::{MailboxError, Message, pac};
    use crate::flexcan::classic::Info;
    use crate::flexcan::filter::{Filter, FilterConfig};

    /// Represents the Enhanced RX FIFO memory area.
    pub(in crate::flexcan) mod fifo {
        use super::{Info, Message, RxMessage};

        /// Gets the oldest unread message from the Enhanced RX FIFO and places it into a `RxMessage`.
        /// If a message is available to return, this function will return it and automatically flag
        /// the FIFO to pop to the next message.
        ///
        /// If the FIFO is empty, returns `None`.
        pub(in crate::flexcan) fn get(info: &Info) -> Option<RxMessage> {
            /// Converts a length/index in bytes to a length/index in words.
            const fn to_words(bytes: usize) -> usize {
                bytes.div_ceil(4)
            }

            // If ERFDA is `0`, then there's no data to read (FIFO is empty).
            if !info.control.regs().erfsr().read().erfda() {
                return None;
            }

            let cs = info.control.regs().erfifo_cs().read();
            let id = info.control.regs().erfifo_id().read();
            let len = (cs.dlc() as usize).min(8);

            /// This const is the maximum number of words we may need to read in this function.
            /// This is defined as the number of words for MAX_PAYLOAD, plus one extra word for ID_HIT.
            /// For Classic CAN, where MAX_PAYLOAD = 8 bytes, MAX_WORDS will equal 3 (first two for payload data, and the last one for ID_HIT)
            const MAX_WORDS: usize = const { to_words(8) + 1 }; // 3

            // Read the FIFO words
            let mut words = [0u32; MAX_WORDS];
            let last_word_index = to_words(len); // The index of the last word for this specific `len`. The word at this index will contain ID_HIT.
            for i in 0..=last_word_index {
                words[i] = info.control.regs().erfifo_data(i).read();
            }

            // At this point we've read all the FIFO data we need, so flag the FIFO to start re-filling.
            // This way, it can get started in the background while we finish up this function call
            info.control.regs().erfsr().write(|w| w.set_erfda(true)); // write 1 to pop the FIFO and get it to load the next frame

            // Get ID_HIT (always one word after the last payload word)
            const SEVEN_BITS: u32 = 0b00000000_00000000_00000000_01111111; // `id_hit` lives in the first 7 bits of the word
            let _id_hit = words[last_word_index] & SEVEN_BITS; // not actually using id_hit for anything yet, but this logic is here in case someone wants to use it for something later

            // Build payload
            let mut payload = [0u8; 8];
            let mut copied = 0;
            while copied < len {
                let n = (len - copied).min(4);
                payload[copied..copied + n].copy_from_slice(&words[copied / 4].to_be_bytes()[..n]);
                copied += n;
            }

            Some(RxMessage {
                inner: Message { cs, id, payload },
            })
        }
    }

    /// Struct representing a RX message.
    ///
    /// This is just a thin wrapper around the `Message` type that is generic to both TX and RX messages.
    /// The reason we need explicit `TxMessage` and `RxMessage` structs is because the CODE field
    /// inside `Message` has different meanings depending on TX or RX.
    pub(in crate::flexcan) struct RxMessage {
        inner: Message,
    }

    /// Possible errors that can occur when trying to convert
    /// a `RxMessage` to a `Frame`.
    pub(in crate::flexcan) enum RxMessageFromError {
        /// Indicates that the std() field inside the `RxMessage`
        /// contained a value larger than 11 bits (`> 0x7FF`).
        StandardIdTooLong,

        /// Indicates that the ext() field inside the `RxMessage`
        /// contained a value larger than 29 bits (`> 0x1FFF_FFFF`)
        ExtendedIdTooLong,
    }

    // Converts a hardware-specific `RxMessage` into a generic `Frame`.
    // Lets you do `let frame: Frame = frame.into()` (where `frame` starts as a `RxMessage`)
    impl TryFrom<RxMessage> for crate::flexcan::classic::frame::Frame {
        type Error = RxMessageFromError;

        fn try_from(message: RxMessage) -> Result<Self, Self::Error> {
            use crate::flexcan::classic::frame::FrameKind;
            use crate::flexcan::id::{ExtendedId, Id, StandardId};

            let kind: FrameKind = if message.inner.cs.rtr() {
                FrameKind::RemoteFrame
            } else {
                FrameKind::DataFrame
            };

            let id: Id = if message.inner.cs.ide() {
                // ide = true means Extended ID
                let std: u16 = message.inner.id.std(); // the upper 11 bits (18 through 29)
                let ext: u32 = message.inner.id.ext(); // the lower 18 bits (0 through 17)
                let full: u32 = ext | ((std as u32) << 18);
                Id::Extended(ExtendedId::new(full).ok_or(RxMessageFromError::ExtendedIdTooLong)?)
            } else {
                // ide = false means Standard ID
                Id::Standard(StandardId::new(message.inner.id.std()).ok_or(RxMessageFromError::StandardIdTooLong)?)
            };

            let length: usize = (message.inner.cs.dlc() as usize).min(8);

            Ok(crate::flexcan::classic::frame::Frame {
                kind,
                id,
                length,
                data: message.inner.payload,
            })
        }
    }

    /// Sets up the RX subsystem.
    /// This function requires `filter_config` to have been validated prior to being passed in here.
    pub(in crate::flexcan) fn setup(info: &Info, filter_config: &FilterConfig) -> Result<(), MailboxError> {
        use embassy_time::Duration;

        use crate::flexcan::id::{ExtendedId, StandardId};

        // Make sure we're frozen before continuing.
        const FREEZE_TIMEOUT: u64 = 10; // ms
        match info.control.freeze(Some(Duration::from_millis(FREEZE_TIMEOUT))) {
            Ok(_) => (),
            Err(_) => {
                return Err(MailboxError::Timeout);
            }
        }

        // Misc basic setup
        info.control.regs().mcr().modify(|m| m.set_rfen(pac::Rfen::Id1)); // Turn off the Legacy FIFO
        info.control
            .regs()
            .ctrl2()
            .modify(|m| m.set_rrs(pac::Rrs::RemoteResponseFrameNotGenerated)); // Store incoming REMOTE frames instead of auto-generating a response frame.
        info.control.regs().erfcr().modify(|m| m.set_erfen(true)); // Enable the Enhanced FIFO
        info.control.regs().erfsr().modify(|m| m.set_erfclr(pac::Erfclr::Clear)); // Clear the Enhanced FIFO
        info.control.regs().erfsr().write(|w| {
            w.set_erfufw(true); // Clear the Enhanced FIFO Underflow Flag (write-1-to-clear)
            w.set_erfovf(true); // Clear the Enhanced FIFO Overflow Flag (write-1-to-clear)
            w.set_erfwmi(pac::Erfwmi::WatermarkYes); // Clear the Enhanced FIFO Watermark Indication Flag (write-1-to-clear)
            w.set_erfda(true); // Clear the Enhanced FIFO Data Available Flag (write-1-to-clear)
        });

        // Make it so every RX triggers an interrupt rather than batching them
        info.control.regs().erfier().modify(|m| {
            m.set_erfdaie(true); // Enable the Enhanced FIFO Data Available Interrupt
            m.set_erfwmiie(false); // Disable the Enhanced RX FIFO Watermark Indication Interrupt (since we're not batching)
            m.set_erfovfie(true); // Enable the Enhanced RX FIFO Overflow Interrupt Enable
        });

        // Set up the ID filteres
        let standard_slots = filter_config.num_standard.next_multiple_of(2); // Round up to a pair of two since hardware needs it
        let nexif = filter_config.num_extended; // NEXIF means the number of extended filter elements (see datasheet page 1538)
        let nfe = nexif + standard_slots / 2 - 1; // Also see datasheet page 1538
        info.control.regs().erfcr().modify(|m| {
            m.set_nexif(nexif as u8);
            m.set_nfe(nfe as u8);
        });

        // Walk the declarative FilterConfig and put each filter in the correct area
        const EXT_ID_MASK: u32 = 0x1FFF_FFFF;
        const STD_ID_MASK: u32 = 0x7FF;
        let standard_base = 2 * filter_config.num_extended;

        let mut ext_idx = 0; // next free extended element index
        let mut std_idx = 0; // next free standard slot (relative to standard_base)
        let mut last_standard_word = 0u32; // remembered so we can pad an odd standard slot

        for filter in filter_config.filters {
            match filter {
                Filter::Extended(id) => {
                    info.control
                        .regs()
                        .erffel(2 * ext_idx)
                        .write_value(pac::Erffel(id.as_raw() & EXT_ID_MASK));
                    info.control
                        .regs()
                        .erffel(2 * ext_idx + 1)
                        .write_value(pac::Erffel(EXT_ID_MASK));
                    ext_idx += 1;
                }
                Filter::ExtendedMasked { id, mask } => {
                    info.control
                        .regs()
                        .erffel(2 * ext_idx)
                        .write_value(pac::Erffel(id.as_raw() & EXT_ID_MASK));
                    info.control
                        .regs()
                        .erffel(2 * ext_idx + 1)
                        .write_value(pac::Erffel(mask.as_raw() & EXT_ID_MASK));
                    ext_idx += 1;
                }
                Filter::Standard(id) => {
                    let word = ((id.as_raw() as u32 & STD_ID_MASK) << 16) | STD_ID_MASK;
                    info.control
                        .regs()
                        .erffel(standard_base + std_idx)
                        .write_value(pac::Erffel(word));
                    last_standard_word = word;
                    std_idx += 1;
                }
                Filter::StandardMasked { id, mask } => {
                    let word = ((id.as_raw() as u32 & STD_ID_MASK) << 16) | (mask.as_raw() as u32 & STD_ID_MASK);
                    info.control
                        .regs()
                        .erffel(standard_base + std_idx)
                        .write_value(pac::Erffel(word));
                    last_standard_word = word;
                    std_idx += 1;
                }
                Filter::AcceptAllStandard => {
                    // Just do the same thing as `StandardMasked` but where the mask and id
                    // are all zeros (since that's what "accept all Standard IDs" means under the hood)
                    let mask = StandardId::ZERO;
                    let id = StandardId::ZERO;

                    let word = ((id.as_raw() as u32 & STD_ID_MASK) << 16) | (mask.as_raw() as u32 & STD_ID_MASK);
                    info.control
                        .regs()
                        .erffel(standard_base + std_idx)
                        .write_value(pac::Erffel(word));
                    last_standard_word = word;
                    std_idx += 1;
                }
                Filter::AcceptAllExtended => {
                    // Just do the same thing as `ExtendedMasked` but where the mask and id
                    // are all zeros (since that's what "accept all Extended IDs" means under the hood)
                    let mask = ExtendedId::ZERO;
                    let id = ExtendedId::ZERO;

                    info.control
                        .regs()
                        .erffel(2 * ext_idx)
                        .write_value(pac::Erffel(id.as_raw() & EXT_ID_MASK));
                    info.control
                        .regs()
                        .erffel(2 * ext_idx + 1)
                        .write_value(pac::Erffel(mask.as_raw() & EXT_ID_MASK));
                    ext_idx += 1;
                }
            }
        }

        // If we have an odd number of standard filters, pad the trailing slot by repeating the
        // last standard word (a duplicate will never win any matches since the earlier element gets found first).
        if std_idx < standard_slots {
            info.control
                .regs()
                .erffel(standard_base + std_idx)
                .write_value(pac::Erffel(last_standard_word));
        }

        Ok(())
    }
}
