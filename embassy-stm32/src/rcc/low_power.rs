use crate::hsem::{self, HardwareSemaphore};

/// It is required to switch to the HSI clock before entering stop mode
/// See an5289-how-to-build-wireless-applications-with-stm32wb-mcus-stmicroelectronics p. 23-24
#[cfg(all(feature = "low-power", stm32wb))]
pub(crate) fn stm32wb_configure_clocks_enter_stop_mode<T: hsem::Instance>(hsem: &mut HardwareSemaphore<T>) {
    use crate::pac::PWR;
    use crate::pac::{rcc::vals::Sw, RCC};
    use hsem::SemId;
    use stm32_metapac::rcc::vals::Smps;

    critical_section::with(|_cs| {
        // Poll Sem3 until granted
        // TODO: Add some kind of timeout here and panic?
        trace!("Polling Sem3 until granted");
        while let Err(_) = hsem.one_step_lock(SemId::Id3) {}
        trace!("Sem3 granted");

        // Get Sem4
        match hsem.one_step_lock(SemId::Id4) {
            Ok(_) => {
                // Sem4 granted
                if PWR.extscr().read().c2ds() {
                    // C2DS set - unlock Sem4
                    hsem.unlock(SemId::Id4, None);
                }
            }
            Err(_) => {
                // Sem4 not granted
                // Set HSION
                RCC.cr().modify(|w| {
                    w.set_hsion(true);
                });

                // Wait for HSIRDY
                while !RCC.cr().read().hsirdy() {}

                // Set SW to HSI
                RCC.cfgr().modify(|w| {
                    w.set_sw(Sw::HSI);
                });

                // Wait for SWS to report HSI
                while !RCC.cfgr().read().sws().eq(&Sw::HSI) {}

                // Set SMPSSEL to HSI
                RCC.smpscr().modify(|w| {
                    w.set_smpssel(Smps::HSI);
                });
            }
        }

        // Unlock Sem3
        hsem.unlock(SemId::Id3, None);
    });
}

/// __Attention__: Must not be called from within a critical section
#[cfg(all(feature = "low-power", stm32wb))]
pub(crate) fn stm32wb_configure_clocks_exit_stop_mode<T: hsem::Instance>(hsem: &mut HardwareSemaphore<T>) {
    use super::{init, CURRENT_RCC_CONFIG};
    use crate::hsem::SemId;

    // Release Sem4
    hsem.unlock(SemId::Id4, None);

    // Enter the critical section afterwards
    critical_section::with(|_cs| {
        // Poll Sem3 until granted
        // TODO: Add some kind of timeout here and panic?
        while let Err(_) = hsem.one_step_lock(SemId::Id3) {}

        // Restore the RCC config from before entering stop mode
        // Diveating from The flow chart of Figure 7 in AN5289
        // We always reconfigure the clocks as we don't know if HSE is suitable for the application
        let config = unsafe { CURRENT_RCC_CONFIG.assume_init() };
        unsafe { init(config) };

        hsem.unlock(SemId::Id3, None);
    });
}
