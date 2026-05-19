#![no_std]
#![no_main]

//! Same as `jpeg_decode` but uses [`Jpeg::new_blocking`] +
//! [`Jpeg::encode_blocking`] / [`Jpeg::decode_blocking`] — no DMA channels,
//! no JPEG IRQ.

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::jpeg::{ChromaSubsampling, ColorSpace, EncodeConfig, Jpeg, PlanarYCbCrMut};
use {defmt_rtt as _, panic_probe as _};

const W: u16 = 64;
const H: u16 = 64;

#[repr(align(4))]
struct Aligned<const N: usize>([u8; N]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Config::default());

    let mut codec = Jpeg::new_blocking(p.JPEG);

    let mut src_buf = Aligned::<{ (W as usize) * (H as usize) }>([0u8; (W as usize) * (H as usize)]);
    let src = &mut src_buf.0;
    for y in 0..H as usize {
        for x in 0..W as usize {
            src[y * W as usize + x] = (x as u8).wrapping_add(y as u8);
        }
    }

    let cfg = EncodeConfig {
        width: W,
        height: H,
        color_space: ColorSpace::Grayscale,
        subsampling: ChromaSubsampling::S444,
        quality: 75,
    };
    let mut jpeg_buf = Aligned::<8192>([0u8; 8192]);
    let n = unwrap!(codec.encode_blocking(src, &cfg, &mut jpeg_buf.0));
    info!("encoded {} bytes", n);

    let mut y_buf = Aligned::<{ (W as usize) * (H as usize) }>([0u8; (W as usize) * (H as usize)]);
    let mut cb_buf = [0u8; 0];
    let mut cr_buf = [0u8; 0];
    let info_out = unwrap!(codec.decode_blocking(
        &jpeg_buf.0[..n],
        PlanarYCbCrMut {
            y: &mut y_buf.0,
            cb: &mut cb_buf,
            cr: &mut cr_buf,
        }
    ));

    info!(
        "decoded {}x{} {:?} ({:?}), y_bytes={}",
        info_out.width, info_out.height, info_out.color_space, info_out.subsampling, info_out.y_bytes
    );
    info!("first 16 decoded Y: {=[u8]:02x}", y_buf.0[..16]);

    defmt::assert_eq!(info_out.width, W);
    defmt::assert_eq!(info_out.height, H);

    info!("done");
    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}
