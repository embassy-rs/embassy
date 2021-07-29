use core::cell::UnsafeCell;
use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;

use cortex_m::peripheral::scb::{Exception, SystemHandler, VectActive};
use cortex_m::peripheral::{NVIC, SCB};
use embassy::interrupt::{Interrupt, InterruptExt};

/// A type which can be used as state with `PeripheralMutex`.
///
/// It needs to be `Send` because `&mut` references are sent back and forth between the 'thread' which owns the `PeripheralMutex` and the interrupt,
/// and `&mut T` is only `Send` where `T: Send`.
///
/// It also requires `'static` to be used safely with `PeripheralMutex::register_interrupt`,
/// because although `Pin` guarantees that the memory of the state won't be invalidated,
/// it doesn't guarantee that the lifetime will last.
pub trait PeripheralState: Send {
    type Interrupt: Interrupt;
    fn on_interrupt(&mut self);
}

pub struct PeripheralMutex<S: PeripheralState> {
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
        // Thread mode can't preempt anything
        VectActive::ThreadMode => false,
        VectActive::Exception(exception) => {
            // `SystemHandler` is a subset of `Exception` for those with configurable priority.
            // There's no built in way to convert between them, so `exception_to_system_handler` was necessary.
            if let Some(system_handler) = exception_to_system_handler(exception) {
                SCB::get_priority(system_handler) < irq.get_priority().into()
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
            NVIC::get_priority(NrWrap(irqn.into())) < irq.get_priority().into()
        }
    }
}

impl<S: PeripheralState + 'static> PeripheralMutex<S> {
    /// Registers `on_interrupt` as the wrapped interrupt's interrupt handler and enables it.
    ///
    /// This requires this `PeripheralMutex`'s `PeripheralState` to live for `'static`,
    /// because `Pin` only guarantees that it's memory won't be repurposed,
    /// not that it's lifetime will last.
    ///
    /// To use non-`'static` `PeripheralState`, use the unsafe `register_interrupt_unchecked`.
    ///
    /// Note: `'static` doesn't mean it _has_ to live for the entire program, like an `&'static T`;
    /// it just means it _can_ live for the entire program - for example, `u8` lives for `'static`.
    pub fn register_interrupt(self: Pin<&mut Self>) {
        // SAFETY: `S: 'static`, so there's no way it's lifetime can expire.
        unsafe { self.register_interrupt_unchecked() }
    }
}

impl<S: PeripheralState> PeripheralMutex<S> {
    /// Create a new `PeripheralMutex` wrapping `irq`, with the initial state `state`.
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

    /// Registers `on_interrupt` as the wrapped interrupt's interrupt handler and enables it.
    ///
    /// # Safety
    /// The lifetime of any data in `PeripheralState` that is accessed by the interrupt handler
    /// must not end without `Drop` being called on this `PeripheralMutex`.
    ///
    /// This can be accomplished by either not accessing any data with a lifetime in `on_interrupt`,
    /// or making sure that nothing like `mem::forget` is used on the `PeripheralMutex`.

    // TODO: this name isn't the best.
    pub unsafe fn register_interrupt_unchecked(self: Pin<&mut Self>) {
        let this = self.get_unchecked_mut();
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

        this.irq.disable();

        // Safety: it's OK to get a &mut to the state, since the irq is disabled.
        let state = unsafe { &mut *this.state.get() };
        let r = f(state);

        this.irq.enable();

        r
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

impl<S: PeripheralState> Drop for PeripheralMutex<S> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();
    }
}
