//! FUS (firmware upgrade services) driver.
//!
//! The upgrade flow follows AN5185 and ST's FUS_CLI example, and is fully
//! self-contained: [`request_upgrade`] takes the ST-signed coprocessor
//! binaries (embedded in the application image by the caller), figures out the
//! current FUS/wireless stack versions, stages the next needed image in the
//! flash download area, drives FUS over IPCC/SHCI, survives the resets FUS
//! performs, and returns once the wireless stack is running.
//!
//! The pending-upgrade flag and the runaway-guard attempt counter live in the
//! `.uninit` linker section: they are never initialized at boot, so they
//! survive the resets FUS performs. This mirrors ST's BLE_Ota example, which
//! keeps its reboot keyword in the first bytes of SRAM1 instead of an RTC
//! backup register.

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::asm::wfi;
use cortex_m::peripheral::SCB;
use embassy_stm32::flash::{FLASH_BASE, FLASH_SIZE, Flash, WRITE_SIZE};
use embassy_stm32::mode::Blocking;
use embassy_stm32::pac;
use embassy_time::{Duration, Timer};

use crate::shci::{SchiSysEventReady, ShciFusGetStateErrorCode};
use crate::sub::sys::Sys;
pub use crate::wb::tables::StackType;

/// Magic number to request an upgrade
const MAGIC_PENDING: u32 = 0x4f2a9c1b;

/// Flash sector size of the STM32WB.
const SECTOR_SIZE: u32 = 0x1000;

/// Runaway guard: number of boots spent on upgrade attempts before giving up.
const MAX_ATTEMPTS: u32 = 20;

/// Set when an upgrade was requested and the device is expected to reboot into
/// FUS to perform it. Never initialized, so it survives resets.
#[unsafe(link_section = ".uninit")]
static UPGRADE_PENDING: AtomicU32 = AtomicU32::new(0);

/// Runaway-guard counter for the number of boots spent on upgrade attempts.
/// Never initialized, so it survives resets.
#[unsafe(link_section = ".uninit")]
static UPGRADE_ATTEMPTS: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" {
    static __sidata: u8;
    static __sdata: u8;
    static __edata: u8;
}

/// Error returned by the FUS upgrade functions.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// An upgrade step requires a signed coprocessor binary that was not passed
    /// to [`request_upgrade`]. The string names the missing parameter.
    MissingImage(&'static str),
    /// A FUS/Sys command failed.
    Command,
    /// FUS reported an error state it could not recover from.
    Fus,
    /// Runaway guard tripped: too many boots were spent on upgrade attempts.
    TooManyAttempts,
}

/// End of the application image in flash (end of `.rodata` plus the `.data` init image).
///
/// Relies on the `__sidata`/`__sdata`/`__edata` symbols provided by the
/// cortex-m-rt linker script.
fn app_flash_end() -> u32 {
    unsafe { &__sidata as *const u8 as u32 + ((&__edata as *const u8).offset_from(&__sdata as *const u8)) as u32 }
}

/// Program `image` into the flash download area and return its address.
///
/// The image is placed at the address recommended by AN5185 (below the secure flash
/// boundary with one sector of margin); the whole download area below it is erased
/// first, so that no stale image footers from earlier attempts remain for FUS to find.
fn stage_image(flash: &mut Flash<'_, Blocking>, image: &[u8]) -> u32 {
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

    info!(
        "erasing download area 0x{:08x}..0x{:08x} (SFSA=0x{:02x})",
        download_base, top, sfsa
    );
    flash
        .blocking_erase(download_base - FLASH_BASE as u32, top - FLASH_BASE as u32)
        .unwrap();

    info!("staging {} bytes at 0x{:08x}", size, addr);
    let full_len = image.len() / WRITE_SIZE * WRITE_SIZE;
    flash
        .blocking_write(addr - FLASH_BASE as u32, &image[..full_len])
        .unwrap();
    if full_len < image.len() {
        let mut tail = [0xFFu8; 16];
        tail[..image.len() - full_len].copy_from_slice(&image[full_len..]);
        flash
            .blocking_write(addr - FLASH_BASE as u32 + full_len as u32, &tail[..WRITE_SIZE])
            .unwrap();
    }

    addr
}

fn upgrade_pending() -> bool {
    UPGRADE_PENDING.load(Ordering::Relaxed) == MAGIC_PENDING
}

fn set_upgrade_pending(pending: bool) {
    UPGRADE_PENDING.store(if pending { MAGIC_PENDING } else { 0 }, Ordering::Relaxed);
}

/// Cancel a pending upgrade request.
///
/// Call this when there is nothing to install, so that a stale pending flag from an
/// earlier aborted attempt is not acted upon (it would make the FUS scan the download
/// area and install whatever image happens to be there).
pub fn cancel_upgrade() {
    set_upgrade_pending(false);
}

/// Perform a firmware upgrade with whatever binaries are supplied.
///
/// Each `Option` carries one ST-signed coprocessor binary (normally embedded with
/// `include_bytes!`): the FUS images used as stepping stones (`fus_0_5_3`,
/// `fus_1_2_0`, `fus_v2`) and the wireless stack (`stack_fw`, of which `stack_version`
/// is the version it installs and `stack_type` its [`StackType`]). Pass `None` for
/// binaries you do not have; the upgrader then gets as far as the supplied set
/// allows:
///
/// - FUS < V1.2.0: install the matching intermediate FUS binary (V0.5.3 or V1.x)
/// - FUS == V1.2.0: install the latest FUS V2
/// - FUS >= V2.0: install `stack_fw` when no wireless stack is installed, when the
///   installed stack has a different type than `stack_type`, or when it is older
///   than `stack_version`. Pass [`StackType::None`] as `stack_type` to accept any
///   installed stack type (version comparison only).
///
/// If the next step needs a binary that was not supplied, any pending request is
/// cancelled and [`Error::MissingImage`] is returned. Otherwise the image is staged
/// in the flash download area and the upgrade is driven to completion, resetting the
/// device as required; on the final boot the new wireless stack is running and
/// `Ok(())` is returned.
///
/// `ready` is the CPU2 ready event from [`crate::TlMbox::init`]. Call this once per
/// boot, after IPCC is up.
pub async fn request_upgrade(
    sys: &mut Sys<'_>,
    flash: &mut Flash<'_, Blocking>,
    ready: SchiSysEventReady,
    fus_0_5_3: Option<&[u8]>,
    fus_1_2_0: Option<&[u8]>,
    fus_v2: Option<&[u8]>,
    stack_fw: Option<&[u8]>,
    stack_version: (u8, u8, u8),
    stack_type: StackType,
) -> Result<(), Error> {
    let fus_version = sys.fus_version();
    // The FUS reports the installed wireless stack version even while it (and not
    // the stack) is running, so take the stack version from whichever table is
    // present. The stack type is only reported by the running wireless stack.
    let running_type = sys.wireless_fw_info().map(|info| info.stack_type());
    let running_stack = sys.wireless_stack_version();
    info!(
        "FUS version: {:?}  wireless stack: {:?} ({:?})",
        fus_version, running_stack, running_type
    );

    // Pick the next image to install; `None` means there is nothing to install.
    let image = match (fus_version, running_stack) {
        // FUS V0.5.3 can only be upgraded by its dedicated binary.
        (Some((0, _, _)), _) => fus_0_5_3.map(Some).ok_or(Error::MissingImage("fus_0_5_3")),
        // Any FUS V1 below V1.2.0 goes through the V1.2.0 binary.
        (Some(fus), _) if fus < (1, 2, 0) => fus_1_2_0.map(Some).ok_or(Error::MissingImage("fus_1_2_0")),
        // FUS V1.2.0 is the stepping stone to the latest FUS V2.
        (Some((1, 2, 0)), _) => fus_v2.map(Some).ok_or(Error::MissingImage("fus_v2")),
        // FUS V2: install the wireless stack when none is installed, when the
        // installed stack has a different type than requested, or when it is older
        // than the supplied stack binary.
        (Some(_), _) => {
            let type_matches = stack_type == StackType::None || running_type == Some(stack_type);
            if type_matches && running_stack.is_some_and(|v| v >= stack_version) {
                Ok(None)
            } else {
                stack_fw.map(Some).ok_or(Error::MissingImage("stack_fw"))
            }
        }
        // No FUS version yet (first boot of a virgin chip): let `boot` initialize FUS.
        (None, _) => Ok(None),
    };

    let image = match image {
        Ok(image) => image,
        Err(e) => {
            // Never leave a stale pending flag behind: `boot` would otherwise
            // install whatever image happens to be in the download area.
            cancel_upgrade();
            return Err(e);
        }
    };

    match image {
        Some(image) => {
            // Runaway guard, survives resets in the counter next to the pending flag.
            // Only trust the persisted counter when a pending flag is actually set:
            // otherwise the value is whatever this RAM happened to hold before
            // (the section is never initialized, even across reflashing).
            let attempts = if upgrade_pending() {
                UPGRADE_ATTEMPTS.load(Ordering::Relaxed) + 1
            } else {
                1
            };
            UPGRADE_ATTEMPTS.store(attempts, Ordering::Relaxed);
            if attempts > MAX_ATTEMPTS {
                error!("too many failed upgrade attempts ({})", attempts);
                return Err(Error::TooManyAttempts);
            }

            // When this boot only deletes the old stack (type change), staging now
            // is wasted: the delete reclaims the download area. `boot` performs the
            // delete; the next boot sees no stack and stages normally.
            let replace = running_stack.is_some()
                && stack_type != StackType::None
                && running_type.map_or(true, |t| t != stack_type);
            if !replace {
                stage_image(flash, image);
            }

            set_upgrade_pending(true);

            if ready == SchiSysEventReady::WirelessFwRunning {
                // Ask the running wireless stack to reboot into FUS.
                start_upgrade(sys).await.map_err(|_| Error::Command)?;
            }
        }
        None => {
            UPGRADE_ATTEMPTS.store(0, Ordering::Relaxed);
            set_upgrade_pending(false);
        }
    }

    // FUS is (or will be after the reset) running: request the upgrade and
    // track it until the new wireless stack runs.
    boot(ready, sys, stack_type).await
}

/// Start the upgrade of firmware; must be called while the wireless stack is running,
/// after the firmware image has been written to the flash download area and the
/// upgrade has been marked pending ([`request_upgrade`] does all of this).
///
/// Requests the device to reboot into FUS by sending two `FUS_GET_STATE` commands,
/// then waits for the FUS-initiated system reset. On the next boot, [`boot`]
/// performs the upgrade. Retries a few times: some FUS versions appear to drop the
/// request if the two commands do not both arrive while it is listening.
pub async fn start_upgrade(sys: &mut Sys<'_>) -> Result<(), Error> {
    for attempt in 1..=3 {
        info!("requesting reboot into FUS (attempt {})", attempt);

        sys.shci_c2_fus_get_state().await.map_err(|_| Error::Command)?;
        sys.shci_c2_fus_get_state().await.map_err(|_| Error::Command)?;

        // Wait for the FUS to reboot us (up to 5s, in 100ms slices so we also log)
        for _ in 0..50 {
            Timer::after(Duration::from_millis(100)).await;
        }
    }

    error!("FUS did not reboot into FUS mode");
    Err(Error::Command)
}

/// Called on boot to drive the FUS upgrade process, or to start the wireless stack.
///
/// `stack_type` is the type of the wireless stack being installed; pass
/// [`StackType::None`] to never delete the installed stack first. The FUS refuses
/// `FUS_FW_UPGRADE` while another wireless stack is installed, so when the types
/// differ the old stack is deleted first (`FUS_FW_DELETE`, AN5185). The installed
/// stack's type is only readable while it runs, so whenever the FUS itself is
/// running, any installed stack is deleted conservatively.
///
/// Returns `Ok(())` once the wireless stack is running (upgrade completed, or nothing
/// to do). Resets the system as required by the FUS state machine; otherwise loops
/// forever, so it never returns `Err` on transient states.
pub async fn boot(ready_event: SchiSysEventReady, sys: &mut Sys<'_>, stack_type: StackType) -> Result<(), Error> {
    let firmware_started = ready_event == SchiSysEventReady::WirelessFwRunning
        && (sys
            .wireless_fw_info()
            .is_some_and(|info| info.version_major() + info.version_minor() > 0)
            // The table keeps the FUS layout until the stack updates it, so also
            // accept the FUS table: it reports the installed stack version.
            || sys.fus_info().is_some_and(|info| info.wireless_stack_version != 0));

    // If wireless firmware is started, then abort the upgrade and return
    if firmware_started {
        set_upgrade_pending(false);
        UPGRADE_ATTEMPTS.store(0, Ordering::Relaxed);

        return Ok(());
    }

    let upgrade_pending = upgrade_pending();
    // The FUS refuses FUS_FW_UPGRADE while another wireless stack is installed, so
    // a type change goes through FUS_FW_DELETE first (AN5185). The installed stack's
    // type is only readable while it runs; when the FUS itself is running, any
    // installed stack is deleted conservatively.
    let delete_first = stack_type != StackType::None
        && sys.fus_info().is_some_and(|info| info.wireless_stack_version != 0)
        && (ready_event != SchiSysEventReady::WirelessFwRunning
            || sys
                .wireless_fw_info()
                .is_some_and(|info| info.stack_type() != stack_type));
    // FUS keeps reporting the last operation's error until a new operation is
    // requested; a failed request is retried a bounded number of times so a
    // genuinely bad image still fails (the caller's runaway guard then applies).
    let mut fwupgrade_retries = 0;

    loop {
        let state = sys.shci_c2_fus_get_state().await.map_err(|_| Error::Command)?;

        match state.state() {
            // FUS_STATE_VALUE_IDLE
            0x00 => {
                if upgrade_pending {
                    // The pending flag is cleared before requesting the upgrade: if the
                    // device reboots before the command is processed, we do not retry on
                    // the next boot with a (possibly erased) download area.
                    set_upgrade_pending(false);

                    if delete_first {
                        info!("deleting installed wireless stack (type change)");
                        sys.shci_c2_fus_fwdelete().await.map_err(|_| Error::Command)?;

                        // The FUS acknowledges the command before applying it, so
                        // resetting right away would abort the delete. Wait until the
                        // stack is actually gone from the FUS device info table.
                        let mut timeout = 0;
                        loop {
                            Timer::after(Duration::from_millis(100)).await;
                            if let Some(info) = sys.fus_info() {
                                if info.wireless_stack_version == 0 {
                                    break;
                                }
                            }
                            timeout += 1;
                            if timeout >= 50 {
                                error!("FUS did not delete the wireless stack");
                                return Err(Error::Fus);
                            }
                        }

                        // The FUS reclaimed the deleted stack's flash; reset so the
                        // next boot stages into the new download area.
                        SCB::sys_reset();
                    }

                    // The FUS may refuse the upgrade briefly after a delete (it is
                    // still reclaiming the deleted stack's flash internally), so
                    // retry a few times before giving up.
                    let mut retries = 0;
                    loop {
                        info!("requesting FUS firmware upgrade");
                        match sys.shci_c2_fus_fwupgrade(0, 0).await {
                            Ok(_) => break,
                            Err(_) => {
                                retries += 1;
                                if retries >= 5 {
                                    error!("FUS refused the firmware upgrade");
                                    return Err(Error::Fus);
                                }
                                Timer::after(Duration::from_secs(1)).await;
                            }
                        }
                    }
                } else {
                    // FUS is idle and the upgrade is complete: start the wireless stack.
                    sys.shci_c2_fus_startws().await.map_err(|_| Error::Command)?;

                    // Wait for the FUS to reboot us into the wireless stack
                    loop {
                        wfi();
                    }
                }
            }
            // FUS upgrade, wireless stack upgrade, or service ongoing
            0x10..=0x3F => {
                trace!("FUS operation ongoing (state 0x{:02x})", state.state());
                Timer::after(Duration::from_secs(1)).await;
            }
            // FUS_STATE_ERROR_STATE_NOT_RUNNING: the wireless stack is running
            0xFE => {
                // The FUS handed control to the wireless stack without resetting us;
                // reset to boot cleanly with the new firmware.
                SCB::sys_reset();
            }
            // FUS_STATE_VALUE_ERROR
            0xFF => match state.error_code() {
                // This is the first time in the life of the product the FUS is involved.
                // After this command, it will be properly initialized; request the
                // device to reboot.
                Some(ShciFusGetStateErrorCode::FusStateErrorErrUnknown) => {
                    SCB::sys_reset();
                }
                error_code => {
                    fwupgrade_retries += 1;
                    if fwupgrade_retries > 5 {
                        error!("FUS reported error: {:?}", error_code);
                        return Err(Error::Fus);
                    }

                    warn!("FUS reported error: {:?}; retrying fwupgrade", error_code);
                    sys.shci_c2_fus_fwupgrade(0, 0).await.map_err(|_| Error::Command)?;
                }
            },
            state => {
                error!("unexpected FUS state 0x{:02x}", state);

                return Err(Error::Fus);
            }
        }
    }
}
