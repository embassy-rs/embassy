#![no_std]
#![no_main]

//! Encode a synthetic grayscale gradient image with the STM32N6 hardware JPEG
//! codec and dump a hex preview of the resulting bitstream over RTT.

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::jpeg::{ChromaSubsampling, ColorSpace, EncodeConfig, Jpeg};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    JPEG => embassy_stm32::jpeg::InterruptHandler<peripherals::JPEG>;
    GPDMA1_CHANNEL0 => embassy_stm32::dma::InterruptHandler<peripherals::GPDMA1_CH0>;
    GPDMA1_CHANNEL1 => embassy_stm32::dma::InterruptHandler<peripherals::GPDMA1_CH1>;
});

const W: u16 = 64;
const H: u16 = 64;

#[repr(align(4))]
struct Aligned<const N: usize>([u8; N]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Config::default());

    // Build an MCU-ordered grayscale source: 64×64 image is 8×8 = 64 MCUs of
    // 8×8 pixels each. For grayscale the buffer layout is identical to a
    // raster image (each MCU's pixels happen to land at the right offsets).
    let mut src_buf = Aligned::<{ (W as usize) * (H as usize) }>([0u8; (W as usize) * (H as usize)]);
    let src = &mut src_buf.0;
    for y in 0..H as usize {
        for x in 0..W as usize {
            src[y * W as usize + x] = (x as u8).wrapping_add(y as u8);
        }
    }

    let mut dst_buf = Aligned::<8192>([0u8; 8192]);
    let dst = &mut dst_buf.0;
    let cfg = EncodeConfig {
        width: W,
        height: H,
        color_space: ColorSpace::Grayscale,
        subsampling: ChromaSubsampling::S444,
        quality: 75,
    };

    info!("before Jpeg::new");
    let mut codec = Jpeg::new(p.JPEG, p.GPDMA1_CH0, p.GPDMA1_CH1, Irqs);
    info!("after Jpeg::new, before encode");
    let n = unwrap!(codec.encode(src, &cfg, dst).await);
    info!("encoded {} bytes, magic={=[u8]:02x}", n, dst[..16]);
    info!("EOI tail={=[u8]:02x}", dst[n - 4..n]);

    // Sanity-check the markers.
    defmt::assert_eq!([dst[0], dst[1]], [0xFF, 0xD8], "missing SOI");
    defmt::assert_eq!([dst[n - 2], dst[n - 1]], [0xFF, 0xD9], "missing EOI");

    info!("done");
    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}
