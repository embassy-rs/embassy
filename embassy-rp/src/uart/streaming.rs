//! streaming uart rx using double-buffered dma
//!
//! overview: lightweight helper to configure uart for rx dma and produce a
//! double-buffered dma `RxStream` fed directly from the uart data register.

use embassy_hal_internal::Peri;

use super::*;
use crate::dma::{self, Channel};

/// streaming uart rx handle
pub struct StreamingUartRx<'buf> {
    pub(super) info: &'static Info,
    pub(super) inner: dma::RxStream<'static, 'buf, AnyChannel, AnyChannel>,
}

impl<'buf> StreamingUartRx<'buf> {
    /// Create a new streaming UART RX (no flow control).
    pub fn new<'d, T: Instance, C0: Channel, C1: Channel>(
        _uart: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>, // error irq wiring
        ch_a: Peri<'static, C0>,
        ch_b: Peri<'static, C1>,
        buf_a: &'buf mut [u8],
        buf_b: &'buf mut [u8],
        uart_config: Config,
    ) -> Self {
        // configure pins/peripheral and enable irq handling in the parent module
        super::Uart::<Async>::init(T::info(), None, Some(rx.into()), None, None, uart_config);
        let info = T::info();

        // enable rx dma and abort transfers on uart error conditions
        info.regs.uartdmacr().write_set(|reg| {
            reg.set_rxdmae(true);
            reg.set_dmaonerr(true);
        });

        Self {
            inner: dma::RxStream::new(
                ch_a.into(),
                ch_b.into(),
                info.regs.uartdr().as_ptr() as *const _,
                info.rx_dreq.into(),
                buf_a,
                buf_b,
            ),
            info,
        }
    }

    /// Build a double-buffered DMA stream using the provided DMA channels and buffers.
    pub async fn next(&mut self) -> Option<dma::RxBufView<'_, 'buf>> {
        self.inner.next().await
    }
}
