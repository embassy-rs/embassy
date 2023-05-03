#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pio::{
    FifoJoin, Pio, PioCommon, PioInstance, PioPin, PioStateMachine, PioStateMachineInstance, ShiftDirection,
};
use embassy_rp::pio_instr_util;
use embassy_rp::relocate::RelocatedProgram;
use embassy_time::{Duration, Timer};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};
pub struct Ws2812<'d, P: PioInstance, const S: usize> {
    sm: PioStateMachineInstance<'d, P, S>,
}

impl<'d, P: PioInstance, const S: usize> Ws2812<'d, P, S> {
    pub fn new(mut pio: PioCommon<'d, P>, mut sm: PioStateMachineInstance<'d, P, S>, pin: impl PioPin) -> Self {
        // Setup sm0

        // prepare the PIO program
        let side_set = pio::SideSet::new(false, 1, false);
        let mut a: pio::Assembler<32> = pio::Assembler::new_with_side_set(side_set);

        const T1: u8 = 2; // start bit
        const T2: u8 = 5; // data bit
        const T3: u8 = 3; // stop bit
        const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;

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

        let relocated = RelocatedProgram::new(&prg);
        pio.write_instr(relocated.origin() as usize, relocated.code());
        pio_instr_util::exec_jmp(&mut sm, relocated.origin());

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        sm.set_set_pins(&[&out_pin]);
        sm.set_sideset_base_pin(&out_pin);
        sm.set_sideset_count(1);

        // Clock config
        // TODO CLOCK_FREQ should come from embassy_rp
        const CLOCK_FREQ: u32 = 125_000_000;
        const WS2812_FREQ: u32 = 800_000;

        let bit_freq = WS2812_FREQ * CYCLES_PER_BIT;
        let mut int = CLOCK_FREQ / bit_freq;
        let rem = CLOCK_FREQ - (int * bit_freq);
        let frac = (rem * 256) / bit_freq;
        // 65536.0 is represented as 0 in the pio's clock divider
        if int == 65536 {
            int = 0;
        }

        sm.set_clkdiv((int << 8) | frac);
        let pio::Wrap { source, target } = relocated.wrap();
        sm.set_wrap(source, target);

        // FIFO config
        sm.set_autopull(true);
        sm.set_fifo_join(FifoJoin::TxOnly);
        sm.set_pull_threshold(24);
        sm.set_out_shift_dir(ShiftDirection::Left);

        sm.set_enable(true);

        Self { sm }
    }

    pub async fn write(&mut self, colors: &[RGB8]) {
        for color in colors {
            let word = (u32::from(color.g) << 24) | (u32::from(color.r) << 16) | (u32::from(color.b) << 8);
            self.sm.wait_push(word).await;
        }
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio { common, sm0, .. } = Pio::new(p.PIO0);

    // This is the number of leds in the string. Helpfully, the sparkfun thing plus and adafruit
    // feather boards for the 2040 both have one built in.
    const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];

    // For the thing plus, use pin 8
    // For the feather, use pin 16
    let mut ws2812 = Ws2812::new(common, sm0, p.PIN_8);

    // Loop forever making RGB values and pushing them out to the WS2812.
    loop {
        for j in 0..(256 * 5) {
            debug!("New Colors:");
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
                debug!("R: {} G: {} B: {}", data[i].r, data[i].g, data[i].b);
            }
            ws2812.write(&data).await;

            Timer::after(Duration::from_micros(5)).await;
        }
    }
}
