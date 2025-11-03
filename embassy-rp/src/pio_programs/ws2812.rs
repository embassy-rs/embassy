//! [ws2812](https://www.sparkfun.com/categories/tags/ws2812)

use embassy_time::Timer;
use fixed::types::U24F8;
use smart_leds::{RGB8, RGBW};

use crate::Peri;
use crate::clocks::clk_sys_freq;
use crate::dma::{AnyChannel, Channel};
use crate::pio::{
    Common, Config, FifoJoin, Instance, LoadedProgram, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};

const T1: u8 = 2; // start bit
const T2: u8 = 5; // data bit
const T3: u8 = 3; // stop bit
const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;

/// Color orders for WS2812B, type RGB8
pub trait RgbColorOrder {
    /// Pack an 8-bit RGB color into a u32
    fn pack(color: RGB8) -> u32;
}

/// Green, Red, Blue order is the common default for WS2812B
pub struct Grb;
impl RgbColorOrder for Grb {
    /// Pack an 8-bit RGB color into a u32 in GRB order
    fn pack(color: RGB8) -> u32 {
        (u32::from(color.g) << 24) | (u32::from(color.r) << 16) | (u32::from(color.b) << 8)
    }
}

/// Red, Green, Blue is used by some WS2812B implementations
pub struct Rgb;
impl RgbColorOrder for Rgb {
    /// Pack an 8-bit RGB color into a u32 in RGB order
    fn pack(color: RGB8) -> u32 {
        (u32::from(color.r) << 24) | (u32::from(color.g) << 16) | (u32::from(color.b) << 8)
    }
}

/// Color orders RGBW strips
pub trait RgbwColorOrder {
    /// Pack an RGB+W color into a u32
    fn pack(color: RGBW<u8>) -> u32;
}

/// Green, Red, Blue, White order is the common default for RGBW strips
pub struct Grbw;
impl RgbwColorOrder for Grbw {
    /// Pack an RGB+W color into a u32 in GRBW order
    fn pack(color: RGBW<u8>) -> u32 {
        (u32::from(color.g) << 24) | (u32::from(color.r) << 16) | (u32::from(color.b) << 8) | u32::from(color.a.0)
    }
}

/// Red, Green, Blue, White order
pub struct Rgbw;
impl RgbwColorOrder for Rgbw {
    /// Pack an RGB+W color into a u32 in RGBW order
    fn pack(color: RGBW<u8>) -> u32 {
        (u32::from(color.r) << 24) | (u32::from(color.g) << 16) | (u32::from(color.b) << 8) | u32::from(color.a.0)
    }
}

/// This struct represents a ws2812 program loaded into pio instruction memory.
pub struct PioWs2812Program<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioWs2812Program<'a, PIO> {
    /// Load the ws2812 program into the given pio
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let side_set = pio::SideSet::new(false, 1, false);
        let mut a: pio::Assembler<32> = pio::Assembler::new_with_side_set(side_set);

        let mut wrap_target = a.label();
        let mut wrap_source = a.label();
        let mut do_zero = a.label();
        a.set_with_side_set(pio::SetDestination::PINDIRS, 1, 0);
        a.bind(&mut wrap_target);
        // Do stop bit
        a.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
        // Do start bit
        a.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
        // Do data bit = 1
        a.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, T2 - 1, 1);
        a.bind(&mut do_zero);
        // Do data bit = 0
        a.nop_with_delay_and_side_set(T2 - 1, 0);
        a.bind(&mut wrap_source);

        let prg = a.assemble_with_wrap(wrap_source, wrap_target);
        let prg = common.load_program(&prg);

        Self { prg }
    }
}

/// Pio backed RGB ws2812 driver
/// Const N is the number of ws2812 leds attached to this pin
pub struct PioWs2812<'d, P: Instance, const S: usize, const N: usize, ORDER>
where
    ORDER: RgbColorOrder,
{
    dma: Peri<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
    _order: core::marker::PhantomData<ORDER>,
}

impl<'d, P: Instance, const S: usize, const N: usize> PioWs2812<'d, P, S, N, Grb> {
    /// Configure a pio state machine to use the loaded ws2812 program.
    /// Uses the default GRB order.
    pub fn new(
        pio: &mut Common<'d, P>,
        sm: StateMachine<'d, P, S>,
        dma: Peri<'d, impl Channel>,
        pin: Peri<'d, impl PioPin>,
        program: &PioWs2812Program<'d, P>,
    ) -> Self {
        Self::with_color_order(pio, sm, dma, pin, program)
    }
}

impl<'d, P: Instance, const S: usize, const N: usize, ORDER> PioWs2812<'d, P, S, N, ORDER>
where
    ORDER: RgbColorOrder,
{
    /// Configure a pio state machine to use the loaded ws2812 program.
    /// Uses the specified color order.
    pub fn with_color_order(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: Peri<'d, impl Channel>,
        pin: Peri<'d, impl PioPin>,
        program: &PioWs2812Program<'d, P>,
    ) -> Self {
        // Setup sm0
        let mut cfg = Config::default();

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);

        cfg.use_program(&program.prg, &[&out_pin]);

        // Clock config, measured in kHz to avoid overflows
        let clock_freq = U24F8::from_num(clk_sys_freq() / 1000);
        let ws2812_freq = U24F8::from_num(800);
        let bit_freq = ws2812_freq * CYCLES_PER_BIT;
        cfg.clock_divider = clock_freq / bit_freq;

        // FIFO config
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 24,
            direction: ShiftDirection::Left,
        };

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            dma: dma.into(),
            sm,
            _order: core::marker::PhantomData,
        }
    }

    /// Write a buffer of [smart_leds::RGB8] to the ws2812 string
    pub async fn write(&mut self, colors: &[RGB8; N]) {
        // Precompute the word bytes from the colors
        let mut words = [0u32; N];
        for i in 0..N {
            words[i] = ORDER::pack(colors[i]);
        }

        // DMA transfer
        self.sm.tx().dma_push(self.dma.reborrow(), &words, false).await;

        Timer::after_micros(55).await;
    }
}

/// Pio backed RGBW ws2812 driver
/// This version is intended for ws2812 leds with 4 addressable lights
/// Const N is the number of ws2812 leds attached to this pin
pub struct RgbwPioWs2812<'d, P: Instance, const S: usize, const N: usize, ORDER>
where
    ORDER: RgbwColorOrder,
{
    dma: Peri<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
    _order: core::marker::PhantomData<ORDER>,
}

impl<'d, P: Instance, const S: usize, const N: usize> RgbwPioWs2812<'d, P, S, N, Grbw> {
    /// Configure a pio state machine to use the loaded ws2812 program.
    /// Uses the default GRBW color order
    pub fn new(
        pio: &mut Common<'d, P>,
        sm: StateMachine<'d, P, S>,
        dma: Peri<'d, impl Channel>,
        pin: Peri<'d, impl PioPin>,
        program: &PioWs2812Program<'d, P>,
    ) -> Self {
        Self::with_color_order(pio, sm, dma, pin, program)
    }
}

impl<'d, P: Instance, const S: usize, const N: usize, ORDER> RgbwPioWs2812<'d, P, S, N, ORDER>
where
    ORDER: RgbwColorOrder,
{
    /// Configure a pio state machine to use the loaded ws2812 program.
    /// Uses the specified color order
    pub fn with_color_order(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: Peri<'d, impl Channel>,
        pin: Peri<'d, impl PioPin>,
        program: &PioWs2812Program<'d, P>,
    ) -> Self {
        // Setup sm0
        let mut cfg = Config::default();

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);

        cfg.use_program(&program.prg, &[&out_pin]);

        // Clock config, measured in kHz to avoid overflows
        let clock_freq = U24F8::from_num(clk_sys_freq() / 1000);
        let ws2812_freq = U24F8::from_num(800);
        let bit_freq = ws2812_freq * CYCLES_PER_BIT;
        cfg.clock_divider = clock_freq / bit_freq;

        // FIFO config
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Left,
        };

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            dma: dma.into(),
            sm,
            _order: core::marker::PhantomData,
        }
    }

    /// Write a buffer of [smart_leds::RGBW] to the ws2812 string
    pub async fn write(&mut self, colors: &[RGBW<u8>; N]) {
        // Precompute the word bytes from the colors
        let mut words = [0u32; N];
        for i in 0..N {
            words[i] = ORDER::pack(colors[i]);
        }

        // DMA transfer
        self.sm.tx().dma_push(self.dma.reborrow(), &words, false).await;

        Timer::after_micros(55).await;
    }
}
