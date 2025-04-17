//! Hardware Semaphore (HSEM)

use embassy_hal_internal::PeripheralType;

use crate::pac;
use crate::rcc::RccPeripheral;
// TODO: This code works for all HSEM implemenations except for the STM32WBA52/4/5xx MCUs.
// Those MCUs have a different HSEM implementation (Secure semaphore lock support,
// Privileged / unprivileged semaphore lock support, Semaphore lock protection via semaphore attribute),
// which is not yet supported by this code.
use crate::Peri;

/// HSEM error.
#[derive(Debug)]
pub enum HsemError {
    /// Locking the semaphore failed.
    LockFailed,
}

/// CPU core.
/// The enum values are identical to the bus master IDs / core Ids defined for each
/// chip family (i.e. stm32h747 see rm0399 table 95)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoreId {
    #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757))]
    /// Cortex-M7, core 1.
    Core0 = 0x3,

    #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757))]
    /// Cortex-M4, core 2.
    Core1 = 0x1,

    #[cfg(not(any(stm32h745, stm32h747, stm32h755, stm32h757)))]
    /// Cortex-M4, core 1
    Core0 = 0x4,

    #[cfg(any(stm32wb, stm32wl))]
    /// Cortex-M0+, core 2.
    Core1 = 0x8,
}

/// Get the current core id
/// This code assume that it is only executed on a Cortex-M M0+, M4 or M7 core.
#[inline(always)]
pub fn get_current_coreid() -> CoreId {
    let cpuid = unsafe { cortex_m::peripheral::CPUID::PTR.read_volatile().base.read() };
    match cpuid & 0x000000F0 {
        #[cfg(any(stm32wb, stm32wl))]
        0x0 => CoreId::Core1,

        #[cfg(not(any(stm32h745, stm32h747, stm32h755, stm32h757)))]
        0x4 => CoreId::Core0,

        #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757))]
        0x4 => CoreId::Core1,

        #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757))]
        0x7 => CoreId::Core0,
        _ => panic!("Unknown Cortex-M core"),
    }
}

/// Translates the core ID to an index into the interrupt registers.
#[inline(always)]
fn core_id_to_index(core: CoreId) -> usize {
    match core {
        CoreId::Core0 => 0,
        #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757, stm32wb, stm32wl))]
        CoreId::Core1 => 1,
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
    pub fn two_step_lock(&mut self, sem_id: u8, process_id: u8) -> Result<(), HsemError> {
        T::regs().r(sem_id as usize).write(|w| {
            w.set_procid(process_id);
            w.set_coreid(get_current_coreid() as u8);
            w.set_lock(true);
        });
        let reg = T::regs().r(sem_id as usize).read();
        match (
            reg.lock(),
            reg.coreid() == get_current_coreid() as u8,
            reg.procid() == process_id,
        ) {
            (true, true, true) => Ok(()),
            _ => Err(HsemError::LockFailed),
        }
    }

    /// Locks the semaphore.
    /// The 1-step procedure consists in a read to lock and check the semaphore in a single step,
    /// carried out from the HSEM_RLRx register.
    pub fn one_step_lock(&mut self, sem_id: u8) -> Result<(), HsemError> {
        let reg = T::regs().rlr(sem_id as usize).read();
        match (reg.lock(), reg.coreid() == get_current_coreid() as u8, reg.procid()) {
            (false, true, 0) => Ok(()),
            _ => Err(HsemError::LockFailed),
        }
    }

    /// Unlocks the semaphore.
    /// Unlocking a semaphore is a protected process, to prevent accidental clearing by a AHB bus
    /// core ID or by a process not having the semaphore lock right.
    pub fn unlock(&mut self, sem_id: u8, process_id: u8) {
        T::regs().r(sem_id as usize).write(|w| {
            w.set_procid(process_id);
            w.set_coreid(get_current_coreid() as u8);
            w.set_lock(false);
        });
    }

    /// Unlocks all semaphores.
    /// All semaphores locked by a COREID can be unlocked at once by using the HSEM_CR
    /// register. Write COREID and correct KEY value in HSEM_CR. All locked semaphores with a
    /// matching COREID are unlocked, and may generate an interrupt when enabled.
    pub fn unlock_all(&mut self, key: u16, core_id: u8) {
        T::regs().cr().write(|w| {
            w.set_key(key);
            w.set_coreid(core_id);
        });
    }

    /// Checks if the semaphore is locked.
    pub fn is_semaphore_locked(&self, sem_id: u8) -> bool {
        T::regs().r(sem_id as usize).read().lock()
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
    pub fn enable_interrupt(&mut self, core_id: CoreId, sem_x: usize, enable: bool) {
        T::regs()
            .ier(core_id_to_index(core_id))
            .modify(|w| w.set_ise(sem_x, enable));
    }

    /// Gets the interrupt flag for the semaphore.
    pub fn is_interrupt_active(&mut self, core_id: CoreId, sem_x: usize) -> bool {
        T::regs().isr(core_id_to_index(core_id)).read().isf(sem_x)
    }

    /// Clears the interrupt flag for the semaphore.
    pub fn clear_interrupt(&mut self, core_id: CoreId, sem_x: usize) {
        T::regs()
            .icr(core_id_to_index(core_id))
            .write(|w| w.set_isc(sem_x, false));
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
