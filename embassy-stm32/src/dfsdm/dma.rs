use core::sync::atomic::{Ordering, compiler_fence};

use super::*;
use crate::dma::{Channel, ReadableRingBuffer};
use crate::rcc::WakeGuard;

pub struct RingBufferedFilter<'d, T, M, P>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
    P: PowerState,
{
    filter: &'d Filter<T, M, P>,
    ring_buf: ReadableRingBuffer<'d, u32>,
    _wake_guard: WakeGuard,
}

#[allow(private_bounds)]
impl<'d, T, M, P> RingBufferedFilter<'d, T, M, P>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
    P: PowerState,
{
    pub fn new_regular<D: Dma<T, M>>(
        filter: &'d Filter<T, M, P>,
        dma: Peri<'d, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'd,
        dma_buf: &'d mut [u32],
    ) -> Self {
        let opts = Default::default();

        // Safety: we forget the struct before this function returns.
        let request = dma.request();

        let mut ring_buf = unsafe {
            ReadableRingBuffer::new(
                Channel::new(dma, irq),
                request,
                T::regs().flt(M::CHANNEL.index()).rdatar().as_ptr() as *mut u32,
                dma_buf,
                opts,
            )
        };

        // Align reads to the scan sequence boundary so that channel assignments
        // never shift after an overrun recovery.
        // ring_buf.set_alignment(dma_buf.len() / 2); // TODO  USE LATER FOR PING PONG

        Self {
            filter,
            _wake_guard: T::RCC_INFO.wake_guard(),
            ring_buf,
        }
    }

    pub fn new_injected<D: Dma<T, M>>(
        filter: &'d Filter<T, M, P>,
        dma: Peri<'d, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'd,
        dma_buf: &'d mut [u32],
    ) -> Self {
        let opts = Default::default();

        // Safety: we forget the struct before this function returns.
        let request = dma.request();

        let mut ring_buf = unsafe {
            ReadableRingBuffer::new(
                Channel::new(dma, irq),
                request,
                T::regs().flt(M::CHANNEL.index()).jdatar().as_ptr() as *mut u32,
                dma_buf,
                opts,
            )
        };

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
