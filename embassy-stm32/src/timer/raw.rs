//! Raw register-level timer driver.
//!
//! This module provides the core functionality for timer drivers. It provides type-safe access to
//! the timer registers, ensuring that only the registers which are available for the given timer
//! can be used.

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralRef;

use super::{
    Advanced1ChInstance, Advanced1ChTim, Advanced2ChInstance, Advanced2ChTim, Advanced4ChInstance, Advanced4ChTim,
    BasicInstance, BasicTim, CcDma, CcDmaInstance, CcDmaTim, ChannelCount, ChannelMarker, CoreInstance, CoreTim,
    General1ChInstance, General1ChTim, General2ChInstance, General2ChTim, General32BitInstance, General32BitTim,
    General4ChInstance, General4ChTim, Info, IsCcDmaTim, IsCoreTim, IsGeneral1ChTim, IsGeneral2ChTim, IsGeneral4ChTim,
    IsMmsTim, IsTrigDmaTim, IsUpDmaTim, MmsInstance, MmsTim, TimerPin, TimerPinMarker, TimerType, TrigDma,
    TrigDmaInstance, TrigDmaTim, UpDma, UpDmaInstance, UpDmaTim,
};
#[cfg(not(timer_l0))]
use super::{IsAdvanced1ChTim, IsAdvanced2ChTim, IsAdvanced4ChTim, IsGeneral32BitTim};
use crate::gpio::{self, SealedPin as _};
use crate::pac::common::{Reg, RW, W};
use crate::pac::timer::regs;
use crate::time::Hertz;
use crate::{dma, into_ref, pac, rcc, Peripheral};

/// Get DMA request and channel that can be used as the update DMA for timer `T`.
pub fn up_dma<'d, T>(dma: impl Peripheral<P = impl UpDma<T>> + 'd) -> dma::ChannelAndRequest<'d>
where
    T: CoreInstance,
{
    into_ref!(dma);
    let request = dma.request();
    dma::ChannelAndRequest {
        channel: dma.map_into(),
        request,
    }
}

/// Get DMA request and channel that can be used as the trigger DMA for timer `T`.
pub fn trig_dma<'d, T>(dma: impl Peripheral<P = impl TrigDma<T>> + 'd) -> dma::ChannelAndRequest<'d>
where
    T: CoreInstance,
{
    into_ref!(dma);
    let request = dma.request();
    dma::ChannelAndRequest {
        channel: dma.map_into(),
        request,
    }
}

/// Get DMA request and channel that can be used as the capture/compare DMA for channel `C` of timer `T`.
pub fn cc_dma<'d, T, C>(dma: impl Peripheral<P = impl CcDma<T, C>> + 'd) -> dma::ChannelAndRequest<'d>
where
    T: CoreInstance,
    C: ChannelMarker,
{
    into_ref!(dma);
    let request = dma.request();
    dma::ChannelAndRequest {
        channel: dma.map_into(),
        request,
    }
}

/// Raw timer pin.
///
/// The only purpose of this struct is to correctly initialize the pin in the constructor and
/// deinitialize it (set it as disconnected) in the destructor. It can be used to implement pin
/// functionality in higher-level drivers.
pub struct RawTimerPin<'d> {
    pin: PeripheralRef<'d, gpio::AnyPin>,
}

impl<'d> RawTimerPin<'d> {
    /// Initializes `pin` as a timer pin `M` for timer instance `T`.
    pub fn new<T: CoreInstance, M: TimerPinMarker>(
        pin: impl Peripheral<P = impl TimerPin<T, M>> + 'd,
        af_type: gpio::AfType,
    ) -> Self {
        into_ref!(pin);
        let af_num = pin.af_num();
        let pin = pin.map_into();
        pin.set_as_af(af_num, af_type);
        Self { pin }
    }
}

impl<'d> Drop for RawTimerPin<'d> {
    fn drop(&mut self) {
        self.pin.set_as_disconnected();
    }
}

/// Raw timer driver.
///
/// This driver provides direct access to the timer registers. Only those registers and register
/// fields which are implemented by timers of type `Tim` are accessible.
///
/// Some registers or register fields (such as CCR) have multiple copies depending on the number of
/// channels provided by the timer peripheral. This driver allows you to access these registers or
/// fields for all channels, even if the `Tim` marker type does not guarantee that the channel is
/// available: for example, you can use [`ccr()`][Self::ccr()] with all four channels, even though
/// this method is available even for 1-channel timers. It is your responsibility to ensure that
/// you use only the channels that are available in the timer peripheral.
///
/// The following table lists the registers and fields with the timer instance that implements them
/// and a method that you can use to access the register.
///
/// | Register | Fields | Instance | Method |
/// | -------- | ------ | -------- | ------ |
// CR1
#[cfg_attr(
    timer_v1,
    doc = "| CR1 | CEN, UDIS, URS, OPM, ARPE, UIFREMAP | [`CoreInstance`] | [`RawTimer::cr1_core()`] "
)]
#[cfg_attr(
    timer_v2,
    doc = "| CR1 | CEN, UDIS, URS, OPM, ARPE, UIFREMAP, DITHEN | [`CoreInstance`] | [`RawTimer::cr1_core()`] "
)]
#[cfg_attr(
    timer_l0,
    doc = "| CR1 | CEN, UDIS, URS, OPM, ARPE | [`CoreInstance`] | [`RawTimer::cr1_core()`] "
)]
#[doc = "| | CKD | [`General1ChInstance`] | [`RawTimer::cr1_1ch()`] "]
#[cfg_attr(
    not(timer_l0),
    doc = "| | DIR, CMS | [`General4ChInstance`] | [`RawTimer::cr1_4ch()`] "
)]
#[cfg_attr(timer_l0, doc = "| | DIR, CMS | [`General2ChInstance`] | [`RawTimer::cr1_2ch()`] ")]
// CR2
#[doc = "| CR2 | MMS | [`MmsInstance`] | [`RawTimer::cr2_mms()`] "]
#[cfg_attr(
    not(timer_l0),
    doc = "| | TI1S | [`General2ChInstance`] | [`RawTimer::cr2_2ch()`], [`RawTimer::cr2_trigdma()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| | CCDS | [`CcDmaInstance`] | [`RawTimer::cr2_ccdma()`], [`RawTimer::cr2_trigdma()`] "
)]
#[cfg_attr(timer_l0, doc = "| | TI1S | [`TrigDmaInstance`] | [`RawTimer::cr2_trigdma()`] ")]
#[cfg_attr(
    not(timer_l0),
    doc = "| | CCPC, CCUS, OIS, OISN | [`Advanced1ChInstance`] | [`RawTimer::cr2_adv1ch()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| | MMS2 | [`Advanced4ChInstance`] | [`RawTimer::cr2_adv4ch()`] "
)]
// SMCR
#[cfg_attr(
    timer_v1,
    doc = "| SMCR | SMS, TS, MSM | [`General2ChInstance`] | [`RawTimer::smcr_2ch()`] "
)]
#[cfg_attr(
    timer_v1,
    doc = "| | ETF, ETPS, ECE, ETP | [`General4ChInstance`] | [`RawTimer::smcr_4ch()`] "
)]
#[cfg_attr(
    timer_v2,
    doc = "| SMCR | SMS, TS, MSM | [`General2ChInstance`] | [`RawTimer::smcr_2ch()`] "
)]
#[cfg_attr(
    timer_v2,
    doc = "| | ETF, ETPS, ECE, ETP, SMSPS | [`General4ChInstance`] | [`RawTimer::smcr_4ch()`] "
)]
#[cfg_attr(timer_v2, doc = "| | SMSPE | [`TrigDmaInstance`] | [`RawTimer::smcr_trigdma()`] ")]
#[cfg_attr(timer_v2, doc = "| | OCCS | [`Advanced4ChInstance`] | [`RawTimer::smcr_adv4ch()`] ")]
#[cfg_attr(
    timer_l0,
    doc = "| SMCR | SMS, TS, MSM, ETF, ETPS, ECE, ETP | [`General2ChInstance`] | [`RawTimer::smcr_2ch()`] "
)]
// DIER
#[doc = "| DIER | UIE | [`CoreInstance`] | [`RawTimer::dier_core()`] "]
#[doc = "| | UDE | [`UpDmaInstance`] | [`RawTimer::dier_updma()`] "]
#[doc = "| | CCIE | [`General1ChInstance`] | [`RawTimer::dier_1ch()`] "]
#[doc = "| | TIE | [`General2ChInstance`] | [`RawTimer::dier_2ch()`] "]
#[doc = "| | CCDE | [`CcDmaInstance`] | [`RawTimer::dier_ccdma()`] "]
#[doc = "| | TDE | [`TrigDmaInstance`] | [`RawTimer::dier_trigdma()`] "]
#[cfg_attr(
    timer_v2,
    doc = "| | IDXIE, DIRIE, IERRIE, TERRIE | [`General4ChInstance`] | [`RawTimer::dier_4ch()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| | COMIE, BIE | [`Advanced1ChInstance`] | [`RawTimer::dier_adv1ch()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| | COMDE | [`Advanced2ChInstance`] | [`RawTimer::dier_adv2ch()`], [`RawTimer::dier_adv4ch()`] "
)]
// SR
#[doc = "| SR | UIF | [`CoreInstance`] | [`RawTimer::sr_core()`] "]
#[doc = "| | CCIF, CCOF | [`General1ChInstance`] | [`RawTimer::sr_1ch()`] "]
#[cfg_attr(
    not(timer_l0),
    doc = "| | TIF | [`General2ChInstance`] | [`RawTimer::sr_2ch()`], [`RawTimer::sr_adv2ch()`] "
)]
#[cfg_attr(
    timer_v2,
    doc = "| | IDXIF, DIRIF, IERRIF, TERRIF | [`General4ChInstance`] | [`RawTimer::sr_4ch()`], [`RawTimer::sr_adv4ch()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| | COMIF, BIF | [`Advanced1ChInstance`] | [`RawTimer::sr_adv1ch()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| | SBIF, CCIF5, CCIF6 | [`Advanced4ChInstance`] | [`RawTimer::sr_adv4ch()`] "
)]
// EGR
#[doc = "| EGR | UG | [`CoreInstance`] | [`RawTimer::egr_core()`] "]
#[doc = "| | CCG | [`General1ChInstance`] | [`RawTimer::egr_1ch()`] "]
#[doc = "| | TG | [`General2ChInstance`] | [`RawTimer::egr_2ch()`] "]
#[cfg_attr(
    not(timer_l0),
    doc = "| | COMG, BG | [`Advanced1ChInstance`] | [`RawTimer::egr_adv1ch()`], [`RawTimer::egr_adv2ch()`] "
)]
// CCMR
#[doc = "| CCMR (input) | CCS, ICPSC, ICF | [`General1ChInstance`] | [`RawTimer::ccmr_input_1ch()`] "]
#[doc = "| CCMR (output) | CCS, OCFE, OCPE, OCM | [`General1ChInstance`] | [`RawTimer::ccmr_output_1ch()`] "]
#[doc = "| | OCCE | [`General4ChInstance`] | [`RawTimer::ccmr_output_4ch()`] "]
#[cfg_attr(
    not(timer_l0),
    doc = "| CCMR3 (output) | OCFE, OCPE, OCM, OCCE | [`Advanced4ChInstance`] | [`RawTimer::ccmr3_output_adv4ch()`] "
)]
// CCER
#[doc = "| CCER | CCE, CCP, CCNP | [`General1ChInstance`] | [`RawTimer::ccer_1ch()`] "]
#[cfg_attr(
    not(timer_l0),
    doc = "| | CCNE | [`Advanced1ChInstance`] | [`RawTimer::ccer_adv1ch()`] "
)]
// CNT
#[doc = "| CNT | 16-bit | [`CoreInstance`] | [`RawTimer::cnt()`] "]
#[cfg_attr(
    not(timer_l0),
    doc = "| | 32-bit | [`General32BitInstance`] | [`RawTimer::cnt_32bit()`] "
)]
// PSC
#[doc = "| PSC | | [`CoreInstance`] | [`RawTimer::arr()`] "]
// ARR
#[doc = "| ARR | 16-bit | [`CoreInstance`] | [`RawTimer::arr()`] "]
#[cfg_attr(
    not(timer_l0),
    doc = "| | 32-bit | [`General32BitInstance`] | [`RawTimer::arr_32bit()`] "
)]
#[cfg_attr(
    timer_v2,
    doc = "| ARR (dither mode) | 16-bit | [`CoreInstance`] | [`RawTimer::arr_dither()`] "
)]
// RCR
#[cfg_attr(
    not(timer_l0),
    doc = "| RCR | REP (8-bit) | [`Advanced1ChInstance`] | [`RawTimer::rcr_adv1ch()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| | REP (16-bit) | [`Advanced4ChInstance`] | [`RawTimer::rcr_adv4ch()`] "
)]
// CCR
#[cfg_attr(not(timer_l0), doc = "| CCR | | [`General1ChInstance`] | [`RawTimer::ccr()`] ")]
#[cfg_attr(
    timer_v2,
    doc = "| CCR (dither mode) | | [`General1ChInstance`] | [`RawTimer::ccr_dither()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| CCR (32-bit) | | [`General32BitInstance`] | [`RawTimer::ccr_32bit()`] "
)]
#[cfg_attr(not(timer_l0), doc = "| CCR5 | | [`Advanced4ChInstance`] | [`RawTimer::ccr5()`] ")]
#[cfg_attr(not(timer_l0), doc = "| | GC5C | [`Advanced4ChInstance`] | [`RawTimer::ccr5()`] ")]
#[cfg_attr(
    timer_v2,
    doc = "| CCR5 (dither mode) | | [`Advanced4ChInstance`] | [`RawTimer::ccr5_dither()`] "
)]
#[cfg_attr(not(timer_l0), doc = "| CCR6 | | [`Advanced4ChInstance`] | [`RawTimer::ccr6()`] ")]
#[cfg_attr(
    timer_v2,
    doc = "| CCR6 (dither mode) | | [`Advanced4ChInstance`] | [`RawTimer::ccr6_dither()`] "
)]
// BDTR
#[cfg_attr(
    timer_v1,
    doc = "| BDTR | DTG, LOCK, OSSI, OSSR, BKE, BKP, ADE, MOE, BKF | [`Advanced1ChInstance`] | [`RawTimer::bdtr()`] "
)]
#[cfg_attr(
    timer_v2,
    doc = "| BDTR | DTG, LOCK, OSSI, OSSR, BKE, BKP, ADE, MOE, BKF, BKDSRM, BKBID | [`Advanced1ChInstance`] | [`RawTimer::bdtr()`] "
)]
// DCR
#[cfg_attr(not(timer_v2), doc = "| DCR | DBA, DBL | [`CcDmaInstance`] | [`RawTimer::dcr()`] ")]
#[cfg_attr(timer_v2, doc = "| DCR | DBA, DBL, DBSS | [`CcDmaInstance`] | [`RawTimer::dcr()`] ")]
// DMAR
#[cfg_attr(
    any(timer_v1, timer_l0),
    doc = "| DMAR | (DMAB) | [`CcDmaInstance`] | [`RawTimer::dmar()`] "
)]
// DTR2
#[cfg_attr(
    timer_v2,
    doc = "| DTR2 | DTGF, DTAE, DTPE | [`Advanced1ChInstance`] | [`RawTimer::dtr2()`] "
)]
// ECR
#[cfg_attr(
    timer_v2,
    doc = "| ECR | IE, IDIR, IBLK, FIDX, IPOS, PW, PWPRSC | [`General4ChInstance`] | [`RawTimer::ecr()`] "
)]
// AF1
#[cfg_attr(
    not(timer_l0),
    doc = "| AF1 | ETRSEL | [`General4ChInstance`] | [`RawTimer::af1_4ch()`], [`RawTimer::af1_adv4ch()`] "
)]
#[cfg_attr(
    not(timer_l0),
    doc = "| | BKINE, BKCMPE, BKDF1BKE, BKINP, BKCMPP | [`Advanced1ChInstance`] | [`RawTimer::af1_adv1ch()`] "
)]
// AF2
#[cfg_attr(
    timer_v1,
    doc = "| AF2 | BK2INE, BK2CMPE, BK2DF1BK1E, BK2INP, BK2CMPP | [`Advanced4ChInstance`] | [`RawTimer::af2_adv4ch()`] "
)]
#[cfg_attr(timer_v2, doc = "| AF2 | OCRSEL | [`CcDmaInstance`] | [`RawTimer::af2_ccdma()`] ")]
#[cfg_attr(
    timer_v2,
    doc = "| | BK2INE, BK2CMPE, BK2INP, BK2CMPP | [`Advanced4ChInstance`] | [`RawTimer::af2_adv4ch()`] "
)]
// TISEL
#[cfg_attr(not(timer_l0), doc = "| TISEL | | [`General1ChInstance`] | [`RawTimer::tisel()`] ")]
pub struct RawTimer<'d, Tim> {
    info: &'d Info,
    kernel_clock: Hertz,
    _phantom: PhantomData<Tim>,
}

#[rustfmt::skip]
macro_rules! impl_new {
    ($marker_ty:ident, $timer_trait:ident, $new:ident) => {
        impl<'d> RawTimer<'d, $marker_ty> {
            #[doc = concat!(
                "Initializes the raw driver from timer `T`, treating it as [`",
                stringify!($timer_trait),
                "`].",
            )]
            pub fn $new<T: $timer_trait>(_tim: impl Peripheral<P = T> + 'd) -> Self {
                rcc::enable_and_reset::<T>();
                Self {
                    info: T::info(),
                    kernel_clock: T::frequency(),
                    _phantom: PhantomData,
                }
            }
        }
    };
}

impl_new!(CoreTim, CoreInstance, new_core);
impl_new!(UpDmaTim, UpDmaInstance, new_up_dma);
impl_new!(MmsTim, MmsInstance, new_mms);
impl_new!(BasicTim, BasicInstance, new_basic);
impl_new!(General1ChTim, General1ChInstance, new_general_1ch);
impl_new!(CcDmaTim, CcDmaInstance, new_cc_dma);
impl_new!(General2ChTim, General2ChInstance, new_general_2ch);
impl_new!(TrigDmaTim, TrigDmaInstance, new_trig_dma);
impl_new!(General4ChTim, General4ChInstance, new_general_4ch);
impl_new!(General32BitTim, General32BitInstance, new_general_32bit);
impl_new!(Advanced1ChTim, Advanced1ChInstance, new_advanced_1ch);
impl_new!(Advanced2ChTim, Advanced2ChInstance, new_advanced_2ch);
impl_new!(Advanced4ChTim, Advanced4ChInstance, new_advanced_4ch);

impl<'d, Tim: IsCoreTim> RawTimer<'d, Tim> {
    /// Get a pointer to the register block for this timer.
    ///
    /// This is a raw pointer to the register block. The actual register block layout varies
    /// depending on the timer type.
    pub fn regs(&self) -> *mut () {
        self.info.regs
    }

    /// Get the kernel clock frequency for this timer.
    ///
    /// Unless you switch the timer to a different clock source, this is the frequency that is fed
    /// into the prescaler to drive the timer.
    pub fn clock_frequency(&self) -> Hertz {
        self.kernel_clock
    }

    /// Get the type of the timer.
    ///
    /// Note that this returns the actual type of the timer peripheral, regardless of the `Tim`
    /// marker.
    pub fn timer_type(&self) -> TimerType {
        self.info.timer_type
    }

    /// Get the number of channels in this timer.
    ///
    /// Note that this returns the actual number of channels supported by the timer peripheral,
    /// regardless of the `Tim` marker.
    pub fn channel_count(&self) -> ChannelCount {
        self.info.timer_type.channel_count()
    }

    /// Get 32-bit registers, if the timer is a [`General32BitInstance`].
    ///
    /// This can be used to optionally use 32-bit counter resolution even if you don't know at
    /// runtime whether you are working with a 32-bit timer or not (i.e., if `Tim:
    /// IsGeneral32BitTim` does hold).
    #[cfg(not(timer_l0))]
    pub fn try_get_32bit_regs(&self) -> Option<pac::timer::Tim32bit> {
        if matches!(self.info.timer_type, TimerType::General32Bit) {
            Some(unsafe { pac::timer::Tim32bit::from_ptr(self.info.regs) })
        } else {
            None
        }
    }

    /// Ensure that outputs are enabled if this is an advanced timer.
    ///
    /// For advanced timers, it is necessary to set bit MOE in register BDTR to enable timer
    /// outputs. This method sets MOE if this is an advanced timer, and does nothing otherwise.
    /// You should use this method when writing generic drivers, to make sure that they work for
    /// both general-purpose and advanced timers.
    pub fn enable_outputs(&self) {
        #[cfg(not(timer_l0))]
        if self.info.timer_type.is_advanced() {
            let regs = unsafe { pac::timer::TimAdv1ch::from_ptr(self.info.regs) };
            regs.bdtr().modify(|w| w.set_moe(true));
        }
    }
}

impl<'d, Tim> Drop for RawTimer<'d, Tim> {
    fn drop(&mut self) {
        self.info.rcc.disable();
    }
}

macro_rules! reg {
    ($self:ident, $block:ident, $reg:ident) => {
        reg!($self, $block, $reg())
    };
    ($self:ident, $block:ident, $reg:ident $args:tt) => {
        unsafe { pac::timer::$block::from_ptr($self.info.regs) }.$reg $args
    }
}

macro_rules! reg_cast {
    ($($reg:ident),* as $reg_ty:ident, $access:ident) => {
        {
            let reg = reg!($($reg),*);
            unsafe { Reg::<regs::$reg_ty, $access>::from_ptr(reg.as_ptr() as *mut _) }
        }
    }
}

impl<'d, Tim> RawTimer<'d, Tim> {
    /// Control register 1.
    pub fn cr1_core(&self) -> Reg<regs::Cr1Core, RW>
    where
        Tim: IsCoreTim,
    {
        reg!(self, TimCore, cr1)
    }

    /// Control register 1.
    pub fn cr1_1ch(&self) -> Reg<regs::Cr11ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, cr1)
    }

    /// Control register 1.
    pub fn cr1_2ch(&self) -> Reg<regs::Cr12ch, RW>
    where
        Tim: IsGeneral2ChTim,
    {
        reg!(self, Tim2ch, cr1)
    }

    /// Control register 1.
    pub fn cr1_4ch(&self) -> Reg<regs::Cr14ch, RW>
    where
        Tim: IsGeneral4ChTim,
    {
        reg!(self, Tim4ch, cr1)
    }

    /// Control register 2.
    pub fn cr2_mms(&self) -> Reg<regs::Cr2Mms, RW>
    where
        Tim: IsMmsTim,
    {
        reg!(self, TimBasic, cr2)
    }

    /// Control register 2.
    pub fn cr2_ccdma(&self) -> Reg<regs::Cr2Ccdma, RW>
    where
        Tim: IsCcDmaTim,
    {
        reg_cast!(self, Tim4ch, cr2 as Cr2Ccdma, RW)
    }

    /// Control register 2.
    pub fn cr2_2ch(&self) -> Reg<regs::Cr22ch, RW>
    where
        Tim: IsGeneral2ChTim,
    {
        reg!(self, Tim2ch, cr2)
    }

    /// Control register 2.
    pub fn cr2_trigdma(&self) -> Reg<regs::Cr2Trigdma, RW>
    where
        Tim: IsTrigDmaTim,
    {
        reg!(self, Tim4ch, cr2)
    }

    /// Control register 2.
    #[cfg(not(timer_l0))]
    pub fn cr2_adv1ch(&self) -> Reg<regs::Cr2Adv1ch, RW>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, cr2)
    }

    /// Control register 2.
    #[cfg(not(timer_l0))]
    pub fn cr2_adv2ch(&self) -> Reg<regs::Cr2Adv2ch, RW>
    where
        Tim: IsAdvanced2ChTim,
    {
        reg!(self, TimAdv2ch, cr2)
    }

    /// Control register 2.
    #[cfg(not(timer_l0))]
    pub fn cr2_adv4ch(&self) -> Reg<regs::Cr2Adv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, cr2)
    }

    /// Slave mode control register.
    pub fn smcr_2ch(&self) -> Reg<regs::Smcr2ch, RW>
    where
        Tim: IsGeneral2ChTim,
    {
        reg!(self, Tim2ch, smcr)
    }

    /// Slave mode control register.
    pub fn smcr_4ch(&self) -> Reg<regs::Smcr4ch, RW>
    where
        Tim: IsGeneral4ChTim,
    {
        reg!(self, Tim4ch, smcr)
    }

    /// Slave mode control register.
    pub fn smcr_trigdma(&self) -> Reg<regs::SmcrTrigdma, RW>
    where
        Tim: IsTrigDmaTim,
    {
        reg_cast!(self, Tim4ch, smcr as SmcrTrigdma, RW)
    }

    /// Slave mode control register.
    #[cfg(not(timer_l0))]
    pub fn smcr_adv4ch(&self) -> Reg<regs::SmcrAdv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, smcr)
    }

    /// DMA/Interrupt enable register.
    pub fn dier_core(&self) -> Reg<regs::DierCore, RW>
    where
        Tim: IsCoreTim,
    {
        reg!(self, TimCore, dier)
    }

    /// DMA/Interrupt enable register
    pub fn dier_updma(&self) -> Reg<regs::DierUpdma, RW>
    where
        Tim: IsUpDmaTim,
    {
        reg!(self, TimBasic, dier)
    }

    /// DMA/Interrupt enable register.
    pub fn dier_1ch(&self) -> Reg<regs::Dier1ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, dier)
    }

    /// DMA/Interrupt enable register.
    pub fn dier_2ch(&self) -> Reg<regs::Dier2ch, RW>
    where
        Tim: IsGeneral2ChTim,
    {
        reg!(self, Tim2ch, dier)
    }

    /// DMA/Interrupt enable register.
    pub fn dier_ccdma(&self) -> Reg<regs::DierCcdma, RW>
    where
        Tim: IsCcDmaTim,
    {
        reg_cast!(self, Tim4ch, dier as DierCcdma, RW)
    }

    /// DMA/Interrupt enable register.
    pub fn dier_trigdma(&self) -> Reg<regs::DierTrigdma, RW>
    where
        Tim: IsTrigDmaTim,
    {
        reg_cast!(self, Tim4ch, dier as DierTrigdma, RW)
    }

    /// DMA/Interrupt enable register.
    pub fn dier_4ch(&self) -> Reg<regs::Dier4ch, RW>
    where
        Tim: IsGeneral4ChTim,
    {
        reg!(self, Tim4ch, dier)
    }

    /// DMA/Interrupt enable register.
    #[cfg(not(timer_l0))]
    pub fn dier_adv1ch(&self) -> Reg<regs::DierAdv1ch, RW>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, dier)
    }

    /// DMA/Interrupt enable register.
    #[cfg(not(timer_l0))]
    pub fn dier_adv2ch(&self) -> Reg<regs::DierAdv2ch, RW>
    where
        Tim: IsAdvanced2ChTim,
    {
        reg!(self, TimAdv2ch, dier)
    }

    /// DMA/Interrupt enable register.
    #[cfg(not(timer_l0))]
    pub fn dier_adv4ch(&self) -> Reg<regs::DierAdv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, dier)
    }

    /// Status register.
    pub fn sr_core(&self) -> Reg<regs::SrCore, RW>
    where
        Tim: IsCoreTim,
    {
        reg!(self, TimCore, sr)
    }

    /// Status register.
    pub fn sr_1ch(&self) -> Reg<regs::Sr1ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, sr)
    }

    /// Status register.
    pub fn sr_2ch(&self) -> Reg<regs::Sr2ch, RW>
    where
        Tim: IsGeneral2ChTim,
    {
        reg!(self, Tim2ch, sr)
    }

    /// Status register.
    pub fn sr_4ch(&self) -> Reg<regs::Sr4ch, RW>
    where
        Tim: IsGeneral4ChTim,
    {
        reg!(self, Tim4ch, sr)
    }

    /// Status register.
    #[cfg(not(timer_l0))]
    pub fn sr_adv1ch(&self) -> Reg<regs::SrAdv1ch, RW>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, sr)
    }

    /// Status register.
    #[cfg(not(timer_l0))]
    pub fn sr_adv2ch(&self) -> Reg<regs::SrAdv2ch, RW>
    where
        Tim: IsAdvanced2ChTim,
    {
        reg!(self, TimAdv2ch, sr)
    }

    /// Status register.
    #[cfg(not(timer_l0))]
    pub fn sr_adv4ch(&self) -> Reg<regs::SrAdv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, sr)
    }

    /// Event generation register.
    pub fn egr_core(&self) -> Reg<regs::EgrCore, W>
    where
        Tim: IsCoreTim,
    {
        reg!(self, TimCore, egr)
    }

    /// Event generation register.
    pub fn egr_1ch(&self) -> Reg<regs::Egr1ch, W>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, egr)
    }

    /// Event generation register.
    pub fn egr_2ch(&self) -> Reg<regs::Egr2ch, W>
    where
        Tim: IsGeneral2ChTim,
    {
        reg!(self, Tim2ch, egr)
    }

    /// Event generation register.
    #[cfg(not(timer_l0))]
    pub fn egr_adv1ch(&self) -> Reg<regs::EgrAdv1ch, W>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, egr)
    }

    /// Event generation register.
    #[cfg(not(timer_l0))]
    pub fn egr_adv2ch(&self) -> Reg<regs::EgrAdv2ch, W>
    where
        Tim: IsAdvanced2ChTim,
    {
        reg!(self, TimAdv2ch, egr)
    }

    /// Capture/compare mode register 1-2 (input mode), for `n` in `0..2` (one register per two
    /// channels).
    pub fn ccmr_input_1ch(&self, n: usize) -> Reg<regs::CcmrInput1ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, ccmr_input(n))
    }

    /// Capture/compare mode register 1-2 (output mode), for `n` in `0..2` (one register per two
    /// channels).
    pub fn ccmr_output_1ch(&self, n: usize) -> Reg<regs::CcmrOutput1ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, ccmr_output(n))
    }

    /// Capture/compare mode register 1-2 (output mode), for `n` in `0..2` (one register per two
    /// channels).
    pub fn ccmr_output_4ch(&self, n: usize) -> Reg<regs::CcmrOutput4ch, RW>
    where
        Tim: IsGeneral4ChTim,
    {
        reg!(self, Tim4ch, ccmr_output(n))
    }

    /// Capture/compare mode register 3 (output mode).
    ///
    /// This register is for channels 5 and 6, which can only be configured as output.
    #[cfg(not(timer_l0))]
    pub fn ccmr3_output_adv4ch(&self) -> Reg<regs::Ccmr3OutputAdv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, ccmr3_output)
    }

    /// Capture/compare enable register.
    pub fn ccer_1ch(&self) -> Reg<regs::Ccer1ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, ccer)
    }

    /// Capture/compare enable register.
    #[cfg(not(timer_l0))]
    pub fn ccer_adv1ch(&self) -> Reg<regs::CcerAdv1ch, RW>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, ccer)
    }

    /// Counter.
    pub fn cnt(&self) -> Reg<regs::CntCore, RW>
    where
        Tim: IsCoreTim,
    {
        reg!(self, TimCore, cnt)
    }

    /// Counter.
    #[cfg(not(timer_l0))]
    pub fn cnt_32bit(&self) -> Reg<u32, RW>
    where
        Tim: IsGeneral32BitTim,
    {
        reg!(self, Tim32bit, cnt)
    }

    /// Prescaler.
    pub fn psc(&self) -> Reg<u16, RW>
    where
        Tim: IsCoreTim,
    {
        reg!(self, TimCore, psc)
    }

    /// Auto-reload register.
    pub fn arr(&self) -> Reg<regs::ArrCore, RW>
    where
        Tim: IsCoreTim,
    {
        reg!(self, TimCore, arr)
    }

    /// Auto-reload register (dither mode enabled).
    #[cfg(timer_v2)]
    pub fn arr_dither(&self) -> Reg<regs::ArrDitherCore, RW>
    where
        Tim: IsCoreTim,
    {
        reg!(self, TimCore, arr_dither)
    }

    /// Auto-reload register.
    #[cfg(not(timer_l0))]
    pub fn arr_32bit(&self) -> Reg<u32, RW>
    where
        Tim: IsGeneral32BitTim,
    {
        reg!(self, Tim32bit, arr)
    }

    /// Repetition counter register.
    #[cfg(not(timer_l0))]
    pub fn rcr_adv1ch(&self) -> Reg<regs::RcrAdv1ch, RW>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, rcr)
    }

    /// Repetition counter register.
    #[cfg(not(timer_l0))]
    pub fn rcr_adv4ch(&self) -> Reg<regs::RcrAdv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, rcr)
    }

    /// Capture/compare register 1-4, for `n` in `0..4` (one register per channel).
    pub fn ccr(&self, n: usize) -> Reg<regs::Ccr1ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, ccr(n))
    }

    /// Capture/compare register 1-4 (dither mode enabled), for `n` in `0..4` (one register per
    /// channel).
    #[cfg(timer_v2)]
    pub fn ccr_dither(&self, n: usize) -> Reg<regs::CcrDither1ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, ccr_dither(n))
    }

    /// Capture/compare register 1-4, for `n` in `0..4` (one register per channel).
    #[cfg(not(timer_l0))]
    pub fn ccr_32bit(&self, n: usize) -> Reg<u32, RW>
    where
        Tim: IsGeneral32BitTim,
    {
        reg!(self, Tim32bit, ccr(n))
    }

    /// Capture/compare register 5.
    #[cfg(not(timer_l0))]
    pub fn ccr5(&self) -> Reg<regs::Ccr5Adv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, ccr5)
    }

    /// Capture/compare register 5 (dither mode enabled).
    #[cfg(timer_v2)]
    pub fn ccr5_dither(&self) -> Reg<regs::Ccr5DitherAdv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, ccr5_dither)
    }

    /// Capture/compare register 6.
    #[cfg(not(timer_l0))]
    pub fn ccr6(&self) -> Reg<regs::Ccr1ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, ccr6)
    }

    /// Capture/compare register 6 (dither mode enabled).
    #[cfg(timer_v2)]
    pub fn ccr6_dither(&self) -> Reg<regs::CcrDither1ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, ccr6_dither)
    }

    /// Break and dead-time register.
    #[cfg(not(timer_l0))]
    pub fn bdtr(&self) -> Reg<regs::BdtrAdv1ch, RW>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, bdtr)
    }

    /// DMA control register.
    pub fn dcr(&self) -> Reg<regs::DcrCcdma, RW>
    where
        Tim: IsCcDmaTim,
    {
        reg!(self, Tim4ch, dcr)
    }

    /// DMA address for full transfer.
    pub fn dmar(&self) -> Reg<u32, RW>
    where
        Tim: IsCcDmaTim,
    {
        reg!(self, Tim4ch, dmar)
    }

    /// Deadtime register 2.
    #[cfg(timer_v2)]
    pub fn dtr2(&self) -> Reg<regs::Dtr2Adv1ch, RW>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, dtr2)
    }

    /// Encoder control register.
    #[cfg(timer_v2)]
    pub fn ecr(&self) -> Reg<regs::Ecr4ch, RW>
    where
        Tim: IsGeneral4ChTim,
    {
        reg!(self, Tim4ch, ecr)
    }

    /// Alternate function register 1.
    #[cfg(not(timer_l0))]
    pub fn af1_4ch(&self) -> Reg<regs::Af14ch, RW>
    where
        Tim: IsGeneral4ChTim,
    {
        reg!(self, Tim4ch, af1)
    }

    /// Alternate function register 1.
    #[cfg(not(timer_l0))]
    pub fn af1_adv1ch(&self) -> Reg<regs::Af1Adv1ch, RW>
    where
        Tim: IsAdvanced1ChTim,
    {
        reg!(self, TimAdv1ch, af1)
    }

    /// Alternate function register 1.
    #[cfg(not(timer_l0))]
    pub fn af1_adv4ch(&self) -> Reg<regs::Af1Adv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, af1)
    }

    /// Alternate function register 2.
    #[cfg(timer_v2)]
    pub fn af2_ccdma(&self) -> Reg<regs::Af2Ccdma, RW>
    where
        Tim: IsCcDmaTim,
    {
        reg!(self, Tim4ch, af2)
    }

    /// Alternate function register 2.
    #[cfg(not(timer_l0))]
    pub fn af2_adv4ch(&self) -> Reg<regs::Af2Adv4ch, RW>
    where
        Tim: IsAdvanced4ChTim,
    {
        reg!(self, TimAdv4ch, af2)
    }

    /// Input selection register.
    #[cfg(not(timer_l0))]
    pub fn tisel(&self) -> Reg<regs::Tisel1ch, RW>
    where
        Tim: IsGeneral1ChTim,
    {
        reg!(self, Tim1ch, tisel)
    }
}
