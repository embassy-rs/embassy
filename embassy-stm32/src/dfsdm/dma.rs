use core::sync::atomic::{Ordering, compiler_fence};

use super::*;
use crate::dma::{Channel, ReadableRingBuffer};
use crate::rcc::WakeGuard;

pub struct RingBufferedFilter<'e, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    filter: &'e dyn FilterDma<T, M>,
    ring_buf: ReadableRingBuffer<'e, u32>,
    _wake_guard: WakeGuard,
}

#[allow(private_bounds)]
impl<'e, T, M> RingBufferedFilter<'e, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    pub fn new_regular<'a, 'd, 't, D: Dma<T, M>>(
        filter: &'e FilterRegular<'a, 'd, 't, T, M, RegDma>,
        dma: Peri<'e, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'e,
        dma_buf: &'e mut [u32],
    ) -> Self {
        let opts = Default::default();

        // Safety: we forget the struct before this function returns.
        let request = dma.request();

        let mut ring_buf =
            unsafe { ReadableRingBuffer::new(Channel::new(dma, irq), request, filter.data_register(), dma_buf, opts) };

        // Align reads to the scan sequence boundary so that channel assignments
        // never shift after an overrun recovery.
        // ring_buf.set_alignment(dma_buf.len() / 2); // TODO  USE LATER FOR PING PONG

        Self {
            filter,
            _wake_guard: T::RCC_INFO.wake_guard(),
            ring_buf,
        }
    }

    pub fn new_injected<'a, 'd, 't, D: Dma<T, M>>(
        filter: &'e FilterInjected<'a, 'd, 't, T, M, InjDma>,
        dma: Peri<'e, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'e,
        dma_buf: &'e mut [u32],
    ) -> Self {
        let opts = Default::default();

        // Safety: we forget the struct before this function returns.
        let request = dma.request();

        let mut ring_buf =
            unsafe { ReadableRingBuffer::new(Channel::new(dma, irq), request, filter.data_register(), dma_buf, opts) };

        // Align reads to the scan sequence boundary so that channel assignments
        // never shift after an overrun recovery.
        // ring_buf.set_alignment(dma_buf.len() / 2); // TODO  USE LATER FOR PING PONG

        Self {
            filter,
            _wake_guard: T::RCC_INFO.wake_guard(),
            ring_buf,
        }
    }

    pub fn start(&mut self) {
        compiler_fence(Ordering::SeqCst);
        self.ring_buf.start();

        // self.regs.start(); DFSDM doesnt need start
    }
    pub fn read_latest(&mut self, measurements: &mut [u32]) -> usize {
        if !self.ring_buf.is_running() {
            self.start();
        }

        self.ring_buf.read_latest(measurements)
    }
}
