//! Async Serial.

use core::future::Future;
use core::marker::PhantomData;
use embassy::interrupt::Interrupt;
use embassy::traits::uart::{Error, Read, ReadUntilIdle, Write};
use embassy::util::InterruptFuture;
use futures::{select_biased, FutureExt};

use crate::hal::{
    dma,
    dma::config::DmaConfig,
    dma::traits::{Channel, DMASet, PeriAddress, Stream},
    dma::{MemoryToPeripheral, PeripheralToMemory, Transfer},
    rcc::Clocks,
    serial,
    serial::config::{Config as SerialConfig, DmaConfig as SerialDmaConfig},
    serial::{Event as SerialEvent, Pins},
};
use crate::interrupt;
use crate::pac;

/// Interface to the Serial peripheral
pub struct Serial<
    USART: PeriAddress<MemSize = u8> + WithInterrupt,
    TSTREAM: Stream + WithInterrupt,
    RSTREAM: Stream + WithInterrupt,
    CHANNEL: Channel,
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

        let (usart, _) = serial::Serial::new(usart, pins, config, clocks)
            .unwrap()
            .release();

        let (tx_stream, rx_stream) = streams;

        Serial {
            tx_stream: Some(tx_stream),
            rx_stream: Some(rx_stream),
            usart: Some(usart),
            tx_int: tx_int,
            rx_int: rx_int,
            usart_int: usart_int,
            channel: PhantomData,
        }
    }
}

impl<USART, TSTREAM, RSTREAM, CHANNEL> Read for Serial<USART, TSTREAM, RSTREAM, CHANNEL>
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
    type ReadFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;

    /// Receives serial data.
    ///
    /// The future is pending until the buffer is completely filled.
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        let static_buf = unsafe { core::mem::transmute::<&'a mut [u8], &'static mut [u8]>(buf) };

        async move {
            let rx_stream = self.rx_stream.take().unwrap();
            let usart = self.usart.take().unwrap();

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

impl<USART, TSTREAM, RSTREAM, CHANNEL> Write for Serial<USART, TSTREAM, RSTREAM, CHANNEL>
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
    type WriteFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;

    /// Sends serial data.
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        #[allow(mutable_transmutes)]
        let static_buf = unsafe { core::mem::transmute::<&'a [u8], &'static mut [u8]>(buf) };

        async move {
            let tx_stream = self.tx_stream.take().unwrap();
            let usart = self.usart.take().unwrap();

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
}

impl<USART, TSTREAM, RSTREAM, CHANNEL> ReadUntilIdle for Serial<USART, TSTREAM, RSTREAM, CHANNEL>
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
    type ReadUntilIdleFuture<'a> = impl Future<Output = Result<usize, Error>> + 'a;

    /// Receives serial data.
    ///
    /// The future is pending until either the buffer is completely full, or the RX line falls idle after receiving some data.
    ///
    /// Returns the number of bytes read.
    fn read_until_idle<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadUntilIdleFuture<'a> {
        let static_buf = unsafe { core::mem::transmute::<&'a mut [u8], &'static mut [u8]>(buf) };

        async move {
            let rx_stream = self.rx_stream.take().unwrap();
            let usart = self.usart.take().unwrap();

            unsafe {
                /*  __HAL_UART_ENABLE_IT(&uart->UartHandle, UART_IT_IDLE); */
                (*USART::ptr()).cr1.modify(|_, w| w.idleie().set_bit());

                /* __HAL_UART_CLEAR_IDLEFLAG(&uart->UartHandle); */
                (*USART::ptr()).sr.read();
                (*USART::ptr()).dr.read();
            };

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

            let total_bytes = RSTREAM::get_number_of_transfers() as usize;

            let fut = InterruptFuture::new(&mut self.rx_int);
            let fut_idle = InterruptFuture::new(&mut self.usart_int);

            rx_transfer.start(|_usart| {});

            futures::future::select(fut, fut_idle).await;

            let (rx_stream, usart, _, _) = rx_transfer.free();

            let remaining_bytes = RSTREAM::get_number_of_transfers() as usize;

            unsafe {
                (*USART::ptr()).cr1.modify(|_, w| w.idleie().clear_bit());
            }
            self.rx_stream.replace(rx_stream);
            self.usart.replace(usart);

            Ok(total_bytes - remaining_bytes)
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

#[cfg(any(feature = "stm32f401", feature = "stm32f410", feature = "stm32f411",))]
usart! {
    USART1 => (USART1),
    USART2 => (USART2),
    USART6 => (USART6),
}

#[cfg(any(feature = "stm32f405", feature = "stm32f407"))]
usart! {
    USART1 => (USART1),
    USART2 => (USART2),
    USART3 => (USART3),
    USART6 => (USART6),

    UART4 => (UART4),
    UART5 => (UART5),
}

#[cfg(feature = "stm32f412")]
usart! {
    USART1 => (USART1),
    USART2 => (USART2),
    USART3 => (USART3),
    USART6 => (USART6),
}

#[cfg(feature = "stm32f413")]
usart! {
    USART1 => (USART1),
    USART2 => (USART2),
    USART3 => (USART3),
    USART6 => (USART6),
    USART7 => (USART7),
    USART8 => (USART8),

    UART5 => (UART5),
    UART9 => (UART9),
    UART10 => (UART10),
}

#[cfg(any(
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f446",
    feature = "stm32f469"
))]
usart! {
    USART1 => (USART1),
    USART2 => (USART2),
    USART3 => (USART3),
    USART6 => (USART6),

    UART4 => (UART4),
    UART5 => (UART5),
//    UART7 => (UART7),
//    UART8 => (UART8),
}
