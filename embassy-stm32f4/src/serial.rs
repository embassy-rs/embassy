//! Async low power UARTE.
//!
//! The peripheral is autmatically enabled and disabled as required to save power.
//! Lowest power consumption can only be guaranteed if the send receive futures
//! are dropped correctly (e.g. not using `mem::forget()`).

use core::cell::UnsafeCell;
use core::cmp::min;
use core::future::Future;
use core::marker::PhantomPinned;
use core::ops::Deref;
use core::pin::Pin;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};
use cortex_m::singleton;

use embassy::util::Signal;
use embedded_dma::{StaticReadBuffer, StaticWriteBuffer, WriteBuffer};

use crate::fmt::assert;
use crate::hal::dma::config::DmaConfig;
use crate::hal::dma::{
    Channel4, Channel7, MemoryToPeripheral, PeripheralToMemory, Stream2, Stream7, StreamsTuple,
    Transfer,
};
use crate::hal::gpio::gpioa::{PA10, PA9};
use crate::hal::gpio::{Alternate, AF10, AF7, AF9};
use crate::hal::gpio::{Floating, Input, Output, PushPull};
use crate::hal::pac;
use crate::hal::prelude::*;
use crate::hal::rcc::Clocks;
use crate::hal::serial::config::{
    Config as SerialConfig, DmaConfig as SerialDmaConfig, Parity, StopBits, WordLength,
};
use crate::hal::serial::{Event as SerialEvent, Serial};
use crate::hal::time::Bps;

use crate::interrupt;

use crate::pac::Interrupt;
use crate::pac::{DMA2, USART1};

use embedded_hal::digital::v2::OutputPin;

// Re-export SVD variants to allow user to directly set values.
// pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

/// Interface to the UARTE peripheral
pub struct Uarte {
    // tx_transfer: Transfer<Stream7<DMA2>, Channel4, USART1, MemoryToPeripheral, &mut [u8; 20]>,
    // rx_transfer: Transfer<Stream2<DMA2>, Channel4, USART1, PeripheralToMemory, &mut [u8; 20]>,
    tx_stream: Option<Stream7<DMA2>>,
    rx_stream: Option<Stream2<DMA2>>,
    usart: Option<USART1>,
}

struct State {
    tx_done: Signal<()>,
    rx_done: Signal<u32>,
}

static STATE: State = State {
    tx_done: Signal::new(),
    rx_done: Signal::new(),
};

impl Uarte {
    pub fn new(
        rxd: PA10<Alternate<AF7>>,
        txd: PA9<Alternate<AF7>>,
        dma: DMA2,
        usart: USART1,
        parity: Parity,
        baudrate: Bps,
        clocks: Clocks,
    ) -> Self {
        // // Enable interrupts
        // uarte.events_endtx.reset();
        // uarte.events_endrx.reset();
        // uarte
        //     .intenset
        //     .write(|w| w.endtx().set().txstopped().set().endrx().set().rxto().set());
        // // TODO: Set interrupt priority?
        // interrupt::unpend(interrupt::UARTE0_UART0);
        // interrupt::enable(interrupt::UARTE0_UART0);

        // Serial<USART1, (PA9<Alternate<AF7>>, PA10<Alternate<AF7>>)>
        let serial = Serial::usart1(
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

        let (usart, _) = serial.release();

        /*
            Note: for our application, it would be approrpiate to listen for idle events,
            and to establish a method to capture data until idle.
        */
        // serial.listen(SerialEvent::Idle);

        // tx_transfer.start(|usart| {
        //     // usart.cr2.modify(|_, w| w.swstart().start());
        // });

        let streams = StreamsTuple::new(dma);

        Uarte {
            tx_stream: Some(streams.7),
            rx_stream: Some(streams.2),
            usart: Some(usart),
        }
    }

    /// Sends serial data.
    ///
    /// `tx_buffer` is marked as static as per `embedded-dma` requirements.
    /// It it safe to use a buffer with a non static lifetime if memory is not
    /// reused until the future has finished.
    pub fn send<'a, B>(&'a mut self, tx_buffer: B) -> SendFuture<'a, B>
    where
        B: WriteBuffer<Word = u8> + 'static,
    {
        let tx_stream = self.tx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();

        SendFuture {
            uarte: self,
            tx_transfer: Transfer::init(
                tx_stream,
                usart,
                tx_buffer,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            ),
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
    pub fn receive<'a, B>(&'a mut self, rx_buffer: B) -> ReceiveFuture<'a, B>
    where
        B: WriteBuffer<Word = u8> + 'static,
    {
        let rx_stream = self.rx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();

        ReceiveFuture {
            uarte: self,
            rx_transfer: Transfer::init(
                rx_stream,
                usart,
                rx_buffer,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .half_transfer_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            ),
        }
    }
}

/// Future for the [`LowPowerUarte::send()`] method.
pub struct SendFuture<'a, B: WriteBuffer<Word = u8> + 'static> {
    uarte: &'a Uarte,
    tx_transfer: Transfer<Stream7<DMA2>, Channel4, USART1, MemoryToPeripheral, B>,
}

impl<'a, B> Drop for SendFuture<'a, B>
where
    B: WriteBuffer<Word = u8> + 'static,
{
    fn drop(self: &mut Self) {}
}

impl<'a, B> Future for SendFuture<'a, B>
where
    B: WriteBuffer<Word = u8> + 'static,
{
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.tx_transfer.is_done() {
            Poll::Ready(())
        } else {
            // self.0.as_mut().tx_transfer.start(|usart| {});

            waker_interrupt!(DMA2_STREAM7, cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Future for the [`Uarte::receive()`] method.
pub struct ReceiveFuture<'a, B: WriteBuffer<Word = u8> + 'static> {
    uarte: &'a Uarte,
    rx_transfer: Transfer<Stream2<DMA2>, Channel4, USART1, PeripheralToMemory, B>,
}

impl<'a, B> Drop for ReceiveFuture<'a, B>
where
    B: WriteBuffer<Word = u8> + 'static,
{
    fn drop(self: &mut Self) {}
}

impl<'a, B> Future for ReceiveFuture<'a, B>
where
    B: WriteBuffer<Word = u8> + 'static,
{
    type Output = B;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<B> {
        //        if !self.transfer.is_none() && self.transfer.unwrap().is_done() {
        //            Poll::Ready(self.buf.take());
        //        } else {
        //            self.transfer = Some(&mut Transfer::init(
        //                StreamsTuple::new(self.uarte.dma).2,
        //                self.uarte.usart,
        //                self.buf,
        //                None,
        //                DmaConfig::default()
        //                    .transfer_complete_interrupt(true)
        //                    .half_transfer_interrupt(true)
        //                    .memory_increment(true)
        //                    .double_buffer(false),
        //            ));

        waker_interrupt!(DMA2_STREAM2, cx.waker().clone());
        Poll::Pending
        //        }
    }
}
