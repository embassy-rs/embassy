//! FCC / CE / Regulatory RF certification test harness for STM32WLE5CC.
//!
//! Runs the transmit and receive modes that test laboratories ask for, one after the other:
//! 1. Continuous wave (unmodulated carrier): `SetTxContinuousWave` (0xD1), high-power PA.
//! 2. Continuous wave through the low-power PA.
//! 3. Continuous preamble (infinite LoRa up-chirps): `SetTxContinuousPreamble` (0xD2).
//! 4. Continuous receive: `SetRx` (0x82) with no timeout.
//! 5. A burst of LoRa packets, one every second, for duty-cycle and spectral-density measurements.
//!
//! **The harness transmits at full power (+22 dBm) as soon as it starts.** Connect the RF output to a 50 ohm load,
//! an attenuator or an antenna first, and check that the frequencies and power are allowed where you are.
//!
//! Board assumptions, change them to match yours:
//! - The RF switch is the two-pin LoRa-E5 type on PB8 and PB5.
//! - The 32 MHz TCXO is powered from PB0 (VDD_TCXO), so the HSE runs in bypass mode. The radio needs the HSE:
//!   without it every transmit or receive command fails with an oscillator start error.
//! - The board has the SMPS inductor, so the radio runs from its DC-DC converter ([`USE_DCDC`]).

#![no_std]
#![no_main]

use defmt::{info, unwrap, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::rcc::{Hse, HseMode, HsePrescaler, Sysclk};
use embassy_stm32::subghz::{RadioInterruptHandler, SubghzControl, SubghzSpi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, peripherals};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use panic_probe as _;

/// What the RF front-end switch is set up for.
#[derive(Clone, Copy)]
enum RfMode {
    Off,
    Rx,
    TxLowPower,
    TxHighPower,
}

/// Two-pin RF switch of the Seeed LoRa-E5 / RAK3172 modules: `ctrl1` high selects RX, `ctrl2` high selects TX.
///
/// How the switch is wired depends on the board, so this lives in the application and not in the driver.
struct RfSwitch<'d> {
    ctrl1: Output<'d>,
    ctrl2: Output<'d>,
}

impl<'d> RfSwitch<'d> {
    fn new(ctrl1: Output<'d>, ctrl2: Output<'d>) -> Self {
        let mut this = Self { ctrl1, ctrl2 };
        this.set_mode(RfMode::Off);
        this
    }

    fn set_mode(&mut self, mode: RfMode) {
        let (rx, tx) = match mode {
            RfMode::Off => (false, false),
            RfMode::Rx => (true, false),
            RfMode::TxLowPower | RfMode::TxHighPower => (false, true),
        };
        self.ctrl1.set_level(rx.into());
        self.ctrl2.set_level(tx.into());
    }
}

bind_interrupts!(struct Irqs {
    SUBGHZ_RADIO => RadioInterruptHandler;
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
});

/// Run the radio from its DC-DC converter. Needs the SMPS inductor on the board: set this to `false` on a board
/// without one, and the radio runs from its LDO instead.
const USE_DCDC: bool = true;
/// Output power of the high-power PA, in dBm (-9 to +22).
const POWER_HP_DBM: i8 = 22;
/// Output power of the low-power PA, in dBm (-17 to +14).
const POWER_LP_DBM: i8 = 14;
/// Packets sent by the burst test, and the time between the start of one and the next.
const BURST_COUNT: u32 = 10;
const BURST_PERIOD: Duration = Duration::from_secs(1);
/// Payload of the burst packets.
const BURST_PAYLOAD: &[u8] = b"Embassy STM32WLE5 FCC burst test";

// SX126x / SUBGHZ Radio Opcodes
const CMD_SET_STANDBY: u8 = 0x80;
const CMD_SET_TX: u8 = 0x83;
const CMD_SET_RX: u8 = 0x82;
const CMD_SET_REGULATOR_MODE: u8 = 0x96;
const CMD_SET_RF_FREQUENCY: u8 = 0x86;
const CMD_SET_PACKET_TYPE: u8 = 0x8A;
const CMD_SET_MODULATION_PARAMS: u8 = 0x8B;
const CMD_SET_PACKET_PARAMS: u8 = 0x8C;
const CMD_SET_PA_CONFIG: u8 = 0x95;
const CMD_SET_TX_PARAMS: u8 = 0x8E;
const CMD_SET_BUFFER_BASE_ADDRESS: u8 = 0x8F;
const CMD_SET_DIO_IRQ_PARAMS: u8 = 0x08;
const CMD_WRITE_BUFFER: u8 = 0x0E;
const CMD_WRITE_REGISTER: u8 = 0x0D;
const CMD_READ_REGISTER: u8 = 0x1D;
const CMD_GET_IRQ_STATUS: u8 = 0x12;
const CMD_CLR_IRQ_STATUS: u8 = 0x02;
const CMD_CALIBRATE: u8 = 0x89;
const CMD_CALIBRATE_IMAGE: u8 = 0x98;
const CMD_SET_TX_CONTINUOUS_WAVE: u8 = 0xD1;
const CMD_SET_TX_CONTINUOUS_PREAMBLE: u8 = 0xD2;
const CMD_GET_STATUS: u8 = 0xC0;

const STDBY_CONFIG_RC: u8 = 0x00;
const REGULATOR_LDO: u8 = 0x00;
const REGULATOR_SMPS: u8 = 0x01;
const PACKET_TYPE_LORA: u8 = 0x01;

const REG_TX_CLAMP: u16 = 0x08D8;
const REG_OCP: u16 = 0x08E7;
/// Over-current protection limit of 140 mA, for the high-power PA.
const OCP_140_MA: u8 = 0x38;

const IRQ_TX_DONE: u16 = 0x0001;
const IRQ_TIMEOUT: u16 = 0x0200;

/// LoRa packet parameters: 8 preamble symbols, explicit header, `payload_len` bytes, CRC on, standard IQ.
///
/// Without a `SetPacketParams` the radio's IQ polarity is left at an unconfigured default that comes out inverted,
/// so the continuous preamble would be sent as down-chirps.
fn lora_packet_params(payload_len: u8) -> [u8; 7] {
    [CMD_SET_PACKET_PARAMS, 0x00, 0x08, 0x00, payload_len, 0x01, 0x00]
}

/// Send a command and wait for the radio to finish it.
async fn command(spi: &mut SubghzSpi<'_, Async>, radio: &mut SubghzControl<'_>, bytes: &[u8]) {
    unwrap!(spi.write(bytes).await);
    radio.wait_on_busy().await;
}

async fn read_register(spi: &mut SubghzSpi<'_, Async>, radio: &mut SubghzControl<'_>, addr: u16) -> u8 {
    let mut buf = [CMD_READ_REGISTER, (addr >> 8) as u8, addr as u8, 0x00, 0x00];
    unwrap!(spi.transfer_in_place(&mut buf).await);
    radio.wait_on_busy().await;
    buf[4]
}

async fn write_register(spi: &mut SubghzSpi<'_, Async>, radio: &mut SubghzControl<'_>, addr: u16, value: u8) {
    command(spi, radio, &[CMD_WRITE_REGISTER, (addr >> 8) as u8, addr as u8, value]).await;
}

/// Helper to configure RF carrier frequency.
async fn set_rf_frequency(spi: &mut SubghzSpi<'_, Async>, radio: &mut SubghzControl<'_>, freq_hz: u32) {
    let freq_raw = ((freq_hz as u64 * 33_554_432) / 32_000_000) as u32;
    let cmd = [
        CMD_SET_RF_FREQUENCY,
        (freq_raw >> 24) as u8,
        (freq_raw >> 16) as u8,
        (freq_raw >> 8) as u8,
        freq_raw as u8,
    ];
    command(spi, radio, &cmd).await;
}

/// Calibrate the image rejection for the band that contains `freq_hz`. It has to be redone after each frequency
/// change, and it matters for the receiver's spurious response.
async fn calibrate_image(spi: &mut SubghzSpi<'_, Async>, radio: &mut SubghzControl<'_>, freq_hz: u32) {
    let (low, high) = if freq_hz > 900_000_000 {
        (0xE1, 0xE9)
    } else if freq_hz > 850_000_000 {
        (0xD7, 0xDB)
    } else if freq_hz > 770_000_000 {
        (0xC1, 0xC5)
    } else if freq_hz > 460_000_000 {
        (0x75, 0x81)
    } else {
        (0x6B, 0x6F)
    };
    command(spi, radio, &[CMD_CALIBRATE_IMAGE, low, high]).await;
}

/// Helper to configure PA and TX output power (+22 dBm HP or +14 dBm LP).
async fn set_tx_power(spi: &mut SubghzSpi<'_, Async>, radio: &mut SubghzControl<'_>, high_power: bool, power_dbm: i8) {
    if high_power {
        // High Power PA (+22 dBm maximum)
        command(spi, radio, &[CMD_SET_PA_CONFIG, 0x04, 0x07, 0x00, 0x01]).await;
    } else {
        // Low Power PA (+14 dBm maximum)
        command(spi, radio, &[CMD_SET_PA_CONFIG, 0x04, 0x00, 0x01, 0x01]).await;
    }

    // SX1262 errata: SetPaConfig resets the TX clamp, which makes the PA less tolerant of antenna mismatch.
    // Set it again after every SetPaConfig.
    let clamp = read_register(spi, radio, REG_TX_CLAMP).await;
    write_register(spi, radio, REG_TX_CLAMP, clamp | 0x1E).await;

    let pwr = if high_power {
        write_register(spi, radio, REG_OCP, OCP_140_MA).await;
        power_dbm.clamp(-9, 22)
    } else {
        power_dbm.clamp(-17, 14)
    };
    command(spi, radio, &[CMD_SET_TX_PARAMS, pwr as u8, 0x02]).await; // 40 µs ramp
}

/// Stop transmission or reception and place radio in Standby RC.
async fn radio_stop(spi: &mut SubghzSpi<'_, Async>, radio: &mut SubghzControl<'_>, rf_switch: &mut RfSwitch<'_>) {
    command(spi, radio, &[CMD_SET_STANDBY, STDBY_CONFIG_RC]).await;
    rf_switch.set_mode(RfMode::Off);
}

/// Start Continuous Wave (CW / Unmodulated Tone) TX.
async fn start_continuous_wave(
    spi: &mut SubghzSpi<'_, Async>,
    radio: &mut SubghzControl<'_>,
    rf_switch: &mut RfSwitch<'_>,
    freq_hz: u32,
    power_dbm: i8,
    high_power: bool,
) {
    radio_stop(spi, radio, rf_switch).await;

    info!(
        "Configuring CW mode: {} Hz @ {} dBm (HP={})",
        freq_hz, power_dbm, high_power
    );
    set_rf_frequency(spi, radio, freq_hz).await;
    set_tx_power(spi, radio, high_power, power_dbm).await;
    calibrate_image(spi, radio, freq_hz).await;

    // Switch RF front-end to TX
    if high_power {
        rf_switch.set_mode(RfMode::TxHighPower);
    } else {
        rf_switch.set_mode(RfMode::TxLowPower);
    }

    // Trigger CW transmission
    command(spi, radio, &[CMD_SET_TX_CONTINUOUS_WAVE]).await;
    info!("Continuous Wave active!");
}

/// Start Continuous Preamble (Infinite Modulated LoRa Chirps) TX.
async fn start_continuous_preamble(
    spi: &mut SubghzSpi<'_, Async>,
    radio: &mut SubghzControl<'_>,
    rf_switch: &mut RfSwitch<'_>,
    freq_hz: u32,
    sf: u8,
    bw: u8,
    power_dbm: i8,
    high_power: bool,
) {
    radio_stop(spi, radio, rf_switch).await;

    info!(
        "Configuring Continuous Preamble mode: {} Hz, SF{}, BW code {} @ {} dBm",
        freq_hz, sf, bw, power_dbm
    );
    command(spi, radio, &[CMD_SET_PACKET_TYPE, PACKET_TYPE_LORA]).await;

    set_rf_frequency(spi, radio, freq_hz).await;
    set_tx_power(spi, radio, high_power, power_dbm).await;
    calibrate_image(spi, radio, freq_hz).await;

    // LoRa Modulation params: [SF, BW, CR=4/5 (0x01), LDRO=0]
    command(spi, radio, &[CMD_SET_MODULATION_PARAMS, sf, bw, 0x01, 0x00]).await;
    command(spi, radio, &lora_packet_params(BURST_PAYLOAD.len() as u8)).await;

    if high_power {
        rf_switch.set_mode(RfMode::TxHighPower);
    } else {
        rf_switch.set_mode(RfMode::TxLowPower);
    }

    command(spi, radio, &[CMD_SET_TX_CONTINUOUS_PREAMBLE]).await;
    info!("Continuous Modulated Preamble active!");
}

/// Start Continuous RX mode for receiver sensitivity and spurious emission checks.
async fn start_continuous_rx(
    spi: &mut SubghzSpi<'_, Async>,
    radio: &mut SubghzControl<'_>,
    rf_switch: &mut RfSwitch<'_>,
    freq_hz: u32,
) {
    radio_stop(spi, radio, rf_switch).await;

    info!("Configuring Continuous RX mode at {} Hz", freq_hz);
    command(spi, radio, &[CMD_SET_PACKET_TYPE, PACKET_TYPE_LORA]).await;
    set_rf_frequency(spi, radio, freq_hz).await;
    calibrate_image(spi, radio, freq_hz).await;

    rf_switch.set_mode(RfMode::Rx);

    // SetRx with timeout 0xFFFFFF = Continuous RX
    command(spi, radio, &[CMD_SET_RX, 0xFF, 0xFF, 0xFF]).await;
    info!("Continuous RX mode active!");
}

/// Send `BURST_COUNT` LoRa packets, one every `BURST_PERIOD`, for duty-cycle and spectral-density measurements.
async fn run_packet_bursts(
    spi: &mut SubghzSpi<'_, Async>,
    radio: &mut SubghzControl<'_>,
    rf_switch: &mut RfSwitch<'_>,
    freq_hz: u32,
    sf: u8,
    bw: u8,
    power_dbm: i8,
    high_power: bool,
) {
    radio_stop(spi, radio, rf_switch).await;

    info!(
        "Configuring packet bursts: {} Hz, SF{}, BW code {} @ {} dBm, {} packets of {} bytes",
        freq_hz,
        sf,
        bw,
        power_dbm,
        BURST_COUNT,
        BURST_PAYLOAD.len()
    );
    command(spi, radio, &[CMD_SET_PACKET_TYPE, PACKET_TYPE_LORA]).await;
    set_rf_frequency(spi, radio, freq_hz).await;
    set_tx_power(spi, radio, high_power, power_dbm).await;
    calibrate_image(spi, radio, freq_hz).await;
    command(spi, radio, &[CMD_SET_MODULATION_PARAMS, sf, bw, 0x01, 0x00]).await;
    command(spi, radio, &lora_packet_params(BURST_PAYLOAD.len() as u8)).await;

    // TxDone and Timeout both raise IRQ 1, which is the interrupt the driver waits on.
    let mask = IRQ_TX_DONE | IRQ_TIMEOUT;
    let [hi, lo] = mask.to_be_bytes();
    command(
        spi,
        radio,
        &[CMD_SET_DIO_IRQ_PARAMS, hi, lo, hi, lo, 0x00, 0x00, 0x00, 0x00],
    )
    .await;

    // The payload is written once. It stays in the radio's buffer between packets.
    let mut frame = [0u8; 2 + BURST_PAYLOAD.len()];
    frame[0] = CMD_WRITE_BUFFER;
    frame[1] = 0x00;
    frame[2..].copy_from_slice(BURST_PAYLOAD);
    command(spi, radio, &frame).await;

    let start = Instant::now();
    for n in 0..BURST_COUNT {
        Timer::at(start + BURST_PERIOD * n).await;

        if high_power {
            rf_switch.set_mode(RfMode::TxHighPower);
        } else {
            rf_switch.set_mode(RfMode::TxLowPower);
        }
        // SetTx with timeout 0: no timeout, the radio stops by itself after the packet.
        command(spi, radio, &[CMD_SET_TX, 0x00, 0x00, 0x00]).await;

        let done = with_timeout(Duration::from_secs(10), radio.await_irq()).await.is_ok();
        let mut irq = [CMD_GET_IRQ_STATUS, 0x00, 0x00, 0x00];
        unwrap!(spi.transfer_in_place(&mut irq).await);
        radio.wait_on_busy().await;
        let status = u16::from_be_bytes([irq[2], irq[3]]);
        command(spi, radio, &[CMD_CLR_IRQ_STATUS, 0xFF, 0xFF]).await;
        rf_switch.set_mode(RfMode::Off);

        if done && status & IRQ_TX_DONE != 0 {
            info!("Packet {}/{} sent", n + 1, BURST_COUNT);
        } else {
            warn!(
                "Packet {}/{} did not finish (IRQ status 0x{:04x})",
                n + 1,
                BURST_COUNT,
                status
            );
            radio_stop(spi, radio, rf_switch).await;
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // The radio runs from the 32 MHz HSE. This board has a TCXO powered from PB0 (VDD_TCXO), which bypass mode
    // turns on. For a crystal use `HseMode::Oscillator`.
    config.rcc.hse = Some(Hse {
        freq: Hertz(32_000_000),
        mode: HseMode::Bypass,
        prescaler: HsePrescaler::Div1,
    });
    config.rcc.sys = Sysclk::Hse;
    let p = embassy_stm32::init(config);

    info!("============================================================");
    info!("STM32WLE5CC FCC / CE Regulatory RF Test Harness Initializing");
    info!("============================================================");

    // 1. Initialize Sub-GHz SPI and Radio controller
    let mut spi = SubghzSpi::new(p.SUBGHZSPI, p.DMA1_CH1, p.DMA1_CH2, Irqs);
    let mut radio = SubghzControl::new(Irqs);

    // 2. Initialize 2-pin RF switch (PB8 = power/enable, PB5 = mode)
    let ctrl_enable = Output::new(p.PB8, Level::Low, Speed::VeryHigh);
    let ctrl_mode = Output::new(p.PB5, Level::Low, Speed::VeryHigh);
    let mut rf_switch = RfSwitch::new(ctrl_enable, ctrl_mode);

    // 3. Hardware reset radio
    radio.reset().await;
    radio.wait_on_busy().await;

    // 4. Standby, regulator mode and a calibration of the radio's blocks
    command(&mut spi, &mut radio, &[CMD_SET_STANDBY, STDBY_CONFIG_RC]).await;
    let regulator = if USE_DCDC { REGULATOR_SMPS } else { REGULATOR_LDO };
    command(&mut spi, &mut radio, &[CMD_SET_REGULATOR_MODE, regulator]).await;
    command(&mut spi, &mut radio, &[CMD_SET_BUFFER_BASE_ADDRESS, 0x00, 0x00]).await;
    command(&mut spi, &mut radio, &[CMD_CALIBRATE, 0x7F]).await;

    // -------------------------------------------------------------
    // TEST 1: Continuous Wave (CW) Tone @ 915 MHz, +22 dBm (US FCC)
    // -------------------------------------------------------------
    info!(
        "\n>>> TEST 1: Continuous Wave (CW) 915.0 MHz @ {} dBm for 5s <<<",
        POWER_HP_DBM
    );
    start_continuous_wave(&mut spi, &mut radio, &mut rf_switch, 915_000_000, POWER_HP_DBM, true).await;
    Timer::after_secs(5).await;
    radio_stop(&mut spi, &mut radio, &mut rf_switch).await;
    info!("Test 1 complete.");

    Timer::after_secs(1).await;

    // -------------------------------------------------------------
    // TEST 2: Continuous Wave (CW) Tone @ 868.1 MHz, +14 dBm (EU CE)
    // -------------------------------------------------------------
    info!(
        "\n>>> TEST 2: Continuous Wave (CW) 868.1 MHz @ {} dBm for 5s <<<",
        POWER_LP_DBM
    );
    start_continuous_wave(&mut spi, &mut radio, &mut rf_switch, 868_100_000, POWER_LP_DBM, false).await;
    Timer::after_secs(5).await;
    radio_stop(&mut spi, &mut radio, &mut rf_switch).await;
    info!("Test 2 complete.");

    Timer::after_secs(1).await;

    // ---------------------------------------------------------------------
    // TEST 3: Continuous LoRa Modulated Preamble @ 915 MHz, SF10, BW 250kHz
    // ---------------------------------------------------------------------
    info!("\n>>> TEST 3: Continuous Modulated LoRa Preamble 915.0 MHz (SF10, BW250kHz) for 5s <<<");
    // BW 0x05 = 250 kHz
    start_continuous_preamble(
        &mut spi,
        &mut radio,
        &mut rf_switch,
        915_000_000,
        10,
        0x05,
        POWER_HP_DBM,
        true,
    )
    .await;
    Timer::after_secs(5).await;
    radio_stop(&mut spi, &mut radio, &mut rf_switch).await;
    info!("Test 3 complete.");

    Timer::after_secs(1).await;

    // -------------------------------------------------------------
    // TEST 4: Continuous RX Mode @ 915 MHz for spurious LO testing
    // -------------------------------------------------------------
    info!("\n>>> TEST 4: Continuous RX mode 915.0 MHz for 5s <<<");
    start_continuous_rx(&mut spi, &mut radio, &mut rf_switch, 915_000_000).await;
    Timer::after_secs(5).await;
    radio_stop(&mut spi, &mut radio, &mut rf_switch).await;
    info!("Test 4 complete.");

    Timer::after_secs(1).await;

    // ------------------------------------------------------------------------
    // TEST 5: Periodic LoRa packet bursts @ 915 MHz, SF10, BW 250kHz, +22 dBm
    // ------------------------------------------------------------------------
    info!(
        "\n>>> TEST 5: {} LoRa packets at 915.0 MHz (SF10, BW250kHz), one per second <<<",
        BURST_COUNT
    );
    run_packet_bursts(
        &mut spi,
        &mut radio,
        &mut rf_switch,
        915_000_000,
        10,
        0x05,
        POWER_HP_DBM,
        true,
    )
    .await;
    radio_stop(&mut spi, &mut radio, &mut rf_switch).await;
    info!("Test 5 complete.");

    // Query status
    let mut status_cmd = [CMD_GET_STATUS, 0x00];
    unwrap!(spi.transfer_in_place(&mut status_cmd).await);
    info!("\nFinal radio status: 0x{:02x}", status_cmd[1]);
    info!("Test sequence finished.");
}
