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
    pub target_fps: Hertz,
    pub drive: Drive,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            use_voltage_output_buffer: false,
            use_segment_muxing: false,
            bias: Default::default(),
            duty: Default::default(),
            voltage_source: Default::default(),
            target_fps: Hertz(30),
            drive: Drive::Medium,
        }
    }
}

/// The number of voltage levels used when driving an LCD.
/// Your LCD datasheet should tell you what to use.
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Bias {
    /// 1/4 bias
    #[default]
    Quarter = 0b00,
    /// 1/2 bias
    Half = 0b01,
    /// 1/3 bias
    Third = 0b10,
}

/// The duty used by the LCD driver.
///
/// This is essentially how many COM pins you're using.
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Duty {
    #[default]
    /// Use a single COM pin
    Static = 0b000,
    /// Use two COM pins
    Half = 0b001,
    /// Use three COM pins
    Third = 0b010,
    /// Use four COM pins
    Quarter = 0b011,
    /// Use eight COM pins.
    ///
    /// In this mode, `COM[7:4]` outputs are available on `SEG[51:48]`.
    /// This allows reducing the number of available segments.
    Eigth = 0b100,
}

/// Whether to use the internal or external voltage source to drive the LCD
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum VoltageSource {
    #[default]
    /// Voltage stepup converter
    Internal,
    /// VLCD pin
    External,
}

/// Defines the pulse duration in terms of ck_ps pulses.
///
/// A short pulse leads to lower power consumption, but displays with high internal resistance
/// may need a longer pulse to achieve satisfactory contrast.
/// Note that the pulse is never longer than one half prescaled LCD clock period.
///
/// Displays with high internal resistance may need a longer drive time to achieve satisfactory contrast.
/// `PermanentHighDrive` is useful in this case if some additional power consumption can be tolerated.
///
/// Basically, for power usage, you want this as low as possible while still being able to use the LCD
/// with a good enough contrast.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Drive {
    /// Zero clock pulse on duration
    Lowest = 0x00,
    /// One clock pulse on duration
    VeryLow = 0x01,
    /// Two clock pulse on duration
    Low = 0x02,
    /// Three clock pulse on duration
    Medium = 0x03,
    /// Four clock pulse on duration
    MediumHigh = 0x04,
    /// Five clock pulse on duration
    High = 0x05,
    /// Six clock pulse on duration
    VeryHigh = 0x06,
    /// Seven clock pulse on duration
    Highest = 0x07,
    /// Enables the highdrive bit of the hardware
    PermanentHighDrive = 0x09,
}

/// LCD driver.
pub struct Lcd<'d, T: Instance> {
    _peri: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Lcd<'d, T> {
    /// Initialize the lcd driver
    pub fn new<const N: usize>(
        _peri: impl Peripheral<P = T> + 'd,
        config: Config,
        vlcd_pin: impl Peripheral<P = impl VlcdPin<T>> + 'd,
        pins: [LcdPin<'d, T>; N],
    ) -> Self {
        rcc::enable_and_reset::<T>();

        into_ref!(vlcd_pin);
        vlcd_pin.set_as_af(vlcd_pin.af_num(), AFType::OutputPushPull);
        vlcd_pin.set_speed(crate::gpio::Speed::VeryHigh);

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

        trace!(
            "lcd_clk: {}, fps: {}, ps: {}, div: {}",
            lcd_clk,
            best_fps_match,
            ps,
            div
        );

        if best_fps_match == u32::MAX || ps > 0xF {
            panic!("Lcd clock error");
        }

        // Set the frame control
        T::regs().fcr().modify(|w| {
            w.set_ps(ps as u8);
            w.set_div(div as u8);
            w.set_cc(0b100); // Init in the middle-ish
            w.set_dead(0b000);
            w.set_pon(config.drive as u8 & 0x07);
            w.set_hd((config.drive as u8 & !0x07) != 0);
        });

        // Wait for the frame control to synchronize
        while !T::regs().sr().read().fcrsf() {}

        // Set the control register values
        T::regs().cr().modify(|w| {
            w.set_bufen(config.use_voltage_output_buffer);
            w.set_mux_seg(config.use_segment_muxing);
            w.set_bias(config.bias as u8);
            w.set_duty(config.duty as u8);
            w.set_vsel(matches!(config.voltage_source, VoltageSource::External));
        });

        // Enable the lcd
        T::regs().cr().modify(|w| w.set_lcden(true));

        // Wait for the lcd to be enabled
        while !T::regs().sr().read().ens() {}

        // Wait for the stepup converter to be ready
        while !T::regs().sr().read().rdy() {}

        Self { _peri: PhantomData }
    }

    /// Change the contrast by changing the voltage being used.
    /// 
    /// This from low at 0 to high at 7.
    pub fn set_contrast_control(&mut self, value: u8) {
        T::regs().fcr().modify(|w| w.set_cc(value));
    }

    /// Change the contrast by introducing a deadtime to the signals
    /// where the voltages are held at 0V.
    /// 
    /// This from no dead time at 0 to high dead time at 7.
    pub fn set_dead_time(&mut self, value: u8) {
        T::regs().fcr().modify(|w: &mut stm32_metapac::lcd::regs::Fcr| w.set_dead(value));
    }

    /// Write frame data to the peripheral.
    /// 
    /// What each bit means depends on the exact microcontroller you use,
    /// which pins are connected to your LCD and also the LCD layout itself.
    /// 
    /// This function blocks until the last update display request has been processed.
    pub fn write_frame(&mut self, data: &[u32; 16]) {
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
pin_trait!(VlcdPin, Instance);

// TODO: pull into build.rs, but the metapack doesn't have this info
pin_trait_impl!(crate::lcd::VlcdPin, LCD, PC3, 11);
pin_trait_impl!(crate::lcd::VlcdPin, LCD, PB2, 11);

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
