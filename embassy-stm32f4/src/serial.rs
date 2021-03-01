//! Async low power Serial.
//!
//! The peripheral is autmatically enabled and disabled as required to save power.
//! Lowest power consumption can only be guaranteed if the send receive futures
//! are dropped correctly (e.g. not using `mem::forget()`).

use core::future::Future;
use core::ptr;
use core::sync::atomic::{self, Ordering};

use embassy::interrupt::InterruptExt;
use embassy::traits::uart::{Error, Uart};
use embassy::util::Signal;

use crate::hal::dma::config::DmaConfig;
use crate::hal::dma::traits::{PeriAddress, Stream};
use crate::hal::dma::{Stream2, Stream7, StreamsTuple, Transfer};
use crate::hal::rcc::Clocks;
use crate::hal::serial::config::{Config as SerialConfig, DmaConfig as SerialDmaConfig};
use crate::hal::serial::Pins;
use crate::hal::serial::{Event as SerialEvent, Serial as HalSerial};
use crate::interrupt;
use crate::pac::{DMA2, USART1};

/// Interface to the Serial peripheral
pub struct Serial<USART: PeriAddress<MemSize = u8>, TSTREAM: Stream, RSTREAM: Stream> {
    tx_stream: Option<TSTREAM>,
    rx_stream: Option<RSTREAM>,
    usart: Option<USART>,
    tx_int: interrupt::DMA2_STREAM7,
    rx_int: interrupt::DMA2_STREAM2,
    usart_int: interrupt::USART1,
}

struct State {
    tx_int: Signal<()>,
    rx_int: Signal<()>,
}

static STATE: State = State {
    tx_int: Signal::new(),
    rx_int: Signal::new(),
};

static mut INSTANCE: *const Serial<USART1, Stream7<DMA2>, Stream2<DMA2>> = ptr::null_mut();

impl Serial<USART1, Stream7<DMA2>, Stream2<DMA2>> {
    // Leaking futures is forbidden!
    pub unsafe fn new<PINS>(
        usart: USART1,
        dma: DMA2,
        pins: PINS,
        tx_int: interrupt::DMA2_STREAM7,
        rx_int: interrupt::DMA2_STREAM2,
        usart_int: interrupt::USART1,
        mut config: SerialConfig,
        clocks: Clocks,
    ) -> Self
    where
        PINS: Pins<USART1>,
    {
        config.dma = SerialDmaConfig::TxRx;
        let mut serial = HalSerial::usart1(usart, pins, config, clocks).unwrap();

        serial.listen(SerialEvent::Idle);
        //        serial.listen(SerialEvent::Txe);

        let (usart, _) = serial.release();

        // Register ISR
        tx_int.set_handler(Self::on_tx_irq);
        rx_int.set_handler(Self::on_rx_irq);
        usart_int.set_handler(Self::on_rx_irq);
        // usart_int.unpend();
        // usart_int.enable();

        let streams = StreamsTuple::new(dma);

        Serial {
            tx_stream: Some(streams.7),
            rx_stream: Some(streams.2),
            usart: Some(usart),
            tx_int: tx_int,
            rx_int: rx_int,
            usart_int: usart_int,
        }
    }

    unsafe fn on_tx_irq(_ctx: *mut ()) {
        let s = &(*INSTANCE);

        s.tx_int.disable();

        STATE.tx_int.signal(());
    }

    unsafe fn on_rx_irq(_ctx: *mut ()) {
        let s = &(*INSTANCE);

        atomic::compiler_fence(Ordering::Acquire);
        s.rx_int.disable();
        s.usart_int.disable();
        atomic::compiler_fence(Ordering::Release);

        STATE.rx_int.signal(());
    }

    unsafe fn on_usart_irq(_ctx: *mut ()) {
        let s = &(*INSTANCE);

        atomic::compiler_fence(Ordering::Acquire);
        s.rx_int.disable();
        s.usart_int.disable();
        atomic::compiler_fence(Ordering::Release);

        STATE.rx_int.signal(());
    }
}

impl Uart for Serial<USART1, Stream7<DMA2>, Stream2<DMA2>> {
    type SendFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
    type ReceiveFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;

    /// Sends serial data.
    fn send<'a>(&'a mut self, buf: &'a [u8]) -> Self::SendFuture<'a> {
        unsafe { INSTANCE = self };

        #[allow(mutable_transmutes)]
        let static_buf = unsafe { core::mem::transmute::<&'a [u8], &'static mut [u8]>(buf) };

        let tx_stream = self.tx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();
        STATE.tx_int.reset();

        async move {
            let mut tx_transfer = Transfer::init(
                tx_stream,
                usart,
                static_buf,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );

            self.tx_int.unpend();
            self.tx_int.enable();
            tx_transfer.start(|_usart| {});

            STATE.tx_int.wait().await;

            let (tx_stream, usart, _buf, _) = tx_transfer.free();
            self.tx_stream.replace(tx_stream);
            self.usart.replace(usart);

            Ok(())
        }
    }

    /// Receives serial data.
    ///
    /// The future is pending until the buffer is completely filled.
    /// A common pattern is to use [`stop()`](ReceiveFuture::stop) to cancel
    /// unfinished transfers after a timeout to prevent lockup when no more data
    /// is incoming.
    fn receive<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReceiveFuture<'a> {
        unsafe { INSTANCE = self };

        let static_buf = unsafe { core::mem::transmute::<&'a mut [u8], &'static mut [u8]>(buf) };
        let rx_stream = self.rx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();
        STATE.rx_int.reset();
        async move {
            let mut rx_transfer = Transfer::init(
                rx_stream,
                usart,
                static_buf,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );
            self.rx_int.unpend();
            self.rx_int.enable();
            rx_transfer.start(|_usart| {});
            STATE.rx_int.wait().await;
            let (rx_stream, usart, _, _) = rx_transfer.free();
            self.rx_stream.replace(rx_stream);
            self.usart.replace(usart);
            Ok(())
        }
    }
}
