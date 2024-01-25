#![macro_use]
#![allow(missing_docs)] // TODO

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::into_ref;

use crate::gpio::sealed::AFType;
use crate::gpio::Speed;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::tsc::regs::{Ier, Ioccr, Iogcsr, Iohcr, Ioscr};
use crate::peripherals;
use crate::{interrupt, Peripheral};

/// Touch Sensor Controller driver.
pub struct Tsc<'d, T: Instance> {
    _tsc: crate::PeripheralRef<'d, T>,
}

pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().isr().read().eoaf() {
            T::regs().ier().modify(|w| w.set_eoaie(false));
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

#[derive(Copy, Clone)]
pub enum ChargeTransferPulseDuration {
    _1x,
    _2x,
    _3x,
    _4x,
    _5x,
    _6x,
    _7x,
    _8x,
    _9x,
    _10x,
    _11x,
    _12x,
    _13x,
    _14x,
    _15x,
    _16x,
}

impl Into<u8> for ChargeTransferPulseDuration {
    fn into(self) -> u8 {
        match self {
            ChargeTransferPulseDuration::_1x => 0b0000,
            ChargeTransferPulseDuration::_2x => 0b0001,
            ChargeTransferPulseDuration::_3x => 0b0010,
            ChargeTransferPulseDuration::_4x => 0b0011,
            ChargeTransferPulseDuration::_5x => 0b0100,
            ChargeTransferPulseDuration::_6x => 0b0101,
            ChargeTransferPulseDuration::_7x => 0b0110,
            ChargeTransferPulseDuration::_8x => 0b0111,
            ChargeTransferPulseDuration::_9x => 0b1000,
            ChargeTransferPulseDuration::_10x => 0b1001,
            ChargeTransferPulseDuration::_11x => 0b1010,
            ChargeTransferPulseDuration::_12x => 0b1011,
            ChargeTransferPulseDuration::_13x => 0b1100,
            ChargeTransferPulseDuration::_14x => 0b1101,
            ChargeTransferPulseDuration::_15x => 0b1110,
            ChargeTransferPulseDuration::_16x => 0b1111,
        }
    }
}

#[derive(Copy, Clone)]
pub enum SpreadSpectrumPrescaler {
    Div1,
    Div2,
}

impl Into<bool> for SpreadSpectrumPrescaler {
    fn into(self) -> bool {
        match self {
            SpreadSpectrumPrescaler::Div1 => false,
            SpreadSpectrumPrescaler::Div2 => true,
        }
    }
}

#[derive(Copy, Clone)]
pub enum PulseGeneratorPrescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

impl Into<u8> for PulseGeneratorPrescaler {
    fn into(self) -> u8 {
        match self {
            PulseGeneratorPrescaler::Div1 => 0b000,
            PulseGeneratorPrescaler::Div2 => 0b001,
            PulseGeneratorPrescaler::Div4 => 0b010,
            PulseGeneratorPrescaler::Div8 => 0b011,
            PulseGeneratorPrescaler::Div16 => 0b100,
            PulseGeneratorPrescaler::Div32 => 0b101,
            PulseGeneratorPrescaler::Div64 => 0b110,
            PulseGeneratorPrescaler::Div128 => 0b111,
        }
    }
}

#[derive(Copy, Clone)]
pub enum MaxCount {
    _255Samples,
    _511Samples,
    _1023Samples,
    _2047Samples,
    _4095Samples,
    _8191Samples,
    _16383Samples,
}

impl Into<u8> for MaxCount {
    fn into(self) -> u8 {
        match self {
            MaxCount::_255Samples => 0b000,
            MaxCount::_511Samples => 0b001,
            MaxCount::_1023Samples => 0b010,
            MaxCount::_2047Samples => 0b011,
            MaxCount::_4095Samples => 0b100,
            MaxCount::_8191Samples => 0b101,
            MaxCount::_16383Samples => 0b110,
        }
    }
}

#[derive(Copy, Clone)]
pub enum PinMode {
    OutputPushPullLow,
    InputFloating,
}

impl Into<bool> for PinMode {
    fn into(self) -> bool {
        match self {
            PinMode::OutputPushPullLow => false,
            PinMode::InputFloating => true,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Polarity {
    FallingEdge,
    RisingHigh,
}

impl Into<bool> for Polarity {
    fn into(self) -> bool {
        match self {
            Polarity::FallingEdge => false,
            Polarity::RisingHigh => true,
        }
    }
}

#[derive(Copy, Clone)]
pub enum AcquisitionMode {
    Normal,
    Synchronized,
}

impl Into<bool> for AcquisitionMode {
    fn into(self) -> bool {
        match self {
            AcquisitionMode::Normal => false,
            AcquisitionMode::Synchronized => true,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Config {
    pub transfer_pulse_high: ChargeTransferPulseDuration,
    pub transfer_pulse_low: ChargeTransferPulseDuration,
    pub spread_spectrum_enable: bool,
    pub spread_spectrum_deviation: u8,
    pub spread_spectrum_prescaler: SpreadSpectrumPrescaler,
    pub pulse_generator_prescaler: PulseGeneratorPrescaler,
    pub max_count_value: MaxCount,
    pub io_default_mode: PinMode,
    pub sync_polarity: Polarity,
    pub acquisition_mode: AcquisitionMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            transfer_pulse_high: ChargeTransferPulseDuration::_1x,
            transfer_pulse_low: ChargeTransferPulseDuration::_1x,
            spread_spectrum_enable: false,
            spread_spectrum_deviation: 0,
            spread_spectrum_prescaler: SpreadSpectrumPrescaler::Div1,
            pulse_generator_prescaler: PulseGeneratorPrescaler::Div1,
            max_count_value: MaxCount::_255Samples,
            io_default_mode: PinMode::OutputPushPullLow,
            sync_polarity: Polarity::FallingEdge,
            acquisition_mode: AcquisitionMode::Normal,
        }
    }
}

impl<'d, T: Instance> Tsc<'d, T> {
    pub fn new(
        tsc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tsc);
        T::enable_and_reset();

        T::regs().cr().modify(|w| w.set_tsce(true));

        // Configure the control register according to the Config values
        T::regs().cr().modify(|w| {
            w.set_ctph(config.transfer_pulse_high.into());
            w.set_ctpl(config.transfer_pulse_low.into());
            w.set_sse(config.spread_spectrum_enable.into());
            w.set_ssd(config.spread_spectrum_deviation.into());
            w.set_sspsc(config.spread_spectrum_prescaler.into());
            w.set_pgpsc(config.pulse_generator_prescaler.into());
            w.set_mcv(config.max_count_value.into());
            w.set_iodef(config.io_default_mode.into());
            w.set_syncpol(config.sync_polarity.into());
            w.set_am(config.acquisition_mode.into());
        });

        // Ensure the rest of the registers are at their reset values
        T::regs().ier().write_value(Ier::default());
        T::regs().iohcr().write_value(Iohcr::default());
        T::regs().ioccr().write_value(Ioccr::default());
        T::regs().ioscr().write_value(Ioscr::default());
        T::regs().iogcsr().write_value(Iogcsr::default());

        T::regs().ier().modify(|w| w.set_eoaie(true));
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Self { _tsc: tsc }
    }

    pub fn include(&self, sampling_pin: impl TscPin<T>, channel_pin: impl TscPin<T>) {
        let sampling_mask = 0x1 << (sampling_pin.group() - 1) * 4 << (sampling_pin.channel() - 1);
        let channel_mask = 0x1 << (channel_pin.group() - 1) * 4 << (channel_pin.channel() - 1);

        let val = T::regs().iohcr().read().0;
        T::regs()
            .iohcr()
            .write_value(Iohcr(val & !sampling_mask & !channel_mask));

        let val = T::regs().ioscr().read().0;
        T::regs().ioscr().write_value(Ioscr(val | sampling_mask));

        let val = T::regs().ioccr().read().0;
        T::regs().ioccr().write_value(Ioccr(val | channel_mask));

        let val = T::regs().iogcsr().read().0;
        T::regs()
            .iogcsr()
            .write_value(Iogcsr(val | (0x1 << (sampling_pin.group() - 1))));

        // This should be condition on the default I/O state configuration, but :shrug:
        sampling_pin.set_as_af(sampling_pin.af(), AFType::OutputOpenDrain);
        sampling_pin.set_speed(Speed::Low);
        channel_pin.set_as_af(channel_pin.af(), AFType::OutputPushPull);
        channel_pin.set_speed(Speed::Low);
    }

    pub async fn read(&mut self) -> [u16; 7] {
        // Request a new acquisition
        T::regs().cr().modify(|w| w.set_start(true));

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::regs().isr().read().eoaf() {
                // Make sure the interrupt is cleared
                T::regs().icr().modify(|w| w.set_eoaic(true));
                T::regs().ier().modify(|w| w.set_eoaie(true));
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        let mut result = [0; 7];
        for i in 0..7 {
            result[i] = T::regs().iogcr(i).read().cnt();
        }

        result
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    pub struct State {
        pub waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                waker: AtomicWaker::new(),
            }
        }
    }

    pub trait InterruptableInstance {
        type Interrupt: crate::interrupt::typelevel::Interrupt;
    }

    pub trait Instance: InterruptableInstance {
        fn regs() -> crate::pac::tsc::Tsc;
        fn state() -> &'static State;
    }

    pub trait TscPin<T: Instance> {
        fn group(&self) -> u8;
        fn channel(&self) -> u8;
        fn af(&self) -> u8;
    }
}

pub trait Instance: sealed::Instance + crate::Peripheral<P = Self> + crate::rcc::RccPeripheral + 'static {}
pub trait TscPin<T: Instance>: sealed::TscPin<T> + crate::gpio::sealed::Pin {}

pin_trait!(G1IO1Pin, Instance);
pin_trait!(G1IO2Pin, Instance);
pin_trait!(G1IO3Pin, Instance);
pin_trait!(G1IO4Pin, Instance);
pin_trait!(G2IO1Pin, Instance);
pin_trait!(G2IO2Pin, Instance);
pin_trait!(G2IO3Pin, Instance);
pin_trait!(G2IO4Pin, Instance);
pin_trait!(G3IO1Pin, Instance);
pin_trait!(G3IO2Pin, Instance);
pin_trait!(G3IO3Pin, Instance);
pin_trait!(G3IO4Pin, Instance);
pin_trait!(G4IO1Pin, Instance);
pin_trait!(G4IO2Pin, Instance);
pin_trait!(G4IO3Pin, Instance);
pin_trait!(G4IO4Pin, Instance);
pin_trait!(G5IO1Pin, Instance);
pin_trait!(G5IO2Pin, Instance);
pin_trait!(G5IO3Pin, Instance);
pin_trait!(G5IO4Pin, Instance);
pin_trait!(G6IO1Pin, Instance);
pin_trait!(G6IO2Pin, Instance);
pin_trait!(G6IO3Pin, Instance);
pin_trait!(G6IO4Pin, Instance);
pin_trait!(G7IO1Pin, Instance);
pin_trait!(G7IO2Pin, Instance);
pin_trait!(G7IO3Pin, Instance);
pin_trait!(G7IO4Pin, Instance);

macro_rules! impl_tsc_pin {
    ($inst:ident, $pin:ident, $grp:expr, $ch:expr, $af:expr) => {
        impl crate::tsc::TscPin<peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::tsc::sealed::TscPin<peripherals::$inst> for crate::peripherals::$pin {
            fn group(&self) -> u8 {
                $grp
            }

            fn channel(&self) -> u8 {
                $ch
            }

            fn af(&self) -> u8 {
                $af
            }
        }
    };
}

foreach_interrupt!(
    ($inst:ident,tsc,TSC,GLOBAL,$irq:ident) => {
        impl crate::tsc::sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::tsc::Tsc {
                crate::pac::$inst
            }

            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
            }
        }

        impl sealed::InterruptableInstance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl crate::tsc::Instance for peripherals::$inst {}
    };
);
