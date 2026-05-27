#![no_std]
#![no_main]

//! Encode → decode round-trip on the STM32N6 hardware JPEG codec.
//!
//! Generates a synthetic 64×64 grayscale gradient, encodes it to JPEG, decodes
//! the resulting bitstream, and reports the decoded dimensions plus a sanity
//! check on the first few luma bytes.

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::jpeg::{ChromaSubsampling, ColorSpace, EncodeConfig, Jpeg, PlanarYCbCrMut};
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

    let mut codec = Jpeg::new(p.JPEG, p.GPDMA1_CH0, p.GPDMA1_CH1, Irqs);

    // 1) Build a known input.
    let mut src_buf = Aligned::<{ (W as usize) * (H as usize) }>([0u8; (W as usize) * (H as usize)]);
    let src = &mut src_buf.0;
    for y in 0..H as usize {
        for x in 0..W as usize {
            src[y * W as usize + x] = (x as u8).wrapping_add(y as u8);
        }
    }

    // 2) Encode.
    let cfg = EncodeConfig {
        width: W,
        height: H,
        color_space: ColorSpace::Grayscale,
        subsampling: ChromaSubsampling::S444,
        quality: 75,
    };
    let mut jpeg_buf = Aligned::<8192>([0u8; 8192]);
    let n = unwrap!(codec.encode(src, &cfg, &mut jpeg_buf.0).await);
    info!("encoded {} bytes", n);

    // 3) Decode.
    let mut y_buf = Aligned::<{ (W as usize) * (H as usize) }>([0u8; (W as usize) * (H as usize)]);
    let mut cb_buf = [0u8; 0];
    let mut cr_buf = [0u8; 0];
    let info_out = unwrap!(
        codec
            .decode(
                &jpeg_buf.0[..n],
                PlanarYCbCrMut {
                    y: &mut y_buf.0,
                    cb: &mut cb_buf,
                    cr: &mut cr_buf,
                }
            )
            .await
    );

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
