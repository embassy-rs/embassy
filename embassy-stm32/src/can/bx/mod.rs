//! Driver for the STM32 bxCAN peripheral.
//!
//! This crate provides a reusable driver for the bxCAN peripheral found in many low- to middle-end
//! STM32 microcontrollers. HALs for compatible chips can reexport this crate and implement its
//! traits to easily expose a featureful CAN driver.
//!
//! # Features
//!
//! - Supports both single- and dual-peripheral configurations (where one bxCAN instance manages the
//!   filters of a secondary instance).
//! - Handles standard and extended frames, and data and remote frames.
//! - Support for interrupts emitted by the bxCAN peripheral.
//! - Transmission respects CAN IDs and protects against priority inversion (a lower-priority frame
//!   may be dequeued when enqueueing a higher-priority one).
//! - Implements the [`embedded-hal`] traits for interoperability.
//! - Support for both RX FIFOs (as [`Rx0`] and [`Rx1`]).
//!
//! # Limitations
//!
//! - Support for querying error states and handling error interrupts is incomplete.
//!

// Deny a few warnings in doctests, since rustdoc `allow`s many warnings by default
#![allow(clippy::unnecessary_operation)] // lint is bugged

//mod embedded_hal;
pub mod filter;

#[allow(clippy::all)] // generated code
use core::cmp::{Ord, Ordering};
use core::convert::Infallible;
use core::marker::PhantomData;
use core::mem;

pub use embedded_can::{ExtendedId, Id, StandardId};

/// CAN Header: includes ID and length
pub type Header = crate::can::frame::Header;

/// Data for a CAN Frame
pub type Data = crate::can::frame::ClassicData;

/// CAN Frame
pub type Frame = crate::can::frame::ClassicFrame;

use crate::can::bx::filter::MasterFilters;

/// A bxCAN peripheral instance.
///
/// This trait is meant to be implemented for a HAL-specific type that represent ownership of
/// the CAN peripheral (and any pins required by it, although that is entirely up to the HAL).
///
/// # Safety
///
/// It is only safe to implement this trait, when:
///
/// * The implementing type has ownership of the peripheral, preventing any other accesses to the
///   register block.
/// * `REGISTERS` is a pointer to that peripheral's register block and can be safely accessed for as
///   long as ownership or a borrow of the implementing type is present.
pub unsafe trait Instance {}

/// A bxCAN instance that owns filter banks.
///
/// In master-slave-instance setups, only the master instance owns the filter banks, and needs to
/// split some of them off for use by the slave instance. In that case, the master instance should
/// implement [`FilterOwner`] and [`MasterInstance`], while the slave instance should only implement
/// [`Instance`].
///
/// In single-instance configurations, the instance owns all filter banks and they can not be split
/// off. In that case, the instance should implement [`Instance`] and [`FilterOwner`].
///
/// # Safety
///
/// This trait must only be implemented if the instance does, in fact, own its associated filter
/// banks, and `NUM_FILTER_BANKS` must be correct.
pub unsafe trait FilterOwner: Instance {
    /// The total number of filter banks available to the instance.
    ///
    /// This is usually either 14 or 28, and should be specified in the chip's reference manual or datasheet.
    const NUM_FILTER_BANKS: u8;
}

/// A bxCAN master instance that shares filter banks with a slave instance.
///
/// In master-slave-instance setups, this trait should be implemented for the master instance.
///
/// # Safety
///
/// This trait must only be implemented when there is actually an associated slave instance.
pub unsafe trait MasterInstance: FilterOwner {}

// TODO: what to do with these?
/*
#[derive(Debug, Copy, Clone, Eq, PartialEq, Format)]
pub enum Error {
    Stuff,
    Form,
    Acknowledgement,
    BitRecessive,
    BitDominant,
    Crc,
    Software,
}*/

/// Error that indicates that an incoming message has been lost due to buffer overrun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OverrunError {
    _priv: (),
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

/// Configuration proxy returned by [`Can::modify_config`].
#[must_use = "`CanConfig` leaves the peripheral in uninitialized state, call `CanConfig::enable` or explicitly drop the value"]
pub struct CanConfig<'a, I: Instance> {
    can: &'a mut Can<I>,
}

impl<I: Instance> CanConfig<'_, I> {
    /// Configures the bit timings.
    ///
    /// You can use <http://www.bittiming.can-wiki.info/> to calculate the `btr` parameter. Enter
    /// parameters as follows:
    ///
    /// - *Clock Rate*: The input clock speed to the CAN peripheral (*not* the CPU clock speed).
    ///   This is the clock rate of the peripheral bus the CAN peripheral is attached to (eg. APB1).
    /// - *Sample Point*: Should normally be left at the default value of 87.5%.
    /// - *SJW*: Should normally be left at the default value of 1.
    ///
    /// Then copy the `CAN_BUS_TIME` register value from the table and pass it as the `btr`
    /// parameter to this method.
    pub fn set_bit_timing(self, bt: crate::can::util::NominalBitTiming) -> Self {
        self.can.set_bit_timing(bt);
        self
    }

    /// Enables or disables loopback mode: Internally connects the TX and RX
    /// signals together.
    pub fn set_loopback(self, enabled: bool) -> Self {
        self.can.canregs.btr().modify(|reg| reg.set_lbkm(enabled));
        self
    }

    /// Enables or disables silent mode: Disconnects the TX signal from the pin.
    pub fn set_silent(self, enabled: bool) -> Self {
        let mode = match enabled {
            false => stm32_metapac::can::vals::Silm::NORMAL,
            true => stm32_metapac::can::vals::Silm::SILENT,
        };
        self.can.canregs.btr().modify(|reg| reg.set_silm(mode));
        self
    }

    /// Enables or disables automatic retransmission of messages.
    ///
    /// If this is enabled, the CAN peripheral will automatically try to retransmit each frame
    /// until it can be sent. Otherwise, it will try only once to send each frame.
    ///
    /// Automatic retransmission is enabled by default.
    pub fn set_automatic_retransmit(self, enabled: bool) -> Self {
        self.can.canregs.mcr().modify(|reg| reg.set_nart(enabled));
        self
    }

    /// Leaves initialization mode and enables the peripheral.
    ///
    /// To sync with the CAN bus, this will block until 11 consecutive recessive bits are detected
    /// on the bus.
    ///
    /// If you want to finish configuration without enabling the peripheral, you can call
    /// [`CanConfig::leave_disabled`] or [`drop`] the [`CanConfig`] instead.
    pub fn enable(mut self) {
        self.leave_init_mode();

        match nb::block!(self.can.enable_non_blocking()) {
            Ok(()) => {}
            Err(void) => match void {},
        }

        // Don't run the destructor.
        mem::forget(self);
    }

    /// Leaves initialization mode, but keeps the peripheral in sleep mode.
    ///
    /// Before the [`Can`] instance can be used, you have to enable it by calling
    /// [`Can::enable_non_blocking`].
    pub fn leave_disabled(mut self) {
        self.leave_init_mode();
    }

    /// Leaves initialization mode, enters sleep mode.
    fn leave_init_mode(&mut self) {
        self.can.canregs.mcr().modify(|reg| {
            reg.set_sleep(true);
            reg.set_inrq(false);
        });
        loop {
            let msr = self.can.canregs.msr().read();
            if msr.slak() && !msr.inak() {
                break;
            }
        }
    }
}

impl<I: Instance> Drop for CanConfig<'_, I> {
    #[inline]
    fn drop(&mut self) {
        self.leave_init_mode();
    }
}

/// Builder returned by [`Can::builder`].
#[must_use = "`CanBuilder` leaves the peripheral in uninitialized state, call `CanBuilder::enable` or `CanBuilder::leave_disabled`"]
pub struct CanBuilder<I: Instance> {
    can: Can<I>,
}

impl<I: Instance> CanBuilder<I> {
    /// Configures the bit timings.
    ///
    /// You can use <http://www.bittiming.can-wiki.info/> to calculate the `btr` parameter. Enter
    /// parameters as follows:
    ///
    /// - *Clock Rate*: The input clock speed to the CAN peripheral (*not* the CPU clock speed).
    ///   This is the clock rate of the peripheral bus the CAN peripheral is attached to (eg. APB1).
    /// - *Sample Point*: Should normally be left at the default value of 87.5%.
    /// - *SJW*: Should normally be left at the default value of 1.
    ///
    /// Then copy the `CAN_BUS_TIME` register value from the table and pass it as the `btr`
    /// parameter to this method.
    pub fn set_bit_timing(mut self, bt: crate::can::util::NominalBitTiming) -> Self {
        self.can.set_bit_timing(bt);
        self
    }
    /// Enables or disables loopback mode: Internally connects the TX and RX
    /// signals together.
    pub fn set_loopback(self, enabled: bool) -> Self {
        self.can.canregs.btr().modify(|reg| reg.set_lbkm(enabled));
        self
    }

    /// Enables or disables silent mode: Disconnects the TX signal from the pin.
    pub fn set_silent(self, enabled: bool) -> Self {
        let mode = match enabled {
            false => stm32_metapac::can::vals::Silm::NORMAL,
            true => stm32_metapac::can::vals::Silm::SILENT,
        };
        self.can.canregs.btr().modify(|reg| reg.set_silm(mode));
        self
    }

    /// Enables or disables automatic retransmission of messages.
    ///
    /// If this is enabled, the CAN peripheral will automatically try to retransmit each frame
    /// until it can be sent. Otherwise, it will try only once to send each frame.
    ///
    /// Automatic retransmission is enabled by default.
    pub fn set_automatic_retransmit(self, enabled: bool) -> Self {
        self.can.canregs.mcr().modify(|reg| reg.set_nart(enabled));
        self
    }

    /// Leaves initialization mode and enables the peripheral.
    ///
    /// To sync with the CAN bus, this will block until 11 consecutive recessive bits are detected
    /// on the bus.
    ///
    /// If you want to finish configuration without enabling the peripheral, you can call
    /// [`CanBuilder::leave_disabled`] instead.
    pub fn enable(mut self) -> Can<I> {
        self.leave_init_mode();

        match nb::block!(self.can.enable_non_blocking()) {
            Ok(()) => self.can,
            Err(void) => match void {},
        }
    }

    /// Returns the [`Can`] interface without enabling it.
    ///
    /// This leaves initialization mode, but keeps the peripheral in sleep mode instead of enabling
    /// it.
    ///
    /// Before the [`Can`] instance can be used, you have to enable it by calling
    /// [`Can::enable_non_blocking`].
    pub fn leave_disabled(mut self) -> Can<I> {
        self.leave_init_mode();
        self.can
    }

    /// Leaves initialization mode, enters sleep mode.
    fn leave_init_mode(&mut self) {
        self.can.canregs.mcr().modify(|reg| {
            reg.set_sleep(true);
            reg.set_inrq(false);
        });
        loop {
            let msr = self.can.canregs.msr().read();
            if msr.slak() && !msr.inak() {
                break;
            }
        }
    }
}

/// Interface to a bxCAN peripheral.
pub struct Can<I: Instance> {
    instance: I,
    canregs: crate::pac::can::Can,
}

impl<I> Can<I>
where
    I: Instance,
{
    /// Creates a [`CanBuilder`] for constructing a CAN interface.
    pub fn builder(instance: I, canregs: crate::pac::can::Can) -> CanBuilder<I> {
        let can_builder = CanBuilder {
            can: Can { instance, canregs },
        };

        canregs.mcr().modify(|reg| {
            reg.set_sleep(false);
            reg.set_inrq(true);
        });
        loop {
            let msr = canregs.msr().read();
            if !msr.slak() && msr.inak() {
                break;
            }
        }

        can_builder
    }

    fn set_bit_timing(&mut self, bt: crate::can::util::NominalBitTiming) {
        let prescaler = u16::from(bt.prescaler) & 0x1FF;
        let seg1 = u8::from(bt.seg1);
        let seg2 = u8::from(bt.seg2) & 0x7F;
        let sync_jump_width = u8::from(bt.sync_jump_width) & 0x7F;
        self.canregs.btr().modify(|reg| {
            reg.set_brp(prescaler - 1);
            reg.set_ts(0, seg1 - 1);
            reg.set_ts(1, seg2 - 1);
            reg.set_sjw(sync_jump_width - 1);
        });
    }

    /// Returns a reference to the peripheral instance.
    ///
    /// This allows accessing HAL-specific data stored in the instance type.
    pub fn instance(&mut self) -> &mut I {
        &mut self.instance
    }

    /// Disables the CAN interface and returns back the raw peripheral it was created from.
    ///
    /// The peripheral is disabled by setting `RESET` in `CAN_MCR`, which causes the peripheral to
    /// enter sleep mode.
    pub fn free(self) -> I {
        self.canregs.mcr().write(|reg| reg.set_reset(true));
        self.instance
    }

    /// Configure bit timings and silent/loop-back mode.
    ///
    /// Calling this method will enter initialization mode.
    pub fn modify_config(&mut self) -> CanConfig<'_, I> {
        self.canregs.mcr().modify(|reg| {
            reg.set_sleep(false);
            reg.set_inrq(true);
        });
        loop {
            let msr = self.canregs.msr().read();
            if !msr.slak() && msr.inak() {
                break;
            }
        }

        CanConfig { can: self }
    }

    /// Configures the automatic wake-up feature.
    ///
    /// This is turned off by default.
    ///
    /// When turned on, an incoming frame will cause the peripheral to wake up from sleep and
    /// receive the frame. If enabled, [`Interrupt::Wakeup`] will also be triggered by the incoming
    /// frame.
    pub fn set_automatic_wakeup(&mut self, enabled: bool) {
        self.canregs.mcr().modify(|reg| reg.set_awum(enabled));
    }

    /// Leaves initialization mode and enables the peripheral (non-blocking version).
    ///
    /// Usually, it is recommended to call [`CanConfig::enable`] instead. This method is only needed
    /// if you want non-blocking initialization.
    ///
    /// If this returns [`WouldBlock`][nb::Error::WouldBlock], the peripheral will enable itself
    /// in the background. The peripheral is enabled and ready to use when this method returns
    /// successfully.
    pub fn enable_non_blocking(&mut self) -> nb::Result<(), Infallible> {
        let msr = self.canregs.msr().read();
        if msr.slak() {
            self.canregs.mcr().modify(|reg| {
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
    pub fn sleep(&mut self) {
        self.canregs.mcr().modify(|reg| {
            reg.set_sleep(true);
            reg.set_inrq(false);
        });
        loop {
            let msr = self.canregs.msr().read();
            if msr.slak() && !msr.inak() {
                break;
            }
        }
    }

    /// Wakes up from sleep mode.
    ///
    /// Note that this will not trigger [`Interrupt::Wakeup`], only reception of an incoming CAN
    /// frame will cause that interrupt.
    pub fn wakeup(&mut self) {
        self.canregs.mcr().modify(|reg| {
            reg.set_sleep(false);
            reg.set_inrq(false);
        });
        loop {
            let msr = self.canregs.msr().read();
            if !msr.slak() && !msr.inak() {
                break;
            }
        }
    }

    /// Puts a CAN frame in a free transmit mailbox for transmission on the bus.
    ///
    /// Frames are transmitted to the bus based on their priority (see [`FramePriority`]).
    /// Transmit order is preserved for frames with identical priority.
    ///
    /// If all transmit mailboxes are full, and `frame` has a higher priority than the
    /// lowest-priority message in the transmit mailboxes, transmission of the enqueued frame is
    /// cancelled and `frame` is enqueued instead. The frame that was replaced is returned as
    /// [`TransmitStatus::dequeued_frame`].
    pub fn transmit(&mut self, frame: &Frame) -> nb::Result<TransmitStatus, Infallible> {
        // Safety: We have a `&mut self` and have unique access to the peripheral.
        unsafe { Tx::<I>::conjure(self.canregs).transmit(frame) }
    }

    /// Returns `true` if no frame is pending for transmission.
    pub fn is_transmitter_idle(&self) -> bool {
        // Safety: Read-only operation.
        unsafe { Tx::<I>::conjure(self.canregs).is_idle() }
    }

    /// Attempts to abort the sending of a frame that is pending in a mailbox.
    ///
    /// If there is no frame in the provided mailbox, or its transmission succeeds before it can be
    /// aborted, this function has no effect and returns `false`.
    ///
    /// If there is a frame in the provided mailbox, and it is canceled successfully, this function
    /// returns `true`.
    pub fn abort(&mut self, mailbox: Mailbox) -> bool {
        // Safety: We have a `&mut self` and have unique access to the peripheral.
        unsafe { Tx::<I>::conjure(self.canregs).abort(mailbox) }
    }

    /// Returns a received frame if available.
    ///
    /// This will first check FIFO 0 for a message or error. If none are available, FIFO 1 is
    /// checked.
    ///
    /// Returns `Err` when a frame was lost due to buffer overrun.
    pub fn receive(&mut self) -> nb::Result<Frame, OverrunError> {
        // Safety: We have a `&mut self` and have unique access to the peripheral.
        let mut rx0 = unsafe { Rx0::<I>::conjure(self.canregs) };
        let mut rx1 = unsafe { Rx1::<I>::conjure(self.canregs) };

        match rx0.receive() {
            Err(nb::Error::WouldBlock) => rx1.receive(),
            result => result,
        }
    }

    /// Returns a reference to the RX FIFO 0.
    pub fn rx0(&mut self) -> Rx0<I> {
        // Safety: We take `&mut self` and the return value lifetimes are tied to `self`'s lifetime.
        unsafe { Rx0::conjure(self.canregs) }
    }

    /// Returns a reference to the RX FIFO 1.
    pub fn rx1(&mut self) -> Rx1<I> {
        // Safety: We take `&mut self` and the return value lifetimes are tied to `self`'s lifetime.
        unsafe { Rx1::conjure(self.canregs) }
    }

    pub(crate) fn split_by_ref(&mut self) -> (Tx<I>, Rx0<I>, Rx1<I>) {
        // Safety: We take `&mut self` and the return value lifetimes are tied to `self`'s lifetime.
        let tx = unsafe { Tx::conjure(self.canregs) };
        let rx0 = unsafe { Rx0::conjure(self.canregs) };
        let rx1 = unsafe { Rx1::conjure(self.canregs) };
        (tx, rx0, rx1)
    }

    /// Consumes this `Can` instance and splits it into transmitting and receiving halves.
    pub fn split(self) -> (Tx<I>, Rx0<I>, Rx1<I>) {
        // Safety: `Self` is not `Copy` and is destroyed by moving it into this method.
        unsafe {
            (
                Tx::conjure(self.canregs),
                Rx0::conjure(self.canregs),
                Rx1::conjure(self.canregs),
            )
        }
    }
}

impl<I: FilterOwner> Can<I> {
    /// Accesses the filter banks owned by this CAN peripheral.
    ///
    /// To modify filters of a slave peripheral, `modify_filters` has to be called on the master
    /// peripheral instead.
    pub fn modify_filters(&mut self) -> MasterFilters<'_, I> {
        unsafe { MasterFilters::new(self.canregs) }
    }
}

/// Interface to the CAN transmitter part.
pub struct Tx<I> {
    _can: PhantomData<I>,
    canregs: crate::pac::can::Can,
}

impl<I> Tx<I>
where
    I: Instance,
{
    unsafe fn conjure(canregs: crate::pac::can::Can) -> Self {
        Self {
            _can: PhantomData,
            canregs,
        }
    }

    /// Puts a CAN frame in a transmit mailbox for transmission on the bus.
    ///
    /// Frames are transmitted to the bus based on their priority (see [`FramePriority`]).
    /// Transmit order is preserved for frames with identical priority.
    ///
    /// If all transmit mailboxes are full, and `frame` has a higher priority than the
    /// lowest-priority message in the transmit mailboxes, transmission of the enqueued frame is
    /// cancelled and `frame` is enqueued instead. The frame that was replaced is returned as
    /// [`TransmitStatus::dequeued_frame`].
    pub fn transmit(&mut self, frame: &Frame) -> nb::Result<TransmitStatus, Infallible> {
        // Get the index of the next free mailbox or the one with the lowest priority.
        let tsr = self.canregs.tsr().read();
        let idx = tsr.code() as usize;

        let frame_is_pending = !tsr.tme(0) || !tsr.tme(1) || !tsr.tme(2);
        let pending_frame = if frame_is_pending {
            // High priority frames are transmitted first by the mailbox system.
            // Frames with identical identifier shall be transmitted in FIFO order.
            // The controller schedules pending frames of same priority based on the
            // mailbox index instead. As a workaround check all pending mailboxes
            // and only accept higher priority frames.
            self.check_priority(0, frame.id().into())?;
            self.check_priority(1, frame.id().into())?;
            self.check_priority(2, frame.id().into())?;

            let all_frames_are_pending = !tsr.tme(0) && !tsr.tme(1) && !tsr.tme(2);
            if all_frames_are_pending {
                // No free mailbox is available. This can only happen when three frames with
                // ascending priority (descending IDs) were requested for transmission and all
                // of them are blocked by bus traffic with even higher priority.
                // To prevent a priority inversion abort and replace the lowest priority frame.
                self.read_pending_mailbox(idx)
            } else {
                // There was a free mailbox.
                None
            }
        } else {
            // All mailboxes are available: Send frame without performing any checks.
            None
        };

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
    /// lower priority (higher ID) than the identifier `id`.
    fn check_priority(&self, idx: usize, id: IdReg) -> nb::Result<(), Infallible> {
        // Read the pending frame's id to check its priority.
        assert!(idx < 3);
        let tir = &self.canregs.tx(idx).tir().read();
        //let tir = &can.tx[idx].tir.read();

        // Check the priority by comparing the identifiers. But first make sure the
        // frame has not finished the transmission (`TXRQ` == 0) in the meantime.
        if tir.txrq() && id <= IdReg::from_register(tir.0) {
            // There's a mailbox whose priority is higher or equal
            // the priority of the new frame.
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }

    fn write_mailbox(&mut self, idx: usize, frame: &Frame) {
        debug_assert!(idx < 3);

        let mb = self.canregs.tx(idx);
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

    fn read_pending_mailbox(&mut self, idx: usize) -> Option<Frame> {
        if self.abort_by_index(idx) {
            debug_assert!(idx < 3);

            let mb = self.canregs.tx(idx);

            let id = IdReg(mb.tir().read().0).id();
            let mut data = [0xff; 8];
            data[0..4].copy_from_slice(&mb.tdlr().read().0.to_ne_bytes());
            data[4..8].copy_from_slice(&mb.tdhr().read().0.to_ne_bytes());
            let len = mb.tdtr().read().dlc();

            Some(Frame::new(Header::new(id, len, false), &data).unwrap())
        } else {
            // Abort request failed because the frame was already sent (or being sent) on
            // the bus. All mailboxes are now free. This can happen for small prescaler
            // values (e.g. 1MBit/s bit timing with a source clock of 8MHz) or when an ISR
            // has preempted the execution.
            None
        }
    }

    /// Tries to abort a pending frame. Returns `true` when aborted.
    fn abort_by_index(&mut self, idx: usize) -> bool {
        self.canregs.tsr().write(|reg| reg.set_abrq(idx, true));

        // Wait for the abort request to be finished.
        loop {
            let tsr = self.canregs.tsr().read();
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
    pub fn abort(&mut self, mailbox: Mailbox) -> bool {
        // If the mailbox is empty, the value of TXOKx depends on what happened with the previous
        // frame in that mailbox. Only call abort_by_index() if the mailbox is not empty.
        let tsr = self.canregs.tsr().read();
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
        let tsr = self.canregs.tsr().read();
        tsr.tme(0) && tsr.tme(1) && tsr.tme(2)
    }

    /// Clears the request complete flag for all mailboxes.
    pub fn clear_interrupt_flags(&mut self) {
        self.canregs.tsr().write(|reg| {
            reg.set_rqcp(0, true);
            reg.set_rqcp(1, true);
            reg.set_rqcp(2, true);
        });
    }
}

/// Interface to receiver FIFO 0.
pub struct Rx0<I> {
    _can: PhantomData<I>,
    canregs: crate::pac::can::Can,
}

impl<I> Rx0<I>
where
    I: Instance,
{
    unsafe fn conjure(canregs: crate::pac::can::Can) -> Self {
        Self {
            _can: PhantomData,
            canregs,
        }
    }

    /// Returns a received frame if available.
    ///
    /// Returns `Err` when a frame was lost due to buffer overrun.
    pub fn receive(&mut self) -> nb::Result<Frame, OverrunError> {
        receive_fifo(self.canregs, 0)
    }
}

/// Interface to receiver FIFO 1.
pub struct Rx1<I> {
    _can: PhantomData<I>,
    canregs: crate::pac::can::Can,
}

impl<I> Rx1<I>
where
    I: Instance,
{
    unsafe fn conjure(canregs: crate::pac::can::Can) -> Self {
        Self {
            _can: PhantomData,
            canregs,
        }
    }

    /// Returns a received frame if available.
    ///
    /// Returns `Err` when a frame was lost due to buffer overrun.
    pub fn receive(&mut self) -> nb::Result<Frame, OverrunError> {
        receive_fifo(self.canregs, 1)
    }
}

fn receive_fifo(canregs: crate::pac::can::Can, fifo_nr: usize) -> nb::Result<Frame, OverrunError> {
    assert!(fifo_nr < 2);
    let rfr = canregs.rfr(fifo_nr);
    let rx = canregs.rx(fifo_nr);

    //let rfr = &can.rfr[fifo_nr];
    //let rx = &can.rx[fifo_nr];

    // Check if a frame is available in the mailbox.
    let rfr_read = rfr.read();
    if rfr_read.fmp() == 0 {
        return Err(nb::Error::WouldBlock);
    }

    // Check for RX FIFO overrun.
    if rfr_read.fovr() {
        rfr.write(|w| w.set_fovr(true));
        return Err(nb::Error::Other(OverrunError { _priv: () }));
    }

    // Read the frame.
    let id = IdReg(rx.rir().read().0).id();
    let mut data = [0xff; 8];
    data[0..4].copy_from_slice(&rx.rdlr().read().0.to_ne_bytes());
    data[4..8].copy_from_slice(&rx.rdhr().read().0.to_ne_bytes());
    let len = rx.rdtr().read().dlc();

    // Release the mailbox.
    rfr.write(|w| w.set_rfom(true));

    Ok(Frame::new(Header::new(id, len, false), &data).unwrap())
}

/// Identifies one of the two receive FIFOs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Fifo {
    /// First receive FIFO
    Fifo0 = 0,
    /// Second receive FIFO
    Fifo1 = 1,
}

/// Identifies one of the three transmit mailboxes.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mailbox {
    /// Transmit mailbox 0
    Mailbox0 = 0,
    /// Transmit mailbox 1
    Mailbox1 = 1,
    /// Transmit mailbox 2
    Mailbox2 = 2,
}

/// Contains information about a frame enqueued for transmission via [`Can::transmit`] or
/// [`Tx::transmit`].
pub struct TransmitStatus {
    dequeued_frame: Option<Frame>,
    mailbox: Mailbox,
}

impl TransmitStatus {
    /// Returns the lower-priority frame that was dequeued to make space for the new frame.
    #[inline]
    pub fn dequeued_frame(&self) -> Option<&Frame> {
        self.dequeued_frame.as_ref()
    }

    /// Returns the [`Mailbox`] the frame was enqueued in.
    #[inline]
    pub fn mailbox(&self) -> Mailbox {
        self.mailbox
    }
}
