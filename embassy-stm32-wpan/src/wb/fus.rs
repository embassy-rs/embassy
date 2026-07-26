use cortex_m::asm::wfi;
use cortex_m::peripheral::SCB;
use embassy_stm32::rtc::AnyRtc;

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

    /// Start the upgrade of firmware; must be called after boot
    pub async fn start_upgrade(&mut self, sys: &mut Sys<'_>) -> Result<(), ()> {
        self.set_upgrade_status(UpgradeStatus::Pending);

        sys.shci_c2_fus_getstate().await?;
        sys.shci_c2_fus_getstate().await?;

        // wait for FUS to reboot us
        loop {
            wfi();
        }
    }

    /// Called on boot to drive the OTA process, or exit the FUS
    pub async fn boot(&mut self, ready_event: SchiSysEventReady, sys: &mut Sys<'_>) -> Result<(), ()> {
        let firmware_started = ready_event == SchiSysEventReady::WirelessFwRunning
            && sys
                .wireless_fw_info()
                .is_some_and(|info| info.version_major() + info.version_minor() > 0);

        let upgrade_status = self.get_upgrade_status();

        // If wireless firmware is started, then abort the upgrade and return
        if firmware_started {
            self.set_upgrade_status(UpgradeStatus::Complete);

            return Ok(());
        }

        // If we cannot get the FUS state, then return with an error
        match sys.shci_c2_fus_getstate().await? {
            ShciFusGetStateErrorCode::FusStateErrorErrUnknown => {
                // This is the first time in the life of the product the FUS is involved.
                // After this command, it will be properly initialized
                // Request the device to reboot to install the wireless firmware

                SCB::sys_reset();
            }
            ShciFusGetStateErrorCode::FusStateErrorNoError if upgrade_status == UpgradeStatus::Complete => {
                // FUS is idle and upgrade is complete, start the wireless stack
                sys.shci_c2_fus_startws().await?;
            }
            ShciFusGetStateErrorCode::FusStateErrorNoError if upgrade_status == UpgradeStatus::Pending => {
                // FUS is idle and upgrade is pending, start the upgrade
                self.set_upgrade_status(UpgradeStatus::Complete);

                sys.shci_c2_fus_fwupgrade(0, 0).await?;
            }
            _ => {}
        }

        // Wait for the FUS to reboot us
        loop {
            wfi();
        }
    }
}
