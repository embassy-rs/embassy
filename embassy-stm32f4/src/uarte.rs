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
use crate::hal::serial::Serial;
use crate::hal::time::Bps;

use crate::interrupt;

use crate::pac::Interrupt;
use crate::pac::{DMA2, USART1};

use embedded_hal::digital::v2::OutputPin;

// Re-export SVD variants to allow user to directly set values.
// pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

/// Interface to the UARTE peripheral
pub struct Uarte {
    instance: Serial<USART1, (PA9<Alternate<AF7>>, PA10<Alternate<AF7>>)>,
    usart: USART1,
    dma: DMA2,
}

struct State {
    tx_done: Signal<()>,
    rx_done: Signal<u32>,
}

static STATE: State = State {
    tx_done: Signal::new(),
    rx_done: Signal::new(),
};

pub struct Pins {
    pub rxd: PA10<Alternate<AF7>>,
    pub txd: PA9<Alternate<AF7>>,
    pub dma: DMA2,
    pub usart: USART1,
}

impl Uarte {
    pub fn new(mut pins: Pins, parity: Parity, baudrate: Bps, clocks: Clocks) -> Self {
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
        let mut serial = Serial::usart1(
            pins.usart,
            (pins.txd, pins.rxd),
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

        // let is_set = dma.hifcr.read().tcif7.bit_is_set();

        Uarte {
            instance: serial,
            dma: pins.dma,
            usart: pins.usart,
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
        SendFuture {
            uarte: self,
            buf: tx_buffer,
            transfer: None,
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
        ReceiveFuture {
            uarte: self,
            buf: rx_buffer,
            transfer: None,
        }
    }
}

/// Future for the [`LowPowerUarte::send()`] method.
pub struct SendFuture<'a, B: WriteBuffer<Word = u8> + 'static> {
    uarte: &'a Uarte,
    transfer: Option<&'a Transfer<Stream7<DMA2>, Channel4, USART1, MemoryToPeripheral, B>>,
    buf: B,
}

impl<'a, B> Drop for SendFuture<'a, B>
where
    B: WriteBuffer<Word = u8> + 'static,
{
    fn drop(self: &mut Self) {
        drop(self.transfer);
    }
}

impl<'a, B> Future for SendFuture<'a, B>
where
    B: WriteBuffer<Word = u8> + 'static,
{
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if !self.transfer.is_none() && self.transfer.unwrap().is_done() {
            Poll::Ready(())
        } else {
            self.transfer = Some(&mut Transfer::init(
                StreamsTuple::new(self.uarte.dma).7,
                self.uarte.usart,
                self.buf,
                // Some(second_buffer),
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            ));

            waker_interrupt!(DMA2_STREAM7, cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Future for the [`Uarte::receive()`] method.
pub struct ReceiveFuture<'a, B: WriteBuffer<Word = u8> + 'static> {
    uarte: &'a Uarte,
    transfer: Option<&'a Transfer<Stream2<DMA2>, Channel4, USART1, PeripheralToMemory, B>>,
    buf: B,
}

impl<'a, B> Drop for ReceiveFuture<'a, B>
where
    B: WriteBuffer<Word = u8> + 'static,
{
    fn drop(self: &mut Self) {
        drop(self.transfer);
    }
}

impl<'a, B> Future for ReceiveFuture<'a, B>
where
    B: WriteBuffer<Word = u8> + 'static,
{
    type Output = B;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<B> {
        if !self.transfer.is_none() && self.transfer.unwrap().is_done() {
            Poll::Ready(self.buf.take());
        } else {
            self.transfer = Some(&mut Transfer::init(
                StreamsTuple::new(self.uarte.dma).2,
                self.uarte.usart,
                self.buf,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .half_transfer_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            ));

            waker_interrupt!(DMA2_STREAM2, cx.waker().clone());
            Poll::Pending
        }
    }
}
