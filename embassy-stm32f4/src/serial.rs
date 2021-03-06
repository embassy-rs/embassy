//! Async low power Serial.
//!
//! The peripheral is autmatically enabled and disabled as required to save power.
//! Lowest power consumption can only be guaranteed if the send receive futures
//! are dropped correctly (e.g. not using `mem::forget()`).

use core::future::Future;
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{self, Ordering};

use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::traits::uart::{Error, Uart};
use embassy::util::InterruptFuture;

use crate::hal::{
    dma,
    dma::config::DmaConfig,
    dma::traits::{Channel, DMASet, PeriAddress, Stream},
    dma::{MemoryToPeripheral, PeripheralToMemory, Transfer},
    rcc::Clocks,
    serial,
    serial::config::{Config as SerialConfig, DmaConfig as SerialDmaConfig},
    serial::{Event as SerialEvent, Pins, Serial as HalSerial},
};
use crate::interrupt;
use crate::pac;

/// Interface to the Serial peripheral
pub struct Serial<
    USART: PeriAddress<MemSize = u8> + WithInterrupt,
    TSTREAM: Stream + WithInterrupt,
    RSTREAM: Stream + WithInterrupt,
    CHANNEL: dma::traits::Channel,
> {
    tx_stream: Option<TSTREAM>,
    rx_stream: Option<RSTREAM>,
    usart: Option<USART>,
    tx_int: TSTREAM::Interrupt,
    rx_int: RSTREAM::Interrupt,
    usart_int: USART::Interrupt,
    channel: PhantomData<CHANNEL>,
}

// static mut INSTANCE: *const Serial<USART1, Stream7<DMA2>, Stream2<DMA2>> = ptr::null_mut();

impl<USART, TSTREAM, RSTREAM, CHANNEL> Serial<USART, TSTREAM, RSTREAM, CHANNEL>
where
    USART: serial::Instance
        + PeriAddress<MemSize = u8>
        + DMASet<TSTREAM, CHANNEL, MemoryToPeripheral>
        + DMASet<RSTREAM, CHANNEL, PeripheralToMemory>
        + WithInterrupt,
    TSTREAM: Stream + WithInterrupt,
    RSTREAM: Stream + WithInterrupt,
    CHANNEL: Channel,
{
    // Leaking futures is forbidden!
    pub unsafe fn new<PINS>(
        usart: USART,
        streams: (TSTREAM, RSTREAM),
        pins: PINS,
        tx_int: TSTREAM::Interrupt,
        rx_int: RSTREAM::Interrupt,
        usart_int: USART::Interrupt,
        mut config: SerialConfig,
        clocks: Clocks,
    ) -> Self
    where
        PINS: Pins<USART>,
    {
        config.dma = SerialDmaConfig::TxRx;
        let mut serial = HalSerial::new(usart, pins, config, clocks).unwrap();

        serial.listen(SerialEvent::Idle);
        //        serial.listen(SerialEvent::Txe);

        let (usart, _) = serial.release();

        let (tx_stream, rx_stream) = streams;

        Serial {
            tx_stream: Some(tx_stream),
            rx_stream: Some(rx_stream),
            usart: Some(usart),
            tx_int: tx_int,
            rx_int: rx_int,
            usart_int: usart_int,
            channel: core::marker::PhantomData,
        }
    }
}

impl<USART, TSTREAM, RSTREAM, CHANNEL> Uart for Serial<USART, TSTREAM, RSTREAM, CHANNEL>
where
    USART: serial::Instance
        + PeriAddress<MemSize = u8>
        + DMASet<TSTREAM, CHANNEL, MemoryToPeripheral>
        + DMASet<RSTREAM, CHANNEL, PeripheralToMemory>
        + WithInterrupt
        + 'static,
    TSTREAM: Stream + WithInterrupt + 'static,
    RSTREAM: Stream + WithInterrupt + 'static,
    CHANNEL: Channel + 'static,
{
    type SendFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
    type ReceiveFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;

    /// Sends serial data.
    fn send<'a>(&'a mut self, buf: &'a [u8]) -> Self::SendFuture<'a> {
        #[allow(mutable_transmutes)]
        let static_buf = unsafe { core::mem::transmute::<&'a [u8], &'static mut [u8]>(buf) };

        let tx_stream = self.tx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();

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

            let fut = InterruptFuture::new(&mut self.tx_int);

            tx_transfer.start(|_usart| {});

            fut.await;

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
        let static_buf = unsafe { core::mem::transmute::<&'a mut [u8], &'static mut [u8]>(buf) };

        let rx_stream = self.rx_stream.take().unwrap();
        let usart = self.usart.take().unwrap();

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

            let fut = InterruptFuture::new(&mut self.rx_int);

            rx_transfer.start(|_usart| {});
            fut.await;

            let (rx_stream, usart, _, _) = rx_transfer.free();
            self.rx_stream.replace(rx_stream);
            self.usart.replace(usart);

            Ok(())
        }
    }
}

mod private {
    pub trait Sealed {}
}

pub trait WithInterrupt: private::Sealed {
    type Interrupt: Interrupt;
}

macro_rules! dma {
     ($($PER:ident => ($dma:ident, $stream:ident),)+) => {
         $(
             impl private::Sealed for dma::$stream<pac::$dma> {}
             impl WithInterrupt for dma::$stream<pac::$dma> {
                 type Interrupt = interrupt::$PER;
             }
         )+
     }
 }

macro_rules! usart {
    ($($PER:ident => ($usart:ident),)+) => {
        $(
            impl private::Sealed for pac::$usart {}
            impl WithInterrupt for pac::$usart {
                type Interrupt = interrupt::$PER;
            }
        )+
    }
}

#[cfg(any(feature = "stm32f405",))]
dma! {
    DMA2_STREAM0 => (DMA2, Stream0),
    DMA2_STREAM1 => (DMA2, Stream1),
    DMA2_STREAM2 => (DMA2, Stream2),
    DMA2_STREAM3 => (DMA2, Stream3),
    DMA2_STREAM4 => (DMA2, Stream4),
    DMA2_STREAM5 => (DMA2, Stream5),
    DMA2_STREAM6 => (DMA2, Stream6),
    DMA2_STREAM7 => (DMA2, Stream7),
    DMA1_STREAM0 => (DMA1, Stream0),
    DMA1_STREAM1 => (DMA1, Stream1),
    DMA1_STREAM2 => (DMA1, Stream2),
    DMA1_STREAM3 => (DMA1, Stream3),
    DMA1_STREAM4 => (DMA1, Stream4),
    DMA1_STREAM5 => (DMA1, Stream5),
    DMA1_STREAM6 => (DMA1, Stream6),
}

#[cfg(any(feature = "stm32f405",))]
usart! {
    USART1 => (USART1),
    USART2 => (USART2),
    USART3 => (USART3),
    UART4 => (UART4),
    UART5 => (UART5),
    USART6 => (USART6),
}
