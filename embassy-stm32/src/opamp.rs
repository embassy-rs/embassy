#![macro_use]

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::Peripheral;

#[cfg(opamp_f3)]
pub struct OpAmpOutput<'d, 'p, T: Instance, P: NonInvertingPin<T>> {
    _inner: &'d OpAmp<'d, T>,
    _input: &'p mut P,
}

pub struct OpAmp<'d, T: Instance> {
    _inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> OpAmp<'d, T> {
    pub fn new(opamp: impl Peripheral<P = T> + 'd) -> Self {
        Self::new_inner(opamp)
    }

    fn new_inner(opamp: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(opamp);

        #[cfg(opamp_f3)]
        T::regs().opampcsr().modify(|w| {
            w.set_opampen(true);
        });

        #[cfg(opamp_g4)]
        T::regs().opamp_csr().modify(|w| {
            w.set_opaen(true);
        });

        Self { _inner: opamp }
    }

    #[cfg(opamp_f3)]
    pub fn buffer_for<'a, 'b, P>(&'a mut self, pin: &'b mut P) -> OpAmpOutput<'a, 'b, T, P>
    where
        P: NonInvertingPin<T>,
    {
        #[cfg(opamp_f3)]
        T::regs().opampcsr().modify(|w| {
            w.set_vp_sel(pin.channel());
        });

        OpAmpOutput {
            _inner: self,
            _input: pin,
        }
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
}

pub trait NonInvertingPin<T: Instance>: sealed::NonInvertingPin<T> {}

pub trait InvertingPin<T: Instance>: sealed::InvertingPin<T> {}

#[cfg(opamp_f3)]
macro_rules! impl_opamp_output {
    ($inst:ident, $adc:ident, $ch:expr) => {
        impl<'d, 'p, P: NonInvertingPin<crate::peripherals::$inst>> crate::adc::sealed::AdcPin<crate::peripherals::$adc>
            for OpAmpOutput<'d, 'p, crate::peripherals::$inst, P>
        {
            fn channel(&self) -> u8 {
                $ch
            }
        }

        impl<'d, 'p, P: NonInvertingPin<crate::peripherals::$inst>> crate::adc::AdcPin<crate::peripherals::$adc>
            for OpAmpOutput<'d, 'p, crate::peripherals::$inst, P>
        {
        }
    };
}

#[cfg(opamp_f3)]
foreach_peripheral!(
    (opamp, OPAMP1) => {
        impl_opamp_output!(OPAMP1, ADC1, 3);
    };
    (opamp, OPAMP2) => {
        impl_opamp_output!(OPAMP2, ADC2, 3);
    };
    (opamp, OPAMP3) => {
        impl_opamp_output!(OPAMP3, ADC3, 1);
    };
    (opamp, OPAMP4) => {
        impl_opamp_output!(OPAMP4, ADC4, 3);
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
macro_rules! impl_opamp_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::opamp::NonInvertingPin<peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::opamp::sealed::NonInvertingPin<peripherals::$inst> for crate::peripherals::$pin {
            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}
