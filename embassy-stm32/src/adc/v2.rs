use core::borrow::BorrowMut;
use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral};
use embassy_time::Timer;
use embedded_hal_02::blocking::delay::DelayUs;
use stm32_metapac::adc::vals;

use crate::adc::{Adc, AdcPin, Instance, Resolution, RxDma, SampleTime};
use crate::dma::ringbuffer::OverrunError;
use crate::dma::{self, ReadableRingBuffer, Transfer};
use crate::interrupt;
use crate::peripherals::ADC1;
use crate::time::Hertz;

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
        if T::regs().sr().read().eoc() {
            T::regs().sr().modify(|w| w.set_eoc(false));
        } else {
            return;
        }
        T::state().waker.wake();
    }
}

pub struct VrefInt;
impl AdcPin<ADC1> for VrefInt {}
impl super::SealedAdcPin<ADC1> for VrefInt {
    fn channel(&self) -> u8 {
        17
    }
}

impl VrefInt {
    /// Time needed for internal voltage reference to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

pub struct Temperature;
impl AdcPin<ADC1> for Temperature {}
impl super::SealedAdcPin<ADC1> for Temperature {
    fn channel(&self) -> u8 {
        cfg_if::cfg_if! {
            if #[cfg(any(stm32f2, stm32f40, stm32f41))] {
                16
            } else {
                18
            }
        }
    }
}

impl Temperature {
    /// Time needed for temperature sensor readings to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub enum Sequence {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
    Sixteen,
}

impl From<Sequence> for u8 {
    fn from(s: Sequence) -> u8 {
        match s {
            Sequence::One => 0,
            Sequence::Two => 1,
            Sequence::Three => 2,
            Sequence::Four => 3,
            Sequence::Five => 4,
            Sequence::Six => 5,
            Sequence::Seven => 6,
            Sequence::Eight => 7,
            Sequence::Nine => 8,
            Sequence::Ten => 9,
            Sequence::Eleven => 10,
            Sequence::Twelve => 11,
            Sequence::Thirteen => 12,
            Sequence::Fourteen => 13,
            Sequence::Fifteen => 14,
            Sequence::Sixteen => 15,
        }
    }
}

impl Into<Sequence> for u8 {
    fn into(self) -> Sequence {
        match self {
            0 => Sequence::One,
            1 => Sequence::Two,
            2 => Sequence::Three,
            3 => Sequence::Four,
            4 => Sequence::Five,
            5 => Sequence::Six,
            6 => Sequence::Seven,
            7 => Sequence::Eight,
            8 => Sequence::Nine,
            9 => Sequence::Ten,
            10 => Sequence::Eleven,
            11 => Sequence::Twelve,
            12 => Sequence::Thirteen,
            13 => Sequence::Fourteen,
            14 => Sequence::Fifteen,
            15 => Sequence::Sixteen,
            _ => panic!("Invalid sequence number"),
        }
    }
}

pub struct Vbat;
impl AdcPin<ADC1> for Vbat {}
impl super::SealedAdcPin<ADC1> for Vbat {
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
        // Datasheet for F2 specifies min frequency 0.6 MHz, and max 30 MHz (with VDDA 2.4-3.6V).
        #[cfg(stm32f2)]
        const MAX_FREQUENCY: Hertz = Hertz(30_000_000);
        // Datasheet for both F4 and F7 specifies min frequency 0.6 MHz, typ freq. 30 MHz and max 36 MHz.
        #[cfg(not(stm32f2))]
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

impl<'d, T> Adc<'d, T>
where
    T: Instance,
{
    pub fn new(adc: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        into_ref!(adc);
        T::enable_and_reset();

        let presc = Prescaler::from_pclk2(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_adcpre(presc.adcpre()));
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
        });

        delay.delay_us(ADC_POWERUP_TIME_US);

        Self {
            adc,
            sample_time: SampleTime::from_bits(0),
        }
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub async fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        self.set_channel_sample_sequence(&[pin.channel()]).await;
        self.convert()
    }

    fn is_on() -> bool {
        T::regs().cr2().read().adon()
    }

    fn stop_adc() {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(false);
        });
    }

    fn start_adc() {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
        });

        // Wait for ADC to power up
        // delay.delay_us(ADC_POWERUP_TIME_US);
    }
    /// Enables internal voltage reference and returns [VrefInt], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vref(&self) -> VrefInt {
        // VrefInt

        VrefInt
    }

    /// Enables internal temperature sensor and returns [Temperature], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    ///
    /// On STM32F42 and STM32F43 this can not be used together with [Vbat]. If both are enabled,
    /// temperature sensor will return vbat value.
    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|w| w.set_tsvrefe(true));

        Temperature
    }

    /// Enables vbat input and returns [Vbat], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    // pub fn enable_vbat(&self) -> Vbat {
    //     T::common_regs().ccr().modify(|reg| {
    //         reg.set_vbate(true);
    //     });

    //     Vbat {}
    // }

    pub async fn set_pin_sample_time(&mut self, pin: &mut impl AdcPin<T>, sample_time: SampleTime) {
        if Self::get_channel_sample_time(pin.channel()) != sample_time {
            let was_on = Self::is_on();
            if was_on {
                Self::stop_adc();
            }
            unsafe {
                Self::set_channel_sample_time(pin.channel(), sample_time);
                // trace!(
                // "Set sample time for channel {} to {:?}",
                // pin.channel(),
                // Self::get_channel_sample_time(pin.channel() as u8)
                // );
            }
            if was_on {
                Self::start_adc();
            }
            // This will make CI pass, but the struct field is no longer relevant as each channel will have an associated sample time.
            self.sample_time = sample_time;
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
    pub async fn set_sample_sequence(
        &mut self,
        sequence: Sequence,
        channel: &mut impl AdcPin<T>,
        sample_time: SampleTime,
    ) {
        let was_on = Self::is_on();
        if !was_on {
            Self::start_adc();
        }

        //Check the sequence is long enough
        T::regs().sqr1().modify(|r| {
            let prev: Sequence = r.l().into();
            trace!("Previous sequence length: {:?}", prev as u8);
            if prev < sequence {
                let new_l: Sequence = sequence.into();
                trace!("Setting sequence length to {:?}", new_l as u8);
                r.set_l(sequence.into())
            } else {
                r.set_l(prev.into())
            }
        });

        //Set this GPIO as an analog input.
        channel.set_as_analog();

        //Set the channel in the right sequence field.
        match sequence {
            Sequence::One => T::regs().sqr3().modify(|w| w.set_sq(0, channel.channel())),
            Sequence::Two => T::regs().sqr3().modify(|w| w.set_sq(1, channel.channel())),
            Sequence::Three => T::regs().sqr3().modify(|w| w.set_sq(2, channel.channel())),
            Sequence::Four => T::regs().sqr3().modify(|w| w.set_sq(3, channel.channel())),
            Sequence::Five => T::regs().sqr3().modify(|w| w.set_sq(4, channel.channel())),
            Sequence::Six => T::regs().sqr3().modify(|w| w.set_sq(5, channel.channel())),
            Sequence::Seven => T::regs().sqr2().modify(|w| w.set_sq(6, channel.channel())),
            Sequence::Eight => T::regs().sqr2().modify(|w| w.set_sq(7, channel.channel())),
            Sequence::Nine => T::regs().sqr2().modify(|w| w.set_sq(8, channel.channel())),
            Sequence::Ten => T::regs().sqr2().modify(|w| w.set_sq(9, channel.channel())),
            Sequence::Eleven => T::regs().sqr2().modify(|w| w.set_sq(10, channel.channel())),
            Sequence::Twelve => T::regs().sqr2().modify(|w| w.set_sq(11, channel.channel())),
            Sequence::Thirteen => T::regs().sqr1().modify(|w| w.set_sq(12, channel.channel())),
            Sequence::Fourteen => T::regs().sqr1().modify(|w| w.set_sq(13, channel.channel())),
            Sequence::Fifteen => T::regs().sqr1().modify(|w| w.set_sq(14, channel.channel())),
            Sequence::Sixteen => T::regs().sqr1().modify(|w| w.set_sq(15, channel.channel())),
        };

        if !was_on {
            Self::stop_adc();
        }

        self.set_channels_sample_time(&[channel.channel()], sample_time);
    }
    /// Sets the sequence to sample the ADC. Must be less than  elements.
    pub async fn set_channel_sample_sequence(&self, sequence: &[u8]) {
        assert!(sequence.len() <= 8);
        let was_on = Self::is_on();
        if !was_on {
            Self::start_adc();
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
            Self::stop_adc();
        }
    }

    fn get_res_clks(res: Resolution) -> u32 {
        match res {
            Resolution::BITS12 => 12,
            Resolution::BITS10 => 11,
            Resolution::BITS8 => 9,
            Resolution::BITS6 => 7,
        }
    }

    fn get_sample_time_clks(sample_time: SampleTime) -> u32 {
        match sample_time {
            SampleTime::CYCLES3 => 3,
            SampleTime::CYCLES15 => 15,
            SampleTime::CYCLES28 => 28,
            SampleTime::CYCLES56 => 56,
            SampleTime::CYCLES84 => 84,
            SampleTime::CYCLES112 => 112,
            SampleTime::CYCLES144 => 144,
            SampleTime::CYCLES480 => 480,
        }
    }

    // pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
    //     let res_clks = Self::get_res_clks(self.());
    //     let us_clks = us * Self::freq().0 / 1_000_000;
    //     let clks = us_clks.saturating_sub(res_clks);
    //     match clks {
    //         0..=3 => SampleTime::CYCLES3,
    //         4..=15 => SampleTime::CYCLES15,
    //         16..=28 => SampleTime::CYCLES28,
    //         29..=56 => SampleTime::CYCLES56,
    //         57..=84 => SampleTime::CYCLES84,
    //         85..=112 => SampleTime::CYCLES112,
    //         113..=144 => SampleTime::CYCLES144,
    //         _ => SampleTime::CYCLES480,
    //     }
    // }

    // pub fn us_for_cfg(&self, res: Resolution, sample_time: SampleTime) -> u32 {
    //     let res_clks = Self::get_res_clks(res);
    //     let sample_clks = Self::get_sample_time_clks(sample_time);
    //     (res_clks + sample_clks) * 1_000_000 / Self::freq().0
    // }

    // pub fn ns_for_cfg(&self, res: Resolution, sample_time: SampleTime) -> u64 {
    //     let res_clks = Self::get_res_clks(res);
    //     let sample_clks = Self::get_sample_time_clks(sample_time);
    //     (res_clks + sample_clks) as u64 * 1_000_000_000 / Self::freq().0 as u64
    // }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        // clear end of conversion flag
        T::regs().sr().modify(|reg| {
            reg.set_eoc(false);
        });

        // Start conversion
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });

        while T::regs().sr().read().strt() == false {
            // spin //wait for actual start
        }
        while T::regs().sr().read().eoc() == false {
            // spin //wait for finish
        }

        T::regs().dr().read().0 as u16
    }

    pub fn start_read_continuous(
        &mut self,
        rxdma: impl RxDma<T>,
        data: &'static mut [u16],
    ) -> ReadableRingBuffer<'static, u16> {
        use crate::dma::{Burst, FlowControl, TransferOptions};
        let rx_src = T::regs().dr().as_ptr() as *mut u16;
        let options: TransferOptions = TransferOptions {
            pburst: Burst::Single,
            mburst: Burst::Single,
            flow_ctrl: FlowControl::Dma,
            priority: dma::Priority::High,
            fifo_threshold: None,
            circular: true,
            half_transfer_ir: true,
            complete_transfer_ir: false,
        };
        let req = rxdma.request();

        let mut transfer = unsafe { ReadableRingBuffer::new(rxdma, req, rx_src, data, options) };
        transfer.start();
        // let mut transfer = unsafe { dma::Transfer::new_read_raw(rxdma, req, rx_src, data, options) };
        // transfer.

        while !transfer.is_running() {}

        //Enable ADC
        let was_on = Self::is_on();
        if !was_on {
            T::regs().cr2().modify(|reg| {
                reg.set_adon(false);
            });
        }

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

        info!("ADC started");
        return transfer;
    }

    pub async fn get_dma_buf<const N: usize>(
        &self,
        transfer: &mut ReadableRingBuffer<'static, u16>,
    ) -> Result<[u16; N], OverrunError> {
        // info!("Getting DMA buffer");

        // while transfer.capacity() > N {

        //     Timer::after_micros(1).await;
        //     // wait for data
        // }
        // info!("ADC stopped");
        // Stop ADC conversions
        // T::regs().cr2().modify(|reg| {
        //     reg.set_adon(false);
        //     reg.set_swstart(false);
        //     reg.set_dma(false);
        // });

        // transfer.request_stop();
        // transfer
        // info!("{}", transfer.get_remaining_transfers());
        // transfer.request_stop();
        // while transfer.is_running() {}

        let mut data_buf = [0u16; N];
        loop {
            match transfer.read_exact(&mut data_buf).await {
                Ok(r) => {
                    return Ok(data_buf);
                }
                Err(_) => {
                    Timer::after_micros(1).await;
                    transfer.clear();
                }
            }
        }
    }

    pub async fn stop_continuous_conversion(&mut self) {
        T::regs().cr2().write(|reg| reg.set_adon(false));
        T::regs().cr2().modify(|reg| {
            reg.set_swstart(false);
            reg.set_dma(false);
        });
        T::regs().cr1().modify(|reg| {
            reg.set_eocie(false);
        });

        while T::regs().cr2().read().adon() {}
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
