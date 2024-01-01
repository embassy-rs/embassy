use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::AtomicU16;
use core::task::Poll;

use super::RxDma;
use embassy_futures::yield_now;
use embassy_hal_internal::interrupt::{InterruptExt, Priority};
use embassy_hal_internal::into_ref;
use embassy_time::Instant;
use embedded_hal_02::blocking::delay::DelayUs;
use stm32_metapac::adc::{self, vals};

use crate::adc::{Adc, AdcPin, Instance, Resolution, SampleTime};
use crate::interrupt::typelevel::Interrupt;
use crate::peripherals::ADC1;
use crate::time::Hertz;
use crate::{interrupt, Peripheral};

const ADC_FREQ: Hertz = crate::rcc::HSI_FREQ;

pub const VDDA_CALIB_MV: u32 = 3300;
pub const ADC_MAX: u32 = (1 << 12) - 1;
pub const VREF_INT: u32 = 1230;

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

/// ADC turn-on time
pub const ADC_POWERUP_TIME_US: u32 = 3;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().sr().read().eoc().to_bits() == 0x01 {
            T::regs().cr1().modify(|w| w.set_eocie(false));
            // ADC_DATA[0].store(T::regs().dr().read().data(), core::sync::atomic::Ordering::Relaxed);
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

fn update_vref<T: Instance>(op: i8) {
    static VREF_STATUS: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

    if op > 0 {
        if VREF_STATUS.fetch_add(1, core::sync::atomic::Ordering::SeqCst) == 0 {
            T::common_regs().ccr().modify(|w| w.set_tsvrefe(true));
        }
    } else {
        if VREF_STATUS.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) == 1 {
            T::common_regs().ccr().modify(|w| w.set_tsvrefe(false));
        }
    }
}

pub struct Vref<T: Instance>(core::marker::PhantomData<T>);
impl<T: Instance> AdcPin<T> for Vref<T> {}
impl<T: Instance> super::sealed::AdcPin<T> for Vref<T> {
    fn channel(&self) -> u8 {
        17
    }
}

impl<T: Instance> Vref<T> {
    /// The value that vref would be if vdda was at 3000mv
    pub fn calibrated_value(&self) -> u16 {
        ADC_MAX as u16
    }

    pub async fn calibrate(&mut self, adc: &mut Adc<'_, T>) -> Calibration {
        let vref_val = adc.read(self).await;
        Calibration {
            vref_cal: self.calibrated_value(),
            vref_val,
        }
    }
}

pub struct Calibration {
    vref_cal: u16,
    vref_val: u16,
}

impl Calibration {
    /// The millivolts that the calibration value was measured at
    pub const CALIBRATION_UV: u32 = 3_000_000;

    /// Returns the measured VddA in microvolts (uV)
    pub fn vdda_uv(&self) -> u32 {
        (Self::CALIBRATION_UV * self.vref_cal as u32) / self.vref_val as u32
    }

    /// Returns the measured VddA as an f32
    pub fn vdda_f32(&self) -> f32 {
        (Self::CALIBRATION_UV as f32 / 1_000.0) * (self.vref_cal as f32 / self.vref_val as f32)
    }

    /// Returns a calibrated voltage value as in microvolts (uV)
    pub fn cal_uv(&self, raw: u16, resolution: super::Resolution) -> u32 {
        (self.vdda_uv() / resolution.to_max_count()) * raw as u32
    }

    /// Returns a calibrated voltage value as an f32
    pub fn cal_f32(&self, raw: u16, resolution: super::Resolution) -> f32 {
        raw as f32 * self.vdda_f32() / resolution.to_max_count() as f32
    }
}

impl<T: Instance> Drop for Vref<T> {
    fn drop(&mut self) {
        update_vref::<T>(-1)
    }
}
pub struct Temperature<T: Instance>(core::marker::PhantomData<T>);
impl<T: Instance> AdcPin<ADC1> for Temperature<T> {}
impl<T: Instance> super::sealed::AdcPin<ADC1> for Temperature<T> {
    fn channel(&self) -> u8 {
        cfg_if::cfg_if! {
            if #[cfg(any(stm32f40, stm32f41))] {
                16
            } else {
                18
            }
        }
    }
}

impl<T: Instance> Temperature<T> {
    /// Time needed for temperature sensor readings to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

pub struct Vbat;
impl AdcPin<ADC1> for Vbat {}
impl super::sealed::AdcPin<ADC1> for Vbat {
    fn channel(&self) -> u8 {
        18
    }
}

enum Prescaler {
    Div2,
    Div4,
    Div6,
    Div8,
}

impl Prescaler {
    fn from_pclk2(freq: Hertz) -> Self {
        // Datasheet for both F4 and F7 specifies min frequency 0.6 MHz, typ freq. 30 MHz and max 36 MHz.
        const MAX_FREQUENCY: Hertz = Hertz(36_000_000);
        let raw_div = freq.0 / MAX_FREQUENCY.0;
        match raw_div {
            0..=1 => Self::Div2,
            2..=3 => Self::Div4,
            4..=5 => Self::Div6,
            6..=7 => Self::Div8,
            _ => panic!("Selected PCLK2 frequency is too high for ADC with largest possible prescaler."),
        }
    }

    fn adcpre(&self) -> crate::pac::adccommon::vals::Adcpre {
        match self {
            Prescaler::Div2 => crate::pac::adccommon::vals::Adcpre::DIV2,
            Prescaler::Div4 => crate::pac::adccommon::vals::Adcpre::DIV4,
            Prescaler::Div6 => crate::pac::adccommon::vals::Adcpre::DIV6,
            Prescaler::Div8 => crate::pac::adccommon::vals::Adcpre::DIV8,
        }
    }
}

impl<'d, T: Instance> Adc<'d, T>
where
    T: Instance,
{
    pub fn new(
        adc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        delay: &mut impl DelayUs<u32>,
    ) -> Self {
        into_ref!(adc);

        T::regs().cr2().modify(|reg| reg.set_adon(true));
        delay.delay_us((1_000_000 * 2) / Self::freq().0 + 1);

        T::enable_and_reset();
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Self {
            adc,
            sample_time: Default::default(),
        }
    }

    pub async fn set_resolution(&mut self, resolution: Resolution) {
        let was_on = Self::is_on();
        if was_on {
            self.stop_adc().await;
        }

        T::regs().cr1().modify(|w| w.set_res(resolution.into()));

        if was_on {
            self.start_adc().await;
        }
    }

    pub fn resolution(&self) -> Resolution {
        match T::regs().cr1().read().res() {
            crate::pac::adc::vals::Res::TWELVEBIT => Resolution::TwelveBit,
            crate::pac::adc::vals::Res::TENBIT => Resolution::TenBit,
            crate::pac::adc::vals::Res::EIGHTBIT => Resolution::EightBit,
            crate::pac::adc::vals::Res::SIXBIT => Resolution::SixBit,
        }
    }

    #[inline(always)]
    fn is_on() -> bool {
        T::regs().cr2().read().adon()
    }

    pub async fn start_adc(&self) {
        // defmt::trace!("Turn ADC on");
        T::regs().cr2().modify(|w| w.set_adon(true));
        //defmt::trace!("Waiting for ADC to turn on");

        let mut t = Instant::now();

        while !T::regs().cr2().read().adon() {
            yield_now().await;
            if t.elapsed() > embassy_time::Duration::from_millis(1000) {
                t = Instant::now();
                defmt::trace!("ADC still not on");
            }
        }

        // defmt::trace!("ADC on");
    }

    fn freq() -> Hertz {
        let div: u8 = T::common_regs().ccr().read().adcpre() as u8 + 1;
        ADC_FREQ / div as u32
    }

    pub async fn stop_adc(&self) {
        if T::regs().cr2().read().adon() {
            //   defmt::trace!("ADC should be on, wait for it to start");
            while !T::regs().cr2().read().adon() {
                yield_now().await;
            }
        }

        // defmt::trace!("Turn ADC off");

        T::regs().cr2().modify(|w| w.set_adon(false));

        // defmt::trace!("Waiting for ADC to turn off");

        while T::regs().cr2().read().adon() {
            yield_now().await;
        }
    }

    pub async fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        self.set_channel_sample_sequence(&[pin.channel()]).await;
        self.convert().await
    }

    async fn wait_sample_ready(&self) {
        // trace!("Waiting for sample channel to be ready");
        while T::regs().sr().read().strt() == adc::vals::Strt::STARTED {
            yield_now().await;
        }
    }

    /// Enables internal voltage reference and returns [VrefInt], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vref(&self) -> Vref<T> {
        update_vref::<T>(1);

        Vref(core::marker::PhantomData)
    }

    /// Enables internal temperature sensor and returns [Temperature], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    ///
    /// On STM32F42 and STM32F43 this can not be used together with [Vbat]. If both are enabled,
    /// temperature sensor will return vbat value.
    pub fn enable_temperature(&self) -> Temperature<T> {
        T::common_regs().ccr().modify(|w| w.set_tsvrefe(true));

        Temperature::<T>(core::marker::PhantomData)
    }

    /// Enables vbat input and returns [Vbat], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vbat(&self) -> Vbat {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbate(true);
        });

        Vbat {}
    }

    pub async fn set_sample_time(&mut self, pin: &mut impl AdcPin<T>, sample_time: SampleTime) {
        if Self::get_channel_sample_time(pin.channel()) != sample_time {
            let was_on = Self::is_on();
            if was_on {
                self.stop_adc().await;
            }
            unsafe {
                Self::set_channel_sample_time(pin.channel(), sample_time);
                trace!(
                    "Set sample time for channel {} to {:?}",
                    pin.channel(),
                    Self::get_channel_sample_time(pin.channel())
                );
            }
            if was_on {
                self.start_adc().await;
            }
        }
    }
    /// Sets the channel sample time
    ///
    /// ## SAFETY:
    /// - ADON == 0 i.e ADC must not be enabled when this is called.
    unsafe fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }

    fn set_channels_sample_time(&mut self, ch: &[u8], sample_time: SampleTime) {
        let ch_iter = ch.iter();
        for idx in ch_iter {
            unsafe {
                Self::set_channel_sample_time(*idx, sample_time);
            }
        }
    }

    fn get_channel_sample_time(ch: u8) -> SampleTime {
        match ch {
            0..=9 => T::regs().smpr2().read().smp(ch as _),
            10..=16 => T::regs().smpr1().read().smp(ch as usize - 10),
            _ => panic!("Invalid channel to sample"),
        }
        .into()
    }
    pub async fn read_sample_sequence(&mut self, sequence: &[u8]) -> [u16; 18] {
        let was_on = Self::is_on();
        if !was_on {
            self.start_adc().await;
        }

        T::regs().cr1().modify(|w| {
            w.set_eocie(false);
            w.set_scan(true);
        });
        self.sample_time = SampleTime::Cycles144;
        self.set_channel_sample_sequence(sequence).await;
        self.set_channels_sample_time(sequence, self.sample_time);

        T::regs().cr2().modify(|w| {
            w.set_swstart(true);
            w.set_cont(adc::vals::Cont::CONTINUOUS);
        }); // swstart cleared by HW

        T::regs().cr1().write(|w| w.set_eocie(true));

        let mut i = 0;
        let mut buf = [0u16; 18];
        let res = poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            if T::regs().sr().read().eoc() == adc::vals::Eoc::COMPLETE {
                buf[i] = T::regs().dr().read().data();
                T::regs().sr().write(|w| w.set_eoc(vals::Eoc::NOTCOMPLETE));
                i = i + 1;
                if i == sequence.len() {
                    Poll::Ready(buf)
                } else {
                    //Unmask interrupt after previous trigger & storing data.
                    T::regs().cr1().write(|w| w.set_eocie(true));

                    Poll::Pending
                }
            } else {
                Poll::Pending
            }
        })
        .await;

        T::regs().cr2().write(|w| {
            w.set_swstart(false);
        }); // swstart cleared by HW

        // T::regs().cr1().write(|w| w.set_eocie(true));
        if !was_on {
            self.stop_adc().await;
        }

        res
    }

    /// Sets the sequence to sample the ADC. Must be less than  elements.
    pub async fn set_channel_sample_sequence(&self, sequence: &[u8]) {
        assert!(sequence.len() <= 16);
        // trace!("Sequence Length: {}", sequence.len());
        let mut iter = sequence.iter();

        T::regs().sqr1().modify(|w| w.set_l((sequence.len() - 1) as _));
        for (idx, ch) in iter.by_ref().take(6).enumerate() {
            T::regs().sqr3().modify(|w| w.set_sq(idx, *ch));
        }
        for (idx, ch) in iter.by_ref().take(6).enumerate() {
            T::regs().sqr2().modify(|w| w.set_sq(idx, *ch));
        }
        for (idx, ch) in iter.by_ref().take(4).enumerate() {
            T::regs().sqr1().modify(|w| w.set_sq(idx, *ch));
        }
    }

    fn get_res_clks(res: Resolution) -> u32 {
        match res {
            Resolution::TwelveBit => 12,
            Resolution::TenBit => 11,
            Resolution::EightBit => 9,
            Resolution::SixBit => 7,
        }
    }

    fn get_sample_time_clks(sample_time: SampleTime) -> u32 {
        match sample_time {
            SampleTime::Cycles3 => 3,
            SampleTime::Cycles15 => 15,
            SampleTime::Cycles28 => 28,
            SampleTime::Cycles56 => 56,
            SampleTime::Cycles84 => 84,
            SampleTime::Cycles112 => 112,
            SampleTime::Cycles144 => 144,
            SampleTime::Cycles480 => 480,
        }
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        let res_clks = Self::get_res_clks(self.resolution());
        let us_clks = us * Self::freq().0 / 1_000_000;
        let clks = us_clks.saturating_sub(res_clks);
        match clks {
            0..=3 => SampleTime::Cycles3,
            4..=15 => SampleTime::Cycles15,
            16..=28 => SampleTime::Cycles28,
            29..=56 => SampleTime::Cycles56,
            57..=84 => SampleTime::Cycles84,
            85..=112 => SampleTime::Cycles112,
            113..=144 => SampleTime::Cycles144,
            _ => SampleTime::Cycles480,
        }
    }

    pub fn us_for_cfg(&self, res: Resolution, sample_time: SampleTime) -> u32 {
        let res_clks = Self::get_res_clks(res);
        let sample_clks = Self::get_sample_time_clks(sample_time);
        (res_clks + sample_clks) * 1_000_000 / Self::freq().0
    }

    pub fn ns_for_cfg(&self, res: Resolution, sample_time: SampleTime) -> u64 {
        let res_clks = Self::get_res_clks(res);
        let sample_clks = Self::get_sample_time_clks(sample_time);
        (res_clks + sample_clks) as u64 * 1_000_000_000 / Self::freq().0 as u64
    }

    /// Perform a single conversion.
    async fn convert(&mut self) -> u16 {
        let was_on = Self::is_on();

        if !was_on {
            self.start_adc().await;
        }

        self.wait_sample_ready().await;

        T::regs().sr().write(|_| {});
        T::regs().cr1().modify(|w| {
            w.set_eocie(true);
            w.set_scan(false);
        });
        T::regs().cr2().modify(|w| {
            w.set_swstart(true);
            w.set_cont(adc::vals::Cont::SINGLE);
        }); // swstart cleared by HW

        let res = poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::regs().sr().read().eoc() == adc::vals::Eoc::COMPLETE {
                let res = T::regs().dr().read().data();
                Poll::Ready(res)
            } else {
                Poll::Pending
            }
        })
        .await;

        if !was_on {
            self.stop_adc().await;
        }

        res
    }
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(false);
        });

        T::disable();
    }
}

// pub async fn read_continuous<S, const N: usize>(
//     &mut self,
//     pin: &mut impl AdcPin<T>,
//     data: &mut [[u16; N]; 2],
//     sampler: &mut S,
// ) where
//     S: FnMut(&[u16; N]) -> SamplerState,
//     RXDMA: RxDma<T>,
// {
//     let rx_request = self.rxdma.request();
//     let rx_src = T::regs().dr().ptr() as *mut u16;

//     unsafe {
//         self.rxdma
//             .start_circular_read(rx_request, rx_src, data.as_mut_ptr(), N * 2, Default::default());
//     }

//     unsafe {
//         Self::set_channel_sample_time(pin.channel(), self.sample_time);
//         T::regs().cr1().modify(|reg| {
//             reg.set_scan(false);
//             reg.set_discen(false);
//         });
//         T::regs().sqr1().modify(|reg| reg.set_l(0));

//         T::regs().cr2().modify(|reg| {
//             reg.set_cont(true); //Goes with circular DMA
//             reg.set_exttrig(true);
//             reg.set_swstart(false);
//             reg.set_extsel(crate::pac::adc::vals::Extsel::SWSTART);
//             reg.set_dma(true);
//         });
//     }

//     // Configure the channel to sample
//     unsafe { T::regs().sqr3().write(|reg| reg.set_sq(0, pin.channel())) }

//     //Enable ADC
//     unsafe {
//         T::regs().cr2().modify(|reg| {
//             reg.set_adon(true);
//             reg.set_swstart(true);
//         });
//     }
//     let mut buf_index = 0;
//     //Loop for retrieving data
//     poll_fn(|cx| {
//         self.rxdma.set_waker(cx.waker());

//         if self.rxdma.is_data_ready() {
//             let sampler_state = sampler(&data[buf_index]);
//             self.rxdma.set_data_processing_done();

//             if sampler_state == SamplerState::Sampled {
//                 buf_index = !buf_index & 0x01; // switch the buffer index (0/1)
//                 return Poll::Pending;
//             } else {
//                 return Poll::Ready(());
//             }
//         } else {
//             return Poll::Pending;
//         }
//     })
//     .await;

//     self.rxdma.request_stop();
//     while self.rxdma.is_running() {}
// }
