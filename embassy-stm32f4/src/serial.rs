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
use crate::hal::dma::traits::{PeriAddress, Stream};
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

        SendFuture {
            uarte: self,
            tx_transfer: Some(tx_transfer),
            // tx_stream: Some(tx_stream),
            // usart: Some(usart),
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
    pub fn receive<'a, B>(
        &'a mut self,
        rx_buffer: B,
    ) -> ReceiveFuture<'a, B, Stream2<DMA2>, Channel4>
    where
        B: WriteBuffer<Word = u8> + 'static,
    {
        let rx_stream = self.rx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();
        let mut rx_transfer = Transfer::init(
            rx_stream,
            usart,
            rx_buffer,
            None,
            DmaConfig::default()
                .transfer_complete_interrupt(true)
                .half_transfer_interrupt(true)
                .memory_increment(true)
                .double_buffer(false),
        );

        ReceiveFuture {
            uarte: self,
            rx_transfer: Some(rx_transfer),
        }
    }
}

/// Future for the [`LowPowerUarte::send()`] method.
pub struct SendFuture<'a, B: WriteBuffer<Word = u8> + 'static> {
    uarte: &'a mut Uarte,
    tx_transfer: Option<Transfer<Stream7<DMA2>, Channel4, USART1, MemoryToPeripheral, B>>,
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
        let Self { uarte, tx_transfer } = unsafe { self.get_unchecked_mut() };
        let mut taken = tx_transfer.take().unwrap();
        if Stream7::<DMA2>::get_transfer_complete_flag() {
            let (tx_stream, usart, buf, _) = taken.free();

            uarte.tx_stream.replace(tx_stream);
            uarte.usart.replace(usart);

            Poll::Ready(())
        } else {
            waker_interrupt!(DMA2_STREAM7, cx.waker().clone());
            taken.start(|usart| {});
            tx_transfer.replace(taken);

            Poll::Pending
        }
    }
}

/// Future for the [`Uarte::receive()`] method.
pub struct ReceiveFuture<'a, B: WriteBuffer<Word = u8> + 'static, STREAM: Stream, CHANNEL> {
    uarte: &'a mut Uarte,
    rx_transfer: Option<Transfer<STREAM, CHANNEL, USART1, PeripheralToMemory, B>>,
}

// pub struct ReceiveFuture<'a, B: WriteBuffer<Word = u8> + 'static, DMA, STREAM, CHANNEL> {
//     uarte: &'a mut Uarte,
//     rx_transfer: Option<Transfer<Stream2<DMA2>, Channel4, USART1, PeripheralToMemory, B>>,
// }

// impl<'a, B> Drop for ReceiveFuture<'a, B, USART1, Stream7<DMA2>, Channel4>
// where
//     B: WriteBuffer<Word = u8> + 'static,
// {
//     fn drop(self: &mut Self) {}
// }

impl<'a, B> Future for ReceiveFuture<'a, B, Stream2<DMA2>, Channel4>
where
    B: WriteBuffer<Word = u8> + 'static + Unpin,
{
    type Output = B;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<B> {
        let Self { uarte, rx_transfer } = unsafe { self.get_unchecked_mut() };
        let mut taken = rx_transfer.take().unwrap();

        if Stream7::<DMA2>::get_transfer_complete_flag() {
            let (rx_stream, usart, buf, _) = rx_transfer.take().unwrap().free();

            uarte.rx_stream.replace(rx_stream);
            uarte.usart.replace(usart);

            Poll::Ready(buf)
        } else {
            waker_interrupt!(DMA2_STREAM2, cx.waker().clone());

            taken.start(|usart| {});
            rx_transfer.replace(taken);

            Poll::Pending
        }
    }
}
