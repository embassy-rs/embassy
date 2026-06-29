//! CRYP AES-GCM hardware vs software roundtrip example.
//!
//! Encrypts and decrypts a 32 KiB payload using the CRYP peripheral in
//! AES-GCM-128 mode, compares the result against the software `aes-gcm`
//! crate, and reports execution time for both paths.

#![no_std]
#![no_main]

use aes_gcm::Aes128Gcm;
use aes_gcm::aead::heapless::Vec;
use aes_gcm::aead::{AeadInPlace, KeyInit};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::cryp::{self, *};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use embassy_time::Instant;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const PAYLOAD_LEN: usize = 32 * 1024;
const TAG_LEN: usize = 16;

// CRYP DMA transfers as 32-bit words; user buffers must be 4-byte aligned.
#[repr(align(4))]
struct Aligned<const N: usize>([u8; N]);

static PAYLOAD: StaticCell<Aligned<PAYLOAD_LEN>> = StaticCell::new();
static CIPHERTEXT: StaticCell<Aligned<PAYLOAD_LEN>> = StaticCell::new();
static PLAINTEXT: StaticCell<Aligned<PAYLOAD_LEN>> = StaticCell::new();
static SW_BUF: StaticCell<Vec<u8, { PAYLOAD_LEN + TAG_LEN }>> = StaticCell::new();
static SW_DECRYPT: StaticCell<Vec<u8, { PAYLOAD_LEN + TAG_LEN }>> = StaticCell::new();

bind_interrupts!(struct Irqs {
    CRYP => cryp::InterruptHandler<peripherals::CRYP>;
    GPDMA1_CHANNEL0 => dma::InterruptHandler<peripherals::GPDMA1_CH0>;
    GPDMA1_CHANNEL1 => dma::InterruptHandler<peripherals::GPDMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(24_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div3,
            mul: PllMul::Mul150,
            divp: Some(PllDiv::Div2),
            divq: None,
            divr: None,
            divs: None,
            divt: None,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.apb5_pre = APBPrescaler::Div2;
        config.rcc.voltage_scale = VoltageScale::High;
    }
    let p = embassy_stm32::init(config);

    let payload = PAYLOAD.init_with(|| {
        let mut buf = Aligned([0u8; PAYLOAD_LEN]);
        let mut i = 0;
        while i < PAYLOAD_LEN {
            buf.0[i] = i as u8;
            i += 1;
        }
        buf
    });
    let ciphertext = CIPHERTEXT.init_with(|| Aligned([0u8; PAYLOAD_LEN]));
    let plaintext = PLAINTEXT.init_with(|| Aligned([0u8; PAYLOAD_LEN]));
    let aad: &[u8] = b"additional data";

    let mut hw_cryp = Cryp::new(p.CRYP, p.GPDMA1_CH0, p.GPDMA1_CH1, Irqs);
    let key: [u8; 16] = [0; 16];
    let iv: [u8; 12] = [0; 12];

    // Hardware path.
    let aes_gcm = AesGcm::new(&key, &iv);
    let hw_start = Instant::now();
    let mut gcm_encrypt = hw_cryp.start(&aes_gcm, Direction::Encrypt).await;
    hw_cryp.aad(&mut gcm_encrypt, aad, true).await;
    hw_cryp
        .payload(&mut gcm_encrypt, &payload.0, &mut ciphertext.0, true)
        .await;
    let encrypt_tag = hw_cryp.finish(gcm_encrypt).await;

    let mut gcm_decrypt = hw_cryp.start(&aes_gcm, Direction::Decrypt).await;
    hw_cryp.aad(&mut gcm_decrypt, aad, true).await;
    hw_cryp
        .payload(&mut gcm_decrypt, &ciphertext.0, &mut plaintext.0, true)
        .await;
    let decrypt_tag = hw_cryp.finish(gcm_decrypt).await;
    let hw_duration = hw_start.elapsed();

    // Software reference for validation and timing comparison.
    let cipher = Aes128Gcm::new(&key.into());
    let sw_buf = SW_BUF.init_with(|| {
        let mut v: Vec<u8, { PAYLOAD_LEN + TAG_LEN }> = Vec::new();
        v.extend_from_slice(&payload.0).unwrap();
        v
    });
    let sw_start = Instant::now();
    cipher.encrypt_in_place(&iv.into(), aad, sw_buf).unwrap();
    let sw_decrypt = SW_DECRYPT.init_with(|| {
        let mut v: Vec<u8, { PAYLOAD_LEN + TAG_LEN }> = Vec::new();
        v.extend_from_slice(sw_buf).unwrap();
        v
    });
    cipher.decrypt_in_place(&iv.into(), aad, sw_decrypt).unwrap();
    let sw_duration = sw_start.elapsed();
    assert_eq!(sw_decrypt.as_slice(), &payload.0[..]);
    let (sw_ciphertext, sw_tag) = sw_buf.split_at(PAYLOAD_LEN);

    info!(
        "HW encrypt: ct match={} tag match={}",
        &ciphertext.0[..] == sw_ciphertext,
        encrypt_tag.as_slice() == sw_tag,
    );
    info!(
        "HW decrypt: pt match={} tag match={}",
        &plaintext.0[..] == &payload.0[..],
        encrypt_tag == decrypt_tag,
    );

    let hw_us = hw_duration.as_micros();
    let sw_us = sw_duration.as_micros();
    let speedup_x100 = if hw_us > 0 { (sw_us * 100) / hw_us } else { 0 };
    info!("Payload: {} bytes", PAYLOAD_LEN);
    info!("Hardware (encrypt+decrypt): {} us", hw_us);
    info!("Software (encrypt+decrypt): {} us", sw_us);
    info!("Speedup: {}.{:02}x", speedup_x100 / 100, speedup_x100 % 100);

    loop {}
}
