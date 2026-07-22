use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::future::poll_fn;
use core::task::Poll;

use crate::adc::{BasicAdcRegs, BorrowedAdcChannel, InjectedAdcRegs, Instance, State};
use crate::atomic::AtomicClear;

/// Injected ADC sequence with owned channels.
pub struct InjectedAdc<'d, R: InjectedAdcRegs> {
    regs: R,
    state: &'static State,
    len: usize,
    _marker: PhantomData<&'d mut ()>,
}

impl<'d, R: InjectedAdcRegs> InjectedAdc<'d, R> {
    pub(crate) fn new<T: Instance<Regs = R>, const N: usize>(
        _channels: [(BorrowedAdcChannel<'d, T>, <T::Regs as BasicAdcRegs>::SampleTime); N],
    ) -> Self {
        Self {
            regs: T::regs(),
            state: T::state(),
            len: N,
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

    /// Reads injected convention result after the end of sequence is detected.
    pub async fn read(&mut self, buf: &mut [u16]) {
        let f = poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            if self.state.injected_eos.clear() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        });

        self.start_injected_conversions();
        f.await;

        self.read_latest(buf);
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

impl<'d, R: InjectedAdcRegs> Drop for InjectedAdc<'d, R> {
    fn drop(&mut self) {
        self.regs.stop_injected();
        compiler_fence(Ordering::SeqCst);
    }
}
