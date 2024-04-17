//! TSC Peripheral Interface

#![macro_use]

pub mod enums;

use crate::gpio::AnyPin;
use crate::{pac::tsc::Tsc as Regs, rcc::RccPeripheral};
use crate::{peripherals, Peripheral};
use embassy_hal_internal::{into_ref, PeripheralRef};

pub use enums::*;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Test error for TSC
    Test,
}

pub enum PinType {
    Channel,
    Sample,
    Shield,
}

pub struct TscGroup {}

pub enum State {
    Reset,
    Ready,
    Busy,
    Error,
}

pub enum GroupStatus {
    Ongoing,
    Complete,
}

pub enum Group {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl Into<usize> for Group {
    fn into(self) -> usize {
        match self {
            Group::One => 0,
            Group::Two => 1,
            Group::Three => 2,
            Group::Four => 3,
            Group::Five => 4,
            Group::Six => 5,
            Group::Seven => 6,
            Group::Eight => 7,
        }
    }
}

pub struct Config {
    pub ct_pulse_high_length: ChargeTransferPulseCycle,
    pub ct_pulse_low_length: ChargeTransferPulseCycle,
    pub spread_spectrum: bool,
    pub spread_spectrum_deviation: u8,
    pub spread_spectrum_prescaler: bool,
    pub pulse_generator_prescaler: PGPrescalerDivider,
    pub max_count_value: u8,
    pub io_default_mode: bool,
    pub synchro_pin_polarity: bool,
    pub acquisition_mode: bool,
    pub max_count_interrupt: bool,
    pub channel_ios: u32,
    pub shield_ios: u32,
    pub sampling_ios: u32,
}

pub struct TscPin<'d, T> {
    pin: PeripheralRef<'d, AnyPin>,
    role: PinType,
}

pub struct PinGroup<'d, A, B, C, D> {
    d1: Option<TscPin<'d, A>>,
    d2: Option<TscPin<'d, B>>,
    d3: Option<TscPin<'d, C>>,
    d4: Option<TscPin<'d, D>>,
}

pub struct TSC<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
    g1: Option<PinGroup<'d, AnyPin, AnyPin, AnyPin, AnyPin>>,
    g2: Option<PinGroup<'d, AnyPin, AnyPin, AnyPin, AnyPin>>,
    g3: Option<PinGroup<'d, AnyPin, AnyPin, AnyPin, AnyPin>>,
    g4: Option<PinGroup<'d, AnyPin, AnyPin, AnyPin, AnyPin>>,
    g5: Option<PinGroup<'d, AnyPin, AnyPin, AnyPin, AnyPin>>,
    g6: Option<PinGroup<'d, AnyPin, AnyPin, AnyPin, AnyPin>>,
    g7: Option<PinGroup<'d, AnyPin, AnyPin, AnyPin, AnyPin>>,
    g8: Option<PinGroup<'d, AnyPin, AnyPin, AnyPin, AnyPin>>,
    state: State,
    config: Config,
}

impl<'d, T: Instance> TSC<'d, T> {
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        g1: Option<
            PinGroup<
                'd,
                impl Peripheral<P = impl G1IO1Pin<T>> + 'd,
                impl Peripheral<P = impl G1IO2Pin<T>> + 'd,
                impl Peripheral<P = impl G1IO3Pin<T>> + 'd,
                impl Peripheral<P = impl G1IO4Pin<T>> + 'd,
            >,
        >,

        g2_d1: Option<impl Peripheral<P = impl G2IO1Pin<T>> + 'd>,
        g2_d2: Option<impl Peripheral<P = impl G2IO2Pin<T>> + 'd>,
        g2_d3: Option<impl Peripheral<P = impl G2IO3Pin<T>> + 'd>,
        g2_d4: Option<impl Peripheral<P = impl G2IO4Pin<T>> + 'd>,

        g3_d1: Option<impl Peripheral<P = impl G3IO1Pin<T>> + 'd>,
        g3_d2: Option<impl Peripheral<P = impl G3IO2Pin<T>> + 'd>,
        g3_d3: Option<impl Peripheral<P = impl G3IO3Pin<T>> + 'd>,
        g3_d4: Option<impl Peripheral<P = impl G3IO4Pin<T>> + 'd>,

        g4_d1: Option<impl Peripheral<P = impl G4IO1Pin<T>> + 'd>,
        g4_d2: Option<impl Peripheral<P = impl G4IO2Pin<T>> + 'd>,
        g4_d3: Option<impl Peripheral<P = impl G4IO3Pin<T>> + 'd>,
        g4_d4: Option<impl Peripheral<P = impl G4IO4Pin<T>> + 'd>,

        g5_d1: Option<impl Peripheral<P = impl G5IO1Pin<T>> + 'd>,
        g5_d2: Option<impl Peripheral<P = impl G5IO2Pin<T>> + 'd>,
        g5_d3: Option<impl Peripheral<P = impl G5IO3Pin<T>> + 'd>,
        g5_d4: Option<impl Peripheral<P = impl G5IO4Pin<T>> + 'd>,

        g6_d1: Option<impl Peripheral<P = impl G6IO1Pin<T>> + 'd>,
        g6_d2: Option<impl Peripheral<P = impl G6IO2Pin<T>> + 'd>,
        g6_d3: Option<impl Peripheral<P = impl G6IO3Pin<T>> + 'd>,
        g6_d4: Option<impl Peripheral<P = impl G6IO4Pin<T>> + 'd>,

        g7_d1: Option<impl Peripheral<P = impl G7IO1Pin<T>> + 'd>,
        g7_d2: Option<impl Peripheral<P = impl G7IO2Pin<T>> + 'd>,
        g7_d3: Option<impl Peripheral<P = impl G7IO3Pin<T>> + 'd>,
        g7_d4: Option<impl Peripheral<P = impl G7IO4Pin<T>> + 'd>,

        g8_d1: Option<impl Peripheral<P = impl G8IO1Pin<T>> + 'd>,
        g8_d2: Option<impl Peripheral<P = impl G8IO2Pin<T>> + 'd>,
        g8_d3: Option<impl Peripheral<P = impl G8IO3Pin<T>> + 'd>,
        g8_d4: Option<impl Peripheral<P = impl G8IO4Pin<T>> + 'd>,
        config: Config,
    ) -> Self {
        into_ref!(peri);

        // Need to check valid pin configuration input
        // Need to configure pin
        Self::new_inner(peri, config)
    }

    fn filter_group() -> Result<PinGroup<'d>, ()> {}

    fn new_inner(peri: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        into_ref!(peri);

        T::enable_and_reset();

        T::REGS.cr().modify(|w| {
            w.set_tsce(true);
            w.set_ctph(config.ct_pulse_high_length.into());
            w.set_ctpl(config.ct_pulse_low_length.into());
            w.set_sse(config.spread_spectrum);
            w.set_ssd(config.spread_spectrum_deviation);
            w.set_sspsc(config.spread_spectrum_prescaler);
            w.set_pgpsc(config.pulse_generator_prescaler.into());
            w.set_mcv(config.max_count_value);
            w.set_syncpol(config.synchro_pin_polarity);
            w.set_am(config.acquisition_mode);
        });

        // Set IO configuration
        // Disable Schmitt trigger hysteresis on all used TSC IOs
        // T::REGS.iohcr().modify(|w| {
        //     w.
        // });

        // Set channel and shield IOs
        // T::REGS.ioccr().modify(|w| {});

        // Set sampling IOs
        // T::REGS.ioscr().modify(|w| {
        //     w.set_g1_io1(val)
        // });

        // Set the groups to be acquired
        // T::REGS.iogcsr().modify(|w| {
        //     w.set_g1e(val);
        // });

        // Disable interrupts
        T::REGS.ier().modify(|w| {
            w.set_eoaie(false);
            w.set_mceie(false);
        });

        // Clear flags
        T::REGS.icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        Self {
            _peri: peri,
            state: State::Ready,
            config,
        }
    }

    pub fn start(&mut self) {
        self.state = State::Busy;

        // Disable interrupts
        T::REGS.ier().modify(|w| {
            w.set_eoaie(false);
            w.set_mceie(false);
        });

        // Clear flags
        T::REGS.icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        // Set the touch sensing IOs not acquired to the default mode
        T::REGS.cr().modify(|w| {
            w.set_iodef(self.config.io_default_mode);
        });

        // Start the acquisition
        T::REGS.cr().modify(|w| {
            w.set_start(true);
        });
    }

    pub fn start_it(&mut self) {
        self.state = State::Busy;

        // Enable interrupts
        T::REGS.ier().modify(|w| {
            w.set_eoaie(true);
            w.set_mceie(self.config.max_count_interrupt);
        });

        // Clear flags
        T::REGS.icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        // Set the touch sensing IOs not acquired to the default mode
        T::REGS.cr().modify(|w| {
            w.set_iodef(self.config.io_default_mode);
        });

        // Start the acquisition
        T::REGS.cr().modify(|w| {
            w.set_start(true);
        });
    }

    pub fn stop(&mut self) {
        T::REGS.cr().modify(|w| {
            w.set_start(false);
        });

        // Set the touch sensing IOs in low power mode
        T::REGS.cr().modify(|w| {
            w.set_iodef(false);
        });

        // Clear flags
        T::REGS.icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        self.state = State::Ready;
    }

    pub fn stop_it(&mut self) {
        T::REGS.cr().modify(|w| {
            w.set_start(false);
        });

        // Set the touch sensing IOs in low power mode
        T::REGS.cr().modify(|w| {
            w.set_iodef(false);
        });

        // Disable interrupts
        T::REGS.ier().modify(|w| {
            w.set_eoaie(false);
            w.set_mceie(false);
        });

        // Clear flags
        T::REGS.icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        self.state = State::Ready;
    }

    pub fn poll_for_acquisition(&mut self) {
        while self.get_state() == State::Busy {}
    }

    pub fn get_state(&mut self) -> State {
        if self.state == State::Busy {
            if T::REGS.isr().read().eoaf() {
                if T::REGS.isr().read().mcef() {
                    self.state = State::Error
                } else {
                    self.state = State::Ready
                }
            }
        }
        self.state
    }

    pub fn group_get_status(&mut self, index: Group) -> GroupStatus {
        // Status bits are set by hardware when the acquisition on the corresponding
        // enabled analog IO group is complete, cleared when new acquisition is started
        let status = match index {
            Group::One => T::REGS.iogcsr().read().g1s(),
            Group::Two => T::REGS.iogcsr().read().g2s(),
            Group::Three => T::REGS.iogcsr().read().g3s(),
            Group::Four => T::REGS.iogcsr().read().g4s(),
            Group::Five => T::REGS.iogcsr().read().g5s(),
            Group::Six => T::REGS.iogcsr().read().g6s(),
            Group::Seven => T::REGS.iogcsr().read().g7s(),
            Group::Eight => T::REGS.iogcsr().read().g8s(),
        };
        match status {
            true => GroupStatus::Complete,
            false => GroupStatus::Ongoing,
        }
    }

    pub fn group_get_value(&mut self, index: Group) -> u16 {
        T::REGS.iogcr(index.into()).read().cnt()
    }

    // pub fn configure_io()

    pub fn discharge_io(&mut self, status: bool) {
        // Set the touch sensing IOs in low power mode
        T::REGS.cr().modify(|w| {
            w.set_iodef(!status);
        });
    }
}

impl<'d, T: Instance> Drop for TSC<'d, T> {
    fn drop(&mut self) {
        //  Need to figure out what to do with the IOs
        T::disable();
    }
}

pub(crate) trait SealedInstance {
    const REGS: Regs;
}

/// TSC instance trait
#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + SealedInstance + RccPeripheral {}

foreach_peripheral!(
    (tsc, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);

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
pin_trait!(G8IO1Pin, Instance);
pin_trait!(G8IO2Pin, Instance);
pin_trait!(G8IO3Pin, Instance);
pin_trait!(G8IO4Pin, Instance);
