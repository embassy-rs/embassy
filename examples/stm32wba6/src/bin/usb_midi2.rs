//! This example demonstrates USB MIDI 2.0 (Universal MIDI Packet / Group Terminal Blocks)
//! on STM32WBA6.
//!
//! Provides dual alternate settings:
//! - Alt 0: MIDI 1.0 backward compatibility (standard event packets).
//! - Alt 1: MIDI 2.0 native UMP transport with Group Terminal Block descriptors.

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::midi::device::{Midi2Config, MidiClass, MidiClassConfig, MidiClassState};
use embassy_usb::class::midi::{GroupTerminalBlockConfig, GroupTerminalBlockType, MidiProtocol};
use embassy_usb::driver::EndpointError;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting STM32WBA6 USB MIDI 2.0 example...");

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

    // Create the driver from the HAL.
    static EP_OUT_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;
    let driver = Driver::new_hs(
        p.USB_OTG_HS,
        p.PD6,
        p.PD7,
        Irqs,
        EP_OUT_BUFFER.init([0u8; 256]),
        usb_config,
    );

    // Create embassy-usb DeviceBuilder.
    let mut usb_builder_config = embassy_usb::Config::new(0x1209, 0x0002);
    usb_builder_config.manufacturer = Some("Embassy");
    usb_builder_config.product = Some("MIDI 2.0 Demo");
    usb_builder_config.serial_number = Some("12345678");
    usb_builder_config.max_power = 100;
    usb_builder_config.max_packet_size_0 = 64;

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 512]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 256]> = StaticCell::new();

    let mut builder = Builder::new(
        driver,
        usb_builder_config,
        CONFIG_DESCRIPTOR.init([0; 512]),
        BOS_DESCRIPTOR.init([0; 256]),
        MSOS_DESCRIPTOR.init([0; 256]),
        CONTROL_BUF.init([0; 256]),
    );

    static GTBS: &[GroupTerminalBlockConfig<'static>] = &[GroupTerminalBlockConfig {
        block_id: 1,
        block_type: GroupTerminalBlockType::Bidirectional,
        first_group: 0,
        num_groups: 1,
        protocol: MidiProtocol::Midi2_0,
        name: Some("WBA65 MIDI 2.0 Port"),
        max_input_bandwidth: 0,
        max_output_bandwidth: 0,
    }];

    static MIDI_STATE: StaticCell<MidiClassState<'static>> = StaticCell::new();
    let midi_state = MIDI_STATE.init(MidiClassState::new());

    let mut midi_config = MidiClassConfig::default();
    midi_config.n_in_jacks = 1;
    midi_config.n_out_jacks = 1;
    midi_config.max_packet_size = 64;
    midi_config.interface_name = Some("MIDI 2.0 Interface");
    midi_config.midi2 = Some(Midi2Config {
        group_terminal_blocks: GTBS,
    });

    let mut midi = MidiClass::new_with_names(&mut builder, midi_state, midi_config);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let midi_fut = async {
        loop {
            midi.wait_connection().await;
            info!("USB MIDI connected!");

            let mut buf = [0u8; 64];
            loop {
                // §3.1.1: alternate setting 0 carries MIDI 1.0 event packets.
                // Alternate setting 1 carries Universal MIDI Packets (§3.2).
                let disconnected = if midi.is_midi2_active() {
                    match midi.read_ump(&mut buf).await {
                        Ok(packets) => {
                            if !midi.is_midi2_active() {
                                continue;
                            }
                            let mut out = [0u8; 64];
                            let mut n = 0;
                            for packet in packets {
                                let Ok(packet) = packet else { break };
                                let Ok(written) = packet.to_bytes(&mut out[n..]) else {
                                    break;
                                };
                                n += written;
                            }
                            if n == 0 {
                                continue;
                            }
                            info!("Received {} UMP bytes, echoing back...", n);
                            if let Err(e) = midi.write_ump_bytes(&out[..n]).await {
                                warn!("Send failed: {:?}", defmt::Debug2Format(&e));
                                true
                            } else {
                                false
                            }
                        }
                        Err(EndpointError::Disabled) => {
                            info!("USB disconnected");
                            true
                        }
                        Err(EndpointError::BufferOverflow) => {
                            warn!("Buffer overflow on read");
                            true
                        }
                    }
                } else {
                    match midi.read_packet(&mut buf).await {
                        Ok(n) => {
                            if midi.is_midi2_active() {
                                continue;
                            }
                            info!("Received {} MIDI 1.0 bytes, echoing back...", n);
                            if let Err(e) = midi.write_packet(&buf[..n]).await {
                                warn!("Send failed: {:?}", defmt::Debug2Format(&e));
                                true
                            } else {
                                false
                            }
                        }
                        Err(EndpointError::Disabled) => {
                            info!("USB disconnected");
                            true
                        }
                        Err(EndpointError::BufferOverflow) => {
                            warn!("Buffer overflow on read");
                            true
                        }
                    }
                };
                if disconnected {
                    break;
                }
            }
        }
    };

    join(usb_fut, midi_fut).await;
}
