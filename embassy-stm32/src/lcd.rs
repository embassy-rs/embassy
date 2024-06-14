//! LCD
use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::gpio::{AFType, AnyPin, SealedPin};
use crate::rcc::{self, RccPeripheral};
use crate::{peripherals, Peripheral};

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    pub use_voltage_output_buffer: bool,
    pub use_segment_muxing: bool,
    pub bias: Bias,
    pub duty: Duty,
    pub voltage_source: VoltageSource,
    pub high_drive: bool,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Bias {
    #[default]
    Quarter = 0b00,
    Half = 0b01,
    Third = 0b10,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Duty {
    #[default]
    Static = 0b000,
    Half = 0b001,
    Third = 0b010,
    Quarter = 0b011,
    /// In this mode, `COM[7:4]` outputs are available on `SEG[51:48]`.
    /// This allows reducing the number of available segments.
    Eigth = 0b100,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum VoltageSource {
    #[default]
    /// Voltage stepup converter
    Internal,
    /// VLCD pin
    External,
}

/// LCD driver.
pub struct Lcd<'d, T: Instance> {
    _peri: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Lcd<'d, T> {
    /// Initialize the lcd driver
    pub fn new<const N: usize>(_peri: impl Peripheral<P = T> + 'd, config: Config, pins: [LcdPin<'d, T>; N]) -> Self {
        rcc::enable_and_reset::<T>();

        for pin in pins {
            pin.pin.set_as_af(pin.af_num, AFType::OutputPushPull);
        }

        T::regs().cr().write(|w| {
            w.set_bufen(config.use_voltage_output_buffer);
            w.set_mux_seg(config.use_segment_muxing);
            w.set_bias(config.bias as u8);
            w.set_duty(config.duty as u8);
            w.set_vsel(matches!(config.voltage_source, VoltageSource::External));
        });

        while !T::regs().sr().read().fcrsf() { }

        T::regs().fcr().modify(|w| {
            w.set_dead(0);
            w.set_pon(0b111);
            // w.set_hd(config.high_drive);
        });
        while !T::regs().sr().read().fcrsf() { }

        for i in 0..8 {
            T::regs().ram_com(i).low().write_value(0);
            T::regs().ram_com(i).high().write_value(0);
        }
        T::regs().sr().write(|w| w.set_udr(true));

        while !T::regs().sr().read().fcrsf() { }

        T::regs().fcr().modify(|w| {
            w.set_ps(2);
            w.set_div(4);
        });
        while !T::regs().sr().read().fcrsf() { }

        T::regs().fcr().modify(|w| {
            w.set_cc(7);
        });
        while !T::regs().sr().read().fcrsf() { }

        T::regs().cr().modify(|w| w.set_lcden(true));

        while !T::regs().sr().read().rdy() { }

        Self { _peri: PhantomData }
    }

    pub fn write_frame(&mut self, data: &[u32; 16]) {
        defmt::info!("{:06b}", T::regs().sr().read().0);

        // Wait until the last update is done
        while T::regs().sr().read().udr() { }

        for i  in 0..8 {
            T::regs().ram_com(i).low().write_value(data[i * 2]);
            T::regs().ram_com(i).low().write_value(data[i * 2 + 1]);
        }
        T::regs().sr().write(|w| w.set_udr(true));
    }
}

impl<'d, T: Instance> Drop for Lcd<'d, T> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

pub struct LcdPin<'d, T: Instance> {
    pin: PeripheralRef<'d, AnyPin>,
    af_num: u8,
    _phantom: PhantomData<T>,
}

impl<'d, T: Instance, Pin: Peripheral<P: SegComPin<T>> + 'd> From<Pin> for LcdPin<'d, T> {
    fn from(value: Pin) -> Self {
        Self::new(value)
    }
}

impl<'d, T: Instance> LcdPin<'d, T> {
    pub fn new(pin: impl Peripheral<P = impl SegComPin<T>> + 'd) -> Self {
        into_ref!(pin);

        let af = pin.af_num();

        Self {
            pin: pin.map_into(),
            af_num: af,
            _phantom: PhantomData,
        }
    }
}

trait SealedInstance: crate::rcc::SealedRccPeripheral {
    fn regs() -> crate::pac::lcd::Lcd;
}

/// DSI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {}

pin_trait!(SegComPin, Instance);

foreach_peripheral!(
    (lcd, $inst:ident) => {
        impl crate::lcd::SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::lcd::Lcd {
                crate::pac::$inst
            }
        }

        impl crate::lcd::Instance for peripherals::$inst {}
    };
);
