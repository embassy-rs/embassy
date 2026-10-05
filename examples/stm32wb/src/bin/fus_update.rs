#![no_std]
#![no_main]

/*
    Self-contained FUS OTA for the STM32WB55.

    This example upgrades the FUS and installs the BLE + MAC 802.15.4 wireless stack
    without any external tool: the ST-signed coprocessor binaries are embedded in the
    application image, programmed into the flash download area by the application
    itself, and installed by the FUS (firmware upgrade services) running on the M0+.

    Setup:

    - Obtain a NUCLEO-STM32WB55 (or any STM32WB55 board reachable by probe-rs).
    - Download the coprocessor binaries from the STM32CubeWB repository
      (see firmware/README.md in this directory for the exact commands).
    - Flash and run this example (a release build is required so the application image
      stays small enough to leave room for the download area):
      cargo run --release --bin fus_update
      (or: probe-rs run --chip STM32WB55RG target/thumbv7em-none-eabi/release/fus_update)

    The example figures out the current FUS/wireless stack versions on its own and
    performs however many upgrade steps are needed, roughly:

    - FUS < V1.2.0:  install the matching intermediate FUS binary (V0.5.3 or V1.x)
    - FUS == V1.2.0: install the latest FUS V2
    - FUS >= V2.0:   install stm32wb5x_BLE_Mac_802_15_4_fw.bin (BLE + MAC combo stack)

    The device resets several times during the process; on the final boot the new
    wireless stack is running and its version is printed.

    Note: extended stack binaries are not supported by the embassy wireless stack at
    this time. Do not attempt to install a stack with "extended" in the name.
*/

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::flash::{Flash, FLASH_BASE, FLASH_SIZE, WRITE_SIZE};
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::pac;
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32::rtc::{AnyRtc, Rtc};
use embassy_stm32_wpan::shci::SchiSysEventReady;
use embassy_stm32_wpan::{TlMbox, fus::FirmwareUpgrader};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

// ST-signed coprocessor binaries from the STM32CubeWB package
// (Projects/STM32WB_Copro_Wireless_Binaries/STM32WB5x). See firmware/README.md.
const FUS_FW_0_5_3: &[u8] = include_bytes!("../../firmware/stm32wb5x_FUS_fw_for_fus_0_5_3.bin");
const FUS_FW_1_2_0: &[u8] = include_bytes!("../../firmware/stm32wb5x_FUS_fw_1_2_0.bin");
const FUS_FW_V2: &[u8] = include_bytes!("../../firmware/stm32wb5x_FUS_fw.bin");
const STACK_FW: &[u8] = include_bytes!("../../firmware/stm32wb5x_BLE_Mac_802_15_4_fw.bin");

/// Version of `stm32wb5x_BLE_Mac_802_15_4_fw.bin` above; installation is skipped when
/// the running wireless stack already reports this version or newer.
const STACK_VERSION: (u8, u8, u8) = (1, 24, 0);

/// Runaway guard: number of boots spent on upgrade attempts before giving up.
const MAX_ATTEMPTS: u32 = 20;

const SECTOR_SIZE: u32 = 0x1000;

/// RTC backup register used to count upgrade attempts (runaway guard).
const ATTEMPTS_REG: usize = 16;

unsafe extern "C" {
    static __sidata: u8;
    static __sdata: u8;
    static __edata: u8;
}

/// End of the application image in flash (end of `.rodata` plus the `.data` init image).
fn app_flash_end() -> u32 {
    unsafe {
        &__sidata as *const u8 as u32 + ((&__edata as *const u8).offset_from(&__sdata as *const u8)) as u32
    }
}

fn decode_version(version: u32) -> (u8, u8, u8) {
    ((version >> 24) as u8, (version >> 16) as u8, (version >> 8) as u8)
}

/// Program `image` into the flash download area and return its address.
///
/// The image is placed at the address recommended by AN5185 (below the secure flash
/// boundary with one sector of margin); the whole download area below it is erased
/// first, so that no stale image footers from earlier attempts remain for FUS to find.
fn stage_image(flash: &mut Flash<'_, embassy_stm32::flash::Blocking>, image: &[u8]) -> u32 {
    // Secure flash start address (SFSA option byte, expressed in sectors).
    let sfsa = pac::FLASH.sfr().read().sfsa() as u32;
    let secure_start = FLASH_BASE as u32 + sfsa * SECTOR_SIZE;
    let flash_end = FLASH_BASE as u32 + FLASH_SIZE as u32;
    let top = secure_start.min(flash_end);

    // Place the image at the address recommended by AN5185: below the secure flash
    // boundary, leaving one sector of margin (FUS v1 rule) plus the four sectors of
    // the NVM data section (FUS v2 rule), so the placement is valid for both.
    let size = image.len() as u32;
    let padded_size = size.div_ceil(WRITE_SIZE as u32) * WRITE_SIZE as u32;
    let addr = (top - padded_size - 5 * SECTOR_SIZE) & !(SECTOR_SIZE - 1);

    // The download area starts right above the application image (which contains the
    // embedded binaries) and must hold the whole image below the secure boundary.
    let download_base = app_flash_end().div_ceil(SECTOR_SIZE) * SECTOR_SIZE;
    core::assert!(
        addr >= download_base && download_base < top,
        "not enough free flash between the application and the secure boundary (build with --release)"
    );

    info!("erasing download area 0x{:08x}..0x{:08x} (SFSA=0x{:02x})", download_base, top, sfsa);
    flash.blocking_erase(download_base - FLASH_BASE as u32, top - FLASH_BASE as u32).unwrap();

    info!("staging {} bytes at 0x{:08x}", size, addr);
    let full_len = image.len() / WRITE_SIZE * WRITE_SIZE;
    flash.blocking_write(addr - FLASH_BASE as u32, &image[..full_len]).unwrap();
    if full_len < image.len() {
        let mut tail = [0xFFu8; 16];
        tail[..image.len() - full_len].copy_from_slice(&image[full_len..]);
        flash
            .blocking_write(addr - FLASH_BASE as u32 + full_len as u32, &tail[..WRITE_SIZE])
            .unwrap();
    }

    addr
}

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc = RccConfig::new_wpan();
    let p = embassy_stm32::init(config);
    info!("STM32WB55 FUS OTA");

    let (rtc, _time_provider) = Rtc::new(p.RTC);
    rtc.write_backup_register(19, rtc.read_backup_register(19).unwrap_or(0) + 1);

    let config = Config::default();
    let mut mbox = TlMbox::init(p.IPCC, Irqs, config);
    let mut flash = Flash::new_blocking(p.FLASH);

    let ready = mbox.sys.read_ready().await.unwrap();

    // The FUS reports the installed wireless stack version even while it (and not the
    // stack) is running, so take the stack version from whichever table is present.
    let fus_raw = mbox.sys.fus_version();
    let stack_raw = mbox
        .sys
        .fus_info()
        .map(|fus| fus.wireless_stack_version)
        .filter(|v| *v != 0)
        .or_else(|| mbox.sys.wireless_fw_info().map(|info| info.version));
    let fus_version = fus_raw.map(decode_version);
    let stack_version = stack_raw.map(decode_version);
    info!("CPU2 ready: {:?}  FUS version: {:?}  wireless stack: {:?}", ready, fus_version, stack_version);

    // Diagnostics: raw versions + boot counter, readable over SWD at
    // 0x40002894.. even when RTT is not attached.
    rtc.write_backup_register(17, fus_raw.unwrap_or(0xFFFF_FFFF));
    rtc.write_backup_register(18, stack_raw.unwrap_or(0xFFFF_FFFF));
    rtc.write_backup_register(20, ready as u32);

    // Decide which image (if any) needs to be installed next.
    let image: Option<&'static [u8]> = match (fus_version, stack_version) {
        // FUS V0.5.3 can only be upgraded by its dedicated binary.
        (Some((0, _, _)), _) => Some(FUS_FW_0_5_3),
        // Any FUS V1 below V1.2.0 goes through the V1.2.0 binary.
        (Some(fus), _) if fus < (1, 2, 0) => Some(FUS_FW_1_2_0),
        // FUS V1.2.0 is the stepping stone to the latest FUS V2.
        (Some((1, 2, 0)), _) => Some(FUS_FW_V2),
        // FUS V2: install the wireless stack unless it is already up to date.
        (Some(_), Some(stack)) if stack >= STACK_VERSION => None,
        (Some(_), _) => Some(STACK_FW),
        // No FUS version yet (first boot of a virgin chip): let `boot()` initialize FUS.
        (None, _) => None,
    };

    match image {
        Some(image) => {
            // Runaway guard.
            let attempts = rtc.read_backup_register(ATTEMPTS_REG).unwrap_or(0) + 1;
            rtc.write_backup_register(ATTEMPTS_REG, attempts);
            core::assert!(attempts <= MAX_ATTEMPTS, "too many failed upgrade attempts");

            stage_image(&mut flash, image);

            let mut upgrader = FirmwareUpgrader::new(rtc, 15);
            upgrader.request_upgrade();
            if ready == SchiSysEventReady::WirelessFwRunning {
                // Ask the running wireless stack to reboot into FUS.
                upgrader.start_upgrade(&mut mbox.sys).await.unwrap();
            }
            // FUS is (or will be after the reset) running: request the upgrade and
            // track it until the new wireless stack runs.
            upgrader.boot(ready, &mut mbox.sys).await.unwrap();
        }
        None => {
            rtc.write_backup_register(ATTEMPTS_REG, 0);

            let mut upgrader = FirmwareUpgrader::new(rtc, 15);
            upgrader.cancel_upgrade();
            upgrader.boot(ready, &mut mbox.sys).await.unwrap();
        }
    }

    info!("wireless stack up to date");
}
