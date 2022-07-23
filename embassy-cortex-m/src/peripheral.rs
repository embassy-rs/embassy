//! Peripheral interrupt handling specific to cortex-m devices.
use core::mem::MaybeUninit;

use cortex_m::peripheral::scb::VectActive;
use cortex_m::peripheral::{NVIC, SCB};
use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};

use crate::interrupt::{Interrupt, InterruptExt, Priority};

/// A type which can be used as state with `PeripheralMutex`.
///
/// It needs to be `Send` because `&mut` references are sent back and forth between the 'thread' which owns the `PeripheralMutex` and the interrupt,
/// and `&mut T` is only `Send` where `T: Send`.
pub trait PeripheralState: Send {
    /// The interrupt that is used for this peripheral.
    type Interrupt: Interrupt;

    /// The interrupt handler that should be invoked for the peripheral. Implementations need to clear the appropriate interrupt flags to ensure the handle will not be called again.
    fn on_interrupt(&mut self);
}

/// A type for storing the state of a peripheral that can be stored in a static.
pub struct StateStorage<S>(MaybeUninit<S>);

impl<S> StateStorage<S> {
    /// Create a new instance for storing peripheral state.
    pub const fn new() -> Self {
        Self(MaybeUninit::uninit())
    }
}

/// A type for a peripheral that keeps the state of a peripheral that can be accessed from thread mode and an interrupt handler in
/// a safe way.
pub struct PeripheralMutex<'a, S: PeripheralState> {
    state: *mut S,
    irq: PeripheralRef<'a, S::Interrupt>,
}

/// Whether `irq` can be preempted by the current interrupt.
pub(crate) fn can_be_preempted(irq: &impl Interrupt) -> bool {
    match SCB::vect_active() {
        // Thread mode can't preempt anything.
        VectActive::ThreadMode => false,
        // Exceptions don't always preempt interrupts,
        // but there isn't much of a good reason to be keeping a `PeripheralMutex` in an exception anyway.
        VectActive::Exception(_) => true,
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

impl<'a, S: PeripheralState> PeripheralMutex<'a, S> {
    /// Create a new `PeripheralMutex` wrapping `irq`, with `init` initializing the initial state.
    ///
    /// Registers `on_interrupt` as the `irq`'s handler, and enables it.
    pub fn new(
        irq: impl Peripheral<P = S::Interrupt> + 'a,
        storage: &'a mut StateStorage<S>,
        init: impl FnOnce() -> S,
    ) -> Self {
        into_ref!(irq);

        if can_be_preempted(&*irq) {
            panic!(
                "`PeripheralMutex` cannot be created in an interrupt with higher priority than the interrupt it wraps"
            );
        }

        let state_ptr = storage.0.as_mut_ptr();

        // Safety: The pointer is valid and not used by anyone else
        // because we have the `&mut StateStorage`.
        unsafe { state_ptr.write(init()) };

        irq.disable();
        irq.set_handler(|p| unsafe {
            // Safety: it's OK to get a &mut to the state, since
            // - We checked that the thread owning the `PeripheralMutex` can't preempt us in `new`.
            //   Interrupts' priorities can only be changed with raw embassy `Interrupts`,
            //   which can't safely store a `PeripheralMutex` across invocations.
            // - We can't have preempted a with() call because the irq is disabled during it.
            let state = &mut *(p as *mut S);
            state.on_interrupt();
        });
        irq.set_handler_context(state_ptr as *mut ());
        irq.enable();

        Self { irq, state: state_ptr }
    }

    /// Access the peripheral state ensuring interrupts are disabled so that the state can be
    /// safely accessed.
    pub fn with<R>(&mut self, f: impl FnOnce(&mut S) -> R) -> R {
        self.irq.disable();

        // Safety: it's OK to get a &mut to the state, since the irq is disabled.
        let state = unsafe { &mut *self.state };
        let r = f(state);

        self.irq.enable();

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
    pub fn priority(&self) -> Priority {
        self.irq.get_priority()
    }
}

impl<'a, S: PeripheralState> Drop for PeripheralMutex<'a, S> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();

        // safety:
        // - we initialized the state in `new`, so we know it's initialized.
        // - the irq is disabled, so it won't preempt us while dropping.
        unsafe { self.state.drop_in_place() }
    }
}
