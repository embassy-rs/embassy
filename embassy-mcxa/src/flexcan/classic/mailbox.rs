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
pub(in crate::flexcan) enum MailboxError {
    /// During a mailbox operation, hardware failed to respond within a reasonable timeframe.
    Timeout,

    /// When trying to read the `CODE` field of a TX message, no known `TxCode` variant matched.
    UnknownTxCode,
}

/// The TX subsystem (for transmitting messages).
pub(in crate::flexcan) mod tx {
    use super::Message;
    use super::pac;
    use crate::flexcan::classic::Info;
    use crate::flexcan::classic::frame::{Frame, Id};
    use super::MailboxError;
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
            info.control.regs().word0(n).write(|w| { *w = word0 });
            info.control.regs().word1(n).write(|w| { *w = word1 });

            info.control.regs().id(n).write(|w| { w.0 = message.inner.id.0 });
            info.control.regs().cs(n).write(|w| { w.0 = message.inner.cs.0 }); // Need to write in CS last because this is when we update CODE (which could trigger a TX dispatch)
        }

        /// Reads one of the 32 message buffers into a `TxMessage`.
        /// * `info` - The type-erased instance handle.
        /// * `n` - The message buffer element to read (0 through 31).
        pub fn read(info: &Info, n: usize) -> TxMessage {
            let cs = info.control.regs().cs(n).read();
            let id = info.control.regs().id(n).read();

            // Read out the payload
            let word0 = info.control.regs().word0(n).read();
            let word1 = info.control.regs().word1(n).read();
            let [b0, b1, b2, b3] = word0.to_be_bytes();
            let [b4, b5, b6, b7] = word1.to_be_bytes();
            let payload = [b0, b1, b2, b3, b4, b5, b6, b7];

            TxMessage { inner: Message { cs, id, payload } }
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
            Err(_) => { return Err(MailboxError::Timeout); }
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
        pub(in crate::flexcan) const ABORT:    u8 = Self::TxAbort as u8;
        pub(in crate::flexcan) const READY:    u8 = Self::TxReady as u8;
        pub(in crate::flexcan) const TANSWER:  u8 = Self::TxTanswer as u8;
    }

    /// Struct representing a TX message.
    /// 
    /// This is just a thin wrapper around the `Message` type that is generic to both TX and RX messages.
    /// The reason we need explicit `TxMessage` and `RxMessage` structs is because the CODE field
    /// inside `Message` has different meanings depending on TX or RX.
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

/// The RX subsystem (for recieving messages)
pub(in crate::flexcan) mod rx {
    use super::MailboxError;
    use crate::flexcan::classic::Info;
    use crate::flexcan::filter::FilterConfig;
    use super::pac;
    use super::Message;

    /// Represents the Enhanced RX FIFO memory area.
    pub(in crate::flexcan) mod fifo {
        use super::Info;
        use super::pac;
        use super::RxMessage;
        use super::Message;
        use super::MailboxError;

        /// Gets the oldest unread message from the Enhanced RX FIFO and places it into a `RxMessage`.
        /// If a message is available to return, this function will return it and automatically flag
        /// the FIFO to pop to the next message.
        /// 
        /// If the FIFO is empty, returns `None`.
        pub(in crate::flexcan) fn get(info: &Info) -> Option<RxMessage> {
            /// Converts a length/index in bytes to a length/index in words.
            const fn to_words(bytes: usize) -> usize { bytes.div_ceil(4) }

            // If ERFDA is `0`, then there's no data to read (FIFO is empty).
            if !info.control.regs().erfsr().read().erfda() {
                return None;
            }

            let cs = info.control.pac_fifocs().read();
            let id = info.control.pac_fifoid().read();

            // Get the length and clamp it to 8 bytes
            let len = (cs.dlc() as usize).min(8);

            /// The maximum number of words we may need to read in this function.
            /// This is defined as the number of words for MAX_PAYLOAD, plus one extra word for ID_HIT.
            /// For Classic CAN, where MAX_PAYLOAD = 8 bytes, MAX_WORDS will equal 3 (first two for payload data, and the last one for ID_HIT)
            const MAX_WORDS: usize = const {to_words(8) + 1}; // 3
            
            // Read the FIFO words
            let mut words = [0u32; MAX_WORDS];
            let last_word_index = to_words(len); // The index of the last word for this specific `len`. The word at this index will contain ID_HIT.
            for i in 0..=last_word_index {
                words[i] = info.control.pac_fifodata(i).read();
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

            Some(RxMessage { inner: Message { cs, id, payload } })
        }
    }

    /// Struct representing a RX message.
    /// 
    /// This is just a thin wrapper around the `Message` type that is generic to both TX and RX messages.
    /// The reason we need explicit `TxMessage` and `RxMessage` structs is because the CODE field
    /// inside `Message` has different meanings depending on TX or RX.
    pub(in crate::flexcan) struct RxMessage{inner: Message}

    /// Sets up the RX subsystem.
    /// This function requires `filter_config` to have been validated prior to being passed in here.
    pub(in crate::flexcan) fn setup(info: &Info, filter_config: &FilterConfig) -> Result<(), MailboxError> {
        use embassy_time::{Duration};

        // Make sure we're frozen before continuing.
        const FREEZE_TIMEOUT: u64 = 10; // ms
        match info.control.freeze(Some(Duration::from_millis(FREEZE_TIMEOUT))) {
            Ok(_) => (),
            Err(_) => { return Err(MailboxError::Timeout); }
        }

        // Misc basic setup
        info.control.regs().mcr().modify(|m| m.set_rfen(pac::Rfen::Id1)); // Turn off the Legacy FIFO
        info.control.regs().ctrl2().modify(|m| m.set_rrs(pac::Rrs::RemoteResponseFrameNotGenerated)); // Store incoming REMOTE frames instead of auto-generating a response frame.
        info.control.regs().erfcr().modify(|m| m.set_erfen(true)); // Enable the Enhanced FIFO
        info.control.regs().erfsr().modify(|m| m.set_erfclr(pac::Erfclr::Clear)); // Clear the Enhanced FIFO
        info.control.regs().erfsr().write(|w| {
            w.set_erfufw(true);                      // Clear the Enhanced FIFO Underflow Flag (write-1-to-clear)
            w.set_erfovf(true);                      // Clear the Enhanced FIFO Overflow Flag (write-1-to-clear)
            w.set_erfwmi(pac::Erfwmi::WatermarkYes); // Clear the Enhanced FIFO Watermark Indication Flag (write-1-to-clear)
            w.set_erfda(true);                       // Clear the Enhanced FIFO Data Available Flag (write-1-to-clear)
        });

        // Make it so every RX triggers an interrupt rather than batching them
        info.control.regs().erfier().modify(|m| {
            m.set_erfdaie(true);    // Enable the Enhanced FIFO Data Available Interrupt
            m.set_erfwmiie(false);  // Disable the Enhanced RX FIFO Watermark Indication Interrupt (since we're not batching)
            m.set_erfovfie(true);   // Enable the Enhanced RX FIFO Overflow Interrupt Enable
        });

        // Set up the ID filteres
        let num_extended = filter_config.extended_ids.len();
        let num_standard = filter_config.standard_ids.len();
        let standard_slots = num_standard.next_multiple_of(2); // Round up to a pair of two since hardware needs it

        let nexif = num_extended; // NEXIF means the number of extended filter elements (see datasheet page 1538)
        let nfe = nexif + standard_slots / 2 - 1; // Also see datasheet page 1538
        info.control.regs().erfcr().modify(|m| {
            m.set_nexif(nexif as u8);
            m.set_nfe(nfe as u8);
        });

        // Extended filters are registers 0 through 2*num_extended, with each being 2 words.
        // Filter Word: extended ID in bits 0 through 28, then 0b000 for the last three
        // Mask Word: RTR Mask = 0, extended ID mask = 0x1FFF_FFFF (so IDs need to be an exact match, and RTR response messages are treated like normal RX messages)
        for(i, &id) in filter_config.extended_ids.iter().enumerate() {
            const MASK: u32 = 0x1FFF_FFFF;
            info.control.regs().erffel(2*i).write_value(pac::Erffel(id & MASK));
            info.control.regs().erffel(2*i + 1).write_value(pac::Erffel(MASK));
        }

        // Standard filters are after the extended filters, and are one register each
        // Standard ID in bits 26::16, standard ID mask in bits 10:0.
        let standard_base = 2 * num_extended;
        for slot in 0..standard_slots {
            // Pad the odd slot by replacing the last real ID (a duplicate will never win any matches since the earlier element gets found first)
            let id = filter_config.standard_ids[if slot < num_standard { slot } else { num_standard - 1}];
            const MASK: u32 = 0x7FF;
            info.control.regs().erffel(standard_base + slot).write_value(pac::Erffel(((id & MASK) << 16) | MASK));
        }

        Ok(())
    }
}