use core::future::Future;
use core::marker::PhantomData;
use core::sync::atomic::{self, Ordering};

use embassy::interrupt::Interrupt;
use embassy::traits::uart::{Error, Uart};
use embassy::util::InterruptFuture;

use crate::hal::{
    dma,
    dma::config::DmaConfig,
    dma::traits::{Channel, DMASet, PeriAddress, Stream},
    dma::{MemoryToPeripheral, PeripheralToMemory, Transfer},
    i2c,
    rcc::Clocks,
    time::KiloHertz,
};
use crate::interrupt;
use crate::pac;

/// Interface to the I2C peripheral
pub struct I2C<
    I2C: PeriAddress<MemSize = u8> + WithInterrupt,
    TSTREAM: Stream + WithInterrupt,
    RSTREAM: Stream + WithInterrupt,
    CHANNEL: dma::traits::Channel,
> {
    tx_stream: Option<TSTREAM>,
    rx_stream: Option<RSTREAM>,
    i2c: Option<I2C>,
    tx_int: TSTREAM::Interrupt,
    rx_int: RSTREAM::Interrupt,
    i2c_int: I2C::Interrupt,
    channel: PhantomData<CHANNEL>,
}

// static mut INSTANCE: *const Serial<I2C1, Stream7<DMA2>, Stream2<DMA2>> = ptr::null_mut();

impl<I2C, TSTREAM, RSTREAM, CHANNEL> I2C<I2C, TSTREAM, RSTREAM, CHANNEL>
where
    I2C: i2c::Instance
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
        i2c: I2C,
        streams: (TSTREAM, RSTREAM),
        pins: PINS,
        speed: KiloHertz,
        tx_int: TSTREAM::Interrupt,
        rx_int: RSTREAM::Interrupt,
        I2C_int: I2C::Interrupt,
        mut config: SerialConfig,
        clocks: Clocks,
    ) -> Self
    where
        PINS: Pins<I2C>,
    {
        let (i2c, _) = I2C::new(i2c, pins, speed, clocks).unwrap().release();

        let (tx_stream, rx_stream) = streams;

        Self {
            tx_stream: Some(tx_stream),
            rx_stream: Some(rx_stream),
            I2C: Some(I2C),
            tx_int: tx_int,
            rx_int: rx_int,
            I2C_int: I2C_int,
            channel: core::marker::PhantomData,
        }
    }
}

impl<I2C, TSTREAM, RSTREAM, CHANNEL> I2C for I2C<I2C, TSTREAM, RSTREAM, CHANNEL>
where
    I2C: i2c::Instance
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
        let I2C = self.I2C.take().unwrap();

        async move {
            let mut tx_transfer = Transfer::init(
                tx_stream,
                I2C,
                static_buf,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );

            let fut = InterruptFuture::new(&mut self.tx_int);

            tx_transfer.start(|_I2C| {});

            fut.await;

            let (tx_stream, I2C, _buf, _) = tx_transfer.free();
            self.tx_stream.replace(tx_stream);
            self.I2C.replace(I2C);

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
        let I2C = self.I2C.take().unwrap();

        async move {
            let mut rx_transfer = Transfer::init(
                rx_stream,
                I2C,
                static_buf,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );

            let fut = InterruptFuture::new(&mut self.rx_int);

            rx_transfer.start(|_I2C| {});
            fut.await;

            let (rx_stream, I2C, _, _) = rx_transfer.free();
            self.rx_stream.replace(rx_stream);
            self.I2C.replace(I2C);

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

macro_rules! I2C {
    ($($PER:ident => ($I2C:ident),)+) => {
        $(
            impl private::Sealed for pac::$I2C {}
            impl WithInterrupt for pac::$I2C {
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
I2C! {
    I2C1 => (I2C1),
    I2C2 => (I2C2),
    I2C3 => (I2C3),
    UART4 => (UART4),
    UART5 => (UART5),
    I2C6 => (I2C6),
}
