use super::*;
use crate::dma::ringbuffer::Error as DmaError;
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
        let mut buf = RingBufferedFilter::<T, M, RegDma>::new_int(self, dma, irq, dma_buf);
        buf.ring_buf.set_alignment(1);
        buf
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
        let alignment = self.popcnt();
        let mut buf = RingBufferedFilter::<T, M, InjDma>::new_int(self, dma, irq, dma_buf);
        buf.ring_buf.set_alignment(alignment);
        buf
    }

    /// Returns number of assigned channels in channelgroup
    fn popcnt(&self) -> usize {
        let bitmask = T::regs().flt(M::CHANNEL.index()).jchgr().read().jchg();
        bitmask.count_ones() as usize
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
        let ring_buf =
            unsafe { ReadableRingBuffer::new(Channel::new(dma, irq), request, filter.data_register(), dma_buf, opts) };

        RingBufferedFilter {
            _dma_marker: PhantomData,
            filter,
            _wake_guard: T::RCC_INFO.wake_guard(),
            ring_buf,
        }
    }

    //TODO docstring should mention that it just trigges the startconverison,
    //meaning in injected one group OR scan, in regular one conversion OR continuous
    //maybe duplicate function for both markers with different docs idk
    pub fn start_conversion(&mut self) {
        self.filter.start_conversion();
    }

    pub fn start(&mut self) {
        self.ring_buf.start();
    }

    pub fn stop(&mut self) {
        self.ring_buf.request_pause();
    }

    pub fn clear(&mut self) {
        self.ring_buf.clear();
    }

    pub fn is_running(&mut self) -> bool {
        self.ring_buf.is_running()
    }

    pub fn capacity(&self) -> usize {
        self.ring_buf.capacity()
    }

    pub fn read_latest(&mut self, buf: &mut [u32]) -> Result<usize, Error> {
        self.autostart()?;

        Ok(self.ring_buf.read_latest(buf))
    }

    pub async fn read(&mut self, buf: &mut [u32]) -> Result<usize, Error> {
        self.autostart()?;

        self.ring_buf.read_exact(buf).await.map_err(remap_dma_error)
    }

    pub fn blocking_read(&mut self, buf: &mut [u32]) -> Result<usize, Error> {
        self.autostart()?;

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(err) => {
                    self.ring_buf.request_pause();

                    return Err(remap_dma_error(err));
                }
            }
        }
    }

    fn autostart(&mut self) -> Result<(), Error> {
        if self.filter.get_and_clear_overrun() {
            return Err(Error::Overrun);
        }

        if !self.ring_buf.is_running() {
            self.start();
        }

        Ok(())
    }
}

fn remap_dma_error(err: DmaError) -> Error {
    match err {
        DmaError::Overrun => Error::Overrun,
        DmaError::DmaUnsynced => Error::PeripheralError,
    }
}
