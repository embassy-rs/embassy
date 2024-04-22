//! TSC Peripheral Interface

#![macro_use]

/// Enums defined for peripheral parameters
pub mod enums;

pub use enums::*;

use crate::gpio::{AFType, AnyPin, Pull};
use crate::pac::tsc::Tsc as Regs;
use crate::rcc::RccPeripheral;
use crate::{peripherals, Peripheral};
use embassy_hal_internal::{into_ref, PeripheralRef};

const TSC_NUM_GROUPS: u32 = 8;

/// Error type defined for TSC
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Test error for TSC
    Test,
}

/// Pin type definition to control IO parameters
pub enum PinType {
    /// Sensing channel pin connected to an electrode
    Channel,
    /// Sampling capacitor pin, one required for every pin group
    Sample,
    /// Shield pin connected to capacitive sensing shield
    Shield,
}

/// Unclear
pub struct TscGroup {}

/// Peripheral state
#[derive(PartialEq, Clone, Copy)]
pub enum State {
    /// Peripheral is being setup or reconfigured
    Reset,
    /// Ready to start acquisition
    Ready,
    /// In process of sensor acquisition
    Busy,
    /// Error occured during acquisition
    Error,
}

/// Individual group status checked after acquisition reported as complete
/// For groups with multiple channel pins, may take longer because acquisitions
/// are done sequentially. Check this status before pulling count for each
/// sampled channel
pub enum GroupStatus {
    /// Acquisition for channel still in progress
    Ongoing,
    /// Acquisition either not started or complete
    Complete,
}

/// Group identifier used to interrogate status
#[allow(missing_docs)]
pub enum Group {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    #[cfg(any(tsc_v2, tsc_v3))]
    Seven,
    #[cfg(tsc_v3)]
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
            #[cfg(any(tsc_v2, tsc_v3))]
            Group::Seven => 6,
            #[cfg(tsc_v3)]
            Group::Eight => 7,
        }
    }
}

/// Peripheral configuration
#[derive(Clone, Copy)]
pub struct Config {
    /// Duration of high state of the charge transfer pulse
    pub ct_pulse_high_length: ChargeTransferPulseCycle,
    /// Duration of the low state of the charge transfer pulse
    pub ct_pulse_low_length: ChargeTransferPulseCycle,
    /// Enable/disable of spread spectrum feature
    pub spread_spectrum: bool,
    /// Adds variable number of periods of the SS clk to pulse high state
    pub spread_spectrum_deviation: SSDeviation,
    /// Selects AHB clock divider used to generate SS clk
    pub spread_spectrum_prescaler: bool,
    /// Selects AHB clock divider used to generate pulse generator clk
    pub pulse_generator_prescaler: PGPrescalerDivider,
    /// Maximum number of charge tranfer pulses that can be generated before error
    pub max_count_value: MaxCount,
    /// Defines config of all IOs when no ongoing acquisition
    pub io_default_mode: bool,
    /// Polarity of sync input pin
    pub synchro_pin_polarity: bool,
    /// Acquisition starts when start bit is set or with sync pin input
    pub acquisition_mode: bool,
    /// Enable max count interrupt
    pub max_count_interrupt: bool,
    /// Channel IO mask
    pub channel_ios: u32,
    /// Shield IO mask
    pub shield_ios: u32,
    /// Sampling IO mask
    pub sampling_ios: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ct_pulse_high_length: ChargeTransferPulseCycle::_1,
            ct_pulse_low_length: ChargeTransferPulseCycle::_1,
            spread_spectrum: false,
            spread_spectrum_deviation: SSDeviation::new(0).unwrap(),
            spread_spectrum_prescaler: false,
            pulse_generator_prescaler: PGPrescalerDivider::_1,
            max_count_value: MaxCount::_255,
            io_default_mode: false,
            synchro_pin_polarity: false,
            acquisition_mode: false,
            max_count_interrupt: false,
            channel_ios: 0,
            shield_ios: 0,
            sampling_ios: 0,
        }
    }
}

/// Pin struct that maintains usage
#[allow(missing_docs)]
pub struct TscPin<'d, T> {
    pin: PeripheralRef<'d, T>,
    role: PinType,
}

/// Input structure for constructor containing peripherals
#[allow(missing_docs)]
pub struct PeriPin<T> {
    pub pin: T,
    pub role: PinType,
}

/// Pin group definition
/// Pins are organized into groups of four IOs, all groups with a
/// sampling channel must also have a sampling capacitor channel.
#[allow(missing_docs)]
pub struct PinGroup<'d, A> {
    pub d1: Option<TscPin<'d, A>>,
    pub d2: Option<TscPin<'d, A>>,
    pub d3: Option<TscPin<'d, A>>,
    pub d4: Option<TscPin<'d, A>>,
}

/// TSC driver
pub struct Tsc<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
    g1: Option<PinGroup<'d, AnyPin>>,
    g2: Option<PinGroup<'d, AnyPin>>,
    g3: Option<PinGroup<'d, AnyPin>>,
    g4: Option<PinGroup<'d, AnyPin>>,
    g5: Option<PinGroup<'d, AnyPin>>,
    g6: Option<PinGroup<'d, AnyPin>>,
    #[cfg(any(tsc_v2, tsc_v3))]
    g7: Option<PinGroup<'d, AnyPin>>,
    #[cfg(tsc_v3)]
    g8: Option<PinGroup<'d, AnyPin>>,
    state: State,
    config: Config,
}

impl<'d, T: Instance> Tsc<'d, T> {
    /// Create new TSC driver
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        // g1_d1: Option<PeriPin<impl Peripheral<P = impl G1IO1Pin<T>> + 'd>>,
        g1_d2: Option<PeriPin<impl Peripheral<P = impl G1IO2Pin<T>> + 'd>>,
        g1_d3: Option<PeriPin<impl Peripheral<P = impl G1IO3Pin<T>> + 'd>>,
        // g1_d4: Option<PeriPin<impl Peripheral<P = impl G1IO4Pin<T>> + 'd>>,

        // g2_d1: Option<impl Peripheral<P = impl G2IO1Pin<T>> + 'd>,
        // g2_d2: Option<impl Peripheral<P = impl G2IO2Pin<T>> + 'd>,
        g2_d3: Option<PeriPin<impl Peripheral<P = impl G2IO3Pin<T>> + 'd>>,
        g2_d4: Option<PeriPin<impl Peripheral<P = impl G2IO4Pin<T>> + 'd>>,

        // g3_d1: Option<impl Peripheral<P = impl G3IO1Pin<T>> + 'd>,
        // g3_d2: Option<impl Peripheral<P = impl G3IO2Pin<T>> + 'd>,
        // g3_d3: Option<impl Peripheral<P = impl G3IO3Pin<T>> + 'd>,
        // g3_d4: Option<impl Peripheral<P = impl G3IO4Pin<T>> + 'd>,
        g4_d1: Option<PeriPin<impl Peripheral<P = impl G4IO1Pin<T>> + 'd>>,
        g4_d2: Option<PeriPin<impl Peripheral<P = impl G4IO2Pin<T>> + 'd>>,
        // g4_d3: Option<impl Peripheral<P = impl G4IO3Pin<T>> + 'd>,
        // g4_d4: Option<impl Peripheral<P = impl G4IO4Pin<T>> + 'd>,

        // g5_d1: Option<impl Peripheral<P = impl G5IO1Pin<T>> + 'd>,
        // g5_d2: Option<impl Peripheral<P = impl G5IO2Pin<T>> + 'd>,
        // g5_d3: Option<impl Peripheral<P = impl G5IO3Pin<T>> + 'd>,
        // g5_d4: Option<impl Peripheral<P = impl G5IO4Pin<T>> + 'd>,

        // g6_d1: Option<impl Peripheral<P = impl G6IO1Pin<T>> + 'd>,
        // g6_d2: Option<impl Peripheral<P = impl G6IO2Pin<T>> + 'd>,
        // g6_d3: Option<impl Peripheral<P = impl G6IO3Pin<T>> + 'd>,
        // g6_d4: Option<impl Peripheral<P = impl G6IO4Pin<T>> + 'd>,

        // g7_d1: Option<impl Peripheral<P = impl G7IO1Pin<T>> + 'd>,
        // g7_d2: Option<impl Peripheral<P = impl G7IO2Pin<T>> + 'd>,
        // g7_d3: Option<impl Peripheral<P = impl G7IO3Pin<T>> + 'd>,
        // g7_d4: Option<impl Peripheral<P = impl G7IO4Pin<T>> + 'd>,

        // g8_d1: Option<impl Peripheral<P = impl G8IO1Pin<T>> + 'd>,
        // g8_d2: Option<impl Peripheral<P = impl G8IO2Pin<T>> + 'd>,
        // g8_d3: Option<impl Peripheral<P = impl G8IO3Pin<T>> + 'd>,
        // g8_d4: Option<impl Peripheral<P = impl G8IO4Pin<T>> + 'd>,
        config: Config,
    ) -> Self {
        let g1_d2 = g1_d2.unwrap();
        let g1_d2_pin = g1_d2.pin;
        let g1_d3 = g1_d3.unwrap();
        let g1_d3_pin = g1_d3.pin;
        let g2_d3 = g2_d3.unwrap();
        let g2_d3_pin = g2_d3.pin;
        let g2_d4 = g2_d4.unwrap();
        let g2_d4_pin = g2_d4.pin;
        let g4_d1 = g4_d1.unwrap();
        let g4_d1_pin = g4_d1.pin;
        let g4_d2 = g4_d2.unwrap();
        let g4_d2_pin = g4_d2.pin;
        into_ref!(peri, g1_d2_pin, g1_d3_pin, g2_d3_pin, g2_d4_pin, g4_d1_pin, g4_d2_pin);

        // Configure pins
        match g1_d2.role {
            PinType::Channel => g1_d2_pin.set_as_af_pull(g1_d2_pin.af_num(), AFType::OutputPushPull, Pull::None),
            PinType::Sample => g1_d2_pin.set_as_af_pull(g1_d2_pin.af_num(), AFType::OutputOpenDrain, Pull::None),
            PinType::Shield => g1_d2_pin.set_as_af_pull(g1_d2_pin.af_num(), AFType::OutputPushPull, Pull::None),
        }
        match g1_d3.role {
            PinType::Channel => g1_d3_pin.set_as_af_pull(g1_d3_pin.af_num(), AFType::OutputPushPull, Pull::None),
            PinType::Sample => g1_d3_pin.set_as_af_pull(g1_d3_pin.af_num(), AFType::OutputOpenDrain, Pull::None),
            PinType::Shield => g1_d3_pin.set_as_af_pull(g1_d3_pin.af_num(), AFType::OutputPushPull, Pull::None),
        }
        let g1 = PinGroup {
            d1: None,
            d2: Some(TscPin {
                pin: g1_d2_pin.map_into(),
                role: g1_d2.role,
            }),
            d3: Some(TscPin {
                pin: g1_d3_pin.map_into(),
                role: g1_d3.role,
            }),
            d4: None,
        };

        match g2_d3.role {
            PinType::Channel => g2_d3_pin.set_as_af_pull(g2_d3_pin.af_num(), AFType::OutputPushPull, Pull::None),
            PinType::Sample => g2_d3_pin.set_as_af_pull(g2_d3_pin.af_num(), AFType::OutputOpenDrain, Pull::None),
            PinType::Shield => g2_d3_pin.set_as_af_pull(g2_d3_pin.af_num(), AFType::OutputPushPull, Pull::None),
        }
        match g2_d4.role {
            PinType::Channel => g2_d4_pin.set_as_af_pull(g2_d4_pin.af_num(), AFType::OutputPushPull, Pull::None),
            PinType::Sample => g2_d4_pin.set_as_af_pull(g2_d4_pin.af_num(), AFType::OutputOpenDrain, Pull::None),
            PinType::Shield => g2_d4_pin.set_as_af_pull(g2_d4_pin.af_num(), AFType::OutputPushPull, Pull::None),
        }
        let g2 = PinGroup {
            d1: None,
            d2: None,
            d3: Some(TscPin {
                pin: g2_d3_pin.map_into(),
                role: g2_d3.role,
            }),
            d4: Some(TscPin {
                pin: g2_d4_pin.map_into(),
                role: g2_d4.role,
            }),
        };

        match g4_d1.role {
            PinType::Channel => g4_d1_pin.set_as_af_pull(g4_d1_pin.af_num(), AFType::OutputPushPull, Pull::None),
            PinType::Sample => g4_d1_pin.set_as_af_pull(g4_d1_pin.af_num(), AFType::OutputOpenDrain, Pull::None),
            PinType::Shield => g4_d1_pin.set_as_af_pull(g4_d1_pin.af_num(), AFType::OutputPushPull, Pull::None),
        }
        match g4_d2.role {
            PinType::Channel => g4_d2_pin.set_as_af_pull(g4_d2_pin.af_num(), AFType::OutputPushPull, Pull::None),
            PinType::Sample => g4_d2_pin.set_as_af_pull(g4_d2_pin.af_num(), AFType::OutputOpenDrain, Pull::None),
            PinType::Shield => g4_d2_pin.set_as_af_pull(g4_d2_pin.af_num(), AFType::OutputPushPull, Pull::None),
        }
        let g4 = PinGroup {
            d1: Some(TscPin {
                pin: g4_d1_pin.map_into(),
                role: g4_d1.role,
            }),
            d2: Some(TscPin {
                pin: g4_d2_pin.map_into(),
                role: g4_d2.role,
            }),
            d3: None,
            d4: None,
        };

        // Need to check valid pin configuration input
        Self::new_inner(
            peri,
            Some(g1),
            Some(g2),
            None,
            Some(g4),
            None,
            None,
            #[cfg(any(tsc_v2, tsc_v3))]
            None,
            #[cfg(tsc_v3)]
            None,
            config,
        )
    }

    // fn configure_pin<'b, G: Pin>(pin: PeripheralRef<'b, G>, role: PinType) {
    //     match role {
    //         PinType::Channel => pin.set_as_af_pull(pin.af_num(), AFType::OutputPushPull, Pull::None),
    //         PinType::Sample => {}
    //         PinType::Shield => {}
    //     }
    // }

    // fn filter_group() -> Option<PinGroup<'d>> {}
    fn extract_groups(io_mask: u32) -> u32 {
        let mut groups: u32 = 0;
        for idx in 0..TSC_NUM_GROUPS {
            if io_mask & (0x0F << idx * 4) != 0 {
                groups |= 1 << idx
            }
        }
        groups
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        g1: Option<PinGroup<'d, AnyPin>>,
        g2: Option<PinGroup<'d, AnyPin>>,
        g3: Option<PinGroup<'d, AnyPin>>,
        g4: Option<PinGroup<'d, AnyPin>>,
        g5: Option<PinGroup<'d, AnyPin>>,
        g6: Option<PinGroup<'d, AnyPin>>,
        #[cfg(any(tsc_v2, tsc_v3))] g7: Option<PinGroup<'d, AnyPin>>,
        #[cfg(tsc_v3)] g8: Option<PinGroup<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(peri);

        T::enable_and_reset();

        T::REGS.cr().modify(|w| {
            w.set_tsce(true);
            w.set_ctph(config.ct_pulse_high_length.into());
            w.set_ctpl(config.ct_pulse_low_length.into());
            w.set_sse(config.spread_spectrum);
            // Prevent invalid configuration for pulse generator prescaler
            if config.ct_pulse_low_length == ChargeTransferPulseCycle::_1
                && (config.pulse_generator_prescaler == PGPrescalerDivider::_1
                    || config.pulse_generator_prescaler == PGPrescalerDivider::_2)
            {
                w.set_pgpsc(PGPrescalerDivider::_4.into());
            } else if config.ct_pulse_low_length == ChargeTransferPulseCycle::_2
                && config.pulse_generator_prescaler == PGPrescalerDivider::_1
            {
                w.set_pgpsc(PGPrescalerDivider::_2.into());
            } else {
                w.set_pgpsc(config.pulse_generator_prescaler.into());
            }
            w.set_ssd(config.spread_spectrum_deviation.into());
            w.set_sspsc(config.spread_spectrum_prescaler);

            w.set_mcv(config.max_count_value.into());
            w.set_syncpol(config.synchro_pin_polarity);
            w.set_am(config.acquisition_mode);
        });

        // Set IO configuration
        // Disable Schmitt trigger hysteresis on all used TSC IOs
        T::REGS
            .iohcr()
            .write(|w| w.0 = config.channel_ios | config.shield_ios | config.sampling_ios);

        // Set channel and shield IOs
        T::REGS.ioccr().write(|w| w.0 = config.channel_ios | config.shield_ios);

        // Set sampling IOs
        T::REGS.ioscr().write(|w| w.0 = config.sampling_ios);

        // Set the groups to be acquired
        T::REGS
            .iogcsr()
            .write(|w| w.0 = Self::extract_groups(config.channel_ios));

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
            g1,
            g2,
            g3,
            g4,
            g5,
            g6,
            #[cfg(any(tsc_v2, tsc_v3))]
            g7,
            #[cfg(tsc_v3)]
            g8,
            state: State::Ready,
            config,
        }
    }

    /// Start charge transfer acquisition
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

    /// Start charge transfer acquisition with interrupts enabled
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

    /// Stop charge transfer acquisition
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

    /// Stop charge transfer acquisition and clear interrupts
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

    /// Wait for end of acquisition
    pub fn poll_for_acquisition(&mut self) {
        while self.get_state() == State::Busy {}
    }

    /// Get current state of acquisition
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

    /// Get the individual group status to check acquisition complete
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
            #[cfg(any(tsc_v2, tsc_v3))]
            Group::Seven => T::REGS.iogcsr().read().g7s(),
            #[cfg(tsc_v3)]
            Group::Eight => T::REGS.iogcsr().read().g8s(),
        };
        match status {
            true => GroupStatus::Complete,
            false => GroupStatus::Ongoing,
        }
    }

    /// Get the count for the acquisiton, valid once group status is set
    pub fn group_get_value(&mut self, index: Group) -> u16 {
        T::REGS.iogcr(index.into()).read().cnt()
    }

    // pub fn configure_io()

    /// Discharge the IOs for subsequent acquisition
    pub fn discharge_io(&mut self, status: bool) {
        // Set the touch sensing IOs in low power mode
        T::REGS.cr().modify(|w| {
            w.set_iodef(!status);
        });
    }
}

impl<'d, T: Instance> Drop for Tsc<'d, T> {
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
