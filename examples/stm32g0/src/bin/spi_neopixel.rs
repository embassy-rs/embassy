#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::word::U5;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const NR_PIXELS: usize = 15;
const BITS_PER_PIXEL: usize = 24; // 24 for rgb, 32 for rgbw
const TOTAL_BITS: usize = NR_PIXELS * BITS_PER_PIXEL;

struct RGB {
    r: u8,
    g: u8,
    b: u8,
}
impl Default for RGB {
    fn default() -> RGB {
        RGB { r: 0, g: 0, b: 0 }
    }
}
pub struct Ws2812 {
    // Note that the U5 type controls the selection of 5 bits to output
    bitbuffer: [U5; TOTAL_BITS],
}

impl Ws2812 {
    pub fn new() -> Ws2812 {
        Ws2812 {
            bitbuffer: [U5(0); TOTAL_BITS],
        }
    }
    fn len(&self) -> usize {
        return NR_PIXELS;
    }
    fn set(&mut self, idx: usize, rgb: RGB) {
        self.render_color(idx, 0, rgb.g);
        self.render_color(idx, 8, rgb.r);
        self.render_color(idx, 16, rgb.b);
    }
    // transform one color byte into an array of 8 byte. Each byte in the array does represent 1 neopixel bit pattern
    fn render_color(&mut self, pixel_idx: usize, offset: usize, color: u8) {
        let mut bits = color as usize;
        let mut idx = pixel_idx * BITS_PER_PIXEL + offset;

        // render one bit in one spi byte. High time first, then the low time
        // clock should be 4 Mhz, 5 bits, each bit is 0.25 us.
        // a one bit is send as a pulse of 0.75 high -- 0.50 low
        // a zero bit is send as a pulse of 0.50 high -- 0.75 low
        // clock frequency for the neopixel is exact 800 khz
        // note that the mosi output should have a resistor to ground of 10k,
        // to assure that between the bursts the line is low
        for _i in 0..8 {
            if idx >= TOTAL_BITS {
                return;
            }
            let pattern = match bits & 0x80 {
                0x80 => 0b0000_1110,
                _ => 0b000_1100,
            };
            bits = bits << 1;
            self.bitbuffer[idx] = U5(pattern);
            idx += 1;
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Start test using spi as neopixel driver");

    let mut config = Config::default();
    config.frequency = Hertz(4_000_000);
    let mut spi = Spi::new_txonly(p.SPI1, p.PB3, p.PB5, p.DMA1_CH3, config); // SCK is unused.

    let mut neopixels = Ws2812::new();

    loop {
        let mut cnt: usize = 0;
        for _i in 0..10 {
            for idx in 0..neopixels.len() {
                let color = match (cnt + idx) % 3 {
                    0 => RGB { r: 0x21, g: 0, b: 0 },
                    1 => RGB { r: 0, g: 0x31, b: 0 },
                    _ => RGB { r: 0, g: 0, b: 0x41 },
                };
                neopixels.set(idx, color);
            }
            cnt += 1;
            // start sending the neopixel bit patters over spi to the neopixel string
            spi.write(&neopixels.bitbuffer).await.ok();
            Timer::after_millis(500).await;
        }
        Timer::after_millis(1000).await;
    }
}
