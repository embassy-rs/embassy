#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::tl_mbox::{Config, TlMbox};
use embassy_stm32::{bind_interrupts, tl_mbox};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => tl_mbox::ReceiveInterruptHandler;
    IPCC_C1_TX => tl_mbox::TransmitInterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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

    info!("waiting for coprocessor to boot");

    let mut payload = [0u8; 6];
    mbox.sys_subsystem
        .read(|event_packet| {
            let _ = event_packet.copy_into_slice(&mut payload);

            let kind = unsafe { (&event_packet.event_serial.kind as *const u8).read_volatile() };

            // means recieved SYS event, which indicates in this case that the coprocessor is ready
            if kind == 0x12 {
                let code = unsafe { (&event_packet.event_serial.event.event_code as *const u8).read_volatile() };
                let payload_len =
                    unsafe { (&event_packet.event_serial.event.payload_len as *const u8).read_volatile() };

                info!(
                    "==> kind: {:#04x}, code: {:#04x}, payload_length: {}, payload: {:#04x}",
                    kind,
                    code,
                    payload_len,
                    payload[3..]
                );
            }
        })
        .await;

    let command_status = mbox.sys_subsystem.schi_c2_mac_802_15_4_init().await;

    info!("command status: {}", command_status);

    info!("Test OK");
    cortex_m::asm::bkpt();

    //
    //    // initialize ble stack, does not return a response
    //    let _ = mbox.mac_802_15_4_init().await;
    //
    //    info!("waiting for sys...");
    //    let event = mbox.sys_subsystem.read_command().await.unwrap();
    //
    //    let size = event.size().unwrap();
    //    // let size = 10;
    //    let event_packet = event.event_packet();
    //    let buf = unsafe { core::slice::from_raw_parts((&event_packet as *const _) as *const u8, size) };
    //
    //    info!("{:#04x}", buf);
    //
    //    info!("resetting BLE");
    //    let _ = mbox.mac_subsystem.write(&[0x01, 0x03, 0x0c, 0x00, 0x00]).await;
    //
    //    info!("waiting for BLE...");
    //    let event_box = mbox.mac_subsystem.read_event().await.unwrap();
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

    info!("Test OK");
    cortex_m::asm::bkpt();
}
