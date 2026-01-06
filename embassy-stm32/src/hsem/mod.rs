//! Hardware Semaphore (HSEM)

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

#[cfg(all(stm32wb, feature = "low-power"))]
use critical_section::CriticalSection;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

// TODO: This code works for all HSEM implemenations except for the STM32WBA52/4/5xx MCUs.
// Those MCUs have a different HSEM implementation (Secure semaphore lock support,
// Privileged / unprivileged semaphore lock support, Semaphore lock protection via semaphore attribute),
// which is not yet supported by this code.
use crate::Peri;
use crate::rcc::{self, RccPeripheral};
use crate::{interrupt, pac};

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

impl CoreId {
    /// Get the current core id
    /// This code assume that it is only executed on a Cortex-M M0+, M4 or M7 core.
    pub fn current() -> Self {
        let cpuid = unsafe { cortex_m::peripheral::CPUID::PTR.read_volatile().base.read() };
        match (cpuid & 0x000000F0) >> 4 {
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
    pub fn to_index(&self) -> usize {
        match &self {
            CoreId::Core0 => 0,
            #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757, stm32wb, stm32wl))]
            CoreId::Core1 => 1,
        }
    }
}

#[cfg(all(not(all(stm32wb, feature = "low-power")), not(stm32wl5x)))]
const PUB_CHANNELS: usize = 6;

#[cfg(stm32wl5x)]
const PUB_CHANNELS: usize = 5;

#[cfg(all(stm32wb, feature = "low-power"))]
const PUB_CHANNELS: usize = 4;

/// TX interrupt handler.
pub struct HardwareSemaphoreInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for HardwareSemaphoreInterruptHandler<T> {
    unsafe fn on_interrupt() {
        let core_id = CoreId::current();
        let isr = T::regs().isr(core_id.to_index()).read();

        for number in 0..5 {
            if isr.isf(number as usize) {
                T::regs()
                    .icr(core_id.to_index())
                    .write(|w| w.set_isc(number as usize, true));

                T::state().waker_for(number).wake();
            }
        }
    }
}

/// Hardware semaphore mutex
pub struct HardwareSemaphoreMutex<'a, T: Instance> {
    index: u8,
    process_id: u8,
    _lifetime: PhantomData<&'a mut T>,
}

impl<'a, T: Instance> Drop for HardwareSemaphoreMutex<'a, T> {
    fn drop(&mut self) {
        let core_id = CoreId::current();

        T::regs()
            .icr(core_id.to_index())
            .write(|w| w.set_isc(self.index as usize, true));

        critical_section::with(|_| {
            T::regs()
                .ier(core_id.to_index())
                .modify(|w| w.set_ise(self.index as usize, false));
        });

        HardwareSemaphoreChannel::<'a, T> {
            index: self.index,
            _lifetime: PhantomData,
        }
        .unlock(self.process_id);
    }
}

/// Hardware semaphore channel
pub struct HardwareSemaphoreChannel<'a, T: Instance> {
    index: u8,
    _lifetime: PhantomData<&'a mut T>,
}

impl<'a, T: Instance> HardwareSemaphoreChannel<'a, T> {
    pub(crate) const fn new(number: u8) -> Self {
        core::assert!(number > 0 && number <= 6);

        Self {
            index: number - 1,
            _lifetime: PhantomData,
        }
    }

    /// Locks the semaphore.
    /// The 2-step lock procedure consists in a write to lock the semaphore, followed by a read to
    /// check if the lock has been successful, carried out from the HSEM_Rx register.
    pub async fn lock(&mut self, process_id: u8) -> HardwareSemaphoreMutex<'a, T> {
        let _scoped_block_stop = T::RCC_INFO.block_stop();
        let core_id = CoreId::current();

        poll_fn(|cx| {
            T::state().waker_for(self.index).register(cx.waker());

            compiler_fence(Ordering::SeqCst);

            critical_section::with(|_| {
                T::regs()
                    .ier(core_id.to_index())
                    .modify(|w| w.set_ise(self.index as usize, true));
            });

            match self.try_lock(process_id) {
                Some(mutex) => Poll::Ready(mutex),
                None => Poll::Pending,
            }
        })
        .await
    }

    /// Try to lock the semaphor
    /// The 2-step lock procedure consists in a write to lock the semaphore, followed by a read to
    /// check if the lock has been successful, carried out from the HSEM_Rx register.
    pub fn try_lock(&mut self, process_id: u8) -> Option<HardwareSemaphoreMutex<'a, T>> {
        if self.two_step_lock(process_id).is_ok() {
            Some(HardwareSemaphoreMutex {
                index: self.index,
                process_id: process_id,
                _lifetime: PhantomData,
            })
        } else {
            None
        }
    }

    /// Locks the semaphore.
    /// The 2-step lock procedure consists in a write to lock the semaphore, followed by a read to
    /// check if the lock has been successful, carried out from the HSEM_Rx register.
    pub fn two_step_lock(&mut self, process_id: u8) -> Result<(), HsemError> {
        T::regs().r(self.index as usize).write(|w| {
            w.set_procid(process_id);
            w.set_coreid(CoreId::current() as u8);
            w.set_lock(true);
        });
        let reg = T::regs().r(self.index as usize).read();
        match (
            reg.lock(),
            reg.coreid() == CoreId::current() as u8,
            reg.procid() == process_id,
        ) {
            (true, true, true) => Ok(()),
            _ => Err(HsemError::LockFailed),
        }
    }

    /// Locks the semaphore.
    /// The 1-step procedure consists in a read to lock and check the semaphore in a single step,
    /// carried out from the HSEM_RLRx register.
    pub fn one_step_lock(&mut self) -> Result<(), HsemError> {
        let reg = T::regs().rlr(self.index as usize).read();
        match (reg.lock(), reg.coreid() == CoreId::current() as u8, reg.procid()) {
            (false, true, 0) => Ok(()),
            _ => Err(HsemError::LockFailed),
        }
    }

    /// Unlocks the semaphore.
    /// Unlocking a semaphore is a protected process, to prevent accidental clearing by a AHB bus
    /// core ID or by a process not having the semaphore lock right.
    pub fn unlock(&mut self, process_id: u8) {
        T::regs().r(self.index as usize).write(|w| {
            w.set_procid(process_id);
            w.set_coreid(CoreId::current() as u8);
            w.set_lock(false);
        });
    }

    /// Return the channel number
    pub const fn channel(&self) -> u8 {
        self.index + 1
    }
}

/// HSEM driver
pub struct HardwareSemaphore<T: Instance> {
    _type: PhantomData<T>,
}

impl<T: Instance> HardwareSemaphore<T> {
    /// Creates a new HardwareSemaphore instance.
    pub fn new<'d>(
        _peripheral: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, HardwareSemaphoreInterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset_without_stop::<T>();

        HardwareSemaphore { _type: PhantomData }
    }

    /// Get a single channel, and keep the global struct
    pub const fn channel_for<'a>(&'a mut self, number: u8) -> HardwareSemaphoreChannel<'a, T> {
        #[cfg(all(stm32wb, feature = "low-power"))]
        core::assert!(number != 3 && number != 4);

        HardwareSemaphoreChannel::new(number)
    }

    /// Split the global struct into channels
    ///
    /// If using low-power mode, channels 3 and 4 will not be returned
    pub const fn split<'a>(self) -> [HardwareSemaphoreChannel<'a, T>; PUB_CHANNELS] {
        [
            HardwareSemaphoreChannel::new(1),
            HardwareSemaphoreChannel::new(2),
            #[cfg(all(not(all(stm32wb, feature = "low-power")), not(stm32wl5x)))]
            HardwareSemaphoreChannel::new(3),
            #[cfg(not(all(stm32wb, feature = "low-power")))]
            HardwareSemaphoreChannel::new(4),
            HardwareSemaphoreChannel::new(5),
            HardwareSemaphoreChannel::new(6),
        ]
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

    /// Sets the clear (unlock) key
    pub fn set_clear_key(&mut self, key: u16) {
        T::regs().keyr().modify(|w| w.set_key(key));
    }

    /// Gets the clear (unlock) key
    pub fn get_clear_key(&mut self) -> u16 {
        T::regs().keyr().read().key()
    }
}

#[cfg(all(stm32wb, feature = "low-power"))]
pub(crate) fn init_hsem(cs: CriticalSection) {
    rcc::enable_and_reset_with_cs::<crate::peripherals::HSEM>(cs);

    unsafe {
        crate::rcc::REFCOUNT_STOP1 = 0;
        crate::rcc::REFCOUNT_STOP2 = 0;
    }
}

struct State {
    wakers: [AtomicWaker; 6],
}

impl State {
    const fn new() -> Self {
        Self {
            wakers: [const { AtomicWaker::new() }; 6],
        }
    }

    const fn waker_for(&self, index: u8) -> &AtomicWaker {
        &self.wakers[index as usize]
    }
}

trait SealedInstance {
    fn regs() -> pac::hsem::Hsem;
    fn state() -> &'static State;
}

/// HSEM instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + Send + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl SealedInstance for crate::peripherals::HSEM {
    fn regs() -> crate::pac::hsem::Hsem {
        crate::pac::HSEM
    }

    fn state() -> &'static State {
        static STATE: State = State::new();
        &STATE
    }
}

foreach_interrupt!(
    ($inst:ident, hsem, $block:ident, $signal_name:ident, $irq:ident) => {
        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);
