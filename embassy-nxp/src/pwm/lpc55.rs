use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use embassy_hal_internal::{Peri, PeripheralType};

use crate::gpio::AnyPin;
use crate::pac::iocon::vals::{PioDigimode, PioFunc, PioMode, PioOd, PioSlew};
use crate::pac::sct0::vals;
use crate::pac::syscon::vals::{SctRst, SctclkselSel};
use crate::pac::{SCT0, SYSCON};

// Since for now the counter is shared, the TOP value has to be kept.
static TOP_VALUE: AtomicU32 = AtomicU32::new(0);
// To check if there are still active instances.
static REF_COUNT: AtomicU8 = AtomicU8::new(0);

/// The configuration of a PWM output.
/// Note the period in clock cycles of an output can be computed as:
/// `(top + 1) * (phase_correct ? 1 : 2) * divider * prescale_factor`
/// By default, the clock used is 96 MHz.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    /// Inverts the PWM output signal.
    pub invert: bool,
    /// Enables phase-correct mode for PWM operation.
    /// In phase-correct mode, the PWM signal is generated in such a way that
    /// the pulse is always centered regardless of the duty cycle.
    /// The output frequency is halved when phase-correct mode is enabled.
    pub phase_correct: bool,
    /// Enables the PWM output, allowing it to generate an output.
    pub enable: bool,
    /// A SYSCON clock divider allows precise control over
    /// the PWM output frequency by gating the PWM counter increment.
    /// A higher value will result in a slower output frequency.
    /// The clock is divided by `divider + 1`.
    pub divider: u8,
    /// Specifies the factor by which the SCT clock is prescaled to produce the unified
    /// counter clock. The counter clock is clocked at the rate of the SCT clock divided by
    /// `PRE + 1`.
    pub prescale_factor: u8,
    /// The output goes high when `compare` is higher than the
    /// counter. A compare of 0 will produce an always low output, while a
    /// compare of `top` will produce an always high output.
    pub compare: u32,
    /// The point at which the counter resets, representing the maximum possible
    /// period. The counter will either wrap to 0 or reverse depending on the
    /// setting of `phase_correct`.
    pub top: u32,
}

impl Config {
    pub fn new(compare: u32, top: u32) -> Self {
        Self {
            invert: false,
            phase_correct: false,
            enable: true,
            divider: 255,
            prescale_factor: 255,
            compare,
            top,
        }
    }
}

/// PWM driver.
pub struct Pwm<'d> {
    _pin: Peri<'d, AnyPin>,
    output: usize,
}

impl<'d> Pwm<'d> {
    pub(crate) fn reset() {
        // Reset SCTimer => Reset counter and halt it.
        // It should be done only once during the initialization of the board.
        SYSCON.presetctrl1().modify(|w| w.set_sct_rst(SctRst::ASSERTED));
        SYSCON.presetctrl1().modify(|w| w.set_sct_rst(SctRst::RELEASED));
    }
    fn new_inner<T: Output>(output: usize, channel: Peri<'d, impl OutputChannelPin<T>>, config: Config) -> Self {
        // Enable clocks (Syscon is enabled by default)
        critical_section::with(|_cs| {
            if !SYSCON.ahbclkctrl0().read().iocon() {
                SYSCON.ahbclkctrl0().modify(|w| w.set_iocon(true));
            }
            if !SYSCON.ahbclkctrl1().read().sct() {
                SYSCON.ahbclkctrl1().modify(|w| w.set_sct(true));
            }
        });

        // Choose the clock for PWM.
        SYSCON.sctclksel().modify(|w| w.set_sel(SctclkselSel::ENUM_0X3));
        // For now, 96 MHz.

        // IOCON Setup
        channel.pio().modify(|w| {
            w.set_func(channel.pin_func());
            w.set_digimode(PioDigimode::DIGITAL);
            w.set_slew(PioSlew::STANDARD);
            w.set_mode(PioMode::INACTIVE);
            w.set_od(PioOd::NORMAL);
        });

        Self::configure(output, &config);
        REF_COUNT.fetch_add(1, Ordering::Relaxed);
        Self {
            _pin: channel.into(),
            output,
        }
    }

    /// Create PWM driver with a single 'a' pin as output.
    #[inline]
    pub fn new_output<T: Output>(
        output: Peri<'d, T>,
        channel: Peri<'d, impl OutputChannelPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(output.number(), channel, config)
    }

    /// Set the PWM config.
    pub fn set_config(&mut self, config: &Config) {
        Self::configure(self.output, config);
    }

    fn configure(output_number: usize, config: &Config) {
        // Stop and reset the counter
        SCT0.ctrl().modify(|w| {
            if config.phase_correct {
                w.set_bidir_l(vals::Bidir::UP_DOWN);
            } else {
                w.set_bidir_l(vals::Bidir::UP);
            }
            w.set_halt_l(true); // halt the counter to make new changes
            w.set_clrctr_l(true); // clear the counter
        });
        // Divides clock by 1-255
        SYSCON.sctclkdiv().modify(|w| w.set_div(config.divider));

        SCT0.config().modify(|w| {
            w.set_unify(vals::Unify::UNIFIED_COUNTER);
            w.set_clkmode(vals::Clkmode::SYSTEM_CLOCK_MODE);
            w.set_noreload_l(true);
            w.set_autolimit_l(true);
        });

        // Before setting the match registers, we have to make sure that `compare` is lower or equal to `top`,
        // otherwise the counter will not reach the match and, therefore, no events will happen.
        assert!(config.compare <= config.top);

        if TOP_VALUE.load(Ordering::Relaxed) == 0 {
            // Match 0 will reset the timer using TOP value
            SCT0.match_(0).modify(|w| {
                w.set_matchn_l((config.top & 0xFFFF) as u16);
                w.set_matchn_h((config.top >> 16) as u16);
            });
        } else {
            panic!("The top value cannot be changed after the initialization.");
        }
        // The actual matches that are used for event logic
        SCT0.match_(output_number + 1).modify(|w| {
            w.set_matchn_l((config.compare & 0xFFFF) as u16);
            w.set_matchn_h((config.compare >> 16) as u16);
        });

        SCT0.match_(15).modify(|w| {
            w.set_matchn_l(0);
            w.set_matchn_h(0);
        });

        // Event configuration
        critical_section::with(|_cs| {
            // If it is already set, don't change
            if SCT0.ev(0).ev_ctrl().read().matchsel() != 15 {
                SCT0.ev(0).ev_ctrl().modify(|w| {
                    w.set_matchsel(15);
                    w.set_combmode(vals::Combmode::MATCH);
                    // STATE + statev, where STATE is a on-board variable.
                    w.set_stateld(vals::Stateld::ADD);
                    w.set_statev(0);
                });
            }
        });
        SCT0.ev(output_number + 1).ev_ctrl().modify(|w| {
            w.set_matchsel((output_number + 1) as u8);
            w.set_combmode(vals::Combmode::MATCH);
            w.set_stateld(vals::Stateld::ADD);
            // STATE + statev, where STATE is a on-board variable.
            w.set_statev(0);
        });

        // Assign events to states
        SCT0.ev(0).ev_state().modify(|w| w.set_statemskn(1 << 0));
        SCT0.ev(output_number + 1)
            .ev_state()
            .modify(|w| w.set_statemskn(1 << 0));
        // TODO(frihetselsker): optimize nxp-pac so that `set_clr` and `set_set` are turned into a bit array.
        if config.invert {
            // Low when event 0 is active
            SCT0.out(output_number).out_clr().modify(|w| w.set_clr(1 << 0));
            // High when event `output_number + 1` is active
            SCT0.out(output_number)
                .out_set()
                .modify(|w| w.set_set(1 << (output_number + 1)));
        } else {
            // High when event 0 is active
            SCT0.out(output_number).out_set().modify(|w| w.set_set(1 << 0));
            // Low when event `output_number + 1` is active
            SCT0.out(output_number)
                .out_clr()
                .modify(|w| w.set_clr(1 << (output_number + 1)));
        }

        if config.phase_correct {
            // Take into account the set matches and reverse their actions while counting back.
            SCT0.outputdirctrl()
                .modify(|w| w.set_setclr(output_number, vals::Setclr::L_REVERSED));
        }

        // State 0 by default
        SCT0.state().modify(|w| w.set_state_l(0));
        // Remove halt and start the actual counter
        SCT0.ctrl().modify(|w| {
            w.set_halt_l(!config.enable);
        });
    }

    /// Read PWM counter.
    #[inline]
    pub fn counter(&self) -> u32 {
        SCT0.count().read().0
    }
}

impl<'d> Drop for Pwm<'d> {
    fn drop(&mut self) {
        REF_COUNT.fetch_sub(1, Ordering::AcqRel);
        if REF_COUNT.load(Ordering::Acquire) == 0 {
            TOP_VALUE.store(0, Ordering::Release);
        }
    }
}

trait SealedOutput {
    /// Output number.
    fn number(&self) -> usize;
}

/// PWM Output.
#[allow(private_bounds)]
pub trait Output: PeripheralType + SealedOutput {}

macro_rules! output {
    ($name:ident, $num:expr) => {
        impl SealedOutput for crate::peripherals::$name {
            fn number(&self) -> usize {
                $num
            }
        }
        impl Output for crate::peripherals::$name {}
    };
}

output!(PWM_OUTPUT0, 0);
output!(PWM_OUTPUT1, 1);
output!(PWM_OUTPUT2, 2);
output!(PWM_OUTPUT3, 3);
output!(PWM_OUTPUT4, 4);
output!(PWM_OUTPUT5, 5);
output!(PWM_OUTPUT6, 6);
output!(PWM_OUTPUT7, 7);
output!(PWM_OUTPUT8, 8);
output!(PWM_OUTPUT9, 9);

/// PWM Output Channel.
pub trait OutputChannelPin<T: Output>: crate::gpio::Pin {
    fn pin_func(&self) -> PioFunc;
}

macro_rules! impl_pin {
    ($pin:ident, $output:ident, $func:ident) => {
        impl crate::pwm::inner::OutputChannelPin<crate::peripherals::$output> for crate::peripherals::$pin {
            fn pin_func(&self) -> PioFunc {
                PioFunc::$func
            }
        }
    };
}

impl_pin!(PIO0_2, PWM_OUTPUT0, ALT3);
impl_pin!(PIO0_17, PWM_OUTPUT0, ALT4);
impl_pin!(PIO1_4, PWM_OUTPUT0, ALT4);
impl_pin!(PIO1_23, PWM_OUTPUT0, ALT2);

impl_pin!(PIO0_3, PWM_OUTPUT1, ALT3);
impl_pin!(PIO0_18, PWM_OUTPUT1, ALT4);
impl_pin!(PIO1_8, PWM_OUTPUT1, ALT4);
impl_pin!(PIO1_24, PWM_OUTPUT1, ALT2);

impl_pin!(PIO0_10, PWM_OUTPUT2, ALT5);
impl_pin!(PIO0_15, PWM_OUTPUT2, ALT4);
impl_pin!(PIO0_19, PWM_OUTPUT2, ALT4);
impl_pin!(PIO1_9, PWM_OUTPUT2, ALT4);
impl_pin!(PIO1_25, PWM_OUTPUT2, ALT2);

impl_pin!(PIO0_22, PWM_OUTPUT3, ALT4);
impl_pin!(PIO0_31, PWM_OUTPUT3, ALT4);
impl_pin!(PIO1_10, PWM_OUTPUT3, ALT4);
impl_pin!(PIO1_26, PWM_OUTPUT3, ALT2);

impl_pin!(PIO0_23, PWM_OUTPUT4, ALT4);
impl_pin!(PIO1_3, PWM_OUTPUT4, ALT4);
impl_pin!(PIO1_17, PWM_OUTPUT4, ALT4);

impl_pin!(PIO0_26, PWM_OUTPUT5, ALT4);
impl_pin!(PIO1_18, PWM_OUTPUT5, ALT4);

impl_pin!(PIO0_27, PWM_OUTPUT6, ALT4);
impl_pin!(PIO1_31, PWM_OUTPUT6, ALT4);

impl_pin!(PIO0_28, PWM_OUTPUT7, ALT4);
impl_pin!(PIO1_19, PWM_OUTPUT7, ALT2);

impl_pin!(PIO0_29, PWM_OUTPUT8, ALT4);

impl_pin!(PIO0_30, PWM_OUTPUT9, ALT4);
