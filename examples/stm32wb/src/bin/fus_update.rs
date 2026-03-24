#![no_std]
#![no_main]

use cortex_m::peripheral::SCB;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::WPAN_DEFAULT;
use embassy_stm32::rtc::Rtc;
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::fus::FirmwareUpgrader;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    /*
        How to make this work:

        - Obtain a NUCLEO-STM32WB55 from your preferred supplier.
        - Download and Install STM32CubeProgrammer.
        - Download stm32wb5x_FUS_fw.bin, stm32wb5x_BLE_Mac_802_15_4_fw.bin, and Release_Notes.html from
          gh:STMicroelectronics/STM32CubeWB@2234d97/Projects/STM32WB_Copro_Wireless_Binaries/STM32WB5x
        - Open STM32CubeProgrammer
        - On the right-hand pane, click "firmware upgrade" to upgrade the st-link firmware.
        - Once complete, click connect to connect to the device.
        - On the left hand pane, click the RSS signal icon to open "Firmware Upgrade Services".
        - In the Release_Notes.html, find the memory address that corresponds to your device for the stm32wb5x_FUS_fw.bin file
        - Select that file, the memory address, "verify download", and then "Firmware Upgrade".
        - Once complete, in the Release_Notes.html, find the memory address that corresponds to your device for the
          stm32wb5x_BLE_Mac_802_15_4_fw.bin file. It should not be the same memory address.
        - Select that file, the memory address, "verify download", and then "Firmware Upgrade".
        - Select "Start Wireless Stack".
        - Disconnect from the device.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    let mut config = embassy_stm32::Config::default();
    config.rcc = WPAN_DEFAULT;
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let (rtc, _time_provider) = Rtc::new(p.RTC);

    let config = Config::default();
    let mut mbox = TlMbox::init(p.IPCC, Irqs, config).await.unwrap();

    match mbox.sys_subsystem.wireless_fw_info() {
        None => info!("not yet initialized"),
        Some(fw_info) => {
            let version_major = fw_info.version_major();
            let version_minor = fw_info.version_minor();
            let subversion = fw_info.subversion();

            let sram2a_size = fw_info.sram2a_size();
            let sram2b_size = fw_info.sram2b_size();

            info!(
                "version {}.{}.{} - SRAM2a {} - SRAM2b {}",
                version_major, version_minor, subversion, sram2a_size, sram2b_size
            );
        }
    }

    let mut updater = FirmwareUpgrader::new(rtc, 15);

    updater.boot(mbox.sys_event, &mut mbox.sys_subsystem).await.unwrap();

    Timer::after(Duration::from_secs(3)).await;

    info!("System Reset");
    defmt::flush();
    SCB::sys_reset();
}
