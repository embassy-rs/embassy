use core::sync::atomic::{Ordering, compiler_fence};

use super::*;
use crate::dma::{Channel, ReadableRingBuffer};
use crate::interrupt::typelevel::Binding;
use crate::rcc::WakeGuard;

pub struct RingBufferedFilter<'e, T, M, DM: DmaMode>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    _dma_marker: PhantomData<DM>,
    filter: &'e mut dyn FilterDma<T, M>,
    ring_buf: ReadableRingBuffer<'e, u32>,
    _wake_guard: WakeGuard,
}

impl<'a, 'd, 't, T, M> FilterRegular<'a, 'd, 't, T, M, RegDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    pub fn ring_buffered<'e, D: Dma<T, M>>(
        self: &'e mut Self,
        dma: Peri<'e, D>,
        irq: impl Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'e,
        dma_buf: &'e mut [u32],
    ) -> RingBufferedFilter<'e, T, M, RegDma> {
        RingBufferedFilter::<T, M, RegDma>::new_int(self, dma, irq, dma_buf)
    }
}

impl<'a, 'd, 't, T, M> FilterInjected<'a, 'd, 't, T, M, InjDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    pub fn ring_buffered<'e, D: Dma<T, M>>(
        self: &'e mut Self,
        dma: Peri<'e, D>,
        irq: impl Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'e,
        dma_buf: &'e mut [u32],
    ) -> RingBufferedFilter<'e, T, M, InjDma> {
        RingBufferedFilter::<T, M, InjDma>::new_int(self, dma, irq, dma_buf)
    }
}

#[allow(private_bounds)]
impl<'e, T, M, DM: DmaMode> RingBufferedFilter<'e, T, M, DM>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    fn new_int<D: Dma<T, M>, DMODE: DmaMode>(
        filter: &'e mut dyn FilterDma<T, M>,
        dma: Peri<'e, D>,
        irq: impl Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'e,
        dma_buf: &'e mut [u32],
    ) -> RingBufferedFilter<'e, T, M, DMODE> {
        let opts = Default::default();

        // SAFETY: `ReadableRingBuffer::new` requires:
        // 1. Exclusive access to the DMA channel - guaranteed by taking ownership of `dma: Peri<'e, D>`
        // 2. The buffer pointer remains valid for the lifetime `'e` - guaranteed by the borrow `dma_buf: &'e mut [u32]`
        // 3. The data register pointer is valid - `filter.data_register()` returns a valid MMIO address
        // 4. No concurrent access to the buffer - the ring buffer is not started until `start()` is called
        // SAFETY: `filter.data_register()` returns a pointer to the DFSDM filter's
        // data register (RDATAR or JDATAR), which is a valid DMA source. The register
        // is memory-mapped and remains accessible for the lifetime of the filter.
        let request = dma.request();
        let mut ring_buf =
            unsafe { ReadableRingBuffer::new(Channel::new(dma, irq), request, filter.data_register(), dma_buf, opts) };

        // Align reads to the scan sequence boundary so that channel assignments
        // never shift after an overrun recovery.
        // ring_buf.set_alignment(dma_buf.len() / 2); // TODO  USE LATER FOR PING PONG

        RingBufferedFilter {
            _dma_marker: PhantomData,
            filter,
            _wake_guard: T::RCC_INFO.wake_guard(),
            ring_buf,
        }
    }

    pub fn start(&mut self) {
        // compiler_fence(Ordering::SeqCst);
        self.ring_buf.start();

        // self.regs.start(); DFSDM doesnt need start
    }

    /// Reads the latest measurements from the DMA ring buffer.
    ///
    /// If the buffer is not yet running, it will be started automatically.
    ///
    /// # Arguments
    /// * `measurements` - Buffer to store the measurements. Must be at least
    ///   as large as the number of samples to read.
    ///
    /// # Returns
    /// The number of samples actually read. This may be less than
    /// `measurements.len()` if fewer samples are available.
    ///
    /// # Note
    /// This function reads the most recent samples, discarding older ones
    /// if the buffer has wrapped around.
    pub fn read_latest(&mut self, measurements: &mut [u32]) -> usize {
        if !self.ring_buf.is_running() {
            self.start();
        }

        self.ring_buf.read_latest(measurements)
    }
}
