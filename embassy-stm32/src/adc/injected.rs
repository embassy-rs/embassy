use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use crate::adc::{BasicAdcRegs, BorrowedAdcChannel, DefaultInstance, InjectedAdcRegs, Instance, State};
use crate::atomic::AtomicClear;
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};

// Implement NoHandler bindings so that when the user requests no interrupt, the binding is satisfied.

#[derive(Clone, Copy)]
pub struct NoHandler<I: Interrupt> {
    _marker: PhantomData<I>,
}

impl<I: Interrupt> crate::interrupt::typelevel::Handler<I> for NoHandler<I> {
    unsafe fn on_interrupt() {}
}

unsafe impl<I: Interrupt + Copy> crate::interrupt::typelevel::Binding<I, NoHandler<I>> for I {}

pub(crate) trait SealedIrqMode: Mode {
    const IRQ: bool;
}

#[allow(private_bounds)]
pub trait IrqMode: SealedIrqMode {
    type Handler<T: DefaultInstance>: Handler<<T as Instance>::Interrupt>;
}

impl IrqMode for Async {
    type Handler<T: DefaultInstance> = crate::adc::InterruptHandler<T>;
}

impl IrqMode for Blocking {
    type Handler<T: DefaultInstance> = NoHandler<T::Interrupt>;
}

impl SealedIrqMode for Async {
    const IRQ: bool = true;
}

impl SealedIrqMode for Blocking {
    const IRQ: bool = false;
}

/// Injected ADC sequence with owned channels.
pub struct InjectedAdc<'d, R: InjectedAdcRegs, M: Mode> {
    regs: R,
    state: &'static State,
    len: usize,
    _mode: M,
    _marker: PhantomData<&'d mut ()>,
}

impl<'d, R: InjectedAdcRegs, M: Mode> InjectedAdc<'d, R, M> {
    pub(crate) fn new<T: Instance<Regs = R>, const N: usize>(
        _channels: [(BorrowedAdcChannel<'d, T>, <T::Regs as BasicAdcRegs>::SampleTime); N],
        mode: M,
    ) -> Self {
        Self {
            regs: T::regs(),
            state: T::state(),
            len: N,
            _mode: mode,
            _marker: PhantomData,
        }
    }

    /// Stops injected convention.
    ///
    /// Any ongoing injected conversion is aborted with partial result discarded.
    pub fn stop_injected_conversions(&mut self) {
        self.regs.stop_injected();
    }

    /// Starts injected convention:
    /// - Immediately if in software trigger mode (JEXTEN = 0)
    /// - At the next active edge of the selected injected hardware trigger (JEXTEN != 0)
    pub fn start_injected_conversions(&mut self) {
        self.regs.start_injected();
    }

    /// Reads latest result directly from the injected convention registers.
    ///
    /// This function is intended to be used in a custom interrupt handler.
    /// For other use cases prefer [`read`](Self::read) function.
    pub fn read_latest(&mut self, buf: &mut [u16]) {
        assert!(
            buf.len() == self.len,
            "Buffer must have as many entries as the sequence"
        );

        self.regs.read_injected(buf);
    }
}

impl<'d, R: InjectedAdcRegs> InjectedAdc<'d, R, Async> {
    /// Reads injected convention result after the end of sequence is detected.
    pub async fn read(&mut self, buf: &mut [u16]) {
        let f = poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            if self.state.injected_done.clear() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        });

        self.start_injected_conversions();
        f.await;

        self.read_latest(buf);
    }
}

impl<'d, R: InjectedAdcRegs, M: Mode> Drop for InjectedAdc<'d, R, M> {
    fn drop(&mut self) {
        self.regs.stop_injected();
        compiler_fence(Ordering::SeqCst);
    }
}
