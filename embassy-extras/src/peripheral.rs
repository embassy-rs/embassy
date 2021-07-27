use core::cell::UnsafeCell;
use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;

use cortex_m::peripheral::scb::{Exception, SystemHandler, VectActive};
use cortex_m::peripheral::{NVIC, SCB};
use embassy::interrupt::{Interrupt, InterruptExt};

/// A version of `PeripheralState` without the `'static` bound,
/// for cases where the compiler can't statically make sure
/// that `on_interrupt` doesn't reference anything which might be invalidated.
///
/// # Safety
/// When types implementing this trait are used with `PeripheralMutex`,
/// no fields referenced by `on_interrupt`'s lifetimes must end without first calling `Drop` on the `PeripheralMutex`.
pub unsafe trait PeripheralStateUnchecked: Send {
    type Interrupt: Interrupt;
    fn on_interrupt(&mut self);
}

/// A type which can be used as state with `PeripheralMutex`.
///
/// It needs to be `Send` because `&mut` references are sent back and forth between the 'thread' which owns the `PeripheralMutex` and the interrupt,
/// and `&mut T` is `Send` where `T: Send`.
///
/// It also requires `'static`, because although `Pin` guarantees that the memory of the state won't be invalidated,
/// it doesn't guarantee that the lifetime will last.
pub trait PeripheralState: Send + 'static {
    type Interrupt: Interrupt;
    fn on_interrupt(&mut self);
}

// SAFETY: `T` has to live for `'static` to implement `PeripheralState`, thus its lifetime cannot end.
unsafe impl<T> PeripheralStateUnchecked for T
where
    T: PeripheralState,
{
    type Interrupt = T::Interrupt;
    fn on_interrupt(&mut self) {
        self.on_interrupt()
    }
}

pub struct PeripheralMutex<S: PeripheralStateUnchecked> {
    state: UnsafeCell<S>,

    irq_setup_done: bool,
    irq: S::Interrupt,

    _not_send: PhantomData<*mut ()>,
    _pinned: PhantomPinned,
}

fn exception_to_system_handler(exception: Exception) -> Option<SystemHandler> {
    match exception {
        Exception::NonMaskableInt | Exception::HardFault => None,
        #[cfg(not(armv6m))]
        Exception::MemoryManagement => Some(SystemHandler::MemoryManagement),
        #[cfg(not(armv6m))]
        Exception::BusFault => Some(SystemHandler::BusFault),
        #[cfg(not(armv6m))]
        Exception::UsageFault => Some(SystemHandler::UsageFault),
        #[cfg(any(armv8m, target_arch = "x86_64"))]
        Exception::SecureFault => Some(SystemHandler::SecureFault),
        Exception::SVCall => Some(SystemHandler::SVCall),
        #[cfg(not(armv6m))]
        Exception::DebugMonitor => Some(SystemHandler::DebugMonitor),
        Exception::PendSV => Some(SystemHandler::PendSV),
        Exception::SysTick => Some(SystemHandler::SysTick),
    }
}

/// Whether `irq` can be preempted by the current interrupt.
pub(crate) fn can_be_preempted(irq: &impl Interrupt) -> bool {
    match SCB::vect_active() {
        // Thread mode can't preempt each other
        VectActive::ThreadMode => false,
        VectActive::Exception(exception) => {
            // `SystemHandler` is a subset of `Exception` for those with configurable priority.
            // There's no built in way to convert between them, so `exception_to_system_handler` was necessary.
            if let Some(system_handler) = exception_to_system_handler(exception) {
                let current_prio = SCB::get_priority(system_handler);
                let irq_prio = irq.get_priority().into();
                if current_prio < irq_prio {
                    true
                } else if current_prio == irq_prio {
                    // When multiple interrupts have the same priority number,
                    // the pending interrupt with the lowest interrupt number takes precedence.
                    (exception.irqn() as i16) < irq.number() as i16
                } else {
                    false
                }
            } else {
                // There's no safe way I know of to maintain `!Send` state across invocations of HardFault or NMI, so that should be fine.
                false
            }
        }
        VectActive::Interrupt { irqn } => {
            #[derive(Clone, Copy)]
            struct NrWrap(u16);
            unsafe impl cortex_m::interrupt::InterruptNumber for NrWrap {
                fn number(self) -> u16 {
                    self.0
                }
            }
            let current_prio = NVIC::get_priority(NrWrap(irqn.into()));
            let irq_prio = irq.get_priority().into();
            if current_prio < irq_prio {
                true
            } else if current_prio == irq_prio {
                // When multiple interrupts have the same priority number,
                // the pending interrupt with the lowest interrupt number takes precedence.
                (irqn as u16) < irq.number()
            } else {
                false
            }
        }
    }
}

impl<S: PeripheralStateUnchecked> PeripheralMutex<S> {
    pub fn new(state: S, irq: S::Interrupt) -> Self {
        if can_be_preempted(&irq) {
            panic!("`PeripheralMutex` cannot be created in an interrupt with higher priority than the interrupt it wraps");
        }

        Self {
            irq,
            irq_setup_done: false,

            state: UnsafeCell::new(state),
            _not_send: PhantomData,
            _pinned: PhantomPinned,
        }
    }

    pub fn register_interrupt(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        if this.irq_setup_done {
            return;
        }

        this.irq.disable();
        this.irq.set_handler(|p| {
            // Safety: it's OK to get a &mut to the state, since
            // - We checked that the thread owning the `PeripheralMutex` can't preempt us in `new`.
            //   Interrupts' priorities can only be changed with raw embassy `Interrupts`,
            //   which can't safely store a `PeripheralMutex` across invocations.
            // - We can't have preempted a with() call because the irq is disabled during it.
            let state = unsafe { &mut *(p as *mut S) };
            state.on_interrupt();
        });
        this.irq
            .set_handler_context((&mut this.state) as *mut _ as *mut ());
        this.irq.enable();

        this.irq_setup_done = true;
    }

    pub fn with<R>(self: Pin<&mut Self>, f: impl FnOnce(&mut S) -> R) -> R {
        let this = unsafe { self.get_unchecked_mut() };

        let was_enabled = this.irq.is_enabled();
        this.irq.disable();

        // Safety: it's OK to get a &mut to the state, since the irq is disabled.
        let state = unsafe { &mut *this.state.get() };
        let r = f(state);

        if was_enabled {
            this.irq.enable();
        }

        r
    }

    /// Enables the wrapped interrupt.
    pub fn enable(&self) {
        // This is fine to do before initialization, because we haven't set the handler yet.
        self.irq.enable()
    }

    /// Disables the wrapped interrupt.
    pub fn disable(&self) {
        self.irq.disable()
    }

    /// Returns whether the wrapped interrupt is enabled.
    pub fn is_enabled(&self) -> bool {
        self.irq.is_enabled()
    }

    /// Returns whether the wrapped interrupt is currently in a pending state.
    pub fn is_pending(&self) -> bool {
        self.irq.is_pending()
    }

    /// Forces the wrapped interrupt into a pending state.
    pub fn pend(&self) {
        self.irq.pend()
    }

    /// Forces the wrapped interrupt out of a pending state.
    pub fn unpend(&self) {
        self.irq.unpend()
    }

    /// Gets the priority of the wrapped interrupt.
    pub fn priority(&self) -> <S::Interrupt as Interrupt>::Priority {
        self.irq.get_priority()
    }
}

impl<S: PeripheralStateUnchecked> Drop for PeripheralMutex<S> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();
    }
}
