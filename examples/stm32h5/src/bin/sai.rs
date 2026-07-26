#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::sai::{
    BitOrder, ClockStrobe, DataSize, FrameSyncOffset, FrameSyncPolarity, MasterClockDivider, Sai, word,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals, sai};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL1 => dma::InterruptHandler<peripherals::GPDMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("SAI Test Signal Start");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;

        // HSE = 8 MHz external crystal
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });

        // PLL2: 8 MHz / 5 = 1.6 MHz * 192 = 307.2 MHz / 25 = 12.288 MHz
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5,
            mul: PllMul::Mul192,
            divp: Some(PllDiv::Div25),
            divq: None,
            divr: None,
        });

        config.rcc.mux.sai1sel = mux::Saisel::Pll2P;
    }
    let p = embassy_stm32::init(config);

    // DMA buffer
    let mut write_buffer = [0u32; 256];

    // Get SAI1 block A (transmitter)
    let (sai_a, _) = sai::split_subblocks(p.SAI1);

    // SAI configuration for I2S format
    let mut sai_config = sai::Config::default();
    sai_config.bit_order = BitOrder::MsbFirst;
    sai_config.slot_count = word::U4(2); // Stereo
    sai_config.data_size = DataSize::Data32; // 32 bits
    sai_config.frame_length = 64; // 2 channels * 32 bits
    sai_config.frame_sync_active_level_length = word::U7(32); // Active level length
    sai_config.master_clock_divider = MasterClockDivider::Div1;
    sai_config.clock_strobe = ClockStrobe::Rising;
    sai_config.frame_sync_offset = FrameSyncOffset::BeforeFirstBit;
    sai_config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;

    let mut sai_a = Sai::new_asynchronous_with_mclk(
        sai_a,
        p.PE5, // SCK_A — bit clock
        p.PE6, // SD_A — data
        p.PE4, // FS_A — LRCLK
        p.PE2, // MCLK_A — master clock
        p.GPDMA1_CH1,
        &mut write_buffer,
        Irqs,
        sai_config,
    );

    info!("SAI initialized, generating test signal");

    // Generate test signal: 1 kHz square wave at 48 kHz sample rate
    // Signal period: 48_000 / 1_000 = 48 samples per period
    let sample_rate = 48_000;
    let test_freq = 1_000;
    let period_samples = sample_rate / test_freq;
    let half_period = period_samples / 2;

    let mut test_data = [0u32; 128]; // 128 samples = 256 bytes (stereo)
    for i in 0..test_data.len() {
        let sample = if (i % period_samples) < half_period {
            0x7FFFFFFF // Maximum amplitude (positive)
        } else {
            0x80000000 // Minimum amplitude (negative)
        };
        // Stereo: same signal on both channels
        test_data[i] = sample;
    }

    info!("Test signal buffer prepared, starting playback loop");

    loop {
        if let Err(e) = sai_a.write(&test_data).await {
            info!("SAI write error: {:?}", e);
        }
    }
}
