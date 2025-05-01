//! Operational Amplifier (OPAMP)
#![macro_use]

use embassy_hal_internal::PeripheralType;

use crate::pac::opamp::vals::*;
use crate::Peri;

/// Performs a busy-wait delay for a specified number of microseconds.
#[cfg(opamp_g4)]
fn blocking_delay_ms(ms: u32) {
    #[cfg(feature = "time")]
    embassy_time::block_for(embassy_time::Duration::from_millis(ms as u64));
    #[cfg(not(feature = "time"))]
    cortex_m::asm::delay(unsafe { crate::rcc::get_freqs() }.sys.to_hertz().unwrap().0 / 1_000 * ms);
}

/// Gain
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum OpAmpGain {
    Mul2,
    Mul4,
    Mul8,
    Mul16,
    #[cfg(opamp_g4)]
    Mul32,
    #[cfg(opamp_g4)]
    Mul64,
}

#[cfg(opamp_g4)]
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

/// OpAmp internal outputs, wired directly to ADC inputs.
///
/// This struct can be used as an ADC input.
#[cfg(opamp_g4)]
pub struct OpAmpInternalOutput<'d, T: Instance> {
    _inner: &'d OpAmp<'d, T>,
}

/// OpAmp driver.
pub struct OpAmp<'d, T: Instance> {
    _inner: Peri<'d, T>,
}

impl<'d, T: Instance> OpAmp<'d, T> {
    /// Create a new driver instance.
    ///
    /// Does not enable the opamp, but does set the speed mode on some families.
    pub fn new(opamp: Peri<'d, T>, #[cfg(opamp_g4)] speed: OpAmpSpeed) -> Self {
        #[cfg(opamp_g4)]
        T::regs().csr().modify(|w| {
            w.set_opahsm(speed == OpAmpSpeed::HighSpeed);
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

        #[cfg(opamp_g4)]
        let vm_sel = VmSel::OUTPUT;
        #[cfg(not(opamp_g4))]
        let vm_sel = VmSel::from_bits(0b11);

        T::regs().csr().modify(|w| {
            w.set_vp_sel(VpSel::from_bits(in_pin.channel()));
            w.set_vm_sel(vm_sel);
            #[cfg(opamp_g4)]
            w.set_opaintoen(false);
            w.set_opampen(true);
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

        #[cfg(opamp_g4)]
        let vm_sel = VmSel::PGA;
        #[cfg(not(opamp_g4))]
        let vm_sel = VmSel::from_bits(0b10);

        #[cfg(opamp_g4)]
        let pga_gain = match gain {
            OpAmpGain::Mul2 => PgaGain::GAIN2,
            OpAmpGain::Mul4 => PgaGain::GAIN4,
            OpAmpGain::Mul8 => PgaGain::GAIN8,
            OpAmpGain::Mul16 => PgaGain::GAIN16,
            OpAmpGain::Mul32 => PgaGain::GAIN32,
            OpAmpGain::Mul64 => PgaGain::GAIN64,
        };
        #[cfg(not(opamp_g4))]
        let pga_gain = PgaGain::from_bits(match gain {
            OpAmpGain::Mul2 => 0b00,
            OpAmpGain::Mul4 => 0b01,
            OpAmpGain::Mul8 => 0b10,
            OpAmpGain::Mul16 => 0b11,
        });

        T::regs().csr().modify(|w| {
            w.set_vp_sel(VpSel::from_bits(in_pin.channel()));
            w.set_vm_sel(vm_sel);
            w.set_pga_gain(pga_gain);
            #[cfg(opamp_g4)]
            w.set_opaintoen(false);
            w.set_opampen(true);
        });

        OpAmpOutput { _inner: self }
    }

    /// Configure the OpAmp as a buffer for the DAC it is connected to,
    /// outputting to the provided output pin, and enable the opamp.
    ///
    /// The output pin is held within the returned [`OpAmpOutput`] struct,
    /// preventing it being used elsewhere. The `OpAmpOutput` can then be
    /// directly used as an ADC input. The opamp will be disabled when the
    /// [`OpAmpOutput`] is dropped.
    #[cfg(opamp_g4)]
    pub fn buffer_dac(&mut self, out_pin: Peri<'_, impl OutputPin<T> + crate::gpio::Pin>) -> OpAmpOutput<'_, T> {
        out_pin.set_as_analog();

        T::regs().csr().modify(|w| {
            use crate::pac::opamp::vals::*;

            w.set_vm_sel(VmSel::OUTPUT);
            w.set_vp_sel(VpSel::DAC3_CH1);
            w.set_opaintoen(false);
            w.set_opampen(true);
        });

        OpAmpOutput { _inner: self }
    }

    /// Configure the OpAmp as a buffer for the provided input pin,
    /// with the output only used internally, and enable the opamp.
    ///
    /// The input pin is configured for analogue mode but not consumed,
    /// so it may be subsequently used for ADC or comparator inputs.
    ///
    /// The returned `OpAmpInternalOutput` struct may be used as an ADC input.
    /// The opamp output will be disabled when it is dropped.
    #[cfg(opamp_g4)]
    pub fn buffer_int(
        &mut self,
        pin: Peri<'_, impl NonInvertingPin<T> + crate::gpio::Pin>,
    ) -> OpAmpInternalOutput<'_, T> {
        pin.set_as_analog();

        T::regs().csr().modify(|w| {
            w.set_vp_sel(VpSel::from_bits(pin.channel()));
            w.set_vm_sel(VmSel::OUTPUT);
            #[cfg(opamp_g4)]
            w.set_opaintoen(true);
            w.set_opampen(true);
        });

        OpAmpInternalOutput { _inner: self }
    }

    /// Configure the OpAmp as a PGA for the provided input pin,
    /// with the output only used internally, and enable the opamp.
    ///
    /// The input pin is configured for analogue mode but not consumed,
    /// so it may be subsequently used for ADC or comparator inputs.
    ///
    /// The returned `OpAmpInternalOutput` struct may be used as an ADC input.
    /// The opamp output will be disabled when it is dropped.
    #[cfg(opamp_g4)]
    pub fn pga_int(
        &mut self,
        pin: Peri<'_, impl NonInvertingPin<T> + crate::gpio::Pin>,
        gain: OpAmpGain,
    ) -> OpAmpInternalOutput<'_, T> {
        pin.set_as_analog();

        let pga_gain = match gain {
            OpAmpGain::Mul2 => PgaGain::GAIN2,
            OpAmpGain::Mul4 => PgaGain::GAIN4,
            OpAmpGain::Mul8 => PgaGain::GAIN8,
            OpAmpGain::Mul16 => PgaGain::GAIN16,
            OpAmpGain::Mul32 => PgaGain::GAIN32,
            OpAmpGain::Mul64 => PgaGain::GAIN64,
        };

        T::regs().csr().modify(|w| {
            w.set_vp_sel(VpSel::from_bits(pin.channel()));
            w.set_vm_sel(VmSel::OUTPUT);
            w.set_pga_gain(pga_gain);
            w.set_opaintoen(true);
            w.set_opampen(true);
        });

        OpAmpInternalOutput { _inner: self }
    }

    /// Configure the OpAmp as a standalone DAC with the inverting input
    /// connected to the provided pin, and the output is connected
    /// internally to an ADC channel.
    ///
    /// The input pin is configured for analogue mode but not consumed,
    /// so it may be subsequently used for ADC or comparator inputs.
    ///
    /// The returned `OpAmpInternalOutput` struct may be used as an ADC
    /// input. The opamp output will be disabled when it is dropped.
    #[cfg(opamp_g4)]
    pub fn standalone_dac_int(
        &mut self,
        m_pin: Peri<'_, impl InvertingPin<T> + crate::gpio::Pin>,
    ) -> OpAmpInternalOutput<'_, T> {
        m_pin.set_as_analog();

        T::regs().csr().modify(|w| {
            use crate::pac::opamp::vals::*;
            w.set_vp_sel(VpSel::DAC3_CH1); // Actually DAC3_CHx
            w.set_vm_sel(VmSel::from_bits(m_pin.channel()));
            w.set_opaintoen(true);
            w.set_opampen(true);
        });

        OpAmpInternalOutput { _inner: self }
    }

    /// Configure the OpAmp as a standalone DAC with the inverting input
    /// connected to the provided pin, and the output connected to the
    /// provided pin.
    ///
    /// The input pin is configured for analogue mode but not consumed,
    /// so it may be subsequently used for ADC or comparator inputs.
    ///
    /// The output pin is held within the returned [`OpAmpOutput`] struct,
    /// preventing it being used elsewhere. The opamp will be disabled when
    /// the [`OpAmpOutput`] is dropped.
    #[cfg(opamp_g4)]
    pub fn standalone_dac_ext(
        &mut self,
        m_pin: Peri<'_, impl InvertingPin<T> + crate::gpio::Pin>,
        out_pin: Peri<'_, impl OutputPin<T> + crate::gpio::Pin>,
    ) -> OpAmpOutput<'_, T> {
        m_pin.set_as_analog();
        out_pin.set_as_analog();

        T::regs().csr().modify(|w| {
            use crate::pac::opamp::vals::*;
            w.set_vp_sel(VpSel::DAC3_CH1); // Actually DAC3_CHx
            w.set_vm_sel(VmSel::from_bits(m_pin.channel()));
            w.set_opaintoen(false);
            w.set_opampen(true);
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
    #[cfg(opamp_g4)]
    pub fn standalone_ext(
        &mut self,
        p_pin: Peri<'d, impl NonInvertingPin<T> + crate::gpio::Pin>,
        m_pin: Peri<'d, impl InvertingPin<T> + crate::gpio::Pin>,
        out_pin: Peri<'d, impl OutputPin<T> + crate::gpio::Pin>,
    ) -> OpAmpOutput<'_, T> {
        p_pin.set_as_analog();
        m_pin.set_as_analog();
        out_pin.set_as_analog();

        T::regs().csr().modify(|w| {
            use crate::pac::opamp::vals::*;
            w.set_vp_sel(VpSel::from_bits(p_pin.channel()));
            w.set_vm_sel(VmSel::from_bits(m_pin.channel()));
            w.set_opaintoen(false);
            w.set_opampen(true);
        });

        OpAmpOutput { _inner: self }
    }

    /// Configure the OpAmp in standalone mode with the non-inverting input
    /// connected to the provided `p_pin`, the inverting input connected to
    /// the `m_pin`, and output is connected to the DAC.
    ///
    /// The input pins are configured for analogue mode but not consumed,
    /// allowing their subsequent use for ADC or comparator inputs.
    ///
    /// The returned `OpAmpOutput` struct may be used as an ADC
    /// input. The opamp output will be disabled when it is dropped.
    #[cfg(opamp_g4)]
    pub fn standalone_int(
        &mut self,
        p_pin: Peri<'d, impl NonInvertingPin<T> + crate::gpio::Pin>,
        m_pin: Peri<'d, impl InvertingPin<T> + crate::gpio::Pin>,
    ) -> OpAmpOutput<'_, T> {
        p_pin.set_as_analog();
        m_pin.set_as_analog();

        T::regs().csr().modify(|w| {
            use crate::pac::opamp::vals::*;
            w.set_vp_sel(VpSel::from_bits(p_pin.channel()));
            w.set_vm_sel(VmSel::from_bits(m_pin.channel()));
            w.set_opaintoen(true);
            w.set_opampen(true);
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
    /// Calibrating a differential pair requires waiting 12ms in the worst case (binary method).
    #[cfg(opamp_g4)]
    pub fn calibrate(&mut self) {
        T::regs().csr().modify(|w| {
            w.set_opampen(true);
            w.set_calon(true);
            w.set_usertrim(true);
        });

        if T::regs().csr().read().opahsm() {
            self.calibrate_differential_pair(OpAmpDifferentialPair::P);
        } else {
            self.calibrate_differential_pair(OpAmpDifferentialPair::P);
            self.calibrate_differential_pair(OpAmpDifferentialPair::N);
        }

        T::regs().csr().modify(|w| {
            w.set_calon(false);
            w.set_opampen(false);
        });
    }

    /// Calibrate differential pair.
    ///
    /// The calibration is done by trying different offset values and
    /// measuring the outcal bit.
    ///
    /// The calibration range is from 0 to 31.
    ///
    /// The result is stored in the OPAMP_CSR register.
    #[cfg(opamp_g4)]
    fn calibrate_differential_pair(&mut self, pair: OpAmpDifferentialPair) {
        let mut low = 0;
        let mut high = 31;

        let calsel = match pair {
            OpAmpDifferentialPair::P => Calsel::PERCENT10,
            OpAmpDifferentialPair::N => Calsel::PERCENT90,
        };

        T::regs().csr().modify(|w| {
            w.set_calsel(calsel);
        });

        while low <= high {
            let mid = (low + high) / 2;

            T::regs().csr().modify(|w| match pair {
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
            // (with a maximum stabilization time remaining below 2 ms in any case) -- RM0440 25.3.7
            blocking_delay_ms(2);

            if !T::regs().csr().read().calout() {
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

impl<'d, T: Instance> Drop for OpAmpOutput<'d, T> {
    fn drop(&mut self) {
        T::regs().csr().modify(|w| {
            w.set_opampen(false);
        });
    }
}

#[cfg(opamp_g4)]
impl<'d, T: Instance> Drop for OpAmpInternalOutput<'d, T> {
    fn drop(&mut self) {
        T::regs().csr().modify(|w| {
            w.set_opampen(false);
        });
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::opamp::Opamp;
}

pub(crate) trait SealedNonInvertingPin<T: Instance> {
    fn channel(&self) -> u8;
}

pub(crate) trait SealedInvertingPin<T: Instance> {
    #[allow(unused)]
    fn channel(&self) -> u8;
}

pub(crate) trait SealedOutputPin<T: Instance> {}

/// Opamp instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}
/// Non-inverting pin trait.
#[allow(private_bounds)]
pub trait NonInvertingPin<T: Instance>: SealedNonInvertingPin<T> {}
/// Inverting pin trait.
#[allow(private_bounds)]
pub trait InvertingPin<T: Instance>: SealedInvertingPin<T> {}
/// Output pin trait.
#[allow(private_bounds)]
pub trait OutputPin<T: Instance>: SealedOutputPin<T> {}

macro_rules! impl_opamp_external_output {
    ($inst:ident, $adc:ident, $ch:expr) => {
        foreach_adc!(
            ($adc, $common_inst:ident, $adc_clock:ident) => {
                impl<'d> crate::adc::SealedAdcChannel<crate::peripherals::$adc>
                    for OpAmpOutput<'d, crate::peripherals::$inst>
                {
                    fn channel(&self) -> u8 {
                        $ch
                    }
                }

                impl<'d> crate::adc::AdcChannel<crate::peripherals::$adc>
                    for OpAmpOutput<'d, crate::peripherals::$inst>
                {
                }
            };
        );
    };
}

foreach_peripheral!(
    (opamp, OPAMP1) => {
        impl_opamp_external_output!(OPAMP1, ADC1, 3);
    };
    (opamp, OPAMP2) => {
        impl_opamp_external_output!(OPAMP2, ADC2, 3);
    };
    (opamp, OPAMP3) => {
        impl_opamp_external_output!(OPAMP3, ADC1, 12);
        impl_opamp_external_output!(OPAMP3, ADC3, 1);
    };
    // OPAMP4 only in STM32G4 Cat 3 devices
    (opamp, OPAMP4) => {
        impl_opamp_external_output!(OPAMP4, ADC1, 11);
        impl_opamp_external_output!(OPAMP4, ADC4, 3);
    };
    // OPAMP5 only in STM32G4 Cat 3 devices
    (opamp, OPAMP5) => {
        impl_opamp_external_output!(OPAMP5, ADC5, 1);
    };
    // OPAMP6 only in STM32G4 Cat 3/4 devices
    (opamp, OPAMP6) => {
        impl_opamp_external_output!(OPAMP6, ADC1, 14);
        impl_opamp_external_output!(OPAMP6, ADC2, 14);
    };
);

#[cfg(opamp_g4)]
macro_rules! impl_opamp_internal_output {
    ($inst:ident, $adc:ident, $ch:expr) => {
        foreach_adc!(
            ($adc, $common_inst:ident, $adc_clock:ident) => {
                impl<'d> crate::adc::SealedAdcChannel<crate::peripherals::$adc>
                    for OpAmpInternalOutput<'d, crate::peripherals::$inst>
                {
                    fn channel(&self) -> u8 {
                        $ch
                    }
                }

                impl<'d> crate::adc::AdcChannel<crate::peripherals::$adc>
                    for OpAmpInternalOutput<'d, crate::peripherals::$inst>
                {
                }
            };
        );
    };
}

#[cfg(opamp_g4)]
foreach_peripheral!(
    (opamp, OPAMP1) => {
        impl_opamp_internal_output!(OPAMP1, ADC1, 13);
    };
    (opamp, OPAMP2) => {
        impl_opamp_internal_output!(OPAMP2, ADC2, 16);
    };
    (opamp, OPAMP3) => {
        impl_opamp_internal_output!(OPAMP3, ADC2, 18);
        // Only in Cat 3/4 devices
        impl_opamp_internal_output!(OPAMP3, ADC3, 13);
    };
    // OPAMP4 only in Cat 3 devices
    (opamp, OPAMP4) => {
        impl_opamp_internal_output!(OPAMP4, ADC5, 5);
    };
    // OPAMP5 only in Cat 3 devices
    (opamp, OPAMP5) => {
        impl_opamp_internal_output!(OPAMP5, ADC5, 3);
    };
    // OPAMP6 only in Cat 3/4 devices
    (opamp, OPAMP6) => {
        // Only in Cat 3 devices
        impl_opamp_internal_output!(OPAMP6, ADC4, 17);
        // Only in Cat 4 devices
        impl_opamp_internal_output!(OPAMP6, ADC3, 17);
    };
);

foreach_peripheral! {
    (opamp, $inst:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::opamp::Opamp {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_opamp_vp_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::opamp::NonInvertingPin<peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::opamp::SealedNonInvertingPin<peripherals::$inst> for crate::peripherals::$pin {
            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_opamp_vn_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::opamp::InvertingPin<peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::opamp::SealedInvertingPin<peripherals::$inst> for crate::peripherals::$pin {
            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_opamp_vout_pin {
    ($inst:ident, $pin:ident) => {
        impl crate::opamp::OutputPin<peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::opamp::SealedOutputPin<peripherals::$inst> for crate::peripherals::$pin {}
    };
}
