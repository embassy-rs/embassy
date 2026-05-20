//! Direct Memory Access (DMA)

#![macro_use]

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::{Context, Poll};
use core::{fmt, mem};

use critical_section::CriticalSection;
use embassy_hal_internal::PeripheralType;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
use mspm0_metapac::common::{RW, Reg};
use mspm0_metapac::dma::regs;
use mspm0_metapac::dma::vals::{self, Autoen, Em, Incr, Preirq, Wdth};

use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::{Peri, interrupt, pac};

/// DMA interrupt handler.
pub struct InterruptHandler<T: ChannelInstance> {
    _marker: PhantomData<T>,
}

impl<T: ChannelInstance> Handler<<T as ChannelInstance>::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_irq(pac::DMA);
    }
}

/// The burst size of a DMA transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurstSize {
    /// The whole block transfer is completed in one transfer without interruption.
    Complete,

    /// The burst size is 8, after 9 transfers the block transfer is interrupted and the priority
    /// is reevaluated.
    _8,

    /// The burst size is 16, after 17 transfers the block transfer is interrupted and the priority
    /// is reevaluated.
    _16,

    /// The burst size is 32, after 32 transfers the block transfer is interrupted and the priority
    /// is reevaluated.
    _32,
}

/// Basic DMA channel driver.
pub struct Channel<'d> {
    id: u8,
    _marker: PhantomData<&'d ()>,
}

impl<'d> Channel<'d> {
    /// Create a new basic DMA channel driver.
    pub fn new<T: ChannelInstance>(
        _ch: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        Self {
            id: T::ID,
            _marker: PhantomData,
        }
    }

    /// Reborrow the channel, allowing it to be used in multiple places.
    pub fn reborrow(&mut self) -> Channel<'_> {
        Channel {
            id: self.id,
            _marker: PhantomData,
        }
    }

    /// Create a new read DMA transfer.
    pub unsafe fn read<'a, SW: Word, DW: Word>(
        &'a mut self,
        trigger_source: u8,
        src: *mut SW,
        dst: &'a mut [DW],
        options: TransferOptions,
    ) -> Result<Transfer<'a>, Error> {
        self.read_raw(trigger_source, src, dst, options)
    }

    /// Create a new read DMA transfer, using raw pointers.
    pub unsafe fn read_raw<'a, SW: Word, DW: Word>(
        &'a mut self,
        trigger_source: u8,
        src: *mut SW,
        dst: *mut [DW],
        options: TransferOptions,
    ) -> Result<Transfer<'a>, Error> {
        verify_transfer::<DW>(dst)?;

        let transfer = Transfer {
            channel: self.reborrow(),
        };
        transfer.channel.configure(
            trigger_source,
            src.cast(),
            SW::width(),
            dst.cast(),
            DW::width(),
            dst.len() as u16,
            false,
            true,
            options,
        );
        transfer.channel.start();

        Ok(transfer)
    }

    /// Create a new write DMA transfer.
    pub unsafe fn write<'a, SW: Word, DW: Word>(
        &'a mut self,
        trigger_source: u8,
        src: &'a [SW],
        dst: *mut DW,
        options: TransferOptions,
    ) -> Result<Transfer<'a>, Error> {
        self.write_raw(trigger_source, src, dst, options)
    }

    /// Create a new write DMA transfer, using raw pointers.
    pub unsafe fn write_raw<'a, SW: Word, DW: Word>(
        &'a mut self,
        trigger_source: u8,
        src: *const [SW],
        dst: *mut DW,
        options: TransferOptions,
    ) -> Result<Transfer<'a>, Error> {
        verify_transfer::<SW>(src)?;

        let transfer = Transfer {
            channel: self.reborrow(),
        };
        transfer.channel.configure(
            trigger_source,
            src.cast(),
            SW::width(),
            dst.cast(),
            DW::width(),
            src.len() as u16,
            true,
            false,
            options,
        );
        transfer.channel.start();

        Ok(transfer)
    }
}

/// Full DMA channel driver.
///
/// This is an extended [`Channel`] driver and can be [reborrowed](Self::reborrow) to be used as a basic
/// channel driver.
pub struct FullChannel<'d>(Channel<'d>);

impl<'d> FullChannel<'d> {
    /// Create a new full DMA channel driver.
    pub fn new<T: FullChannelInstance>(
        ch: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        Self(Channel::new(ch, irq))
    }

    /// Reborrow the full channel, allowing it to be used in multiple places.
    pub fn reborrow_full(&mut self) -> FullChannel<'_> {
        FullChannel(self.0.reborrow())
    }

    /// Reborrow the channel as a basic [`Channel`], allowing it to be used in multiple places.
    pub fn reborrow(&mut self) -> Channel<'_> {
        self.0.reborrow()
    }
}

/// DMA channel instance.
#[allow(private_bounds)]
pub trait ChannelInstance: SealedChannel + PeripheralType {
    /// Interrupt type for this DMA channel.
    type Interrupt: Interrupt;
}

/// Full DMA channel instance.
#[allow(private_bounds)]
pub trait FullChannelInstance: ChannelInstance {}

#[allow(private_bounds)]
pub trait Word: SealedWord + 'static {
    /// Size in bytes for the width.
    fn size() -> isize;
}

impl SealedWord for u8 {
    fn width() -> vals::Wdth {
        vals::Wdth::BYTE
    }
}
impl Word for u8 {
    fn size() -> isize {
        1
    }
}

impl SealedWord for u16 {
    fn width() -> vals::Wdth {
        vals::Wdth::HALF
    }
}
impl Word for u16 {
    fn size() -> isize {
        2
    }
}

impl SealedWord for u32 {
    fn width() -> vals::Wdth {
        vals::Wdth::WORD
    }
}
impl Word for u32 {
    fn size() -> isize {
        4
    }
}

impl SealedWord for u64 {
    fn width() -> vals::Wdth {
        vals::Wdth::LONG
    }
}
impl Word for u64 {
    fn size() -> isize {
        8
    }
}

// TODO: u128 (LONGLONG) support. G350x does support it, but other parts do not such as C110x. More metadata is
// needed to properly enable this.
// impl SealedWord for u128 {
//     fn width() -> vals::Wdth {
//         vals::Wdth::LONGLONG
//     }
// }
// impl Word for u128 {
//     fn size() -> isize {
//         16
//     }
// }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The DMA transfer is too large.
    ///
    /// The hardware limits the DMA to 16384 transfers per channel at a time. This means that transferring
    /// 16384 `u8` and 16384 `u64` are equivalent, since the DMA must copy 16384 values.
    TooManyTransfers,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TooManyTransfers => write!(f, "too many transfers"),
        }
    }
}

impl core::error::Error for Error {}

/// DMA transfer mode for basic channels.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransferMode {
    /// Each DMA trigger will transfer a single value.
    Single,

    /// Each DMA trigger will transfer the complete block with one trigger.
    Block,
}

/// DMA transfer options.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// DMA transfer mode.
    pub mode: TransferMode,
    // TODO: Read and write stride.
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            mode: TransferMode::Single,
        }
    }
}

/// DMA transfer.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a> {
    channel: Channel<'a>,
}

impl<'a> Transfer<'a> {
    /// Software trigger source.
    ///
    /// Using this trigger source means that a transfer will start immediately rather than waiting for
    /// a hardware event. This can be useful if you want to do a DMA accelerated memcpy.
    pub const SOFTWARE_TRIGGER: u8 = 0;

    /// Request the transfer to resume.
    pub fn resume(&mut self) {
        self.channel.resume();
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause();
    }

    /// Return whether this transfer is still running.
    ///
    /// If this returns [`false`], it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_stop`].
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Blocking wait until the transfer finishes.
    pub fn blocking_wait(mut self) {
        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);

        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);

        // Prevent drop from being called since we ran to completion (drop will try to pause).
        mem::forget(self);
    }
}

impl<'a> Unpin for Transfer<'a> {}
impl<'a> Future for Transfer<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state: &ChannelState = &STATE[self.channel.id as usize];

        state.waker.register(cx.waker());

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);

        if self.channel.is_running() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        self.channel.request_pause();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);
    }
}

// impl details

fn verify_transfer<W: Word>(ptr: *const [W]) -> Result<(), Error> {
    if ptr.len() > (u16::MAX as usize) {
        return Err(Error::TooManyTransfers);
    }

    // TODO: Stride checks

    Ok(())
}

fn convert_burst_size(value: BurstSize) -> vals::Burstsz {
    match value {
        BurstSize::Complete => vals::Burstsz::INFINITI,
        BurstSize::_8 => vals::Burstsz::BURST_8,
        BurstSize::_16 => vals::Burstsz::BURST_16,
        BurstSize::_32 => vals::Burstsz::BURST_32,
    }
}

fn convert_mode(mode: TransferMode) -> vals::Tm {
    match mode {
        TransferMode::Single => vals::Tm::SINGLE,
        TransferMode::Block => vals::Tm::BLOCK,
    }
}

const CHANNEL_COUNT: usize = crate::_generated::DMA_CHANNELS;
static STATE: [ChannelState; CHANNEL_COUNT] = [const { ChannelState::new() }; CHANNEL_COUNT];

struct ChannelState {
    waker: AtomicWaker,
}

impl ChannelState {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

/// SAFETY: Must only be called once.
///
/// Changing the burst size mid transfer may have some odd behavior.
pub(crate) unsafe fn init(_cs: CriticalSection, burst_size: BurstSize, round_robin: bool) {
    pac::DMA.prio().modify(|prio| {
        prio.set_burstsz(convert_burst_size(burst_size));
        prio.set_roundrobin(round_robin);
    });
    pac::DMA.int_event(0).imask().modify(|w| {
        w.set_dataerr(true);
        w.set_addrerr(true);
    });

    interrupt::DMA.enable();
}

pub(crate) trait SealedWord {
    fn width() -> vals::Wdth;
}

pub(crate) trait SealedChannel {
    const ID: u8;
}

impl<'d> Channel<'d> {
    #[inline]
    fn tctl(&self) -> Reg<regs::Tctl, RW> {
        pac::DMA.trig(self.id as usize).tctl()
    }

    #[inline]
    fn ctl(&self) -> Reg<regs::Ctl, RW> {
        pac::DMA.chan(self.id as usize).ctl()
    }

    #[inline]
    fn sa(&self) -> Reg<u32, RW> {
        pac::DMA.chan(self.id as usize).sa()
    }

    #[inline]
    fn da(&self) -> Reg<u32, RW> {
        pac::DMA.chan(self.id as usize).da()
    }

    #[inline]
    fn sz(&self) -> Reg<regs::Sz, RW> {
        pac::DMA.chan(self.id as usize).sz()
    }

    #[inline]
    fn mask_interrupt(&self, enable: bool) {
        // Enabling interrupts is an RMW operation.
        critical_section::with(|_cs| {
            pac::DMA.int_event(0).imask().modify(|w| {
                w.set_ch(self.id as usize, enable);
            });
        })
    }

    /// # Safety
    ///
    /// - `src` must be valid for the lifetime of the transfer.
    /// - `dst` must be valid for the lifetime of the transfer.
    unsafe fn configure(
        &self,
        trigger_sel: u8,
        src: *const u32,
        src_wdth: Wdth,
        dst: *const u32,
        dst_wdth: Wdth,
        transfer_count: u16,
        increment_src: bool,
        increment_dst: bool,
        options: TransferOptions,
    ) {
        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);

        self.ctl().modify(|w| {
            // SLAU 5.2.5:
            // "The DMATSEL bits should be modified only when the DMACTLx.DMAEN bit is
            //  0; otherwise, unpredictable DMA triggers can occur."
            //
            // We also want to stop any transfers before setup.
            w.set_en(false);
            w.set_req(false);

            // Not every part supports auto enable, so force its value to 0.
            w.set_autoen(Autoen::NONE);
            w.set_preirq(Preirq::PREIRQ_DISABLE);
            w.set_srcwdth(src_wdth);
            w.set_dstwdth(dst_wdth);
            w.set_srcincr(if increment_src {
                Incr::INCREMENT
            } else {
                Incr::UNCHANGED
            });
            w.set_dstincr(if increment_dst {
                Incr::INCREMENT
            } else {
                Incr::UNCHANGED
            });

            w.set_em(Em::NORMAL);
            // Single and block will clear the enable bit when the transfers finish.
            w.set_tm(convert_mode(options.mode));
        });

        self.tctl().write(|w| {
            w.set_tsel(trigger_sel);
            // Basic channels do not implement cross triggering.
            w.set_tint(vals::Tint::EXTERNAL);
        });

        self.sz().write(|w| {
            w.set_size(transfer_count);
        });

        self.sa().write_value(src as u32);
        self.da().write_value(dst as u32);

        // Enable the channel.
        self.ctl().modify(|w| {
            // FIXME: Why did putting set_req later fix some transfers
            w.set_en(true);
            w.set_req(true);
        });
    }

    fn start(&self) {
        self.mask_interrupt(true);

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);

        // Request the DMA transfer to start.
        self.ctl().modify(|w| {
            w.set_req(true);
        });
    }

    fn resume(&self) {
        self.mask_interrupt(true);

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);

        self.ctl().modify(|w| {
            // w.set_en(true);
            w.set_req(true);
        });
    }

    fn request_pause(&self) {
        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);

        // Stop the transfer.
        //
        // SLAU846 5.2.6:
        // "A DMA block transfer in progress can be stopped by clearing the DMAEN bit"
        self.ctl().modify(|w| {
            // w.set_en(false);
            w.set_req(false);
        });
    }

    fn is_running(&self) -> bool {
        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::SeqCst);

        let ctl = self.ctl().read();

        // Is the transfer requested?
        ctl.req()
            // Is the channel enabled?
            && ctl.en()
    }
}

// Some future parts will have no DMA.
#[allow(unused_macros)]
macro_rules! impl_dma_channel {
    ($instance: ident, $num: expr) => {
        impl crate::dma::SealedChannel for crate::peripherals::$instance {
            const ID: u8 = $num;
        }

        impl crate::dma::ChannelInstance for crate::peripherals::$instance {
            // TODO: For chips with multiple DMAs, pick correct instance
            type Interrupt = crate::interrupt::typelevel::DMA;
        }
    };
}

// C1104 has no full DMA channels.
#[allow(unused_macros)]
macro_rules! impl_full_dma_channel {
    ($instance: ident, $num: expr) => {
        impl_dma_channel!($instance, $num);

        impl crate::dma::FullChannelInstance for crate::peripherals::$instance {}
    };
}

fn on_irq(dma: pac::dma::Dma) {
    use crate::BitIter;

    let events = dma.int_event(0);
    let mis = events.mis().read();

    // TODO: Handle DATAERR and ADDRERR? However we do not know which channel causes an error.
    if mis.dataerr() {
        panic!("DMA data error");
    } else if mis.addrerr() {
        panic!("DMA address error")
    }

    // Ignore preirq interrupts (values greater than 16).
    for i in BitIter(mis.0 & 0x0000_FFFF) {
        if let Some(state) = STATE.get(i as usize) {
            state.waker.wake();

            // Notify the future that the counter size hit zero
            events.imask().modify(|w| {
                w.set_ch(i as usize, false);
            });
        }
    }
}
