//! This example demonstrates a USB Chip/Smart Card Interface Device (CCID)
//! running on an STM32WBA65 microcontroller over High-Speed USB.
//!
//! You can test and communicate with this device using `pcsc_scan` or `opensc-tool`:
//! ```bash
//! opensc-tool -a   # displays the ATR
//! opensc-tool -s 00:a4:04:00:00 # sends SELECT APDU
//! ```

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::ccid::device::{CcidCommand, CcidDevice, Config as CcidConfig, State as CcidState};
use embassy_usb::class::ccid::{
    CCID_ERR_NONE, CCID_ICC_PRESENT_ACTIVE, CCID_ICC_PRESENT_INACTIVE, CCID_STATUS_CMD_FAILED, CCID_STATUS_CMD_SUCCESS,
};
use embassy_usb::driver::EndpointError;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

/// Standard ISO 7816-4 Answer-to-Reset (ATR) indicating an active smart card.
const SMARTCARD_ATR: &[u8] = &[0x3B, 0x80, 0x80, 0x01, 0x01];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting USB CCID Smart Card on STM32WBA65");

    let mut config = Config::default();

    {
        use embassy_stm32::rcc::*;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,   // PLLM = 1 → HSI / 1 = 16 MHz
            mul: PllMul::Mul30,        // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
            divr: Some(PllDiv::Div5),  // PLLR = 5 → 96 MHz (Sysclk)
            divq: Some(PllDiv::Div10), // PLLQ = 10 → 48 MHz
            divp: Some(PllDiv::Div30), // PLLP = 30 → 16 MHz (USB_OTG_HS)
            frac: Some(0),             // Fractional part (disabled)
        });
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
    }

    let p = embassy_stm32::init(config);

    // Create the USB OTG HS driver.
    let mut ep_out_buffer = [0u8; 1024];
    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;
    let driver = Driver::new_hs(p.USB_OTG_HS, p.PD6, p.PD7, Irqs, &mut ep_out_buffer, usb_config);

    let mut usb_dev_config = embassy_usb::Config::new(0x1209, 0x0003);
    usb_dev_config.manufacturer = Some("Embassy");
    usb_dev_config.product = Some("STM32WBA65 CCID Smart Card");
    usb_dev_config.serial_number = Some("12345678");
    usb_dev_config.max_power = 100;
    usb_dev_config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 128];

    let mut state = CcidState::new();

    let mut builder = Builder::new(
        driver,
        usb_dev_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let ccid_config = CcidConfig {
        interface_name: Some("Smart Card Reader"),
        ..Default::default()
    };
    let ccid = CcidDevice::new(&mut builder, &mut state, ccid_config);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    let (mut reader, mut writer) = ccid.split();

    let ccid_fut = async {
        let mut msg_buf = [0u8; 512];
        let mut card_active = true;

        loop {
            reader.wait_connection().await;
            info!("CCID interface connected");

            loop {
                match reader.read_command(&mut msg_buf).await {
                    Ok(cmd) => match cmd {
                        CcidCommand::IccPowerOn { slot, seq, .. } => {
                            info!("CCID PowerOn slot={}, seq={}", slot, seq);
                            card_active = true;
                            let status = CCID_STATUS_CMD_SUCCESS | CCID_ICC_PRESENT_ACTIVE;
                            if let Err(e) = writer
                                .send_data_block(slot, seq, status, CCID_ERR_NONE, SMARTCARD_ATR)
                                .await
                            {
                                warn!("Failed to send ATR: {:?}", e);
                            }
                        }
                        CcidCommand::IccPowerOff { slot, seq } => {
                            info!("CCID PowerOff slot={}, seq={}", slot, seq);
                            card_active = false;
                            let status = CCID_STATUS_CMD_SUCCESS | CCID_ICC_PRESENT_INACTIVE;
                            if let Err(e) = writer.send_slot_status(slot, seq, status, CCID_ERR_NONE, 0).await {
                                warn!("Failed to send slot status: {:?}", e);
                            }
                        }
                        CcidCommand::GetSlotStatus { slot, seq } => {
                            info!("CCID GetSlotStatus slot={}, seq={}", slot, seq);
                            let icc = if card_active {
                                CCID_ICC_PRESENT_ACTIVE
                            } else {
                                CCID_ICC_PRESENT_INACTIVE
                            };
                            let status = CCID_STATUS_CMD_SUCCESS | icc;
                            if let Err(e) = writer.send_slot_status(slot, seq, status, CCID_ERR_NONE, 0).await {
                                warn!("Failed to send slot status: {:?}", e);
                            }
                        }
                        CcidCommand::XfrBlock { slot, seq, apdu, .. } => {
                            info!("CCID XfrBlock APDU (len={}): {=[u8]}", apdu.len(), apdu);

                            // Respond with echo / success SW1=0x90, SW2=0x00
                            let mut response = [0u8; 64];
                            let resp_len = if apdu.len() >= 4 && apdu[0] == 0x00 && apdu[1] == 0xA4 {
                                // SELECT AID: return success 90 00
                                response[0] = 0x90;
                                response[1] = 0x00;
                                2
                            } else {
                                // Echo up to 62 bytes of APDU data + 90 00
                                let copy_len = apdu.len().min(62);
                                response[..copy_len].copy_from_slice(&apdu[..copy_len]);
                                response[copy_len] = 0x90;
                                response[copy_len + 1] = 0x00;
                                copy_len + 2
                            };

                            let status = CCID_STATUS_CMD_SUCCESS | CCID_ICC_PRESENT_ACTIVE;
                            if let Err(e) = writer
                                .send_data_block(slot, seq, status, CCID_ERR_NONE, &response[..resp_len])
                                .await
                            {
                                warn!("Failed to send APDU response: {:?}", e);
                            }
                        }
                        CcidCommand::GetParameters { slot, seq } => {
                            info!("CCID GetParameters slot={}, seq={}", slot, seq);
                            let params = [0x11, 0x10, 0x00, 0x15, 0x00, 0xFE, 0x00];
                            let status = CCID_STATUS_CMD_SUCCESS | CCID_ICC_PRESENT_ACTIVE;
                            if let Err(e) = writer
                                .send_parameters(slot, seq, status, CCID_ERR_NONE, 1, &params)
                                .await
                            {
                                warn!("Failed to send parameters: {:?}", e);
                            }
                        }
                        CcidCommand::SetParameters { slot, seq, .. } | CcidCommand::ResetParameters { slot, seq } => {
                            let params = [0x11, 0x10, 0x00, 0x15, 0x00, 0xFE, 0x00];
                            let status = CCID_STATUS_CMD_SUCCESS | CCID_ICC_PRESENT_ACTIVE;
                            _ = writer
                                .send_parameters(slot, seq, status, CCID_ERR_NONE, 1, &params)
                                .await;
                        }
                        CcidCommand::Abort { slot, seq } => {
                            info!("CCID Abort slot={}, seq={}", slot, seq);
                            let status = CCID_STATUS_CMD_SUCCESS | CCID_ICC_PRESENT_ACTIVE;
                            _ = writer.send_slot_status(slot, seq, status, CCID_ERR_NONE, 0).await;
                        }
                        CcidCommand::Unknown {
                            message_type,
                            slot,
                            seq,
                            ..
                        } => {
                            warn!("Unknown CCID message type: {:#04x}", message_type);
                            let status = CCID_STATUS_CMD_FAILED | CCID_ICC_PRESENT_ACTIVE;
                            _ = writer.send_slot_status(slot, seq, status, 0x00, 0).await;
                        }
                    },
                    Err(EndpointError::Disabled) => {
                        info!("CCID interface disabled/disconnected");
                        break;
                    }
                    Err(e) => {
                        warn!("CCID read error: {:?}", e);
                    }
                }
            }
        }
    };

    join(usb_fut, ccid_fut).await;
}
