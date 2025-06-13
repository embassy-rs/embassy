//! Hardware Semaphore (HSEM)

use embassy_hal_internal::PeripheralType;

use crate::pac;
use crate::rcc::RccPeripheral;
// TODO: This code works for all HSEM implementations except for the STM32WBA52/4/5xx MCUs.
// Those MCUs have a different HSEM implementation (Secure semaphore lock support,
// Privileged / unprivileged semaphore lock support, Semaphore lock protection via semaphore attribute),
// which is not yet supported by this code.
use crate::Peri;

/// HSEM error.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HsemError {
    /// Locking the semaphore failed.
    LockFailed,
}

/// HSEM identifier
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SemId {
    /// Semaphore 0
    Id0 = 0,
    /// Semaphore 1
    Id1 = 1,
    /// Semaphore 2
    Id2 = 2,
    /// Semaphore 3
    Id3 = 3,
    /// Semaphore 4
    Id4 = 4,
    /// Semaphore 5
    Id5 = 5,
    /// Semaphore 6
    Id6 = 6,
    /// Semaphore 7
    Id7 = 7,
    /// Semaphore 8
    Id8 = 8,
    /// Semaphore 9
    Id9 = 9,
    /// Semaphore 10
    Id10 = 10,
    /// Semaphore 11
    Id11 = 11,
    /// Semaphore 12
    Id12 = 12,
    /// Semaphore 13
    Id13 = 13,
    /// Semaphore 14
    Id14 = 14,
    /// Semaphore 15
    Id15 = 15,
    /// Semaphore 16
    Id16 = 16,
    /// Semaphore 17
    Id17 = 17,
    /// Semaphore 18
    Id18 = 18,
    /// Semaphore 19
    Id19 = 19,
    /// Semaphore 20
    Id20 = 20,
    /// Semaphore 21
    Id21 = 21,
    /// Semaphore 22
    Id22 = 22,
    /// Semaphore 23
    Id23 = 23,
    /// Semaphore 24
    Id24 = 24,
    /// Semaphore 25
    Id25 = 25,
    /// Semaphore 26
    Id26 = 26,
    /// Semaphore 27
    Id27 = 27,
    /// Semaphore 28
    Id28 = 28,
    /// Semaphore 29
    Id29 = 29,
    /// Semaphore 30
    Id30 = 30,
    /// Semaphore 31
    Id31 = 31,
}

impl From<SemId> for usize {
    fn from(id: SemId) -> Self {
        id as usize
    }
}

/// CPU core.
/// The enum values are identical to the bus master IDs / core Ids defined for each
/// chip family (i.e. stm32h747 see rm0399 table 95)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoreId {
    /// Main processor
    Core0 = 0,
    /// Coprocessor
    Core1,
}

impl From<CoreId> for usize {
    fn from(core: CoreId) -> Self {
        core as usize
    }
}

/// The core ID used in the HSEM_RLRx register.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RlrCoreId {
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

impl From<CoreId> for RlrCoreId {
    #[rustfmt::skip]
    fn from(core: CoreId) -> Self {
        match core {
            #[cfg(any(
                any(
                    stm32h723, stm32h725, stm32h730, stm32h733, stm32h735, stm32h742, stm32h743, stm32h745, stm32h747, stm32h750,
                    stm32h753, stm32h755, stm32h757,
                ),
                any(stm32wb, stm32wl))
            )]
            CoreId::Core0 => RlrCoreId::Core0,
            #[cfg(any(
                not(any(stm32wb, stm32wl, stm32h7a3, stm32h7b3, stm32h7b0)),
                any(stm32wb, stm32wl))
            )]
            CoreId::Core1 => RlrCoreId::Core1,
        }
    }
}

impl CoreId {
    /// Returns the ID of the current running core.
    pub fn get_current() -> Self {
        #[cfg(any(
            all(stm32wl, not(feature = "_core-cm0p")),
            all(not(stm32wl), any(feature = "_core-cm7", not(feature = "_core-cm4"))),
        ))]
        return CoreId::Core0;

        #[cfg(any(all(not(stm32wl), feature = "_core-cm4"), all(stm32wl, feature = "_core-cm0p")))]
        return CoreId::Core1;
    }
}

/// HSEM driver
pub struct HardwareSemaphore<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> HardwareSemaphore<'d, T> {
    /// Creates a new HardwareSemaphore instance.
    pub fn new(peripheral: Peri<'d, T>) -> Self {
        HardwareSemaphore { _peri: peripheral }
    }

    /// Locks the semaphore.
    /// The 2-step lock procedure consists in a write to lock the semaphore, followed by a read to
    /// check if the lock has been successful, carried out from the HSEM_Rx register.
    pub fn two_step_lock(&mut self, sem_id: SemId, process_id: u8) -> Result<(), HsemError> {
        T::regs().r(sem_id.into()).write(|w| {
            w.set_procid(process_id);
            w.set_coreid(RlrCoreId::from(CoreId::get_current()) as u8);
            w.set_lock(true);
        });
        let reg = T::regs().r(sem_id.into()).read();
        match (
            reg.lock(),
            reg.coreid() == RlrCoreId::from(CoreId::get_current()) as u8,
            reg.procid() == process_id,
        ) {
            (true, true, true) => Ok(()),
            _ => Err(HsemError::LockFailed),
        }
    }

    /// Locks the semaphore.
    /// The 1-step procedure consists in a read to lock and check the semaphore in a single step,
    /// carried out from the HSEM_RLRx register.
    pub fn one_step_lock(&mut self, sem_id: SemId) -> Result<(), HsemError> {
        let reg = T::regs().rlr(sem_id.into()).read();

        match (
            reg.lock(),
            reg.coreid() == RlrCoreId::from(CoreId::get_current()) as u8,
            reg.procid(),
        ) {
            (true, true, 0) => Ok(()),
            _ => Err(HsemError::LockFailed),
        }
    }

    /// Unlocks the semaphore.
    /// Unlocking a semaphore is a protected process, to prevent accidental clearing by a AHB bus
    /// core ID or by a process not having the semaphore lock right.
    pub fn unlock(&mut self, sem_id: SemId, process_id: Option<u8>) {
        let process_id = process_id.unwrap_or(0);

        T::regs().r(sem_id.into()).write(|w| {
            w.set_procid(process_id);
            w.set_coreid(RlrCoreId::from(CoreId::get_current()) as u8);
            w.set_lock(false);
        });
    }

    /// Unlocks all semaphores.
    /// All semaphores locked by a COREID can be unlocked at once by using the HSEM_CR
    /// register. Write COREID and correct KEY value in HSEM_CR. All locked semaphores with a
    /// matching COREID are unlocked, and may generate an interrupt when enabled.
    pub fn unlock_all(&mut self, key: u16, core_id: CoreId) {
        T::regs().cr().write(|w| {
            w.set_key(key);
            w.set_coreid(RlrCoreId::from(core_id) as u8);
        });
    }

    /// Checks if the semaphore is locked.
    pub fn is_semaphore_locked(&self, sem_id: SemId) -> bool {
        T::regs().r(sem_id.into()).read().lock()
    }

    /// Sets the clear (unlock) key
    pub fn set_clear_key(&mut self, key: u16) {
        T::regs().keyr().modify(|w| w.set_key(key));
    }

    /// Gets the clear (unlock) key
    pub fn get_clear_key(&mut self) -> u16 {
        T::regs().keyr().read().key()
    }

    /// Sets the interrupt enable bit for the semaphore.
    pub fn enable_interrupt(&mut self, core_id: CoreId, sem_id: SemId, enable: bool) {
        T::regs()
            .ier(core_id.into())
            .modify(|w| w.set_ise(sem_id.into(), enable));
    }

    /// Gets the interrupt flag for the semaphore.
    pub fn is_interrupt_active(&mut self, core_id: CoreId, sem_id: SemId) -> bool {
        T::regs().isr(core_id.into()).read().isf(sem_id.into())
    }

    /// Clears the interrupt flag for the semaphore.
    pub fn clear_interrupt(&mut self, core_id: CoreId, sem_id: SemId) {
        T::regs().icr(core_id.into()).write(|w| w.set_isc(sem_id.into(), false));
    }
}

trait SealedInstance {
    fn regs() -> pac::hsem::Hsem;
}

/// HSEM instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + Send + 'static {}

impl SealedInstance for crate::peripherals::HSEM {
    fn regs() -> crate::pac::hsem::Hsem {
        crate::pac::HSEM
    }
}
impl Instance for crate::peripherals::HSEM {}
