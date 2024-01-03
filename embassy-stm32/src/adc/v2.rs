use core::borrow::{Borrow, BorrowMut};
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::Deref;
use core::task::Poll;

use embassy_futures::yield_now;
use embassy_hal_internal::{atomic_ring_buffer, into_ref, PeripheralRef};
use embassy_time::Instant;
use embedded_hal_02::blocking::delay::DelayUs;
use embedded_hal_02::blocking::i2c::Read;
use embedded_hal_02::can::Frame;
use futures::future::OrElse;
use stm32_metapac::adc::{self, vals};
use stm32_metapac::common::R;

use super::RxDma;
use crate::adc::{sample_time, ADCState, Adc, AdcPin, Instance, Resolution, SampleTime};
use crate::dma::ringbuffer::{DmaCtrl, ReadableDmaRingBuffer};
use crate::dma::{dma, Channel, DoubleBuffered, NoDma, ReadableRingBuffer, Transfer};
use crate::interrupt::typelevel;
use crate::peripherals::{ADC1, DMA1, DMA1_CH0, DMA1_CH1};
use crate::time::Hertz;
use crate::Peripheral;
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

// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

// impl<T: Instance> typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
//     unsafe fn on_interrupt() {
//         if T::regs().sr().read().eoc().to_bits() == 0x01 {
//             // T::regs().cr1().write(|w| w.set_eocie(false));
//             trace!("EOC interrupt");
//             T::regs().sr().write(|w| w.set_eoc(vals::Eoc::NOTCOMPLETE));
//             T::regs().sr().write(|w| w.set_strt(vals::Strt::NOTSTARTED));
//         } else {
//             let ovr = T::regs().sr().read().ovr();
//             let strt = T::regs().sr().read().strt();
//             trace!("ovr: {:?}, strt: {:?}", ovr.to_bits(), strt.to_bits());
//             return;
//         }

//         T::state().waker.wake();
//     }
// }

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

    pub async fn calibrate<RXDMA: RxDma<T>>(&mut self, adc: &mut Adc<'_, T, RXDMA>) -> Calibration {
        let vref_val = Adc::read(adc, self);
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

/// The state of a continuously running sampler
#[derive(PartialEq)]
pub enum SamplerState {
    Sampled,
    Stopped,
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
/* : crate::dma::Channel + crate::adc::RxDma<T>>*/
impl<'d, T: Instance, RXDMA> Adc<'d, T, RXDMA>
where
    RXDMA: RxDma<T> + Peripheral<P = RXDMA>,
{
    pub fn new(
        _adc: impl Peripheral<P = T> + 'd,
        rxdma: impl Peripheral<P = RXDMA> + 'd,
        data: &'static mut [u16],
        delay: &mut impl DelayUs<u32>,
    ) -> Self {
        into_ref!(_adc);
        into_ref!(rxdma);

        T::enable_and_reset();

        T::regs().cr2().modify(|reg| reg.set_adon(true));
        delay.delay_us((1_000_000 * 2) / Self::freq().0 + 1);

        Self {
            sample_time: SampleTime::Cycles480,
            calibrated_vdda: VDDA_CALIB_MV,
            phantom: PhantomData,
            transfer: None,
            rxdma: Some(rxdma),
            data,
            state: ADCState::Off,
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

    pub fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        self.set_channel_sample_sequence(&[pin.channel()]);
        self.convert()
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

        Vref::<T>(core::marker::PhantomData)
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

    /// Sets the sequence to sample the ADC. Must be less than  elements.
    pub async fn set_channel_sample_sequence(&self, sequence: &[u8]) {
        assert!(sequence.len() <= 8);
        let was_on = Self::is_on();
        if !was_on {
            self.start_adc().await;
        }
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

        if !was_on {
            self.stop_adc().await;
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
    fn convert(&mut self) -> u16 {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });
        while T::regs().sr().read().strt() == vals::Strt::NOTSTARTED {}
        while T::regs().sr().read().eoc() == vals::Eoc::NOTCOMPLETE {}

        T::regs().dr().read().0 as u16
    }

    pub fn start_read_continuous(&mut self)
    where
        RXDMA: RxDma<T>,
    {
        use crate::dma::{Burst, FlowControl, TransferOptions};
        let rxdma = match self.rxdma.take() {
            Some(rxdma) => rxdma,
            None => panic!("DMA already taken"),
        };

        let rx_request = rxdma.request();
        let rx_src = T::regs().dr().as_ptr() as *mut u16;
        let options = TransferOptions {
            pburst: Burst::Single,
            mburst: Burst::Single,
            flow_ctrl: FlowControl::Dma,
            fifo_threshold: None,
            circular: true,
            half_transfer_ir: false,
            complete_transfer_ir: true,
        };

        self.transfer = Some(unsafe { Transfer::new_read_raw(rxdma, rx_request, rx_src, self.data, options) });

        //Enable ADC
        let was_on = Self::is_on();
        if !was_on {
            T::regs().cr2().modify(|reg| {
                reg.set_adon(false);
            });
        }

        unsafe { Self::set_channel_sample_time(0, self.sample_time) };
        unsafe { Self::set_channel_sample_time(1, self.sample_time) };
        // unsafe { Self::set_channel_sample_time(2, self.sample_time) };
        // unsafe { Self::set_channel_sample_time(3, self.sample_time) };
        // unsafe { Self::set_channel_sample_time(4, self.sample_time) };
        // unsafe { Self::set_channel_sample_time(5, self.sample_time) };

        // Configure the channel to sample
        T::regs().sqr1().modify(|reg| reg.set_l(2));
        T::regs().sqr3().modify(|reg| {
            reg.set_sq(0, 0);
            reg.set_sq(1, 1);
            // reg.set_sq(2, 2);
            // reg.set_sq(3, 3);
            // reg.set_sq(4, 4);
            // reg.set_sq(5, 5);
        });

        T::regs().cr1().modify(|reg| {
            reg.set_scan(true);
            reg.set_discen(false);
            reg.set_eocie(true);
        });

        T::regs().cr2().modify(|reg| {
            reg.set_cont(vals::Cont::CONTINUOUS); //Goes with circular DMA
            reg.set_swstart(false);
            reg.set_dma(true);
            reg.set_dds(vals::Dds::CONTINUOUS);
            reg.set_eocs(vals::Eocs::EACHCONVERSION);
        });

        //Enable ADC
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });

        // while T::regs().sr().read().strt() != vals::Strt::STARTED {}

        //Loop for retrieving data
        // let mut buf_index = 0;
        // poll_fn(|cx: &mut core::task::Context<'_>| {
        //     dma.set_waker(cx.waker());

        //     let complete = dma.get_complete_count();
        //     let remaining_xfr = dma.get_remaining_transfers();
        //     let running = dma.is_running();

        //     // trace!(
        //     //     "n_complete: {}, remaining_xfr: {}, running: {}",
        //     //     complete,
        //     //     remaining_xfr,
        //     //     running
        //     // );

        //     // trace!("buf: {}", &data);

        //     if running {
        //         if complete > 0 {
        //             //trace!("Buffer0 accessible, reading Buffer0");

        //             //data.clone_from_slice(self.data);
        //             let sampler_state: SamplerState = sampler(&data);
        //             //dma.reset_complete_count();
        //             // T::regs().cr2().write(|reg| reg.set_swstart(true));

        //             if sampler_state == SamplerState::Sampled {
        //                 // buf_index = !buf_index & 0x01; // switch the buffer index (0/1)
        //                 return Poll::Pending;
        //             } else {
        //                 // dma.set_waker(cx.waker());

        //                 dma.reset_complete_count();
        //                 return Poll::Ready(());
        //             }
        //         } else {
        //             //trace!("Buffer0 not accessible, but DMA still running, Reading Buffer1");
        //             //  T::regs().cr2().write(|reg| reg.set_swstart(true));

        //             // data.clone_from_slice(self.data);
        //             let sampler_state: SamplerState = sampler(&data);

        //             if sampler_state == SamplerState::Sampled {
        //                 return Poll::Pending;
        //             } else {
        //                 return Poll::Ready(());
        //             }
        //         }
        //     } else {
        //         // trace!("DMA not running, transfer completed.");
        //         if remaining_xfr == 0 {
        //             return Poll::Ready(());
        //         } else {
        //             T::regs().cr2().write(|reg| reg.set_swstart(true));
        //             dma.reset_complete_count();
        //             return Poll::Pending;
        //         }
        //     }
        // })
        // .await;
        // dma.request_stop();
        // while dma.is_running() {}

        // trace!("buf: {}", self.data);
        // //Enable ADC
        // T::regs().cr2().modify(|reg| {
        //     reg.set_swstart(false);
        //     reg.set_adon(false);
        // });
        // }
    }

    pub fn get_dma_buf<const N: usize>(&self, buf: &mut [u16; N]) {
        // trace!("buf: {}", self.data[0]);
        while self.borrow().transfer.as_ref().unwrap().get_complete_count() < 1 {}
        buf.copy_from_slice(self.data);
    }
}

impl<'d, T: Instance, RXDMA: dma::Channel> Drop for Adc<'d, T, RXDMA> {
    fn drop(&mut self) {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(false);
        });

        T::disable();
    }
}
