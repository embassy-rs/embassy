#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::ptr;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::tl_mbox::cmd::{CommandPacket, CommandSerial};
use embassy_stm32::tl_mbox::consts::TlPacketType;
use embassy_stm32::tl_mbox::shci::{ShciConfigParam, ShciOpcode};
use embassy_stm32::tl_mbox::{Config, PacketHeader, TlMbox};
use embassy_stm32::{bind_interrupts, pac, tl_mbox};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => tl_mbox::ReceiveInterruptHandler;
    IPCC_C1_TX => tl_mbox::TransmitInterruptHandler;
});

#[embassy_executor::task]
async fn read_sys() {
    let mut sys_subsystem = tl_mbox::sys::SysSubsystem::new();

    let mut payload = [0u8; 6];

    loop {
        sys_subsystem
            .read(|event_packet| {
                let _ = event_packet.copy_into_slice(&mut payload);

                let kind = unsafe { (&event_packet.event_serial.kind as *const u8).read_volatile() };

                // means recieved SYS event, which indicates in this case that the coprocessor is ready
                let code = unsafe { (&event_packet.event_serial.event.event_code as *const u8).read_volatile() };
                let payload_len =
                    unsafe { (&event_packet.event_serial.event.payload_len as *const u8).read_volatile() };

                info!(
                    "sys event ==> kind: {:#04x}, code: {:#04x}, payload_length: {}, payload: {:#04x}",
                    kind,
                    code,
                    payload_len,
                    payload[3..]
                );
            })
            .await;

        break;
    }
}

#[embassy_executor::task]
async fn read_ble() {
    let mut ble_subsystem = tl_mbox::ble::BleSubsystem::new();

    let mut payload = [0u8; 6];

    loop {
        ble_subsystem
            .read(|event_packet| {
                let _ = event_packet.copy_into_slice(&mut payload);

                let kind = unsafe { (&event_packet.event_serial.kind as *const u8).read_volatile() };

                // means recieved SYS event, which indicates in this case that the coprocessor is ready
                let code = unsafe { (&event_packet.event_serial.event.event_code as *const u8).read_volatile() };
                let payload_len =
                    unsafe { (&event_packet.event_serial.event.payload_len as *const u8).read_volatile() };

                info!(
                    "ble event ==> kind: {:#04x}, code: {:#04x}, payload_length: {}, payload: {:#04x}",
                    kind,
                    code,
                    payload_len,
                    payload[3..]
                );
            })
            .await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    /*
        How to make this work:

        - Obtain a NUCLEO-STM32WB55 from your preferred supplier.
        - Download and Install STM32CubeProgrammer.
        - Download stm32wb5x_FUS_fw.bin, stm32wb5x_BLE_Stack_full_fw.bin, and Release_Notes.html from
          gh:STMicroelectronics/STM32CubeWB@2234d97/Projects/STM32WB_Copro_Wireless_Binaries/STM32WB5x
        - Open STM32CubeProgrammer
        - On the right-hand pane, click "firmware upgrade" to upgrade the st-link firmware.
        - Once complete, click connect to connect to the device.
        - On the left hand pane, click the RSS signal icon to open "Firmware Upgrade Services".
        - In the Release_Notes.html, find the memory address that corresponds to your device for the stm32wb5x_FUS_fw.bin file
        - Select that file, the memory address, "verify download", and then "Firmware Upgrade".
        - Once complete, in the Release_Notes.html, find the memory address that corresponds to your device for the
          stm32wb5x_BLE_Stack_full_fw.bin file. It should not be the same memory address.
        - Select that file, the memory address, "verify download", and then "Firmware Upgrade".
        - Select "Start Wireless Stack".
        - Disconnect from the device.
        - In the examples folder for stm32wb, modify the memory.x file to match your target device.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let config = Config::default();
    let mut mbox = TlMbox::new(p.IPCC, Irqs, config);

    unsafe {
        pac::EXTI.cpu(1).imr(1).modify(|w| {
            w.set_line(4, true);
            w.set_line(6, true);
        });
    }

    info!("waiting for coprocessor to boot");

    spawner.spawn(read_sys()).unwrap();

    Timer::after(Duration::from_millis(500)).await;

    spawner.spawn(read_ble()).unwrap();

    Timer::after(Duration::from_millis(500)).await;

    //    let mut config_param: ShciConfigParam = Default::default();
    //
    //    config_param.device_id = 495 as u16;
    //    config_param.revision_id = 2003 as u16;
    //
    //    let command_status = mbox.sys_subsystem.shci_c2_config(Default::default()).await;
    //
    //    info!("command status: {}", command_status);
    //
    //    let command_status = mbox.sys_subsystem.shci_c2_ble_init(Default::default()).await;
    //
    //    info!("command status: {}", command_status);

    //    let command_status = mbox
    //        .sys_subsystem
    //        .write_and_get_response(
    //            TlPacketType::SysCmd,
    //            0xFC68,
    //            &[
    //                0x4C, 0x9C, 0x00, 0x08, 0x88, 0x00, 0x00, 0x20, 0x8C, 0x00, 0x00, 0x20, 0x26, 0x04, 0x04,
    //            ],
    //        )
    //        .await
    //        .payload[0];
    //
    //    info!("command status: {:x}", command_status);

    Timer::after(Duration::from_secs(3)).await;

    let command_status = mbox
        .sys_subsystem
        .write_and_get_response(
            TlPacketType::SysCmd,
            0xFC75,
            &[
                0x0F, 0x00, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x20, 0x95, 0x04,
            ],
        )
        .await
        .payload[0];

    info!("command status: {:x}", command_status);

    let command_status = mbox
        .sys_subsystem
        .write_and_get_response(
            TlPacketType::SysCmd,
            0xFC66,
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x00, 0x08, 0x00, 0x40, 0x05, 0x02, 0x01, 0x12,
                0x27, 0x9C, 0x00, 0xF4, 0x01, 0x00, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0x48, 0x01, 0x01, 0x00, 0x00, 0x20,
                0x00, 0x00, 0x00, 0x03, 0x72, 0x06, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00,
            ],
        )
        .await
        .payload[0];

    info!("command status: {:x}", command_status);

    Timer::after(Duration::from_millis(100)).await;

    mbox.ble_subsystem
        .write_and_get_response(TlPacketType::BleCmd, 0xc03, &[])
        .await;

    Timer::after(Duration::from_secs(3)).await;

    info!("Test OK");
    cortex_m::asm::bkpt();

    //    let p = embassy_stm32::init(Default::default());
    //    info!("Hello World!");
    //
    //    let config = Config::default();
    //    let mut mbox = TlMbox::new(p.IPCC, Irqs, config);
    //
    //    info!("waiting for coprocessor to boot");
    //    let event_box = mbox.sys_subsystem.read().await.unwrap();
    //
    //    let mut payload = [0u8; 6];
    //    event_box.copy_into_slice(&mut payload).unwrap();
    //
    //    let event_packet = event_box.event_packet();
    //    let kind = event_packet.event_serial.kind;
    //
    //    // means recieved SYS event, which indicates in this case that the coprocessor is ready
    //    if kind == 0x12 {
    //        let code = event_packet.event_serial.event.event_code;
    //        let payload_len = event_packet.event_serial.event.payload_len;
    //
    //        info!(
    //            "==> kind: {:#04x}, code: {:#04x}, payload_length: {}, payload: {:#04x}",
    //            kind,
    //            code,
    //            payload_len,
    //            payload[3..]
    //        );
    //    }
    //
    //    // initialize ble stack, does not return a response
    //    let _ = mbox.ble_init(Default::default()).await;
    //
    //    info!("resetting BLE");
    //    let _ = mbox.ble_subsystem.write(&[0x01, 0x03, 0x0c, 0x00, 0x00]).await;
    //
    //    info!("waiting for BLE...");
    //    let event_box = mbox.ble_subsystem.read().await.unwrap();
    //
    //    info!("BLE ready");
    //    cortex_m::asm::bkpt();
    //
    //    let mut payload = [0u8; 7];
    //    event_box.copy_into_slice(&mut payload).unwrap();
    //
    //    let event_packet = event_box.event_packet();
    //    let kind = event_packet.event_serial.kind;
    //
    //    let code = event_packet.event_serial.event.event_code;
    //    let payload_len = event_packet.event_serial.event.payload_len;
    //
    //    info!(
    //        "==> kind: {:#04x}, code: {:#04x}, payload_length: {}, payload: {:#04x}",
    //        kind,
    //        code,
    //        payload_len,
    //        payload[3..]
    //    );
    //
    //    info!("Test OK");
    //    cortex_m::asm::bkpt();
}
