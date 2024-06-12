use core::cmp::Ordering;
use core::convert::Infallible;

pub use embedded_can::{ExtendedId, Id, StandardId};
use stm32_metapac::can::vals::Lec;

use super::{Mailbox, TransmitStatus};
use crate::can::enums::BusError;
use crate::can::frame::{Envelope, Frame, Header};

pub(crate) struct Registers(pub crate::pac::can::Can);

impl Registers {
    pub fn enter_init_mode(&self) {
        self.0.mcr().modify(|reg| {
            reg.set_sleep(false);
            reg.set_inrq(true);
        });
        loop {
            let msr = self.0.msr().read();
            if !msr.slak() && msr.inak() {
                break;
            }
        }
    }

    // Leaves initialization mode, enters sleep mode.
    pub fn leave_init_mode(&self) {
        self.0.mcr().modify(|reg| {
            reg.set_sleep(true);
            reg.set_inrq(false);
        });
        loop {
            let msr = self.0.msr().read();
            if msr.slak() && !msr.inak() {
                break;
            }
        }
    }

    pub fn set_bit_timing(&self, bt: crate::can::util::NominalBitTiming) {
        let prescaler = u16::from(bt.prescaler) & 0x1FF;
        let seg1 = u8::from(bt.seg1);
        let seg2 = u8::from(bt.seg2) & 0x7F;
        let sync_jump_width = u8::from(bt.sync_jump_width) & 0x7F;
        self.0.btr().modify(|reg| {
            reg.set_brp(prescaler - 1);
            reg.set_ts(0, seg1 - 1);
            reg.set_ts(1, seg2 - 1);
            reg.set_sjw(sync_jump_width - 1);
        });
    }

    /// Enables or disables silent mode: Disconnects the TX signal from the pin.
    pub fn set_silent(&self, enabled: bool) {
        let mode = match enabled {
            false => stm32_metapac::can::vals::Silm::NORMAL,
            true => stm32_metapac::can::vals::Silm::SILENT,
        };
        self.0.btr().modify(|reg| reg.set_silm(mode));
    }

    /// Enables or disables automatic retransmission of messages.
    ///
    /// If this is enabled, the CAN peripheral will automatically try to retransmit each frame
    /// until it can be sent. Otherwise, it will try only once to send each frame.
    ///
    /// Automatic retransmission is enabled by default.
    pub fn set_automatic_retransmit(&self, enabled: bool) {
        self.0.mcr().modify(|reg| reg.set_nart(enabled));
    }

    /// Enables or disables loopback mode: Internally connects the TX and RX
    /// signals together.
    pub fn set_loopback(&self, enabled: bool) {
        self.0.btr().modify(|reg| reg.set_lbkm(enabled));
    }

    /// Configures the automatic wake-up feature.
    ///
    /// This is turned off by default.
    ///
    /// When turned on, an incoming frame will cause the peripheral to wake up from sleep and
    /// receive the frame. If enabled, [`Interrupt::Wakeup`] will also be triggered by the incoming
    /// frame.
    #[allow(dead_code)]
    pub fn set_automatic_wakeup(&self, enabled: bool) {
        self.0.mcr().modify(|reg| reg.set_awum(enabled));
    }

    /// Leaves initialization mode and enables the peripheral (non-blocking version).
    ///
    /// Usually, it is recommended to call [`CanConfig::enable`] instead. This method is only needed
    /// if you want non-blocking initialization.
    ///
    /// If this returns [`WouldBlock`][nb::Error::WouldBlock], the peripheral will enable itself
    /// in the background. The peripheral is enabled and ready to use when this method returns
    /// successfully.
    pub fn enable_non_blocking(&self) -> nb::Result<(), Infallible> {
        let msr = self.0.msr().read();
        if msr.slak() {
            self.0.mcr().modify(|reg| {
                reg.set_abom(true);
                reg.set_sleep(false);
            });
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }

    /// Puts the peripheral in a sleep mode to save power.
    ///
    /// While in sleep mode, an incoming CAN frame will trigger [`Interrupt::Wakeup`] if enabled.
    #[allow(dead_code)]
    pub fn sleep(&mut self) {
        self.0.mcr().modify(|reg| {
            reg.set_sleep(true);
            reg.set_inrq(false);
        });
        loop {
            let msr = self.0.msr().read();
            if msr.slak() && !msr.inak() {
                break;
            }
        }
    }

    /// Wakes up from sleep mode.
    ///
    /// Note that this will not trigger [`Interrupt::Wakeup`], only reception of an incoming CAN
    /// frame will cause that interrupt.
    #[allow(dead_code)]
    pub fn wakeup(&self) {
        self.0.mcr().modify(|reg| {
            reg.set_sleep(false);
            reg.set_inrq(false);
        });
        loop {
            let msr = self.0.msr().read();
            if !msr.slak() && !msr.inak() {
                break;
            }
        }
    }

    pub fn curr_error(&self) -> Option<BusError> {
        if !self.0.msr().read().erri() {
            // This ensures that once a single error instance has
            // been acknowledged and forwared to the bus message consumer
            // we don't continue to re-forward the same error occurrance for an
            // in-definite amount of time.
            return None;
        }

        // Since we have not already acknowledge the error, and the interrupt was
        // disabled in the ISR, we will acknowledge the current error and re-enable the interrupt
        // so futher errors are captured
        self.0.msr().modify(|m| m.set_erri(true));
        self.0.ier().modify(|i| i.set_errie(true));

        let err = self.0.esr().read();
        if err.boff() {
            return Some(BusError::BusOff);
        } else if err.epvf() {
            return Some(BusError::BusPassive);
        } else if err.ewgf() {
            return Some(BusError::BusWarning);
        } else if err.lec() != Lec::NOERROR {
            return Some(match err.lec() {
                Lec::STUFF => BusError::Stuff,
                Lec::FORM => BusError::Form,
                Lec::ACK => BusError::Acknowledge,
                Lec::BITRECESSIVE => BusError::BitRecessive,
                Lec::BITDOMINANT => BusError::BitDominant,
                Lec::CRC => BusError::Crc,
                Lec::CUSTOM => BusError::Software,
                Lec::NOERROR => unreachable!(),
            });
        }
        None
    }

    /// Enables or disables FIFO scheduling of outgoing mailboxes.
    ///
    /// If this is enabled, mailboxes are scheduled based on the time when the transmit request bit of the mailbox was set.
    ///
    /// If this is disabled, mailboxes are scheduled based on the priority of the frame in the mailbox.
    pub fn set_tx_fifo_scheduling(&self, enabled: bool) {
        self.0.mcr().modify(|w| w.set_txfp(enabled))
    }

    /// Checks if FIFO scheduling of outgoing mailboxes is enabled.
    pub fn tx_fifo_scheduling_enabled(&self) -> bool {
        self.0.mcr().read().txfp()
    }

    /// Puts a CAN frame in a transmit mailbox for transmission on the bus.
    ///
    /// The behavior of this function depends on wheter or not FIFO scheduling is enabled.
    /// See [`Self::set_tx_fifo_scheduling()`] and [`Self::tx_fifo_scheduling_enabled()`].
    ///
    /// # Priority based scheduling
    ///
    /// If FIFO scheduling is disabled, frames are transmitted to the bus based on their
    /// priority (see [`FramePriority`]). Transmit order is preserved for frames with identical
    /// priority.
    ///
    /// If all transmit mailboxes are full, and `frame` has a higher priority than the
    /// lowest-priority message in the transmit mailboxes, transmission of the enqueued frame is
    /// cancelled and `frame` is enqueued instead. The frame that was replaced is returned as
    /// [`TransmitStatus::dequeued_frame`].
    ///
    /// # FIFO scheduling
    ///
    /// If FIFO scheduling is enabled, frames are transmitted in the order that they are passed to this function.
    ///
    /// If all transmit mailboxes are full, this function returns [`nb::Error::WouldBlock`].
    pub fn transmit(&self, frame: &Frame) -> nb::Result<TransmitStatus, Infallible> {
        // Check if FIFO scheduling is enabled.
        let fifo_scheduling = self.0.mcr().read().txfp();

        // Get the index of the next free mailbox or the one with the lowest priority.
        let tsr = self.0.tsr().read();
        let idx = tsr.code() as usize;

        let frame_is_pending = !tsr.tme(0) || !tsr.tme(1) || !tsr.tme(2);
        let all_frames_are_pending = !tsr.tme(0) && !tsr.tme(1) && !tsr.tme(2);

        let pending_frame;
        if fifo_scheduling && all_frames_are_pending {
            // FIFO scheduling is enabled and all mailboxes are full.
            // We will not drop a lower priority frame, we just report WouldBlock.
            return Err(nb::Error::WouldBlock);
        } else if !fifo_scheduling && frame_is_pending {
            // Priority scheduling is enabled and alteast one mailbox is full.
            //
            // In this mode, the peripheral transmits high priority frames first.
            // Frames with identical priority should be transmitted in FIFO order,
            // but the controller schedules pending frames of same priority based on the
            // mailbox index. As a workaround check all pending mailboxes and only accept
            // frames with a different priority.
            self.check_priority(0, frame.id().into())?;
            self.check_priority(1, frame.id().into())?;
            self.check_priority(2, frame.id().into())?;

            if all_frames_are_pending {
                // No free mailbox is available. This can only happen when three frames with
                // ascending priority (descending IDs) were requested for transmission and all
                // of them are blocked by bus traffic with even higher priority.
                // To prevent a priority inversion abort and replace the lowest priority frame.
                pending_frame = self.read_pending_mailbox(idx);
            } else {
                // There was a free mailbox.
                pending_frame = None;
            }
        } else {
            // Either we have FIFO scheduling and at-least one free mailbox,
            // or we have priority scheduling and all mailboxes are free.
            // No further checks are needed.
            pending_frame = None
        }

        self.write_mailbox(idx, frame);

        let mailbox = match idx {
            0 => Mailbox::Mailbox0,
            1 => Mailbox::Mailbox1,
            2 => Mailbox::Mailbox2,
            _ => unreachable!(),
        };
        Ok(TransmitStatus {
            dequeued_frame: pending_frame,
            mailbox,
        })
    }

    /// Returns `Ok` when the mailbox is free or if it contains pending frame with a
    /// different priority from the identifier `id`.
    fn check_priority(&self, idx: usize, id: IdReg) -> nb::Result<(), Infallible> {
        // Read the pending frame's id to check its priority.
        assert!(idx < 3);
        let tir = &self.0.tx(idx).tir().read();

        // Check the priority by comparing the identifiers. But first make sure the
        // frame has not finished the transmission (`TXRQ` == 0) in the meantime.
        if tir.txrq() && id == IdReg::from_register(tir.0) {
            // There's a mailbox whose priority is equal to the priority of the new frame.
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }

    fn write_mailbox(&self, idx: usize, frame: &Frame) {
        debug_assert!(idx < 3);

        let mb = self.0.tx(idx);
        mb.tdtr().write(|w| w.set_dlc(frame.header().len() as u8));

        mb.tdlr()
            .write(|w| w.0 = u32::from_ne_bytes(frame.data()[0..4].try_into().unwrap()));
        mb.tdhr()
            .write(|w| w.0 = u32::from_ne_bytes(frame.data()[4..8].try_into().unwrap()));
        let id: IdReg = frame.id().into();
        mb.tir().write(|w| {
            w.0 = id.0;
            w.set_txrq(true);
        });
    }

    fn read_pending_mailbox(&self, idx: usize) -> Option<Frame> {
        if self.abort_by_index(idx) {
            debug_assert!(idx < 3);

            let mb = self.0.tx(idx);

            let id = IdReg(mb.tir().read().0);
            let mut data = [0xff; 8];
            data[0..4].copy_from_slice(&mb.tdlr().read().0.to_ne_bytes());
            data[4..8].copy_from_slice(&mb.tdhr().read().0.to_ne_bytes());
            let len = mb.tdtr().read().dlc();

            Some(Frame::new(Header::new(id.id(), len, id.rtr()), &data).unwrap())
        } else {
            // Abort request failed because the frame was already sent (or being sent) on
            // the bus. All mailboxes are now free. This can happen for small prescaler
            // values (e.g. 1MBit/s bit timing with a source clock of 8MHz) or when an ISR
            // has preempted the execution.
            None
        }
    }

    /// Tries to abort a pending frame. Returns `true` when aborted.
    fn abort_by_index(&self, idx: usize) -> bool {
        self.0.tsr().write(|reg| reg.set_abrq(idx, true));

        // Wait for the abort request to be finished.
        loop {
            let tsr = self.0.tsr().read();
            if false == tsr.abrq(idx) {
                break tsr.txok(idx) == false;
            }
        }
    }

    /// Attempts to abort the sending of a frame that is pending in a mailbox.
    ///
    /// If there is no frame in the provided mailbox, or its transmission succeeds before it can be
    /// aborted, this function has no effect and returns `false`.
    ///
    /// If there is a frame in the provided mailbox, and it is canceled successfully, this function
    /// returns `true`.
    pub fn abort(&self, mailbox: Mailbox) -> bool {
        // If the mailbox is empty, the value of TXOKx depends on what happened with the previous
        // frame in that mailbox. Only call abort_by_index() if the mailbox is not empty.
        let tsr = self.0.tsr().read();
        let mailbox_empty = match mailbox {
            Mailbox::Mailbox0 => tsr.tme(0),
            Mailbox::Mailbox1 => tsr.tme(1),
            Mailbox::Mailbox2 => tsr.tme(2),
        };
        if mailbox_empty {
            false
        } else {
            self.abort_by_index(mailbox as usize)
        }
    }

    /// Returns `true` if no frame is pending for transmission.
    pub fn is_idle(&self) -> bool {
        let tsr = self.0.tsr().read();
        tsr.tme(0) && tsr.tme(1) && tsr.tme(2)
    }

    pub fn receive_frame_available(&self) -> bool {
        if self.0.rfr(0).read().fmp() != 0 {
            true
        } else if self.0.rfr(1).read().fmp() != 0 {
            true
        } else {
            false
        }
    }

    pub fn receive_fifo(&self, fifo: RxFifo) -> Option<Envelope> {
        // Generate timestamp as early as possible
        #[cfg(feature = "time")]
        let ts = embassy_time::Instant::now();

        use crate::pac::can::vals::Ide;

        let fifo_idx = match fifo {
            RxFifo::Fifo0 => 0usize,
            RxFifo::Fifo1 => 1usize,
        };
        let rfr = self.0.rfr(fifo_idx);
        let fifo = self.0.rx(fifo_idx);

        // If there are no pending messages, there is nothing to do
        if rfr.read().fmp() == 0 {
            return None;
        }

        let rir = fifo.rir().read();
        let id: embedded_can::Id = if rir.ide() == Ide::STANDARD {
            embedded_can::StandardId::new(rir.stid()).unwrap().into()
        } else {
            let stid = (rir.stid() & 0x7FF) as u32;
            let exid = rir.exid() & 0x3FFFF;
            let id = (stid << 18) | (exid);
            embedded_can::ExtendedId::new(id).unwrap().into()
        };
        let rdtr = fifo.rdtr().read();
        let data_len = rdtr.dlc();
        let rtr = rir.rtr() == stm32_metapac::can::vals::Rtr::REMOTE;

        #[cfg(not(feature = "time"))]
        let ts = rdtr.time();

        let mut data: [u8; 8] = [0; 8];
        data[0..4].copy_from_slice(&fifo.rdlr().read().0.to_ne_bytes());
        data[4..8].copy_from_slice(&fifo.rdhr().read().0.to_ne_bytes());

        let frame = Frame::new(Header::new(id, data_len, rtr), &data).unwrap();
        let envelope = Envelope { ts, frame };

        rfr.modify(|v| v.set_rfom(true));

        Some(envelope)
    }
}

/// Identifier of a CAN message.
///
/// Can be either a standard identifier (11bit, Range: 0..0x3FF) or a
/// extendended identifier (29bit , Range: 0..0x1FFFFFFF).
///
/// The `Ord` trait can be used to determine the frame’s priority this ID
/// belongs to.
/// Lower identifier values have a higher priority. Additionally standard frames
/// have a higher priority than extended frames and data frames have a higher
/// priority than remote frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct IdReg(u32);

impl IdReg {
    const STANDARD_SHIFT: u32 = 21;

    const EXTENDED_SHIFT: u32 = 3;

    const IDE_MASK: u32 = 0x0000_0004;

    const RTR_MASK: u32 = 0x0000_0002;

    /// Creates a new standard identifier (11bit, Range: 0..0x7FF)
    ///
    /// Panics for IDs outside the allowed range.
    fn new_standard(id: StandardId) -> Self {
        Self(u32::from(id.as_raw()) << Self::STANDARD_SHIFT)
    }

    /// Creates a new extendended identifier (29bit , Range: 0..0x1FFFFFFF).
    ///
    /// Panics for IDs outside the allowed range.
    fn new_extended(id: ExtendedId) -> IdReg {
        Self(id.as_raw() << Self::EXTENDED_SHIFT | Self::IDE_MASK)
    }

    fn from_register(reg: u32) -> IdReg {
        Self(reg & 0xFFFF_FFFE)
    }

    /// Returns the identifier.
    fn to_id(self) -> Id {
        if self.is_extended() {
            Id::Extended(unsafe { ExtendedId::new_unchecked(self.0 >> Self::EXTENDED_SHIFT) })
        } else {
            Id::Standard(unsafe { StandardId::new_unchecked((self.0 >> Self::STANDARD_SHIFT) as u16) })
        }
    }

    /// Returns the identifier.
    fn id(self) -> embedded_can::Id {
        if self.is_extended() {
            embedded_can::ExtendedId::new(self.0 >> Self::EXTENDED_SHIFT)
                .unwrap()
                .into()
        } else {
            embedded_can::StandardId::new((self.0 >> Self::STANDARD_SHIFT) as u16)
                .unwrap()
                .into()
        }
    }

    /// Returns `true` if the identifier is an extended identifier.
    fn is_extended(self) -> bool {
        self.0 & Self::IDE_MASK != 0
    }

    /// Returns `true` if the identifer is part of a remote frame (RTR bit set).
    fn rtr(self) -> bool {
        self.0 & Self::RTR_MASK != 0
    }
}

impl From<&embedded_can::Id> for IdReg {
    fn from(eid: &embedded_can::Id) -> Self {
        match eid {
            embedded_can::Id::Standard(id) => IdReg::new_standard(StandardId::new(id.as_raw()).unwrap()),
            embedded_can::Id::Extended(id) => IdReg::new_extended(ExtendedId::new(id.as_raw()).unwrap()),
        }
    }
}

impl From<IdReg> for embedded_can::Id {
    fn from(idr: IdReg) -> Self {
        idr.id()
    }
}

/// `IdReg` is ordered by priority.
impl Ord for IdReg {
    fn cmp(&self, other: &Self) -> Ordering {
        // When the IDs match, data frames have priority over remote frames.
        let rtr = self.rtr().cmp(&other.rtr()).reverse();

        let id_a = self.to_id();
        let id_b = other.to_id();
        match (id_a, id_b) {
            (Id::Standard(a), Id::Standard(b)) => {
                // Lower IDs have priority over higher IDs.
                a.as_raw().cmp(&b.as_raw()).reverse().then(rtr)
            }
            (Id::Extended(a), Id::Extended(b)) => a.as_raw().cmp(&b.as_raw()).reverse().then(rtr),
            (Id::Standard(a), Id::Extended(b)) => {
                // Standard frames have priority over extended frames if their Base IDs match.
                a.as_raw()
                    .cmp(&b.standard_id().as_raw())
                    .reverse()
                    .then(Ordering::Greater)
            }
            (Id::Extended(a), Id::Standard(b)) => {
                a.standard_id().as_raw().cmp(&b.as_raw()).reverse().then(Ordering::Less)
            }
        }
    }
}

impl PartialOrd for IdReg {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum RxFifo {
    Fifo0,
    Fifo1,
}
