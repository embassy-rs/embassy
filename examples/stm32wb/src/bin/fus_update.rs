#![no_std]
#![no_main]

/*
    Self-contained FUS OTA for the STM32WB55.

    This example upgrades the FUS and installs the BLE + MAC 802.15.4 wireless stack
    without any external tool: the ST-signed coprocessor binaries are embedded in the
    application image, programmed into the flash download area by the application
    itself, and installed by the FUS (firmware upgrade services) running on the M0+.
    All of the logic lives in `embassy_stm32_wpan::TlMbox::with_firmware`.

    Setup:

    - Obtain a NUCLEO-STM32WB55 (or any STM32WB55 board reachable by probe-rs).
    - Download the coprocessor binaries from the STM32CubeWB repository
      (see firmware/README.md in this directory for the exact commands).
    - Flash and run this example (a release build is required so the application image
      stays small enough to leave room for the download area):
      cargo run --release --bin fus_update
      (or: probe-rs run --chip STM32WB55RG target/thumbv7em-none-eabihf/release/fus_update)

    The upgrader figures out the current FUS/wireless stack versions on its own and
    performs however many upgrade steps the supplied binaries allow, roughly:

    - FUS < V1.2.0:  install the matching intermediate FUS binary (V0.5.3 or V1.x)
    - FUS == V1.2.0: install the latest FUS V2
    - FUS >= V2.0:   install stm32wb5x_BLE_Mac_802_15_4_fw.bin (BLE + MAC combo stack,
                     replacing any stack of a different type)

    Binaries that are commented out below are passed as `None`; if the upgrade path
    needs one of them, `with_firmware` returns `Error::MissingImage` and tells you
    which. The device resets several times during the process; on the final boot the
    new wireless stack is running and its version is printed.

    Note: extended stack binaries are not supported by the embassy wireless stack at
    this time. Do not attempt to install a stack with "extended" in the name.
*/

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::flash::Flash;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::{Config as RccConfig, StopMode, WakeGuard};
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::fus::StackType;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

// ST-signed coprocessor binaries from the STM32CubeWB package
// (Projects/STM32WB_Copro_Wireless_Binaries/STM32WB5x). See firmware/README.md.

#[cfg(feature = "fw_fus")]
const FUS_FW_0_5_3: &[u8] = include_bytes!("../../firmware/stm32wb5x_FUS_fw_for_fus_0_5_3.bin");
#[cfg(feature = "fw_fus")]
const FUS_FW_1_2_0: &[u8] = include_bytes!("../../firmware/stm32wb5x_FUS_fw_1_2_0.bin");
#[cfg(feature = "fw_fus")]
const FUS_FW_V2: &[u8] = include_bytes!("../../firmware/stm32wb5x_FUS_fw.bin");

#[cfg(not(feature = "fw_fus"))]
const FUS_FW_0_5_3: &[u8] = &[];
#[cfg(not(feature = "fw_fus"))]
const FUS_FW_1_2_0: &[u8] = &[];
#[cfg(not(feature = "fw_fus"))]
const FUS_FW_V2: &[u8] = &[];

#[cfg(feature = "fw_mac_ble")]
const STACK_FW: &[u8] = include_bytes!("../../firmware/stm32wb5x_BLE_Mac_802_15_4_fw.bin");

#[cfg(feature = "fw_hci")]
const STACK_FW: &[u8] = include_bytes!("../../firmware/stm32wb5x_BLE_HCILayer_fw.bin");

#[cfg(not(any(feature = "fw_hci", feature = "fw_mac_ble")))]
const STACK_FW: &[u8] = &[];

/// Version of the stack binary selected above; installation is skipped when
/// the running wireless stack already reports this version or newer. (Both
/// `stm32wb5x_BLE_Mac_802_15_4_fw.bin` and `stm32wb5x_BLE_HCILayer_fw.bin` are
/// V1.24.0 at the downloaded revision.)
const STACK_VERSION: (u8, u8, u8) = (1, 24, 0);

/// Wireless stack type of `STACK_FW` above; a stack of a different type is
/// replaced even when its version is current.
#[cfg(feature = "fw_hci")]
const STACK_TYPE: StackType = StackType::BleHci;
#[cfg(not(feature = "fw_hci"))]
const STACK_TYPE: StackType = StackType::BleMacStatic;

/// Pass an embedded binary only if it is present: the `include_bytes!` lines above
/// stay commented out until the files are downloaded (see firmware/README.md).
fn img(binary: &'static [u8]) -> Option<&'static [u8]> {
    if binary.is_empty() { None } else { Some(binary) }
}

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc = RccConfig::new_wpan();
    let p = embassy_stm32::init(config);
    info!("STM32WB55 FUS OTA");

    let _guard = WakeGuard::new(StopMode::Stop1);

    let mut flash = Flash::new_blocking(p.FLASH);

    let mbox = TlMbox::with_firmware(
        p.IPCC,
        Irqs,
        Config::default(),
        &mut flash,
        img(FUS_FW_0_5_3),
        img(FUS_FW_1_2_0),
        img(FUS_FW_V2),
        img(STACK_FW),
        STACK_VERSION,
        STACK_TYPE,
    )
    .await
    .unwrap();

    info!(
        "wireless stack up to date: version {:?}",
        mbox.sys.wireless_stack_version()
    );
}
