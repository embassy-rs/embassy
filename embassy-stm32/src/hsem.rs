//! Hardware Semaphore (HSEM)
const HSEM: crate::pac::hsem::Hsem = crate::pac::HSEM;

/// Locking the HardwareSemaphore failed.
#[derive(Debug)]
pub struct HsemLockFailed;

/// CPU Core.
///
/// The enum values are identical to the bus master IDs defined for each chip family.
///
/// On some chips, the Reference Manual calls this value MASTERID instead of COREID.
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoreId {
    #[cfg(any(stm32h7a3, stm32h7b3, stm32h7b0))]
    /// Cortex-M7, core 1. MASTERID = 1
    Core0 = 1,

    #[cfg(any(
        stm32h723, stm32h725, stm32h730, stm32h733, stm32h735, stm32h742, stm32h743, stm32h745, stm32h747, stm32h750,
        stm32h753, stm32h755, stm32h757,
    ))]
    /// Cortex-M7, core 1. MASTERID = 3
    Core0 = 3,

    #[cfg(any(stm32wb, stm32wl))]
    /// Cortex-M4, core 1
    Core0 = 4,

    #[cfg(not(any(stm32wb, stm32wl, stm32h7a3, stm32h7b3, stm32h7b0)))]
    /// Cortex-M4, core 2
    Core1 = 1,

    #[cfg(any(stm32wb, stm32wl))]
    /// Cortex M0+, core 2
    Core1 = 8,
}

impl CoreId {
    #[cfg(any(
        all(stm32wl, not(feature = "_core-cm0p")),
        all(not(stm32wl), any(feature = "_core-cm7", not(feature = "_core-cm4"))),
    ))]
    const fn current() -> CoreId {
        CoreId::Core0
    }

    #[cfg(any(all(not(stm32wl), feature = "_core-cm4"), all(stm32wl, feature = "_core-cm0p")))]
    const fn current() -> CoreId {
        CoreId::Core1
    }

    fn as_index(&self) -> usize {
        match self {
            CoreId::Core0 => 0,
            #[cfg(not(any(stm32h7a3, stm32h7b3, stm32h7b0)))]
            CoreId::Core1 => 1,
        }
    }
}

/// TODO
pub struct HardwareSemaphore<'d> {
    _peri: crate::Peri<'d, crate::peripherals::HSEM>,
}

impl<'d> HardwareSemaphore<'d> {
    /// Create a new HardwareSemaphore instance.
    pub fn new(peri: crate::Peri<'d, crate::peripherals::HSEM>) -> Self {
        HardwareSemaphore { _peri: peri }
    }

    /// Locks the semaphore via the 2-step (write) lock procedure
    ///
    /// The two-step procedure consists of a write to lock the semaphore, followed by a read
    /// to check if the lock has been successful, carried out from the HSEM_Rx retgister.
    pub fn two_step_lock(&mut self, sem_id: u8, process_id: u8) -> Result<(), HsemLockFailed> {
        HSEM.r(sem_id as usize).write(|w| {
            w.set_procid(process_id);
            w.set_coreid(CoreId::current() as u8);
            w.set_lock(true);
        });

        let reg = HSEM.r(sem_id as usize).read();

        match (
            reg.lock(),
            reg.coreid() == CoreId::current() as u8,
            reg.procid() == process_id,
        ) {
            (true, true, true) => Ok(()),
            v => {
                error!("{}: {}", v, CoreId::current() as u8);
                Err(HsemLockFailed)
            }
        }
    }

    /// Locks the semaphore via the 1-step (read) lock procedure
    ///
    /// The one-step procedure consists of a read to lock and check the semaphore in a single
    /// step, carried out from the HSEM_RLRx register.
    pub fn one_step_lock(&mut self, sem_id: u8) -> Result<(), HsemLockFailed> {
        let reg = HSEM.rlr(sem_id as usize).read();

        match (reg.lock(), reg.coreid() == CoreId::current() as u8, reg.procid()) {
            (_, true, 0) => Ok(()),
            v => {
                error!("{}: Current: {} Actual: {}", v, CoreId::current() as u8, reg.coreid());
                Err(HsemLockFailed)
            }
        }
    }

    /// Unlocks the semaphore
    ///
    /// Unlocking a semaphore is a protected process, to prevent accidental clearing by an AHB
    /// bus core ID or by a process not having the semaphore lock right.
    pub fn unlock(&mut self, sem_id: u8, process_id: u8) {
        HSEM.r(sem_id as usize).write(|w| {
            w.set_procid(process_id);
            w.set_coreid(CoreId::current() as u8);
            w.set_lock(false);
        });
    }

    /// Unlocks all semahpores.
    ///
    /// All semaphores locked by a COREID can be unlocked at once by using the HSEM_CR register.
    /// Write COREID and correct KEY value in HSEM_CR. All locked semaphores with a matching
    /// COREID are unlocked, and may generate an interrupt if enabled.
    pub fn unlock_all(&mut self, key: u16, core_id: u8) {
        HSEM.cr().write(|w| {
            w.set_key(key);
            w.set_coreid(core_id);
        })
    }

    /// Checks if the semaphore is locked.
    pub fn is_semaphore_locked(&self, sem_id: u8) -> bool {
        HSEM.r(sem_id as usize).read().lock()
    }

    /// Sets the clear (unlock) key
    pub fn set_clear_key(&mut self, key: u16) {
        HSEM.keyr().modify(|w| w.set_key(key));
    }

    /// Gets the clear (unlock) key
    pub fn get_clear_key(&mut self) -> u16 {
        HSEM.keyr().read().key()
    }

    /// Sets the interrupt enable bit for the semaphore.
    pub fn enable_interrupt(&mut self, core_id: CoreId, sem_x: usize, enable: bool) {
        HSEM.ier(core_id.as_index()).modify(|w| w.set_ise(sem_x, enable));
    }

    /// Clears the interrupt flag for the semaphore.
    pub fn clear_interrupt(&mut self, core_id: CoreId, sem_x: usize) {
        HSEM.icr(core_id.as_index()).write(|w| w.set_isc(sem_x, false));
    }

    /// Gets the interrupt flag for the semaphore.
    pub fn is_interrupt_active(&mut self, core_id: CoreId, sem_x: usize) -> bool {
        HSEM.isr(core_id.as_index()).read().isf(sem_x)
    }
}
