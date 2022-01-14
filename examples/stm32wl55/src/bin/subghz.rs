#![no_std]
#![no_main]
#![macro_use]
#![allow(dead_code)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::channel::signal::Signal;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy_stm32::dma::NoDma;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::subghz::*;
use embassy_stm32::Peripherals;
use example_common::unwrap;

const PING_DATA: &str = "PING";
const DATA_LEN: u8 = PING_DATA.len() as u8;
const PING_DATA_BYTES: &[u8] = PING_DATA.as_bytes();
const PREAMBLE_LEN: u16 = 5 * 8;

const RF_FREQ: RfFreq = RfFreq::from_frequency(867_500_000);

const SYNC_WORD: [u8; 8] = [0x79, 0x80, 0x0C, 0xC0, 0x29, 0x95, 0xF8, 0x4A];
const SYNC_WORD_LEN: u8 = SYNC_WORD.len() as u8;
const SYNC_WORD_LEN_BITS: u8 = SYNC_WORD_LEN * 8;

const TX_BUF_OFFSET: u8 = 128;
const RX_BUF_OFFSET: u8 = 0;
const LORA_PACKET_PARAMS: LoRaPacketParams = LoRaPacketParams::new()
    .set_crc_en(true)
    .set_preamble_len(PREAMBLE_LEN)
    .set_payload_len(DATA_LEN)
    .set_invert_iq(false)
    .set_header_type(HeaderType::Fixed);

const LORA_MOD_PARAMS: LoRaModParams = LoRaModParams::new()
    .set_bw(LoRaBandwidth::Bw125)
    .set_cr(CodingRate::Cr45)
    .set_ldro_en(true)
    .set_sf(SpreadingFactor::Sf7);

// configuration for +10 dBm output power
// see table 35 "PA optimal setting and operating modes"
const PA_CONFIG: PaConfig = PaConfig::new()
    .set_pa_duty_cycle(0x1)
    .set_hp_max(0x0)
    .set_pa(PaSel::Lp);

const TCXO_MODE: TcxoMode = TcxoMode::new()
    .set_txco_trim(TcxoTrim::Volts1pt7)
    .set_timeout(Timeout::from_duration_sat(
        core::time::Duration::from_millis(10),
    ));

const TX_PARAMS: TxParams = TxParams::new()
    .set_power(0x0D)
    .set_ramp_time(RampTime::Micros40);

fn config() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = embassy_stm32::rcc::ClockSrc::HSE32;
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: embassy::executor::Spawner, p: Peripherals) {
    let mut led1 = Output::new(p.PB15, Level::High, Speed::Low);
    let mut led2 = Output::new(p.PB9, Level::Low, Speed::Low);
    let mut led3 = Output::new(p.PB11, Level::Low, Speed::Low);

    let button = Input::new(p.PA0, Pull::Up);
    let mut pin = ExtiInput::new(button, p.EXTI0);

    static IRQ_SIGNAL: Signal<()> = Signal::new();
    let radio_irq = interrupt::take!(SUBGHZ_RADIO);
    radio_irq.set_handler(|_| {
        IRQ_SIGNAL.signal(());
        unsafe { interrupt::SUBGHZ_RADIO::steal() }.disable();
    });

    let mut radio = SubGhz::new(p.SUBGHZSPI, p.PA5, p.PA7, p.PA6, NoDma, NoDma);

    defmt::info!("Radio ready for use");

    led1.set_low();

    led2.set_high();

    unwrap!(radio.set_standby(StandbyClk::Rc));
    unwrap!(radio.set_tcxo_mode(&TCXO_MODE));
    unwrap!(radio.set_standby(StandbyClk::Hse));
    unwrap!(radio.set_regulator_mode(RegMode::Ldo));
    unwrap!(radio.set_buffer_base_address(TX_BUF_OFFSET, RX_BUF_OFFSET));
    unwrap!(radio.set_pa_config(&PA_CONFIG));
    unwrap!(radio.set_pa_ocp(Ocp::Max60m));
    unwrap!(radio.set_tx_params(&TX_PARAMS));
    unwrap!(radio.set_packet_type(PacketType::LoRa));
    unwrap!(radio.set_lora_sync_word(LoRaSyncWord::Public));
    unwrap!(radio.set_lora_mod_params(&LORA_MOD_PARAMS));
    unwrap!(radio.set_lora_packet_params(&LORA_PACKET_PARAMS));
    unwrap!(radio.calibrate_image(CalibrateImage::ISM_863_870));
    unwrap!(radio.set_rf_frequency(&RF_FREQ));

    defmt::info!("Status: {:?}", unwrap!(radio.status()));

    led2.set_low();

    loop {
        pin.wait_for_rising_edge().await;
        led3.set_high();
        unwrap!(radio.set_irq_cfg(&CfgIrq::new().irq_enable_all(Irq::TxDone)));
        unwrap!(radio.write_buffer(TX_BUF_OFFSET, PING_DATA_BYTES));
        unwrap!(radio.set_tx(Timeout::DISABLED));

        radio_irq.enable();
        IRQ_SIGNAL.wait().await;

        let (_, irq_status) = unwrap!(radio.irq_status());
        if irq_status & Irq::TxDone.mask() != 0 {
            defmt::info!("TX done");
        }
        unwrap!(radio.clear_irq_status(irq_status));
        led3.set_low();
    }
}
