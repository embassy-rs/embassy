//! LCD
use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};

use crate::gpio::{AfType, Flex, SealedPin};
use crate::peripherals;
use crate::rcc::{self, RccPeripheral};
use crate::time::Hertz;

#[cfg(any(stm32u0, stm32l073, stm32l083))]
const NUM_SEGMENTS: u8 = 52;
#[cfg(any(stm32wb, stm32l4x6, stm32l15x, stm32l162, stm32l4x3, stm32l4x6))]
const NUM_SEGMENTS: u8 = 44;
#[cfg(any(stm32l053, stm32l063, stm32l100))]
const NUM_SEGMENTS: u8 = 32;

const MAX_PINS: usize = 32;

/// LCD configuration struct
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct Config {
    #[cfg(lcd_v2)]
    /// Enable the voltage output buffer for higher driving capability.
    ///
    /// The LCD driving capability is improved as buffers prevent the LCD capacitive loads from loading the resistor
    /// bridge unacceptably and interfering with its voltage generation.
    pub use_voltage_output_buffer: bool,
    /// Enable SEG pin remapping. SEG[31:28] multiplexed with SEG[43:40]
    pub use_segment_muxing: bool,
    /// Bias selector
    pub bias: Bias,
    /// Duty selector
    pub duty: Duty,
    /// Internal or external voltage source
    pub voltage_source: VoltageSource,
    /// The frequency used to update the LCD with.
    /// Should be between ~30 and ~100. Lower is better for power consumption, but has lower visual fidelity.
    pub target_fps: Hertz,
    /// LCD driver selector
    pub drive: Drive,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(lcd_v2)]
            use_voltage_output_buffer: false,
            use_segment_muxing: false,
            bias: Default::default(),
            duty: Default::default(),
            voltage_source: Default::default(),
            target_fps: Hertz(60),
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

impl Duty {
    fn num_com_pins(&self) -> u8 {
        match self {
            Duty::Static => 1,
            Duty::Half => 2,
            Duty::Third => 3,
            Duty::Quarter => 4,
            Duty::Eigth => 8,
        }
    }
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

/// Clock dividers for the LCD frame control register, as computed by [`calc_clock_dividers`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClockDividers {
    /// `PS[3:0]`: `ck_ps = lcd_clk / 2^ps`
    ps: u8,
    /// `DIV[3:0]`: `ck_div = ck_ps / (16 + div)`
    div: u8,
}

impl ClockDividers {
    /// Frequency of `ck_div` in Hz.
    fn ck_div(&self, lcd_clk: Hertz) -> u32 {
        lcd_clk.0 / ((1 << self.ps) * (self.div as u32 + 16))
    }

    /// Resulting frame rate in Hz.
    fn fps(&self, lcd_clk: Hertz, duty: Duty) -> u32 {
        self.ck_div(lcd_clk) / duty.num_com_pins() as u32
    }
}

/// Find the `PS`/`DIV` combination that gets closest to `target_fps` without going below it.
///
/// Returns `None` if no combination that fits the `PS`/`DIV` register fields reaches `target_fps`.
fn calc_clock_dividers(lcd_clk: Hertz, target_fps: Hertz, duty: Duty) -> Option<ClockDividers> {
    let duty_divider = duty.num_com_pins() as u32;
    let target_clock = target_fps.0 * duty_divider;
    let target_division = lcd_clk.0 / target_clock;

    let mut best: Option<(ClockDividers, u32)> = None;

    for trial_div in 0..0xFu8 {
        // The largest power of two prescaler that doesn't push the frame rate below the target.
        // Rounding *up* to the next power of two would overdivide the clock, which only ever
        // yields a frame rate below the target.
        // `PS` is a 4-bit field, so skip prescalers that don't fit.
        let Some(trial_ps) = (target_division / (trial_div as u32 + 16))
            .checked_ilog2()
            .filter(|ps| *ps <= 0xF)
        else {
            continue;
        };
        let trial = ClockDividers {
            ps: trial_ps as u8,
            div: trial_div,
        };
        let fps = trial.fps(lcd_clk, duty);

        if fps < target_fps.0 {
            continue;
        }

        if best.is_none_or(|(_, best_fps)| fps < best_fps) {
            best = Some((trial, fps));
        }
    }

    best.map(|(dividers, _)| dividers)
}

/// LCD driver.
pub struct Lcd<'d, T: Instance> {
    _peri: PhantomData<&'d mut T>,
    _pins: heapless::Vec<Flex<'d>, MAX_PINS>,
    duty: Duty,
    ck_div: u32,
}

impl<'d, T: Instance> Lcd<'d, T> {
    /// Initialize the lcd driver.
    ///
    /// The `pins` parameter must contain *all* segment and com pins that are connected to the LCD.
    /// This is not further checked by this driver. Pins not routed to the LCD can be used for other purposes.
    pub fn new<const N: usize>(
        _peripheral: Peri<'d, T>,
        config: Config,
        vlcd_pin: Peri<'_, impl VlcdPin<T>>,
        mut pins: [LcdPin<'d, T>; N],
    ) -> Self {
        rcc::enable_and_reset::<T>();

        vlcd_pin.set_as_af(
            vlcd_pin.af_num(),
            AfType::output(crate::gpio::OutputType::PushPull, crate::gpio::Speed::VeryHigh),
        );

        assert_eq!(
            pins.iter().filter(|pin| !pin.is_seg).count(),
            config.duty.num_com_pins() as usize,
            "The number of provided COM pins is not the same as the duty configures"
        );

        // Set the pins
        for pin in pins.iter_mut() {
            pin.pin.pin.set_as_af(
                pin.af_num,
                AfType::output(crate::gpio::OutputType::PushPull, crate::gpio::Speed::VeryHigh),
            );
        }

        // Initialize the display ram to 0
        for i in 0..8 {
            T::regs().ram_com(i).low().write_value(0);
            T::regs().ram_com(i).high().write_value(0);
        }

        // Calculate the clock dividers
        let Some(lcd_clk) = (unsafe { rcc::get_freqs().rtc.to_hertz() }) else {
            panic!("The LCD driver needs the RTC/LCD clock to be running");
        };
        let Some(dividers) = calc_clock_dividers(lcd_clk, config.target_fps, config.duty) else {
            panic!("Lcd clock error");
        };
        let ClockDividers { ps, div } = dividers;
        let ck_div = dividers.ck_div(lcd_clk);

        trace!(
            "lcd_clk: {}, fps: {}, ps: {}, div: {}, ck_div: {}",
            lcd_clk,
            dividers.fps(lcd_clk, config.duty),
            ps,
            div,
            ck_div
        );

        // Set the frame control
        T::regs().fcr().modify(|w| {
            w.set_ps(ps);
            w.set_div(div);
            w.set_cc(0b100); // Init in the middle-ish
            w.set_dead(0b000);
            w.set_pon(config.drive as u8 & 0x07);
            w.set_hd((config.drive as u8 & !0x07) != 0);
        });

        // Wait for the frame control to synchronize
        while !T::regs().sr().read().fcrsf() {}

        // Set the control register values
        T::regs().cr().modify(|w| {
            #[cfg(lcd_v2)]
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

        let mut lcd_pins = heapless::Vec::new();
        for p in pins.into_iter() {
            unwrap!(lcd_pins.push(p.pin));
        }

        Self {
            _peri: PhantomData,
            duty: config.duty,
            ck_div,
            _pins: lcd_pins,
        }
    }

    /// Change the contrast by changing the voltage being used.
    ///
    /// This is from low at 0 to high at 7.
    pub fn set_contrast_control(&mut self, value: u8) {
        assert!((0..=7).contains(&value));
        T::regs().fcr().modify(|w| w.set_cc(value));
    }

    /// Change the contrast by introducing a deadtime to the signals
    /// where the voltages are held at 0V.
    ///
    /// This is from no dead time at 0 to high dead time at 7.
    pub fn set_dead_time(&mut self, value: u8) {
        assert!((0..=7).contains(&value));
        T::regs()
            .fcr()
            .modify(|w: &mut stm32_metapac::lcd::regs::Fcr| w.set_dead(value));
    }

    /// Write data into the display RAM. This overwrites the data already in it for the specified com index.
    ///
    /// The `com_index` value determines which part of the RAM is written to.
    /// The `segments` value is a bitmap where each bit represents whether a pixel is turned on or off.
    ///
    /// This function waits last update request to be finished, but does not submit the buffer to the LCD with a new request.
    /// Submission has to be done manually using [Self::submit_frame].
    pub fn write_com_segments(&mut self, com_index: u8, segments: u64) {
        while T::regs().sr().read().udr() {}

        assert!(
            com_index < self.duty.num_com_pins(),
            "Com index cannot be higher than number of configured com pins (through the Duty setting in the config)"
        );

        assert!(
            segments.leading_zeros() >= 64 - self.num_segments() as u32,
            "Invalid segment pixel set",
        );

        T::regs()
            .ram_com(com_index as usize)
            .low()
            .write_value((segments & 0xFFFF_FFFF) as u32);
        T::regs()
            .ram_com(com_index as usize)
            .high()
            .write_value(((segments >> 32) & 0xFFFF_FFFF) as u32);
    }

    /// Read the data from the display RAM.
    ///
    /// The `com_index` value determines which part of the RAM is read from.
    ///
    /// This function waits for the last update request to be finished.
    pub fn read_com_segments(&self, com_index: u8) -> u64 {
        while T::regs().sr().read().udr() {}

        assert!(
            com_index < self.duty.num_com_pins(),
            "Com index cannot be higher than number of configured com pins (through the Duty setting in the config)"
        );

        let low = T::regs().ram_com(com_index as usize).low().read();
        let high = T::regs().ram_com(com_index as usize).high().read();

        ((high as u64) << 32) | low as u64
    }

    /// Submit the current RAM data to the LCD.
    ///
    /// This function waits until the RAM is writable, but does not wait for the frame to be drawn.
    pub fn submit_frame(&mut self) {
        while T::regs().sr().read().udr() {}
        // Clear the update done flag
        T::regs().sr().write(|w| w.set_udd(true));
        // Set the update request flag
        T::regs().sr().write(|w| w.set_udr(true));
    }

    /// Get the number of segments that are supported on this LCD
    pub fn num_segments(&self) -> u8 {
        match self.duty {
            Duty::Eigth => NUM_SEGMENTS - 4, // With 8 coms, 4 of the segment pins turn into com pins
            _ => NUM_SEGMENTS,
        }
    }

    /// Get the pixel mask for the current LCD setup.
    /// This is a mask of all bits that are allowed to be set in the [Self::write_com_segments] function.
    pub fn segment_pixel_mask(&self) -> u64 {
        (1 << self.num_segments()) - 1
    }

    /// Get the number of COM pins that were configured through the Drive config
    pub fn num_com_pins(&self) -> u8 {
        self.duty.num_com_pins()
    }

    /// Set the blink behavior on some pixels.
    ///
    /// The blink frequency is an approximation. It's divided from the clock selected by the FPS.
    /// Play with the FPS value if you want the blink frequency to be more accurate.
    ///
    /// If a blink frequency cannot be attained, this function will panic.
    pub fn set_blink(&mut self, selector: BlinkSelector, freq: BlinkFreq) {
        // Freq * 100 to be able to do integer math
        let scaled_blink_freq = match freq {
            BlinkFreq::Hz0_25 => 25,
            BlinkFreq::Hz0_5 => 50,
            BlinkFreq::Hz1 => 100,
            BlinkFreq::Hz2 => 200,
            BlinkFreq::Hz4 => 400,
        };

        let desired_divider = self.ck_div * 100 / scaled_blink_freq;
        let target_divider = desired_divider.next_power_of_two();
        let power_divisions = target_divider.trailing_zeros();

        trace!(
            "Setting LCD blink frequency -> desired_divider: {}, target_divider: {}",
            desired_divider, target_divider
        );

        assert!(
            (8..=1024).contains(&target_divider),
            "LCD blink frequency cannot be attained"
        );

        T::regs().fcr().modify(|reg| {
            reg.set_blinkf((power_divisions - 3) as u8);
            reg.set_blink(selector as u8);
        })
    }
}

impl<'d, T: Instance> Drop for Lcd<'d, T> {
    fn drop(&mut self) {
        // Disable the lcd
        T::regs().cr().modify(|w| w.set_lcden(false));
        rcc::disable::<T>();
    }
}

/// Blink frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlinkFreq {
    /// 0.25 hz
    Hz0_25,
    /// 0.5 hz
    Hz0_5,
    /// 1 hz
    Hz1,
    /// 2 hz
    Hz2,
    /// 4 hz
    Hz4,
}

/// Blink pixel selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlinkSelector {
    /// No pixels blink
    None = 0b00,
    /// The SEG0, COM0 pixel blinks if the pixel is set
    Seg0Com0 = 0b01,
    /// The SEG0 pixel of all COMs blinks if the pixel is set
    Seg0ComAll = 0b10,
    /// All pixels blink if the pixel is set
    All = 0b11,
}

/// A type-erased pin that can be configured as an LCD pin.
/// This is used for passing pins to the new function in the array.
pub struct LcdPin<'d, T: Instance> {
    pin: Flex<'d>,
    af_num: u8,
    is_seg: bool,
    _marker: PhantomData<T>,
}

impl<'d, T: Instance> LcdPin<'d, T> {
    /// Construct an LCD pin from any pin that supports it
    pub fn new_seg(pin: Peri<'d, impl SegPin<T>>) -> Self {
        let af = pin.af_num();

        Self {
            pin: Flex::new(pin),
            af_num: af,
            is_seg: true,
            _marker: PhantomData,
        }
    }

    /// Construct an LCD pin from any pin that supports it
    pub fn new_com(pin: Peri<'d, impl ComPin<T>>) -> Self {
        let af = pin.af_num();

        Self {
            pin: Flex::new(pin),
            af_num: af,
            is_seg: false,
            _marker: PhantomData,
        }
    }
}

trait SealedInstance: crate::rcc::SealedRccPeripheral + PeripheralType {
    fn regs() -> crate::pac::lcd::Lcd;
}

/// DSI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {}

pin_trait!(SegPin, Instance);
pin_trait!(ComPin, Instance);
pin_trait!(VlcdPin, Instance);

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

#[cfg(test)]
mod tests {
    use super::*;

    const LSE: Hertz = Hertz(32_768);

    fn check(clk: Hertz, fps: Hertz, duty: Duty, ps: u8, div: u8, expected_fps: u32) {
        let d = calc_clock_dividers(clk, fps, duty).unwrap();
        assert_eq!(d, ClockDividers { ps, div });
        assert_eq!(d.fps(clk, duty), expected_fps);
        assert!(expected_fps >= fps.0);
    }

    #[test]
    fn derived_values() {
        let d = ClockDividers { ps: 5, div: 1 };
        assert_eq!(d.ck_div(LSE), 32_768 / (32 * 17));
        assert_eq!(d.fps(LSE, Duty::Static), 60);
        assert_eq!(d.fps(LSE, Duty::Quarter), 15);
    }

    #[test]
    fn exact_power_of_two_division() {
        // 32768 / 60 = 546; 546 / 17 = 32 = 2^5 -> fps = 32768 / (32 * 17) = 60
        check(LSE, Hertz(60), Duty::Static, 5, 1, 60);
        // 32768 / (64 * 4) = 128; 128 / 16 = 8 = 2^3 -> fps = 32768 / (8 * 16) / 4 = 64
        check(LSE, Hertz(64), Duty::Quarter, 3, 0, 64);
    }

    #[test]
    fn non_power_of_two_division() {
        // Regression: 32768 / 61 = 537, and 537 / (div + 16) is never a power of two.
        // Rounding up to the next power of two overdivided the clock for every `div`,
        // leaving no candidate above the target and panicking with "Lcd clock error".
        check(LSE, Hertz(61), Duty::Static, 5, 0, 64);
        check(LSE, Hertz(31), Duty::Static, 6, 0, 32);
        check(Hertz(1_000_000), Hertz(60), Duty::Quarter, 8, 0, 61);
        check(Hertz(1_000_000), Hertz(100), Duty::Eigth, 6, 3, 102);
    }

    #[test]
    fn picks_closest_fps_at_or_above_target() {
        for clk in [LSE, Hertz(1_000_000), Hertz(4_000_000)] {
            for duty in [Duty::Static, Duty::Half, Duty::Third, Duty::Quarter, Duty::Eigth] {
                for fps in 30..=100 {
                    let d = calc_clock_dividers(clk, Hertz(fps), duty).unwrap();
                    let d_fps = d.fps(clk, duty);
                    assert!(d_fps >= fps, "{clk} {fps} {duty:?}: {d:?}");

                    // Brute force: no (ps, div) yields a frame rate closer to the target.
                    for ps in 0..=0xFu8 {
                        for div in 0..0xFu8 {
                            let f = ClockDividers { ps, div }.fps(clk, duty);
                            assert!(
                                f < fps || f >= d_fps,
                                "{clk} {fps} {duty:?}: ps={ps} div={div} gives {f} < {d_fps}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unreachable_target() {
        // 32768 / 16 / 8 = 256 fps is the fastest possible frame rate at eighth duty.
        assert!(calc_clock_dividers(LSE, Hertz(256), Duty::Eigth).is_some());
        assert_eq!(calc_clock_dividers(LSE, Hertz(257), Duty::Eigth), None);
        assert_eq!(calc_clock_dividers(LSE, Hertz(100_000), Duty::Static), None);
    }
}
