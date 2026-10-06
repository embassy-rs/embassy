//! Injected conversions.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use super::{
    Adc, AdcRegs, BorrowedAdcChannel, ConversionMode, Exten, InjectedAdcTrigger, Instance, RegularAdcTrigger,
    RingBufferedAdc, RxDma, SampleTimeOf, State, check_dma_len,
};
use crate::atomic::AtomicClear;
use crate::interrupt::typelevel::Handler;
use crate::mode::{Async, Blocking, Mode, NoHandler};

/// Injected conversion operations, implemented by the ADC generations that have injected
/// conversions.
pub(crate) trait InjectedRegs: AdcRegs {
    /// Set the injected trigger and enable the end-of-injected-sequence interrupt.
    fn configure_injected_trigger(self, trigger: (u8, Exten), interrupt: bool);
    fn start_injected(self);
    fn stop_injected(self);
    fn read_injected(self, data: &mut [u16]);
}

/// Interrupt mode of an [`InjectedAdc`].
#[allow(private_bounds)]
pub trait InjectedMode: Mode {
    /// The interrupt handler that must be bound for this mode.
    type Handler<T: Instance>: Handler<<T as Instance>::Interrupt>;
}

impl InjectedMode for Async {
    type Handler<T: Instance> = super::InterruptHandler<T>;
}

impl InjectedMode for Blocking {
    type Handler<T: Instance> = NoHandler<T::Interrupt>;
}

/// Injected ADC sequence with owned channels.
#[allow(private_bounds)]
pub struct InjectedAdc<'d, R: InjectedRegs, M: Mode> {
    regs: R,
    state: &'static State,
    len: usize,
    _mode: M,
    _marker: PhantomData<&'d mut ()>,
}

#[allow(private_bounds)]
impl<'d, R: InjectedRegs, M: Mode> InjectedAdc<'d, R, M> {
    pub(crate) fn new<T: Instance<Regs = R>, const N: usize>(
        _channels: [(BorrowedAdcChannel<'d, T>, SampleTimeOf<T>); N],
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

    /// Stops injected conversions.
    ///
    /// Any ongoing injected conversion is aborted with partial result discarded.
    pub fn stop_injected_conversions(&mut self) {
        self.regs.stop_injected();
    }

    /// Starts injected conversions:
    /// - Immediately if in software trigger mode (JEXTEN = 0)
    /// - At the next active edge of the selected injected hardware trigger (JEXTEN != 0)
    pub fn start_injected_conversions(&mut self) {
        self.regs.start_injected();
    }

    /// Reads the latest result directly from the injected data registers.
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

#[allow(private_bounds)]
impl<'d, R: InjectedRegs> InjectedAdc<'d, R, Async> {
    /// Reads the injected conversion result after the end of sequence is detected.
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

impl<'d, R: InjectedRegs, M: Mode> Drop for InjectedAdc<'d, R, M> {
    fn drop(&mut self) {
        self.regs.stop_injected();
        compiler_fence(Ordering::SeqCst);
    }
}

#[allow(private_bounds)]
impl<'d, T: Instance, M: Mode> Adc<'d, T, M>
where
    T::Regs: InjectedRegs,
{
    /// Configures the ADC for injected conversions.
    ///
    /// Injected conversions are separate from the regular conversion sequence and are typically
    /// triggered by software or an external event. This method sets up a fixed-length sequence of
    /// injected channels with specified sample times, the trigger source, and whether the end-of-sequence
    /// interrupt should be enabled.
    ///
    /// # Parameters
    /// - `sequence`: An array of tuples containing the ADC channels and their sample times. The length
    ///   `N` determines the number of injected ranks to configure (maximum 4).
    /// - `trigger`: The trigger source that starts the injected conversion sequence.
    /// - `mode`: [`Async`] to read results with the interrupt (the ADC interrupt must be bound),
    ///   [`Blocking`] to read them with [`InjectedAdc::read_latest`].
    ///
    /// # Returns
    /// An `InjectedAdc` instance that represents the configured injected sequence.
    ///
    /// # Panics
    /// This function will panic if `sequence` is empty or longer than the number of injected ranks.
    pub fn setup_injected_conversions<'a, const N: usize, IM: InjectedMode>(
        self,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, IM::Handler<T>> + 'a,
        sequence: [(BorrowedAdcChannel<'a, T>, SampleTimeOf<T>); N],
        trigger: InjectedAdcTrigger<T>,
        mode: IM,
    ) -> InjectedAdc<'a, T::Regs, IM> {
        assert!(N != 0, "Read sequence cannot be empty");
        assert!(
            N <= T::Regs::INJECTED_RANKS,
            "Read sequence cannot be more than {} in length",
            T::Regs::INJECTED_RANKS
        );

        let r = T::regs();
        r.stop_injected();
        r.configure_sequence(
            sequence
                .iter()
                .map(|(channel, sample_time)| ((channel.channel, channel.is_differential), *sample_time)),
            true,
        );

        r.enable();
        r.configure_injected_trigger((trigger.trigger, trigger.edge), IM::ASYNC);

        if IM::ASYNC {
            use crate::interrupt::typelevel::Interrupt;
            T::Interrupt::unpend();
            unsafe { T::Interrupt::enable() };
        }

        r.start_injected();

        core::mem::forget(self);

        InjectedAdc::new(sequence, mode)
    }

    /// Configures ADC for both regular conversions with a ring-buffered DMA and injected conversions.
    ///
    /// # Parameters
    /// - `dma`: The DMA peripheral to use for the ring-buffered ADC transfers.
    /// - `dma_buf`: The buffer to store DMA-transferred samples for regular conversions.
    /// - `regular_sequence`: The sequence of channels and their sample times for regular conversions.
    /// - `regular_trigger`: The trigger for regular conversions (`None` for continuous).
    /// - `injected_sequence`: An array of channels and sample times for injected conversions (length `N`).
    /// - `injected_trigger`: The trigger source for injected conversions.
    ///
    /// Injected conversions are typically used with interrupts. If ADC1 and ADC2 are used in dual mode,
    /// it is recommended to enable interrupts only for the ADC whose sequence takes the longest to complete.
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. `RingBufferedAdc` — the configured ADC for regular conversions using DMA.
    /// 2. `InjectedAdc` — the configured ADC for injected conversions.
    #[allow(clippy::too_many_arguments)]
    pub fn into_ring_buffered_and_injected<'a, 'b, const N: usize, D: RxDma<T>, IM: InjectedMode>(
        self,
        dma: crate::Peri<'a, D>,
        dma_buf: &'a mut [u16],
        _irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'a
        + crate::interrupt::typelevel::Binding<T::Interrupt, IM::Handler<T>>
        + 'b,
        regular_sequence: impl ExactSizeIterator<Item = (BorrowedAdcChannel<'a, T>, SampleTimeOf<T>)>,
        regular_trigger: Option<RegularAdcTrigger<T>>,
        injected_sequence: [(BorrowedAdcChannel<'b, T>, SampleTimeOf<T>); N],
        injected_trigger: InjectedAdcTrigger<T>,
        mode: IM,
    ) -> (RingBufferedAdc<'a, T::Regs>, InjectedAdc<'b, T::Regs, IM>) {
        let sequence_len = regular_sequence.len();

        check_dma_len::<T>(sequence_len, Some(dma_buf.len()));

        assert!(N != 0, "Read sequence cannot be empty");
        assert!(
            N <= T::Regs::INJECTED_RANKS,
            "Read sequence cannot be more than {} in length",
            T::Regs::INJECTED_RANKS
        );

        let r = T::regs();
        r.stop();
        r.stop_injected();

        r.configure_sequence(
            regular_sequence.map(|(channel, sample_time)| ((channel.channel, channel.is_differential), sample_time)),
            false,
        );
        r.configure_sequence(
            injected_sequence
                .iter()
                .map(|(channel, sample_time)| ((channel.channel, channel.is_differential), *sample_time)),
            true,
        );

        r.enable();
        r.configure_dma(ConversionMode::Repeated(regular_trigger.map(|t| (t.trigger, t.edge))));
        r.configure_injected_trigger((injected_trigger.trigger, injected_trigger.edge), IM::ASYNC);

        if IM::ASYNC {
            use crate::interrupt::typelevel::Interrupt;
            T::Interrupt::unpend();
            unsafe { T::Interrupt::enable() };
        }

        r.start_injected();

        core::mem::forget(self);

        (
            RingBufferedAdc::new(dma, _irq, dma_buf, sequence_len),
            InjectedAdc::new(injected_sequence, mode),
        )
    }
}
