//! Hardware Semaphore (HSEM)

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
use core::task::Poll;

use critical_section::CriticalSection;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use interrupt::typelevel::Interrupt;

// TODO: This code works for all HSEM implemenations except for the STM32WBA52/4/5xx MCUs.
// Those MCUs have a different HSEM implementation (Secure semaphore lock support,
// Privileged / unprivileged semaphore lock support, Semaphore lock protection via semaphore attribute),
// which is not yet supported by this code.
use crate::Peri;
use crate::cpu::CoreId;
use crate::peripherals::HSEM;
use crate::rcc::RccPeripheral;
use crate::{interrupt, pac};

/// HSEM error.
#[derive(Debug)]
pub enum HsemError {
    /// Locking the semaphore failed.
    LockFailed,
}

const CHANNELS: usize = 6;

#[cfg(all(not(all(stm32wb, feature = "low-power")), not(all(stm32wl5x, feature = "low-power"))))]
const PUB_CHANNELS: usize = 6;

#[cfg(all(stm32wl5x, feature = "low-power"))]
const PUB_CHANNELS: usize = 5;

#[cfg(all(stm32wb, feature = "low-power"))]
const PUB_CHANNELS: usize = 4;

/// TX interrupt handler.
pub struct HardwareSemaphoreInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for HardwareSemaphoreInterruptHandler<T> {
    unsafe fn on_interrupt() {
        let core = CoreId::current();
        // Get pending semaphore bits from masked ISR for the current core
        let mut pending = T::regs().misr(core.to_index().into()).read().0;

        T::regs().icr(core.to_index().into()).write(|w| {
            while pending != 0 {
                // Index of lowest set bit
                let n = pending.trailing_zeros() as u8;

                // Clear lowest set bit.
                // Safe when pending != 0 enforced by while predicate
                pending &= pending - 1;

                w.set_isc(n.into(), true);
                T::state().flag_for(n).store(true, Ordering::Release);
                T::state().waker_for(n).wake();
            }
        });
    }
}

struct ActiveInterrupt<T: Instance> {
    core: CoreId,
    index: u8,
    _marker: PhantomData<T>,
}

impl<T: Instance> ActiveInterrupt<T> {
    pub fn new(core: CoreId, index: u8) -> Self {
        T::regs()
            .ier(core.to_index().into())
            .modify(|w| w.set_ise(index.into(), true));

        Self {
            core,
            index,
            _marker: PhantomData,
        }
    }
}

impl<T: Instance> Drop for ActiveInterrupt<T> {
    fn drop(&mut self) {
        T::regs()
            .ier(self.core.to_index().into())
            .modify(|w| w.set_ise(self.index.into(), false));
    }
}

/// Hardware semaphore mutex. The semaphore is unlocked when the guard is dropped
pub struct HardwareSemaphoreMutex<'a, T: Instance> {
    index: u8,
    process_id: u8,
    _lifetime: PhantomData<&'a mut T>,
}

impl<'a, T: Instance> Drop for HardwareSemaphoreMutex<'a, T> {
    /// Unlock the semaphore when this guard is dropped
    fn drop(&mut self) {
        HardwareSemaphoreChannel::<'a, T> {
            index: self.index,
            _lifetime: PhantomData,
        }
        .unlock(self.process_id);
    }
}

#[derive(Copy, Clone)]
/// Hardware semaphore channel
pub struct HardwareSemaphoreChannel<'a, T: Instance> {
    index: u8,
    _lifetime: PhantomData<&'a mut T>,
}

impl<'a, T: Instance> HardwareSemaphoreChannel<'a, T> {
    pub(crate) const fn new(number: u8) -> Self {
        core::assert!(number > 0 && number <= CHANNELS as u8);

        Self {
            index: number - 1,
            _lifetime: PhantomData,
        }
    }

    /// Asynchronously lock a hardware semaphore with 1-step lock procedure and interrupts
    /// This lock does not use process ID, and should only be used to synchronize between cores.
    /// Locking a 1 step lock from the same AHB bus master ID will read the semaphore as locked
    pub async fn fast_lock(&mut self) -> HardwareSemaphoreMutex<'a, T> {
        if let Some(lock) = self.try_fast_lock() {
            return lock;
        }

        let _scoped_wake_guard = T::RCC_INFO.wake_guard();
        let core = CoreId::current();

        let _irq = self.clear_and_enable_interupt(core);
        poll_fn(|cx| {
            T::state().waker_for(self.index).register(cx.waker());

            compiler_fence(Ordering::SeqCst);
            match self.try_fast_lock() {
                Some(lock) => Poll::Ready(lock),
                None => Poll::Pending,
            }
        })
        .await
    }

    /// Asynchronously lock a hardware semaphore with 2-step lock procedure and interrupts outlined in RM0399 11.3.7.
    ///
    /// - Try to lock the semaphore and return if it is obtained
    /// - If the lock fails:
    ///   - Clear pending interrupt status and retry the lock
    ///   - If the lock is obtained, return
    ///   - If the lock fails, register the waker and enable interrupt in IER
    ///
    pub async fn lock(&mut self, process_id: u8) -> HardwareSemaphoreMutex<'a, T> {
        if let Some(lock) = self.try_lock(process_id) {
            return lock;
        }

        let _scoped_wake_guard = T::RCC_INFO.wake_guard();
        let core = CoreId::current();

        let _irq = self.clear_and_enable_interupt(core);
        poll_fn(|cx| {
            T::state().waker_for(self.index).register(cx.waker());

            compiler_fence(Ordering::SeqCst);
            match self.try_lock(process_id) {
                Some(lock) => Poll::Ready(lock),
                None => Poll::Pending,
            }
        })
        .await
    }

    /// Locks the semaphore in a blocking wait loop
    pub fn blocking_lock(&mut self, process_id: u8) -> HardwareSemaphoreMutex<'a, T> {
        loop {
            if let Some(lock) = self.try_lock(process_id) {
                return lock;
            }
        }
    }

    /// Lock and unlock the semaphore to notify a listening core
    pub fn blocking_notify(&mut self) {
        while self.one_step_lock().is_err() {}
        cortex_m::asm::dsb();
        self.unlock(0);
    }

    /// Blocking poll for semaphore interrupt flag
    pub fn blocking_listen(&mut self) {
        let core = CoreId::current();

        T::state().flag_for(core.to_index()).store(false, Ordering::Release);

        let _irq = self.clear_and_enable_interupt(core);
        // Wait for the semaphore interrupt flag
        while !T::state().flag_for(core.to_index()).load(Ordering::Relaxed) {}
    }

    /// Asynchronous listen for a notification interrupt when this semaphore channel is unlocked
    pub async fn listen(&mut self) {
        let _scoped_wake_guard = T::RCC_INFO.wake_guard();
        let core = CoreId::current();

        T::state().flag_for(core.to_index()).store(false, Ordering::Release);

        let _irq = self.clear_and_enable_interupt(core);
        // Wait for the semaphore interrupt flag
        poll_fn(|cx| {
            T::state().waker_for(self.index).register(cx.waker());

            compiler_fence(Ordering::SeqCst);

            if T::state().flag_for(core.to_index()).load(Ordering::Acquire) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Clear interrupts for this semaphore and return an active interrupt
    #[inline]
    fn clear_and_enable_interupt(&self, core: CoreId) -> ActiveInterrupt<T> {
        T::regs()
            .icr(core.to_index().into())
            .write(|w| w.set_isc(self.index.into(), true));

        ActiveInterrupt::new(core, self.index)
    }

    #[cfg(all(stm32wb, feature = "low-power"))]
    /// Try to fast lock a semaphore, and leave the irq enabled if it fails
    pub(crate) fn try_fast_lock_with_interrupt(&mut self) -> Option<HardwareSemaphoreMutex<'a, T>> {
        let core = CoreId::current();
        let _irq = self.clear_and_enable_interupt(core);

        let res = self.try_fast_lock();

        if res.is_none() {
            core::mem::forget(_irq);
        }

        res
    }

    /// Try to lock the semaphore
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

    /// Try to a lock a single step semaphore.
    /// This should only be used from different cores, as fast locks do not use process IDs
    pub fn try_fast_lock(&mut self) -> Option<HardwareSemaphoreMutex<'a, T>> {
        if self.one_step_lock().is_ok() {
            Some(HardwareSemaphoreMutex {
                index: self.index,
                process_id: 0,
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
    ///
    /// This is only safe to use when acquiring from different cores/AHB masters,
    /// as a 1-step is only exclusive per core, AHB bus master ID
    pub fn one_step_lock(&mut self) -> Result<(), HsemError> {
        let reg = T::regs().rlr(self.index as usize).read();
        match (reg.lock(), reg.coreid() == CoreId::current() as u8, reg.procid()) {
            (true, true, 0) => Ok(()),
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
        critical_section::with(|cs| init_hsem(cs));

        HardwareSemaphore { _type: PhantomData }
    }

    /// Get a single channel, and keep the global struct
    pub const fn channel_for<'a>(&'a mut self, number: u8) -> HardwareSemaphoreChannel<'a, T> {
        #[cfg(all(stm32wb, feature = "low-power"))]
        core::assert!(number != 3 && number != 4);
        #[cfg(all(stm32wl5x, feature = "low-power"))]
        core::assert!(number != 3);

        HardwareSemaphoreChannel::new(number)
    }

    /// Split the global struct into channels
    ///
    /// If using low-power mode, channels 3 and 4 will not be returned
    pub const fn split<'a>(self) -> [HardwareSemaphoreChannel<'a, T>; PUB_CHANNELS] {
        [
            HardwareSemaphoreChannel::new(1),
            HardwareSemaphoreChannel::new(2),
            #[cfg(not(all(any(stm32wb, stm32wl5x), feature = "low-power")))]
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

pub(crate) fn init_hsem(cs: CriticalSection) {
    // Do not attempt to reset the HSEM, as a race condition and deadlock can occur between cores
    // It is assumed the HSEM is already initialized during `init_primary()`
    crate::rcc::enable_with_cs::<HSEM>(cs);

    <HSEM as Instance>::Interrupt::unpend();
    unsafe {
        <HSEM as Instance>::Interrupt::enable();
    }
}

#[cfg(any(all(stm32wb, feature = "low-power"), feature = "_dual-core"))]
pub(crate) const fn get_hsem<'a>(index: usize) -> HardwareSemaphoreChannel<'a, crate::peripherals::HSEM> {
    HardwareSemaphoreChannel::new(index as u8)
}

struct State {
    flags: [AtomicBool; CHANNELS],
    wakers: [AtomicWaker; CHANNELS],
}

impl State {
    const fn new() -> Self {
        Self {
            wakers: [const { AtomicWaker::new() }; CHANNELS],
            flags: [const { AtomicBool::new(false) }; CHANNELS],
        }
    }

    const fn waker_for(&self, index: u8) -> &AtomicWaker {
        &self.wakers[index as usize]
    }

    const fn flag_for(&self, index: u8) -> &AtomicBool {
        &self.flags[index as usize]
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
    ($inst:ident, hsem, $block:ident, GLOBAL, $irq:ident) => {
        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);
