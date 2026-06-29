#![no_std]
#![no_main]

//! Same as `jpeg_encode` but uses [`Jpeg::new_blocking`] +
//! [`Jpeg::encode_blocking`] — no DMA channels, no JPEG IRQ. Useful when DMA
//! channels are scarce.

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::jpeg::{ChromaSubsampling, ColorSpace, EncodeConfig, Jpeg};
use {defmt_rtt as _, panic_probe as _};

const W: u16 = 64;
const H: u16 = 64;

#[repr(align(4))]
struct Aligned<const N: usize>([u8; N]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Config::default());

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

    let mut codec = Jpeg::new_blocking(p.JPEG);
    let n = unwrap!(codec.encode_blocking(src, &cfg, dst));
    info!("encoded {} bytes, magic={=[u8]:02x}", n, dst[..16]);
    info!("EOI tail={=[u8]:02x}", dst[n - 4..n]);

    defmt::assert_eq!([dst[0], dst[1]], [0xFF, 0xD8], "missing SOI");
    defmt::assert_eq!([dst[n - 2], dst[n - 1]], [0xFF, 0xD9], "missing EOI");

    info!("done");
    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}
