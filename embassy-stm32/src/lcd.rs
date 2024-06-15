//! LCD
use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::gpio::{AFType, AnyPin, SealedPin};
use crate::rcc::{self, RccPeripheral};
use crate::time::Hertz;
use crate::{peripherals, Peripheral};

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub use_voltage_output_buffer: bool,
    pub use_segment_muxing: bool,
    pub bias: Bias,
    pub duty: Duty,
    pub voltage_source: VoltageSource,
    pub high_drive: bool,
    pub target_fps: Hertz,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            use_voltage_output_buffer: Default::default(),
            use_segment_muxing: Default::default(),
            bias: Default::default(),
            duty: Default::default(),
            voltage_source: Default::default(),
            high_drive: Default::default(),
            target_fps: Hertz(30),
        }
    }
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

        // Set the pins
        for pin in pins {
            pin.pin.set_as_af(pin.af_num, AFType::OutputPushPull);
            pin.pin.set_speed(crate::gpio::Speed::VeryHigh);
        }

        // Initialize the display ram to 0
        for i in 0..8 {
            T::regs().ram_com(i).low().write_value(0);
            T::regs().ram_com(i).high().write_value(0);
        }

        // Calculate the clock dividers
        let Some(lcd_clk) = (unsafe { rcc::get_freqs().rtc }) else {
            panic!("The LCD driver needs the RTC/LCD clock to be running");
        };
        let duty_divider = match config.duty {
            Duty::Static => 1,
            Duty::Half => 2,
            Duty::Third => 3,
            Duty::Quarter => 4,
            Duty::Eigth => 8,
        };
        let target_clock = config.target_fps.0 * duty_divider;
        let target_division = lcd_clk.0 / target_clock;

        let mut ps = 0;
        let mut div = 0;
        let mut best_fps_match = u32::MAX;

        for trial_div in 0..0xF {
            let trial_ps = (target_division / (trial_div + 16))
                .next_power_of_two()
                .trailing_zeros();
            let fps = lcd_clk.0 / ((1 << trial_ps) * (trial_div + 16)) / duty_divider;

            if fps < config.target_fps.0 {
                continue;
            }

            if fps < best_fps_match {
                ps = trial_ps;
                div = trial_div;
                best_fps_match = fps;
            }
        }

        trace!("lcd_clk: {}, fps: {}, ps: {}, div: {}", lcd_clk, best_fps_match, ps, div);

        if best_fps_match == u32::MAX || ps > 0xF {
            panic!("Lcd clock error");
        }

        // Set the frame control
        T::regs().fcr().modify(|w| {
            w.0 = 0x7C5C41;
            // w.set_ps(ps as u8);
            // w.set_div(div as u8);
            // w.set_cc(0);
            // w.set_dead(0);
            // w.set_pon(0);
            // // w.set_hd(config.high_drive);
        });

        // Wait for the frame control to synchronize
        while !T::regs().sr().read().fcrsf() {}

        // Set the control register values
        T::regs().cr().modify(|w| {
            w.0 = 0x4E;
            // w.set_bufen(config.use_voltage_output_buffer);
            // w.set_mux_seg(config.use_segment_muxing);
            // w.set_bias(config.bias as u8);
            // w.set_duty(config.duty as u8);
            // w.set_vsel(matches!(config.voltage_source, VoltageSource::External));
        });

        // Enable the lcd
        T::regs().cr().modify(|w| w.set_lcden(true));

        // Wait for the lcd to be enabled
        while !T::regs().sr().read().ens() {}

        // Wait for the stepup converter to be ready
        while !T::regs().sr().read().rdy() {}

        Self { _peri: PhantomData }
    }

    pub fn write_frame(&mut self, data: &[u32; 16]) {
        defmt::info!("{:06b}", T::regs().sr().read().0);

        // Wait until the last update is done
        while T::regs().sr().read().udr() {}

        for i in 0..8 {
            T::regs().ram_com(i).low().write_value(data[i * 2]);
            T::regs().ram_com(i).low().write_value(data[i * 2 + 1]);
        }

        // Clear the update done flag
        T::regs().sr().write(|w| w.set_udd(true));
        // Set the update request flag
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
