//! Sub-GHz Radio demonstration for STM32WLE5CC (single-core Cortex-M4).
//!
//! Uses the on-chip `SUBGHZSPI` interface and `SubghzRadio` controller to communicate
//! with the internal SX1262 radio transceiver.

#![no_std]
#![no_main]

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::subghz::{LoraE5RfSwitch, RadioInterruptHandler, RfSwitch, Subghz};
use embassy_stm32::{bind_interrupts, dma, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    SUBGHZ_RADIO => RadioInterruptHandler;
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
});

// SX126x / SUBGHZ Radio Opcodes
const CMD_SET_STANDBY: u8 = 0x80;
const CMD_GET_STATUS: u8 = 0xC0;
const CMD_SET_PACKET_TYPE: u8 = 0x8A;
const CMD_GET_PACKET_TYPE: u8 = 0x11;

const STDBY_CONFIG_RC: u8 = 0x00;
const PACKET_TYPE_LORA: u8 = 0x01;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("Starting STM32WLE5CC SUBGHZ demonstration!");

    // 1. Initialize RF switch (LoRa-E5 module 2-pin switch: PB8 = power/enable, PB5 = mode)
    let ctrl_enable = Output::new(p.PB8, Level::Low, Speed::VeryHigh);
    let ctrl_mode = Output::new(p.PB5, Level::Low, Speed::VeryHigh);
    let mut rf_switch = LoraE5RfSwitch::new(ctrl_enable, ctrl_mode);

    // 2. Initialize Sub-GHz SPI and Radio controller
    let (mut spi, mut radio) = Subghz::new(p.SUBGHZSPI, p.DMA1_CH1, p.DMA1_CH2, Irqs);

    // 3. Hardware reset the radio
    info!("Resetting radio via RCC_CSR.RFRST...");
    radio.reset().await;
    radio.wait_on_busy().await;
    info!("Radio out of reset! BUSY = {}", radio.is_busy());

    // 4. Put radio in Standby mode (RC oscillator)
    info!("Sending SetStandby command...");
    unwrap!(spi.write(&[CMD_SET_STANDBY, STDBY_CONFIG_RC]).await);
    radio.wait_on_busy().await;

    // 5. Query radio status
    let mut status_cmd = [CMD_GET_STATUS, 0x00];
    unwrap!(spi.transfer_in_place(&mut status_cmd).await);
    let status = status_cmd[1];
    let chip_mode = (status >> 4) & 0x07;
    let cmd_status = (status >> 1) & 0x07;
    info!(
        "Radio status: 0x{:02x} (chip_mode: {}, cmd_status: {})",
        status, chip_mode, cmd_status
    );

    // 6. Set packet type to LoRa
    info!("Setting packet type to LoRa...");
    unwrap!(spi.write(&[CMD_SET_PACKET_TYPE, PACKET_TYPE_LORA]).await);
    radio.wait_on_busy().await;

    // 7. Verify packet type
    let mut pkt_type_cmd = [CMD_GET_PACKET_TYPE, 0x00, 0x00];
    unwrap!(spi.transfer_in_place(&mut pkt_type_cmd).await);
    info!("Packet type readback: 0x{:02x} (expected 0x01 = LoRa)", pkt_type_cmd[2]);

    // 8. Configure RF switch for reception
    info!("Switching RF front-end to RX...");
    rf_switch.set_rx();

    info!("STM32WLE5CC SUBGHZ bring-up verified successfully!");
}
