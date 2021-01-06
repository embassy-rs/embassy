//! Async low power Serial.
//!
//! The peripheral is autmatically enabled and disabled as required to save power.
//! Lowest power consumption can only be guaranteed if the send receive futures
//! are dropped correctly (e.g. not using `mem::forget()`).

use core::future::Future;
use core::ptr;
use core::sync::atomic::{self, Ordering};
use core::task::{Context, Poll};

use embassy::interrupt::OwnedInterrupt;
use embassy::util::Signal;
use embedded_dma::{StaticReadBuffer, StaticWriteBuffer, WriteBuffer};

use crate::hal::dma::config::DmaConfig;
use crate::hal::dma::traits::{PeriAddress, Stream};
use crate::hal::dma::{
    Channel4, MemoryToPeripheral, PeripheralToMemory, Stream2, Stream7, StreamsTuple, Transfer,
};
use crate::hal::gpio::gpioa::{PA10, PA9};
use crate::hal::gpio::{Alternate, AF7};
use crate::hal::prelude::*;
use crate::hal::rcc::Clocks;
use crate::hal::serial::config::{
    Config as SerialConfig, DmaConfig as SerialDmaConfig, Parity, StopBits, WordLength,
};
use crate::hal::serial::{Event as SerialEvent, Serial as HalSerial};
use crate::hal::time::Bps;

use crate::interrupt;

use crate::pac::Interrupt;
use crate::pac::{DMA2, USART1};

/// Interface to the Serial peripheral
pub struct Serial<USART: PeriAddress<MemSize = u8>, TSTREAM: Stream, RSTREAM: Stream> {
    tx_stream: Option<TSTREAM>,
    rx_stream: Option<RSTREAM>,
    usart: Option<USART>,
    tx_int: interrupt::DMA2_STREAM7Interrupt,
    rx_int: interrupt::DMA2_STREAM2Interrupt,
    usart_int: interrupt::USART1Interrupt,
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
    pub fn new(
        txd: PA9<Alternate<AF7>>,
        rxd: PA10<Alternate<AF7>>,
        tx_int: interrupt::DMA2_STREAM7Interrupt,
        rx_int: interrupt::DMA2_STREAM2Interrupt,
        usart_int: interrupt::USART1Interrupt,
        dma: DMA2,
        usart: USART1,
        parity: Parity,
        baudrate: Bps,
        clocks: Clocks,
    ) -> Self {
        let mut serial = HalSerial::usart1(
            usart,
            (txd, rxd),
            SerialConfig {
                baudrate: baudrate,
                wordlength: WordLength::DataBits8,
                parity: Parity::ParityNone,
                stopbits: StopBits::STOP1,
                dma: SerialDmaConfig::TxRx,
            },
            clocks,
        )
        .unwrap();

        serial.listen(SerialEvent::Idle);
        //        serial.listen(SerialEvent::Txe);

        let (usart, _) = serial.release();

        // Register ISR
        tx_int.set_handler(Self::on_tx_irq, core::ptr::null_mut());
        rx_int.set_handler(Self::on_rx_irq, core::ptr::null_mut());
        usart_int.set_handler(Self::on_rx_irq, core::ptr::null_mut());
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

    /// Sends serial data.
    ///
    /// `tx_buffer` is marked as static as per `embedded-dma` requirements.
    /// It it safe to use a buffer with a non static lifetime if memory is not
    /// reused until the future has finished.
    pub fn send<'a, B: 'a>(&'a mut self, tx_buffer: B) -> impl Future<Output = ()> + 'a
    where
        B: StaticWriteBuffer<Word = u8>,
    {
        unsafe { INSTANCE = self };

        let tx_stream = self.tx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();
        STATE.tx_int.reset();

        async move {
            let mut tx_transfer = Transfer::init(
                tx_stream,
                usart,
                tx_buffer,
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
        }
    }

    /// Receives serial data.
    ///
    /// The future is pending until the buffer is completely filled.
    /// A common pattern is to use [`stop()`](ReceiveFuture::stop) to cancel
    /// unfinished transfers after a timeout to prevent lockup when no more data
    /// is incoming.
    ///
    /// `rx_buffer` is marked as static as per `embedded-dma` requirements.
    /// It it safe to use a buffer with a non static lifetime if memory is not
    /// reused until the future has finished.
    pub fn receive<'a, B: 'a>(&'a mut self, rx_buffer: B) -> impl Future<Output = B> + 'a
    where
        B: StaticWriteBuffer<Word = u8> + Unpin,
    {
        unsafe { INSTANCE = self };

        let rx_stream = self.rx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();
        STATE.rx_int.reset();

        async move {
            let mut rx_transfer = Transfer::init(
                rx_stream,
                usart,
                rx_buffer,
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

            let (rx_stream, usart, buf, _) = rx_transfer.free();
            self.rx_stream.replace(rx_stream);
            self.usart.replace(usart);

            buf
        }
    }
}
