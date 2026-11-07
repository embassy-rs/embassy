//! Operational Amplifier (OPAMP) for STM32C5.
//!
//! The C5 OPAMP (`opamp_c5`) has a different register layout from the
//! G4-style `opamp_v5` block, so it lives in its own file. Only the C531,
//! C532 and C542 lines have an OPAMP (OPAMP1).
//!
//! Not yet covered: the internal output to ADC/COMP, the DAC as non-inverting
//! input, the filtering PGA modes and the timer-controlled input/gain mode.
#![macro_use]

use embassy_hal_internal::PeripheralType;

use crate::Peri;
use crate::pac::opamp::vals::*;
use crate::rcc::RccInfo;
use crate::wait::block_for_us;

/// Gain
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum OpAmpGain {
    Mul2,
    Mul4,
    Mul8,
    Mul16,
}

enum OpAmpDifferentialPair {
    P,
    N,
}

/// Speed
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub enum OpAmpSpeed {
    Normal,
    HighSpeed,
}

/// OpAmp external outputs, wired to a GPIO pad.
///
/// This struct can also be used as an ADC input.
pub struct OpAmpOutput<'d, T: Instance> {
    _inner: &'d OpAmp<'d, T>,
}

/// OpAmp driver.
pub struct OpAmp<'d, T: Instance> {
    _inner: Peri<'d, T>,
}

impl<'d, T: Instance> OpAmp<'d, T> {
    /// Create a new driver instance.
    ///
    /// Does not enable the opamp, but does set the speed mode.
    pub fn new(opamp: Peri<'d, T>, speed: OpAmpSpeed) -> Self {
        T::info().rcc.enable_and_reset();
        T::regs().opamp_csr().modify(|w| {
            w.set_opahsm(match speed {
                OpAmpSpeed::Normal => Opahsm::NormalSpeed,
                OpAmpSpeed::HighSpeed => Opahsm::HighSpeed,
            });
        });

        Self { _inner: opamp }
    }

    /// Configure the OpAmp as a buffer for the provided input pin,
    /// outputting to the provided output pin, and enable the opamp.
    ///
    /// The input pin is configured for analogue mode but not consumed,
    /// so it may subsequently be used for ADC or comparator inputs.
    ///
    /// The output pin is held within the returned [`OpAmpOutput`] struct,
    /// preventing it being used elsewhere. The `OpAmpOutput` can then be
    /// directly used as an ADC input. The opamp will be disabled when the
    /// [`OpAmpOutput`] is dropped.
    pub fn buffer_ext(
        &mut self,
        in_pin: Peri<'_, impl NonInvertingPin<T> + crate::gpio::Pin>,
        out_pin: Peri<'_, impl OutputPin<T> + crate::gpio::Pin>,
    ) -> OpAmpOutput<'_, T> {
        in_pin.set_as_analog();
        out_pin.set_as_analog();

        T::regs().opamp_csr().modify(|w| {
            w.set_vp_sel(VpSel::from_bits(in_pin.channel()));
            w.set_vm_sel(VmSel::Follower);
            w.set_opaintoen(Opaintoen::OpampVout);
            w.set_opaen(true);
        });

        OpAmpOutput { _inner: self }
    }

    /// Configure the OpAmp as a PGA for the provided input pin,
    /// outputting to the provided output pin, and enable the opamp.
    ///
    /// The input pin is configured for analogue mode but not consumed,
    /// so it may subsequently be used for ADC or comparator inputs.
    ///
    /// The output pin is held within the returned [`OpAmpOutput`] struct,
    /// preventing it being used elsewhere. The `OpAmpOutput` can then be
    /// directly used as an ADC input. The opamp will be disabled when the
    /// [`OpAmpOutput`] is dropped.
    pub fn pga_ext(
        &mut self,
        in_pin: Peri<'_, impl NonInvertingPin<T> + crate::gpio::Pin>,
        out_pin: Peri<'_, impl OutputPin<T> + crate::gpio::Pin>,
        gain: OpAmpGain,
    ) -> OpAmpOutput<'_, T> {
        in_pin.set_as_analog();
        out_pin.set_as_analog();

        let pga_gain = match gain {
            OpAmpGain::Mul2 => PgaGain::Gain2,
            OpAmpGain::Mul4 => PgaGain::Gain4,
            OpAmpGain::Mul8 => PgaGain::Gain8,
            OpAmpGain::Mul16 => PgaGain::Gain16,
        };

        T::regs().opamp_csr().modify(|w| {
            w.set_vp_sel(VpSel::from_bits(in_pin.channel()));
            w.set_vm_sel(VmSel::Pga);
            w.set_pga_gain(pga_gain);
            w.set_opaintoen(Opaintoen::OpampVout);
            w.set_opaen(true);
        });

        OpAmpOutput { _inner: self }
    }

    /// Configure the OpAmp as a PGA for the provided input pin,
    /// outputting to the provided output pin, and enable the opamp.
    ///
    /// The ground node of the gain network is connected to the provided VINM0 bias pin.
    ///
    /// The input and bias pins are configured for analogue mode but not consumed,
    /// so it may subsequently be used for ADC or comparator inputs.
    ///
    /// The output pin is held within the returned [`OpAmpOutput`] struct,
    /// preventing it being used elsewhere. The `OpAmpOutput` can then be
    /// directly used as an ADC input. The opamp will be disabled when the
    /// [`OpAmpOutput`] is dropped.
    pub fn pga_biased_ext(
        &mut self,
        in_pin: Peri<'_, impl NonInvertingPin<T> + crate::gpio::Pin>,
        bias_pin: Peri<'_, impl BiasPin<T> + crate::gpio::Pin>,
        out_pin: Peri<'_, impl OutputPin<T> + crate::gpio::Pin>,
        gain: OpAmpGain,
    ) -> OpAmpOutput<'_, T> {
        in_pin.set_as_analog();
        bias_pin.set_as_analog();
        out_pin.set_as_analog();

        let pga_gain = match gain {
            OpAmpGain::Mul2 => PgaGain::Gain2InputVinm0,
            OpAmpGain::Mul4 => PgaGain::Gain4InputVinm0,
            OpAmpGain::Mul8 => PgaGain::Gain8InputVinm0,
            OpAmpGain::Mul16 => PgaGain::Gain16InputVinm0,
        };

        T::regs().opamp_csr().modify(|w| {
            w.set_vp_sel(VpSel::from_bits(in_pin.channel()));
            w.set_vm_sel(VmSel::Pga);
            w.set_pga_gain(pga_gain);
            w.set_opaintoen(Opaintoen::OpampVout);
            w.set_opaen(true);
        });

        OpAmpOutput { _inner: self }
    }

    /// Configure the OpAmp in standalone mode with the non-inverting input
    /// connected to the provided `p_pin`, the inverting input connected to
    /// the `m_pin`, and output to the provided `out_pin`.
    ///
    /// The input pins are configured for analogue mode but not consumed,
    /// allowing their subsequent use for ADC or comparator inputs.
    ///
    /// The output pin is held within the returned [`OpAmpOutput`] struct,
    /// preventing it being used elsewhere. The opamp will be disabled when
    /// the [`OpAmpOutput`] is dropped.
    pub fn standalone_ext(
        &mut self,
        p_pin: Peri<'d, impl NonInvertingPin<T> + crate::gpio::Pin>,
        m_pin: Peri<'d, impl InvertingPin<T> + crate::gpio::Pin>,
        out_pin: Peri<'d, impl OutputPin<T> + crate::gpio::Pin>,
    ) -> OpAmpOutput<'_, T> {
        p_pin.set_as_analog();
        m_pin.set_as_analog();
        out_pin.set_as_analog();

        T::regs().opamp_csr().modify(|w| {
            w.set_vp_sel(VpSel::from_bits(p_pin.channel()));
            w.set_vm_sel(VmSel::from_bits(m_pin.channel()));
            w.set_opaintoen(Opaintoen::OpampVout);
            w.set_opaen(true);
        });

        OpAmpOutput { _inner: self }
    }

    /// Calibrates the operational amplifier.
    ///
    /// This function enables the opamp and sets the user trim mode for calibration.
    /// Depending on the speed mode of the opamp, it calibrates the differential pair inputs.
    /// For normal speed, both the P and N differential pairs are calibrated,
    /// while for high-speed mode, only the P differential pair is calibrated.
    ///
    /// The procedure follows the one used for the G4 opamp and has not yet
    /// been checked against the C5 reference manual or on hardware.
    ///
    /// Calibrating a differential pair requires waiting 12ms in the worst case (binary method).
    pub fn calibrate(&mut self) {
        T::regs().opamp_csr().modify(|w| {
            w.set_opaen(true);
            w.set_calon(true);
            w.set_usertrim(true);
        });

        if T::regs().opamp_csr().read().opahsm() == Opahsm::HighSpeed {
            self.calibrate_differential_pair(OpAmpDifferentialPair::P);
        } else {
            self.calibrate_differential_pair(OpAmpDifferentialPair::P);
            self.calibrate_differential_pair(OpAmpDifferentialPair::N);
        }

        T::regs().opamp_csr().modify(|w| {
            w.set_calon(false);
            w.set_opaen(false);
        });
    }

    /// Calibrate differential pair.
    ///
    /// The calibration is done by trying different offset values and
    /// measuring the calout bit.
    ///
    /// The calibration range is from 0 to 31.
    ///
    /// The result is stored in the OPAMP_CSR register.
    fn calibrate_differential_pair(&mut self, pair: OpAmpDifferentialPair) {
        let mut low = 0;
        let mut high = 31;

        let calsel = match pair {
            OpAmpDifferentialPair::P => Calsel::Percent10,
            OpAmpDifferentialPair::N => Calsel::Percent90,
        };

        T::regs().opamp_csr().modify(|w| {
            w.set_calsel(calsel);
        });

        while low <= high {
            let mid = (low + high) / 2;

            T::regs().opamp_csr().modify(|w| match pair {
                OpAmpDifferentialPair::P => {
                    #[cfg(feature = "defmt")]
                    defmt::debug!("opamp p calibration. offset: {}", mid);
                    w.set_trimoffsetp(mid);
                }
                OpAmpDifferentialPair::N => {
                    #[cfg(feature = "defmt")]
                    defmt::debug!("opamp n calibration. offset: {}", mid);
                    w.set_trimoffsetn(mid);
                }
            });

            // The closer the trimming value is to the optimum trimming value, the longer it takes to stabilize
            // (with a maximum stabilization time remaining below 2 ms in any case)
            block_for_us(2_000);

            if !T::regs().opamp_csr().read().calout() {
                if mid == 0 {
                    break;
                }
                high = mid - 1;
            } else {
                if mid == 31 {
                    break;
                }
                low = mid + 1;
            }
        }
    }
}

impl<'d, T: Instance> Drop for OpAmp<'d, T> {
    fn drop(&mut self) {
        T::info().rcc.disable();
    }
}

impl<'d, T: Instance> Drop for OpAmpOutput<'d, T> {
    fn drop(&mut self) {
        T::regs().opamp_csr().modify(|w| {
            w.set_opaen(false);
        });
    }
}

pub(crate) struct Info {
    rcc: RccInfo,
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn regs() -> crate::pac::opamp::Opamp;
}

analog_pin_trait!(NonInvertingPin, Instance);
analog_pin_trait!(InvertingPin, Instance);
analog_pin_trait!(BiasPin, Instance);
analog_pin_trait!(OutputPin, Instance);

/// Opamp instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

macro_rules! impl_opamp_external_output {
    ($inst:ident, $adc:ident, $ch:expr) => {
        foreach_adc!(
            ($adc, $common_inst:ident, $block:ident, $family:ident) => {
                impl<'d> crate::adc::SealedAdcChannel<crate::peripherals::$adc>
                    for crate::opamp::OpAmpOutput<'d, crate::peripherals::$inst>
                {
                    fn channel(&self) -> u8 {
                        $ch
                    }
                }

                impl<'d> crate::adc::AdcChannel<'d, crate::peripherals::$adc>
                    for crate::opamp::OpAmpOutput<'d, crate::peripherals::$inst>
                {
                }
            };
        );
    };
}

foreach_peripheral! {
    (opamp, $inst:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn info() -> &'static Info {
                use crate::rcc::SealedRccPeripheral;
                static INFO: Info = Info {
                    rcc: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }
            fn regs() -> crate::pac::opamp::Opamp {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {
        }
    };
}
