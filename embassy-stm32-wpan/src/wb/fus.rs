use cortex_m::asm::wfi;
use cortex_m::peripheral::SCB;
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

/// Administers FUS upgrades
///
/// The upgrade flow follows AN5185 and ST's FUS_CLI example:
///
/// 1. Write the signed firmware image (wireless stack or FUS) into the flash download
///    area, below the secure flash boundary (see the `Release_Notes.html` of the
///    STM32WB coprocessor binaries for the address computation).
/// 2. If the wireless stack is running, call [`Self::start_upgrade`] to request a reboot
///    into FUS (the pending status survives the reset in the given RTC backup register).
/// 3. Call [`Self::boot`] on every boot after the CPU2 ready event. It requests the
///    upgrade from FUS, tracks it with `FUS_GET_STATE` polling, and resets the system
///    when done. Once the new wireless stack is running it returns `Ok(())`.
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

    /// Mark an upgrade as pending; call before writing the firmware image to flash, so
    /// that [`Self::boot`] requests the upgrade from FUS after the next reset.
    pub fn request_upgrade(&mut self) {
        self.set_upgrade_status(UpgradeStatus::Pending);
    }

    /// Cancel a pending upgrade request.
    ///
    /// Call this when there is nothing to install, so that a stale pending flag from an
    /// earlier aborted attempt is not acted upon (it would make the FUS scan the download
    /// area and install whatever image happens to be there).
    pub fn cancel_upgrade(&mut self) {
        self.set_upgrade_status(UpgradeStatus::Complete);
    }

    /// Start the upgrade of firmware; must be called while the wireless stack is running,
    /// after the firmware image has been written to the flash download area and
    /// [`Self::request_upgrade`] has been called.
    ///
    /// Requests the device to reboot into FUS by sending two `FUS_GET_STATE` commands,
    /// then waits for the FUS-initiated system reset. On the next boot, [`Self::boot`]
    /// performs the upgrade. Retries a few times: some FUS versions appear to drop the
    /// request if the two commands do not both arrive while it is listening.
    pub async fn start_upgrade(&mut self, sys: &mut Sys<'_>) -> Result<(), ()> {
        for attempt in 1..=3 {
            info!("requesting reboot into FUS (attempt {})", attempt);

            sys.shci_c2_fus_get_state().await?;
            sys.shci_c2_fus_get_state().await?;

            // Wait for the FUS to reboot us (up to 5s, in 100ms slices so we also log)
            for _ in 0..50 {
                Timer::after(Duration::from_millis(100)).await;
            }
        }

        error!("FUS did not reboot into FUS mode");
        Err(())
    }

    /// Called on boot to drive the FUS upgrade process, or to start the wireless stack.
    ///
    /// Returns `Ok(())` once the wireless stack is running (upgrade completed, or nothing
    /// to do). Resets the system as required by the FUS state machine; otherwise loops
    /// forever, so it never returns `Err` on transient states.
    pub async fn boot(&mut self, ready_event: SchiSysEventReady, sys: &mut Sys<'_>) -> Result<(), ()> {
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

            return Ok(());
        }

        let upgrade_pending = self.get_upgrade_status() == UpgradeStatus::Pending;
        // FUS keeps reporting the last operation's error until a new operation is
        // requested; a failed request is retried a bounded number of times so a
        // genuinely bad image still fails (the caller's runaway guard then applies).
        let mut fwupgrade_retries = 0;

        loop {
            let state = sys.shci_c2_fus_get_state().await?;

            match state.state() {
                // FUS_STATE_VALUE_IDLE
                0x00 => {
                    if upgrade_pending {
                        // The pending flag is cleared before requesting the upgrade: if the
                        // device reboots before the command is processed, we do not retry on
                        // the next boot with a (possibly erased) download area.
                        self.set_upgrade_status(UpgradeStatus::Complete);

                        info!("requesting FUS firmware upgrade");
                        sys.shci_c2_fus_fwupgrade(0, 0).await?;
                    } else {
                        // FUS is idle and the upgrade is complete: start the wireless stack.
                        sys.shci_c2_fus_startws().await?;

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
                            return Err(());
                        }

                        warn!("FUS reported error: {:?}; retrying fwupgrade", error_code);
                        sys.shci_c2_fus_fwupgrade(0, 0).await?;
                    }
                },
                state => {
                    error!("unexpected FUS state 0x{:02x}", state);

                    return Err(());
                }
            }
        }
    }
}
