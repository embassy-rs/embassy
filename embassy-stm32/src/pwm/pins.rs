use crate::gpio::OptionalPin;

#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

pub(crate) mod sealed {
    use crate::gpio::sealed::OptionalPin;

    pub trait Channel1Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
    pub trait Channel1ComplementaryPin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }

    pub trait Channel2Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
    pub trait Channel2ComplementaryPin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }

    pub trait Channel3Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
    pub trait Channel3ComplementaryPin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }

    pub trait Channel4Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
    pub trait Channel4ComplementaryPin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }

    pub trait ExternalTriggerPin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }

    pub trait BreakInputPin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
    pub trait BreakInputComparator1Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
    pub trait BreakInputComparator2Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }

    pub trait BreakInput2Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
    pub trait BreakInput2Comparator1Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
    pub trait BreakInput2Comparator2Pin<Timer>: OptionalPin {
        unsafe fn configure(&mut self);
    }
}
pub trait Channel1Pin<Timer>: sealed::Channel1Pin<Timer> + OptionalPin + 'static {}
pub trait Channel1ComplementaryPin<Timer>:
    sealed::Channel1ComplementaryPin<Timer> + OptionalPin + 'static
{
}

pub trait Channel2Pin<Timer>: sealed::Channel2Pin<Timer> + 'static {}
pub trait Channel2ComplementaryPin<Timer>:
    sealed::Channel2ComplementaryPin<Timer> + OptionalPin + 'static
{
}

pub trait Channel3Pin<Timer>: sealed::Channel3Pin<Timer> + 'static {}
pub trait Channel3ComplementaryPin<Timer>:
    sealed::Channel3ComplementaryPin<Timer> + OptionalPin + 'static
{
}

pub trait Channel4Pin<Timer>: sealed::Channel4Pin<Timer> + 'static {}
pub trait Channel4ComplementaryPin<Timer>:
    sealed::Channel4ComplementaryPin<Timer> + OptionalPin + 'static
{
}

pub trait ExternalTriggerPin<Timer>:
    sealed::ExternalTriggerPin<Timer> + OptionalPin + 'static
{
}

pub trait BreakInputPin<Timer>: sealed::BreakInputPin<Timer> + OptionalPin + 'static {}
pub trait BreakInputComparator1Pin<Timer>:
    sealed::BreakInputComparator1Pin<Timer> + OptionalPin + 'static
{
}
pub trait BreakInputComparator2Pin<Timer>:
    sealed::BreakInputComparator2Pin<Timer> + OptionalPin + 'static
{
}

pub trait BreakInput2Pin<Timer>: sealed::BreakInput2Pin<Timer> + OptionalPin + 'static {}
pub trait BreakInput2Comparator1Pin<Timer>:
    sealed::BreakInput2Comparator1Pin<Timer> + OptionalPin + 'static
{
}
pub trait BreakInput2Comparator2Pin<Timer>:
    sealed::BreakInput2Comparator2Pin<Timer> + OptionalPin + 'static
{
}

macro_rules! impl_no_pin {
    ($timer:ident, $signal:ident) => {
        impl crate::pwm::pins::sealed::$signal<crate::peripherals::$timer> for crate::gpio::NoPin {
            unsafe fn configure(&mut self) {}
        }
        impl crate::pwm::pins::$signal<crate::peripherals::$timer> for crate::gpio::NoPin {}
    };
}

#[allow(unused)]
macro_rules! impl_pin {
    ($timer:ident, $signal:ident, $pin:ident, $af:expr) => {
        impl crate::pwm::pins::sealed::$signal<crate::peripherals::$timer>
            for crate::peripherals::$pin
        {
            unsafe fn configure(&mut self) {
                use crate::gpio::sealed::{AFType, Pin};
                use crate::gpio::Speed;
                self.set_low();
                self.set_speed(Speed::VeryHigh);
                self.set_as_af($af, AFType::OutputPushPull);
            }
        }

        impl crate::pwm::pins::$signal<crate::peripherals::$timer> for crate::peripherals::$pin {}
    };
}
