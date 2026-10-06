use cortex_m::asm::wfi;
use cortex_m::peripheral::SCB;
use embassy_stm32::flash::{Blocking, FLASH_BASE, FLASH_SIZE, Flash, WRITE_SIZE};
use embassy_stm32::pac;
use embassy_stm32::rtc::AnyRtc;
use embassy_time::{Duration, Timer};

use crate::shci::{SchiSysEventReady, ShciFusGetStateErrorCode};
use crate::sub::sys::Sys;

#[derive(Clone, Copy, PartialEq)]
enum UpgradeStatus {
    Pending,
    Complete,
}

/// Magic number to request an upgrade
const MAGIC_PENDING: u32 = 0x4f2a9c1b;

/// Flash sector size of the STM32WB.
const SECTOR_SIZE: u32 = 0x1000;

/// Runaway guard: number of boots spent on upgrade attempts before giving up.
const MAX_ATTEMPTS: u32 = 20;

unsafe extern "C" {
    static __sidata: u8;
    static __sdata: u8;
    static __edata: u8;
}

/// Error returned by [`FirmwareUpgrader`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// An upgrade step requires a signed coprocessor binary that was not passed
    /// to [`FirmwareUpgrader::request_upgrade`]. The string names the missing
    /// parameter.
    MissingImage(&'static str),
    /// A FUS/Sys command failed.
    Command,
    /// FUS reported an error state it could not recover from.
    Fus,
    /// Runaway guard tripped: too many boots were spent on upgrade attempts.
    TooManyAttempts,
}

fn decode_version(version: u32) -> (u8, u8, u8) {
    ((version >> 24) as u8, (version >> 16) as u8, (version >> 8) as u8)
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

/// Administers FUS upgrades.
///
/// The upgrade flow follows AN5185 and ST's FUS_CLI example, and is fully
/// self-contained: [`Self::request_upgrade`] takes the ST-signed coprocessor
/// binaries (embedded in the application image by the caller), figures out the
/// current FUS/wireless stack versions, stages the next needed image in the
/// flash download area, drives FUS over IPCC/SHCI, survives the resets FUS
/// performs, and returns once the wireless stack is running.
///
/// `backup_register` selects the RTC backup register holding the pending
/// upgrade flag across resets; the next register (`backup_register + 1`) is
/// used as the runaway-guard attempt counter.
pub struct FirmwareUpgrader<T: AnyRtc> {
    rtc: T,
    backup_register: usize,
}

impl<T: AnyRtc> FirmwareUpgrader<T> {
    pub fn new(rtc: T, backup_register: usize) -> Self {
        Self { rtc, backup_register }
    }

    fn get_upgrade_status(&mut self) -> UpgradeStatus {
        if self.rtc.read_backup_register(self.backup_register).unwrap_or_default() == MAGIC_PENDING {
            UpgradeStatus::Pending
        } else {
            UpgradeStatus::Complete
        }
    }

    fn set_upgrade_status(&mut self, upgrade_status: UpgradeStatus) {
        self.rtc.write_backup_register(
            self.backup_register,
            match upgrade_status {
                UpgradeStatus::Complete => 0,
                UpgradeStatus::Pending => MAGIC_PENDING,
            },
        )
    }

    /// Cancel a pending upgrade request.
    ///
    /// Call this when there is nothing to install, so that a stale pending flag from an
    /// earlier aborted attempt is not acted upon (it would make the FUS scan the download
    /// area and install whatever image happens to be there).
    pub fn cancel_upgrade(&mut self) {
        self.set_upgrade_status(UpgradeStatus::Complete);
    }

    /// Perform a firmware upgrade with whatever binaries are supplied.
    ///
    /// Each `Option` carries one ST-signed coprocessor binary (normally embedded with
    /// `include_bytes!`): the FUS images used as stepping stones (`fus_0_5_3`,
    /// `fus_1_2_0`, `fus_v2`) and the wireless stack (`stack_fw`, of which `stack_version`
    /// is the version it installs). Pass `None` for binaries you do not have; the
    /// upgrader then gets as far as the supplied set allows:
    ///
    /// - FUS < V1.2.0: install the matching intermediate FUS binary (V0.5.3 or V1.x)
    /// - FUS == V1.2.0: install the latest FUS V2
    /// - FUS >= V2.0: install `stack_fw`, unless the running wireless stack already
    ///   reports `stack_version` or newer
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
        &mut self,
        sys: &mut Sys<'_>,
        flash: &mut Flash<'_, Blocking>,
        ready: SchiSysEventReady,
        fus_0_5_3: Option<&[u8]>,
        fus_1_2_0: Option<&[u8]>,
        fus_v2: Option<&[u8]>,
        stack_fw: Option<&[u8]>,
        stack_version: (u8, u8, u8),
    ) -> Result<(), Error> {
        let fus_version = sys.fus_version().map(decode_version);
        // The FUS reports the installed wireless stack version even while it (and not
        // the stack) is running, so take the stack version from whichever table is present.
        let running_stack = sys
            .fus_info()
            .map(|fus| fus.wireless_stack_version)
            .filter(|v| *v != 0)
            .or_else(|| sys.wireless_fw_info().map(|info| info.version))
            .map(decode_version);
        info!("FUS version: {:?}  wireless stack: {:?}", fus_version, running_stack);

        // Pick the next image to install; `None` means there is nothing to install.
        let image = match (fus_version, running_stack) {
            // FUS V0.5.3 can only be upgraded by its dedicated binary.
            (Some((0, _, _)), _) => fus_0_5_3.map(Some).ok_or(Error::MissingImage("fus_0_5_3")),
            // Any FUS V1 below V1.2.0 goes through the V1.2.0 binary.
            (Some(fus), _) if fus < (1, 2, 0) => fus_1_2_0.map(Some).ok_or(Error::MissingImage("fus_1_2_0")),
            // FUS V1.2.0 is the stepping stone to the latest FUS V2.
            (Some((1, 2, 0)), _) => fus_v2.map(Some).ok_or(Error::MissingImage("fus_v2")),
            // FUS V2: install the wireless stack unless it is already up to date.
            (Some(_), Some(stack)) if stack >= stack_version => Ok(None),
            (Some(_), _) => stack_fw.map(Some).ok_or(Error::MissingImage("stack_fw")),
            // No FUS version yet (first boot of a virgin chip): let `boot` initialize FUS.
            (None, _) => Ok(None),
        };

        let image = match image {
            Ok(image) => image,
            Err(e) => {
                // Never leave a stale pending flag behind: `boot` would otherwise
                // install whatever image happens to be in the download area.
                self.cancel_upgrade();
                return Err(e);
            }
        };

        match image {
            Some(image) => {
                // Runaway guard, survives resets in the register next to the pending flag.
                let attempts = self.rtc.read_backup_register(self.backup_register + 1).unwrap_or(0) + 1;
                self.rtc.write_backup_register(self.backup_register + 1, attempts);
                if attempts > MAX_ATTEMPTS {
                    error!("too many failed upgrade attempts ({})", attempts);
                    return Err(Error::TooManyAttempts);
                }

                stage_image(flash, image);

                self.set_upgrade_status(UpgradeStatus::Pending);

                if ready == SchiSysEventReady::WirelessFwRunning {
                    // Ask the running wireless stack to reboot into FUS.
                    self.start_upgrade(sys).await.map_err(|_| Error::Command)?;
                }
            }
            None => {
                self.rtc.write_backup_register(self.backup_register + 1, 0);
                self.set_upgrade_status(UpgradeStatus::Complete);
            }
        }

        // FUS is (or will be after the reset) running: request the upgrade and
        // track it until the new wireless stack runs.
        self.boot(ready, sys).await
    }

    /// Start the upgrade of firmware; must be called while the wireless stack is running,
    /// after the firmware image has been written to the flash download area and the
    /// upgrade has been marked pending ([`Self::request_upgrade`] does all of this).
    ///
    /// Requests the device to reboot into FUS by sending two `FUS_GET_STATE` commands,
    /// then waits for the FUS-initiated system reset. On the next boot, [`Self::boot`]
    /// performs the upgrade. Retries a few times: some FUS versions appear to drop the
    /// request if the two commands do not both arrive while it is listening.
    pub async fn start_upgrade(&mut self, sys: &mut Sys<'_>) -> Result<(), Error> {
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
    /// Returns `Ok(())` once the wireless stack is running (upgrade completed, or nothing
    /// to do). Resets the system as required by the FUS state machine; otherwise loops
    /// forever, so it never returns `Err` on transient states.
    pub async fn boot(&mut self, ready_event: SchiSysEventReady, sys: &mut Sys<'_>) -> Result<(), Error> {
        let firmware_started = ready_event == SchiSysEventReady::WirelessFwRunning
            && (sys
                .wireless_fw_info()
                .is_some_and(|info| info.version_major() + info.version_minor() > 0)
                // The table keeps the FUS layout until the stack updates it, so also
                // accept the FUS table: it reports the installed stack version.
                || sys.fus_info().is_some_and(|info| info.wireless_stack_version != 0));

        // If wireless firmware is started, then abort the upgrade and return
        if firmware_started {
            self.set_upgrade_status(UpgradeStatus::Complete);
            self.rtc.write_backup_register(self.backup_register + 1, 0);

            return Ok(());
        }

        let upgrade_pending = self.get_upgrade_status() == UpgradeStatus::Pending;
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
                        self.set_upgrade_status(UpgradeStatus::Complete);

                        info!("requesting FUS firmware upgrade");
                        sys.shci_c2_fus_fwupgrade(0, 0).await.map_err(|_| Error::Command)?;
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
}
