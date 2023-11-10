#![macro_use]

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::Peripheral;

#[derive(Clone, Copy)]
pub enum OpAmpGain {
    Mul1,
    Mul2,
    Mul4,
    Mul8,
    Mul16,
}

#[derive(Clone, Copy)]
pub enum OpAmpSpeed {
    Normal,
    HighSpeed,
}

#[cfg(opamp_g4)]
impl From<OpAmpSpeed> for crate::pac::opamp::vals::OpampCsrOpahsm {
    fn from(v: OpAmpSpeed) -> Self {
        match v {
            OpAmpSpeed::Normal => crate::pac::opamp::vals::OpampCsrOpahsm::NORMAL,
            OpAmpSpeed::HighSpeed => crate::pac::opamp::vals::OpampCsrOpahsm::HIGHSPEED,
        }
    }
}

/// OpAmp external outputs, wired to a GPIO pad.
///
/// The GPIO output pad is held by this struct to ensure it cannot be used elsewhere.
///
/// This struct can also be used as an ADC input.
pub struct OpAmpOutput<'d, 'p, T: Instance, P: OutputPin<T>> {
    _inner: &'d OpAmp<'d, T>,
    _output: &'p mut P,
}

/// OpAmp internal outputs, wired directly to ADC inputs.
///
/// This struct can be used as an ADC input.
pub struct OpAmpInternalOutput<'d, T: Instance> {
    _inner: &'d OpAmp<'d, T>,
}

/// OpAmp driver.
pub struct OpAmp<'d, T: Instance> {
    _inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> OpAmp<'d, T> {
    /// Create a new driver instance.
    ///
    /// Enables the OpAmp and configures the speed, but
    /// does not set any other configuration.
    pub fn new(opamp: impl Peripheral<P = T> + 'd, #[cfg(opamp_g4)] speed: OpAmpSpeed) -> Self {
        into_ref!(opamp);

        #[cfg(opamp_f3)]
        T::regs().opampcsr().modify(|w| {
            w.set_opampen(true);
        });

        #[cfg(opamp_g4)]
        T::regs().opamp_csr().modify(|w| {
            w.set_opaen(true);
            w.set_opahsm(speed.into());
        });

        Self { _inner: opamp }
    }

    /// Configure the OpAmp as a buffer for the provided input pin,
    /// outputting to the provided output pin.
    ///
    /// The input pin is configured for analogue mode but not consumed,
    /// so it may subsequently be used for ADC or comparator inputs.
    ///
    /// The output pin is held within the returned [`OpAmpOutput`] struct,
    /// preventing it being used elsewhere. The `OpAmpOutput` can then be
    /// directly used as an ADC input.
    pub fn buffer_ext<'a, 'b, IP, OP>(
        &'a mut self,
        in_pin: &IP,
        out_pin: &'b mut OP,
        gain: OpAmpGain,
    ) -> OpAmpOutput<'a, 'b, T, OP>
    where
        IP: NonInvertingPin<T> + crate::gpio::sealed::Pin,
        OP: OutputPin<T> + crate::gpio::sealed::Pin,
    {
        in_pin.set_as_analog();
        out_pin.set_as_analog();

        let (vm_sel, pga_gain) = match gain {
            OpAmpGain::Mul1 => (0b11, 0b00),
            OpAmpGain::Mul2 => (0b10, 0b00),
            OpAmpGain::Mul4 => (0b10, 0b01),
            OpAmpGain::Mul8 => (0b10, 0b10),
            OpAmpGain::Mul16 => (0b10, 0b11),
        };

        #[cfg(opamp_f3)]
        T::regs().opampcsr().modify(|w| {
            w.set_vp_sel(in_pin.channel());
            w.set_vm_sel(vm_sel);
            w.set_pga_gain(pga_gain);
            w.set_opampen(true);
        });

        #[cfg(opamp_g4)]
        T::regs().opamp_csr().modify(|w| {
            use crate::pac::opamp::vals::*;

            w.set_vp_sel(OpampCsrVpSel::from_bits(in_pin.channel()));
            w.set_vm_sel(OpampCsrVmSel::from_bits(vm_sel));
            w.set_pga_gain(OpampCsrPgaGain::from_bits(pga_gain));
            w.set_opaintoen(OpampCsrOpaintoen::OUTPUTPIN);
            w.set_opaen(true);
        });

        OpAmpOutput {
            _inner: self,
            _output: out_pin,
        }
    }

    /// Configure the OpAmp as a buffer for the provided input pin,
    /// with the output only used internally.
    ///
    /// The input pin is configured for analogue mode but not consumed,
    /// so it may be subsequently used for ADC or comparator inputs.
    ///
    /// The returned `OpAmpInternalOutput` struct may be used as an ADC input.
    #[cfg(opamp_g4)]
    pub fn buffer_int<'a, P>(&'a mut self, pin: &P, gain: OpAmpGain) -> OpAmpInternalOutput<'a, T>
    where
        P: NonInvertingPin<T> + crate::gpio::sealed::Pin,
    {
        pin.set_as_analog();

        let (vm_sel, pga_gain) = match gain {
            OpAmpGain::Mul1 => (0b11, 0b00),
            OpAmpGain::Mul2 => (0b10, 0b00),
            OpAmpGain::Mul4 => (0b10, 0b01),
            OpAmpGain::Mul8 => (0b10, 0b10),
            OpAmpGain::Mul16 => (0b10, 0b11),
        };

        T::regs().opamp_csr().modify(|w| {
            use crate::pac::opamp::vals::*;
            w.set_vp_sel(OpampCsrVpSel::from_bits(pin.channel()));
            w.set_vm_sel(OpampCsrVmSel::from_bits(vm_sel));
            w.set_pga_gain(OpampCsrPgaGain::from_bits(pga_gain));
            w.set_opaintoen(OpampCsrOpaintoen::ADCCHANNEL);
            w.set_opaen(true);
        });

        OpAmpInternalOutput { _inner: self }
    }
}

impl<'d, T: Instance> Drop for OpAmp<'d, T> {
    fn drop(&mut self) {
        #[cfg(opamp_f3)]
        T::regs().opampcsr().modify(|w| {
            w.set_opampen(false);
        });

        #[cfg(opamp_g4)]
        T::regs().opamp_csr().modify(|w| {
            w.set_opaen(false);
        });
    }
}

pub trait Instance: sealed::Instance + 'static {}

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::opamp::Opamp;
    }

    pub trait NonInvertingPin<T: Instance> {
        fn channel(&self) -> u8;
    }

    pub trait InvertingPin<T: Instance> {
        fn channel(&self) -> u8;
    }

    pub trait OutputPin<T: Instance> {}
}

pub trait NonInvertingPin<T: Instance>: sealed::NonInvertingPin<T> {}
pub trait InvertingPin<T: Instance>: sealed::InvertingPin<T> {}
pub trait OutputPin<T: Instance>: sealed::OutputPin<T> {}

macro_rules! impl_opamp_external_output {
    ($inst:ident, $adc:ident, $ch:expr) => {
        foreach_adc!(
            ($adc, $common_inst:ident, $adc_clock:ident) => {
                impl<'d, 'p, P: OutputPin<crate::peripherals::$inst>> crate::adc::sealed::AdcPin<crate::peripherals::$adc>
                    for OpAmpOutput<'d, 'p, crate::peripherals::$inst, P>
                {
                    fn channel(&self) -> u8 {
                        $ch
                    }
                }

                impl<'d, 'p, P: OutputPin<crate::peripherals::$inst>> crate::adc::AdcPin<crate::peripherals::$adc>
                    for OpAmpOutput<'d, 'p, crate::peripherals::$inst, P>
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
        impl_opamp_external_output!(OPAMP3, ADC3, 1);
    };
    // OPAMP4 only in STM32G4 Cat 3 devices
    (opamp, OPAMP4) => {
        impl_opamp_external_output!(OPAMP4, ADC4, 3);
    };
    // OPAMP5 only in STM32G4 Cat 3 devices
    (opamp, OPAMP5) => {
        impl_opamp_external_output!(OPAMP5, ADC5, 1);
    };
    // OPAMP6 only in STM32G4 Cat 3/4 devices
    (opamp, OPAMP6) => {
        impl_opamp_external_output!(OPAMP6, ADC1, 14);
    };
);

#[cfg(opamp_g4)]
macro_rules! impl_opamp_internal_output {
    ($inst:ident, $adc:ident, $ch:expr) => {
        foreach_adc!(
            ($adc, $common_inst:ident, $adc_clock:ident) => {
                impl<'d> crate::adc::sealed::AdcPin<crate::peripherals::$adc>
                    for OpAmpInternalOutput<'d, crate::peripherals::$inst>
                {
                    fn channel(&self) -> u8 {
                        $ch
                    }
                }

                impl<'d> crate::adc::AdcPin<crate::peripherals::$adc>
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
        impl sealed::Instance for crate::peripherals::$inst {
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
        impl crate::opamp::sealed::NonInvertingPin<peripherals::$inst> for crate::peripherals::$pin {
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
        impl crate::opamp::sealed::OutputPin<peripherals::$inst> for crate::peripherals::$pin {}
    };
}
