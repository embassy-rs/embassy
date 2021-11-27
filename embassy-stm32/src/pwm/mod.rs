use crate::gpio;
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use stm32_metapac::timer::vals::Ocm;

pub struct Pwm<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

// TIM2

pub struct Ch1 {}
pub struct Ch2 {}
pub struct Ch3 {}
pub struct Ch4 {}

#[derive(Clone, Copy)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
}

impl<'d, T: Instance> Pwm<'d, T> {
    pub fn new<F: Into<Hertz>>(
        _tim: impl Unborrow<Target = T> + 'd,
        ch1: impl Unborrow<Target = impl PwmPin<T, Ch1>> + 'd,
        ch2: impl Unborrow<Target = impl PwmPin<T, Ch2>> + 'd,
        ch3: impl Unborrow<Target = impl PwmPin<T, Ch3>> + 'd,
        ch4: impl Unborrow<Target = impl PwmPin<T, Ch4>> + 'd,
        freq: F,
    ) -> Self {
        unborrow!(ch1, ch2, ch3, ch4);

        T::enable();
        T::reset();
        let r = T::regs();

        let mut this = Pwm {
            phantom: PhantomData,
        };
        unsafe {
            ch1.configure();
            ch2.configure();
            ch3.configure();
            ch4.configure();
        }

        unsafe {
            use stm32_metapac::timer::vals::Dir;
            this.set_freq(freq);
            r.cr1().write(|w| {
                w.set_cen(true);
                w.set_dir(Dir::UP)
            });

            this.set_ocm(Channel::Ch1, Ocm::PWMMODE1);
            this.set_ocm(Channel::Ch2, Ocm::PWMMODE1);
            this.set_ocm(Channel::Ch3, Ocm::PWMMODE1);
            this.set_ocm(Channel::Ch4, Ocm::PWMMODE1);
        }
        this
    }

    unsafe fn set_ocm(&mut self, channel: Channel, mode: Ocm) {
        let r = T::regs();
        match channel {
            Channel::Ch1 => r.ccmr_output(0).modify(|w| w.set_ocm(0, mode)),
            Channel::Ch2 => r.ccmr_output(0).modify(|w| w.set_ocm(1, mode)),
            Channel::Ch3 => r.ccmr_output(1).modify(|w| w.set_ocm(0, mode)),
            Channel::Ch4 => r.ccmr_output(1).modify(|w| w.set_ocm(1, mode)),
        }
    }

    unsafe fn set_enable(&mut self, channel: Channel, enable: bool) {
        let r = T::regs();
        match channel {
            Channel::Ch1 => r.ccer().modify(|w| w.set_cce(0, enable)),
            Channel::Ch2 => r.ccer().modify(|w| w.set_cce(1, enable)),
            Channel::Ch3 => r.ccer().modify(|w| w.set_cce(2, enable)),
            Channel::Ch4 => r.ccer().modify(|w| w.set_cce(3, enable)),
        }
    }

    pub fn enable(&mut self, channel: Channel) {
        unsafe { self.set_enable(channel, true) }
    }

    pub fn disable(&mut self, channel: Channel) {
        unsafe { self.set_enable(channel, false) }
    }

    pub fn set_freq<F: Into<Hertz>>(&mut self, freq: F) {
        use core::convert::TryInto;
        let clk = T::frequency();
        let r = T::regs();
        let freq: Hertz = freq.into();
        let ticks: u32 = clk.0 / freq.0;
        let psc: u16 = (ticks / (1 << 16)).try_into().unwrap();
        let arr: u16 = (ticks / (u32::from(psc) + 1)).try_into().unwrap();
        unsafe {
            r.psc().write(|w| w.set_psc(psc));
            r.arr().write(|w| w.set_arr(arr));
        }
    }

    pub fn get_max_duty(&self) -> u32 {
        let r = T::regs();
        unsafe { r.arr().read().arr() as u32 }
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u32) {
        use core::convert::TryInto;
        assert!(duty < self.get_max_duty());
        let duty: u16 = duty.try_into().unwrap();
        let r = T::regs();
        unsafe {
            match channel {
                Channel::Ch1 => r.ccr(0).modify(|w| w.set_ccr(duty)),
                Channel::Ch2 => r.ccr(1).modify(|w| w.set_ccr(duty)),
                Channel::Ch3 => r.ccr(2).modify(|w| w.set_ccr(duty)),
                Channel::Ch4 => r.ccr(3).modify(|w| w.set_ccr(duty)),
            }
        }
    }
}

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::timer::TimGp16;
    }
}

pub trait Instance: sealed::Instance + Sized + RccPeripheral + 'static {}

#[allow(unused)]
macro_rules! impl_timer {
    ($inst:ident) => {
        impl crate::pwm::sealed::Instance for crate::peripherals::$inst {
            fn regs() -> crate::pac::timer::TimGp16 {
                crate::pac::timer::TimGp16(crate::pac::$inst.0)
            }
        }

        impl crate::pwm::Instance for crate::peripherals::$inst {}
    };
}

pub trait PwmPin<Timer, Channel>: gpio::OptionalPin {
    unsafe fn configure(&mut self);
}

impl<Timer, Channel> PwmPin<Timer, Channel> for gpio::NoPin {
    unsafe fn configure(&mut self) {}
}

#[allow(unused)]
macro_rules! impl_pwm_pin {
    ($timer:ident, $channel:ident, $pin:ident, $af:expr) => {
        impl crate::pwm::PwmPin<crate::peripherals::$timer, crate::pwm::$channel>
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
    };
}

crate::pac::peripherals!(
    (timer, $inst:ident) => { impl_timer!($inst); };
);

crate::pac::peripheral_pins!(
    ($inst:ident, timer,TIM_GP16, $pin:ident, CH1, $af:expr) => {
        impl_pwm_pin!($inst, Ch1, $pin, $af);
    };
    ($inst:ident, timer,TIM_GP16, $pin:ident, CH2, $af:expr) => {
        impl_pwm_pin!($inst, Ch2, $pin, $af);
    };
    ($inst:ident, timer,TIM_GP16, $pin:ident, CH3, $af:expr) => {
        impl_pwm_pin!($inst, Ch3, $pin, $af);
    };
    ($inst:ident, timer,TIM_GP16, $pin:ident, CH4, $af:expr) => {
        impl_pwm_pin!($inst, Ch4, $pin, $af);
    };
);
