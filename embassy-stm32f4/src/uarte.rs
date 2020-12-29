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
use embedded_dma::{StaticReadBuffer, StaticWriteBuffer};

use crate::fmt::assert;
use crate::hal::dma::config::DmaConfig;
use crate::hal::dma::{Channel4, PeripheralToMemory, Stream2, StreamsTuple, Transfer};
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

        let isr = pins.dma.hisr;0

        Uarte { instance: serial, dma: pins.dma, usart: pins.usart }
    }

    /// Sets the baudrate, parity and assigns the pins to the UARTE peripheral.
    // TODO: Make it take the same `Pins` structs nrf-hal (with optional RTS/CTS).
    //    // TODO: #[cfg()] for smaller device variants without port register (nrf52810, ...).
    //    pub fn configure(
    //        &mut self,
    //        rxd: &Pin<Input<Floating>>,
    //        txd: &mut Pin<Output<PushPull>>,
    //        parity: Parity,
    //        baudrate: Baudrate,
    //    ) {
    //        let uarte = &self.instance;
    //        assert!(uarte.enable.read().enable().is_disabled());
    //
    //        uarte.psel.rxd.write(|w| {
    //            let w = unsafe { w.pin().bits(rxd.pin()) };
    //            let w = w.port().bit(rxd.port().bit());
    //            w.connect().connected()
    //        });
    //
    //        txd.set_high().unwrap();
    //        uarte.psel.txd.write(|w| {
    //            let w = unsafe { w.pin().bits(txd.pin()) };
    //            let w = w.port().bit(txd.port().bit());
    //            w.connect().connected()
    //        });
    //
    //        uarte.baudrate.write(|w| w.baudrate().variant(baudrate));
    //        uarte.config.write(|w| w.parity().variant(parity));
    //    }

    //    fn enable(&mut self) {
    //        self.instance.enable.write(|w| w.enable().enabled());
    //    }

    /// Sends serial data.
    ///
    /// `tx_buffer` is marked as static as per `embedded-dma` requirements.
    /// It it safe to use a buffer with a non static lifetime if memory is not
    /// reused until the future has finished.
    pub fn send<'a, B>(&'a mut self, tx_buffer: B) -> SendFuture<'a, B>
    where
        B: StaticReadBuffer<Word = u8>,
    {
        // Panic if TX is running which can happen if the user has called
        // `mem::forget()` on a previous future after polling it once.
        assert!(!self.tx_started());

        self.enable();

        SendFuture {
            uarte: self,
            buf: tx_buffer,
        }
    }

    fn tx_started(&self) -> bool {
        // self.instance.events_txstarted.read().bits() != 0
        false
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
        B: StaticWriteBuffer<Word = u8>,
    {
        // Panic if RX is running which can happen if the user has called
        // `mem::forget()` on a previous future after polling it once.
        assert!(!self.rx_started());

        self.enable();

        ReceiveFuture {
            uarte: self,
            buf: Some(rx_buffer),
        }
    }

    fn rx_started(&self) -> bool {
        self.instance.events_rxstarted.read().bits() != 0
    }
}

/// Future for the [`LowPowerUarte::send()`] method.
pub struct SendFuture<'a, B> {
    uarte: &'a Uarte,
    buf: B,
}

impl<'a, B> Drop for SendFuture<'a, B> {
    fn drop(self: &mut Self) {
        if self.uarte.tx_started() {
            trace!("stoptx");

            // Stop the transmitter to minimize the current consumption.
            self.uarte
                .instance
                .tasks_stoptx
                .write(|w| unsafe { w.bits(1) });
            self.uarte.instance.events_txstarted.reset();
        }
    }
}

impl<'a, B> Future for SendFuture<'a, B>
where
    B: StaticReadBuffer<Word = u8>,
{
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.is_ready() {
            Poll::Ready(())
        } else {
            // Start DMA transaction
            let uarte = &self.uarte.instance;

            STATE.tx_done.reset();

            let (ptr, len) = unsafe { self.buf.read_buffer() };
            // assert!(len <= EASY_DMA_SIZE);
            // TODO: panic if buffer is not in SRAM

            compiler_fence(Ordering::SeqCst);
            // uarte.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
            // uarte
            //     .txd
            //     .maxcnt
            //     .write(|w| unsafe { w.maxcnt().bits(len as _) });

            // Start the DMA transfer
            // See https://github.com/mwkroening/async-stm32f1xx/blob/78c46d1bff124eae4ebc7a2f4d40e6ed74def8b5/src/serial.rs#L118-L129
            //     https://github.com/stm32-rs/stm32f1xx-hal/blob/68fd3d6f282173816fd3181e795988d314cb17d0/src/serial.rs#L649-L671

            // let first_buffer = singleton!(: [u8; 128] = [0; 128]).unwrap();
            // let second_buffer = singleton!(: [u8; 128] = [0; 128]).unwrap();
            // let triple_buffer = Some(singleton!(: [u8; 128] = [0; 128]).unwrap());

            let transfer = Transfer::init(
                StreamsTuple::new(self.dma).2,
                self.usart,
                self.buf,
                // Some(second_buffer),
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );

            waker_interrupt!(DMA2_STREAM2, cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Future for the [`Uarte::receive()`] method.
pub struct ReceiveFuture<'a, B> {
    uarte: &'a Uarte,
    buf: Option<B>,
}

impl<'a, B> Drop for ReceiveFuture<'a, B> {
    fn drop(self: &mut Self) {
        if self.uarte.rx_started() {
            trace!("stoprx");

            self.uarte
                .instance
                .tasks_stoprx
                .write(|w| unsafe { w.bits(1) });
            self.uarte.instance.events_rxstarted.reset();
        }
    }
}

impl<'a, B> Future for ReceiveFuture<'a, B>
where
    B: StaticWriteBuffer<Word = u8>,
{
    type Output = B;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<B> {
        if self.is_ready() {
            Poll::Ready(())
        } else {
            // Start DMA transaction
            compiler_fence(Ordering::SeqCst);
            // uarte.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
            // uarte
            //     .txd
            //     .maxcnt
            //     .write(|w| unsafe { w.maxcnt().bits(len as _) });

            // Start the DMA transfer
            // See https://github.com/mwkroening/async-stm32f1xx/blob/78c46d1bff124eae4ebc7a2f4d40e6ed74def8b5/src/serial.rs#L118-L129
            //     https://github.com/stm32-rs/stm32f1xx-hal/blob/68fd3d6f282173816fd3181e795988d314cb17d0/src/serial.rs#L649-L671

            // let first_buffer = singleton!(: [u8; 128] = [0; 128]).unwrap();
            // let second_buffer = singleton!(: [u8; 128] = [0; 128]).unwrap();
            // let triple_buffer = Some(singleton!(: [u8; 128] = [0; 128]).unwrap());

            let transfer = Transfer::init(
                StreamsTuple::new(self.dma).7,
                self.usart,
                self.buf,
                // Some(second_buffer),
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );

            waker_interrupt!(DMA2_STREAM7, cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Future for the [`receive()`] method.
impl<'a, B> ReceiveFuture<'a, B> {
    /// Stops the ongoing reception and returns the number of bytes received.
    pub async fn stop(mut self) -> (B, usize) {
        let buf = self.buf.take().unwrap();
        drop(self);
        let len = STATE.rx_done.wait().await;
        (buf, len as _)
    }
}
