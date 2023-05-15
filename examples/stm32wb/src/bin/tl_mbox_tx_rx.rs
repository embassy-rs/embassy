#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::interrupt;
use embassy_stm32::ipcc::{Config, Ipcc};
use embassy_stm32::tl_mbox::TlMbox;
use {defmt_rtt as _, panic_probe as _};

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
        - Disconnect from the device.
        - In the examples folder for stm32wb, modify the memory.x file to match your target device.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let config = Config::default();
    let mut ipcc = Ipcc::new(p.IPCC, config);

    let rx_irq = interrupt::take!(IPCC_C1_RX);
    let tx_irq = interrupt::take!(IPCC_C1_TX);

    let mbox = TlMbox::init(&mut ipcc, rx_irq, tx_irq);

    // initialize ble stack, does not return a response
    // mbox.shci_ble_init(&mut ipcc, Default::default());

    info!("waiting for coprocessor to boot");
    let event_box = mbox.read().await;

    let mut payload = [0u8; 6];
    event_box.copy_into_slice(&mut payload).unwrap();

    let event_packet = event_box.evt();
    let kind = event_packet.evt_serial.kind;

    // means recieved SYS event, which indicates in this case that the coprocessor is ready
    if kind == 0x12 {
        let code = event_packet.evt_serial.evt.evt_code;
        let payload_len = event_packet.evt_serial.evt.payload_len;

        info!(
            "==> kind: {:#04x}, code: {:#04x}, payload_length: {}, payload: {:#04x}",
            kind,
            code,
            payload_len,
            payload[3..]
        );
    }

    mbox.shci_ble_init(&mut ipcc, Default::default());

    info!("resetting BLE");
    mbox.send_ble_cmd(&mut ipcc, &[0x01, 0x03, 0x0c]);

    let event_box = mbox.read().await;

    let mut payload = [0u8; 7];
    event_box.copy_into_slice(&mut payload).unwrap();

    let event_packet = event_box.evt();
    let kind = event_packet.evt_serial.kind;

    let code = event_packet.evt_serial.evt.evt_code;
    let payload_len = event_packet.evt_serial.evt.payload_len;

    info!(
        "==> kind: {:#04x}, code: {:#04x}, payload_length: {}, payload: {:#04x}",
        kind, code, payload_len, payload
    );

    loop {}
}
