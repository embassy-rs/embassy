use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{self, Ordering};

use embassy::interrupt::Interrupt;
use embassy::traits::i2c::{I2c, SevenBitAddress};
use embassy::util::InterruptFuture;

use crate::hal::{
    dma,
    dma::config::DmaConfig,
    dma::traits::{Channel, DMASet, PeriAddress, Stream},
    dma::{MemoryToPeripheral, PeripheralToMemory, Transfer},
    i2c,
    i2c::Pins,
    rcc::Clocks,
    time::KiloHertz,
};
use crate::interrupt;
use crate::pac;

pub enum Error {}

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

impl<TI2C, TSTREAM, RSTREAM, CHANNEL> I2C<TI2C, TSTREAM, RSTREAM, CHANNEL>
where
    TI2C: i2c::Instance
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
        i2c: TI2C,
        streams: (TSTREAM, RSTREAM),
        pins: PINS,
        speed: KiloHertz,
        clocks: Clocks,
        tx_int: TSTREAM::Interrupt,
        rx_int: RSTREAM::Interrupt,
        i2c_int: TI2C::Interrupt,
    ) -> Self
    where
        PINS: Pins<TI2C>,
    {
        let (i2c, _) = i2c::I2c::new(i2c, pins, speed, clocks).release();

        i2c.cr2.write(|w| w.dmaen().set_bit());

        let (tx_stream, rx_stream) = streams;

        Self {
            tx_stream: Some(tx_stream),
            rx_stream: Some(rx_stream),
            i2c: Some(i2c),
            tx_int: tx_int,
            rx_int: rx_int,
            i2c_int: i2c_int,
            channel: core::marker::PhantomData,
        }
    }
}

impl<TI2C, TSTREAM, RSTREAM, CHANNEL> I2c for I2C<TI2C, TSTREAM, RSTREAM, CHANNEL>
where
    TI2C: i2c::Instance
        + PeriAddress<MemSize = u8>
        + DMASet<TSTREAM, CHANNEL, MemoryToPeripheral>
        + DMASet<RSTREAM, CHANNEL, PeripheralToMemory>
        + WithInterrupt
        + 'static,
    TSTREAM: Stream + WithInterrupt + 'static,
    RSTREAM: Stream + WithInterrupt + 'static,
    CHANNEL: Channel + 'static,
{
    /// Error type
    type Error = Error;

    type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
    type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
    type WriteReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(
        self: Pin<&'a mut Self>,
        address: SevenBitAddress,
        buffer: &mut [u8],
    ) -> Self::ReadFuture<'a> {
        let s = unsafe { self.get_unchecked_mut() };

        let rx_stream = s.rx_stream.take().unwrap();
        let i2c = s.i2c.take().unwrap();
        async move {
            s.rx_stream.replace(rx_stream);
            s.i2c.replace(i2c);

            Ok(())
        }
    }

    fn write<'a>(
        self: Pin<&'a mut Self>,
        address: SevenBitAddress,
        bytes: &[u8],
    ) -> Self::WriteFuture<'a> {
        let s = unsafe { self.get_unchecked_mut() };

        #[allow(mutable_transmutes)]
        let static_buf: &'static mut [u8] = unsafe { core::mem::transmute(bytes) };

        let tx_stream = s.tx_stream.take().unwrap();
        let i2c = s.i2c.take().unwrap();
        async move {
            let mut tx_transfer = Transfer::init(
                tx_stream,
                i2c,
                static_buf,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );

            let fut = InterruptFuture::new(&mut s.tx_int);

            tx_transfer.start(|_i2c| {});

            fut.await;

            let fut = InterruptFuture::new(&mut s.i2c_int);
            // Send a STOP condition
            s.i2c.cr1.modify(|_, w| w.stop().set_bit());

            // Wait for STOP condition to transmit.
            fut.await;

            let (tx_stream, i2c, _buf, _) = tx_transfer.free();
            s.tx_stream.replace(tx_stream);
            s.i2c.replace(i2c);

            Ok(())
        }
    }

    fn write_read<'a>(
        self: Pin<&'a mut Self>,
        address: SevenBitAddress,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Self::WriteReadFuture<'a> {
        async move { Ok(()) }
    }
}

// impl<I2C, TSTREAM, RSTREAM, CHANNEL> I2c for I2C<I2C, TSTREAM, RSTREAM, CHANNEL>
// where
//     I2C: i2c::Instance
//         + PeriAddress<MemSize = u8>
//         + DMASet<TSTREAM, CHANNEL, MemoryToPeripheral>
//         + DMASet<RSTREAM, CHANNEL, PeripheralToMemory>
//         + WithInterrupt
//         + 'static,
//     TSTREAM: Stream + WithInterrupt + 'static,
//     RSTREAM: Stream + WithInterrupt + 'static,
//     CHANNEL: Channel + 'static,
// {
//     type SendFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
//     type ReceiveFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
//
//     /// Sends serial data.
//     fn send<'a>(&'a mut self, buf: &'a [u8]) -> Self::SendFuture<'a> {
//         #[allow(mutable_transmutes)]
//         let static_buf = unsafe { core::mem::transmute::<&'a [u8], &'static mut [u8]>(buf) };
//
//         let tx_stream = self.tx_stream.take().unwrap();
//         let I2C = self.I2C.take().unwrap();
//
//         async move {
//             let mut tx_transfer = Transfer::init(
//                 tx_stream,
//                 I2C,
//                 static_buf,
//                 None,
//                 DmaConfig::default()
//                     .transfer_complete_interrupt(true)
//                     .memory_increment(true)
//                     .double_buffer(false),
//             );
//
//             let fut = InterruptFuture::new(&mut self.tx_int);
//
//             tx_transfer.start(|_I2C| {});
//
//             fut.await;
//
//             let (tx_stream, I2C, _buf, _) = tx_transfer.free();
//             self.tx_stream.replace(tx_stream);
//             self.I2C.replace(I2C);
//
//             Ok(())
//         }
//     }
//
//     /// Receives serial data.
//     ///
//     /// The future is pending until the buffer is completely filled.
//     /// A common pattern is to use [`stop()`](ReceiveFuture::stop) to cancel
//     /// unfinished transfers after a timeout to prevent lockup when no more data
//     /// is incoming.
//     fn receive<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReceiveFuture<'a> {
//         let static_buf = unsafe { core::mem::transmute::<&'a mut [u8], &'static mut [u8]>(buf) };
//
//         let rx_stream = self.rx_stream.take().unwrap();
//         let I2C = self.I2C.take().unwrap();
//
//         async move {
//             let mut rx_transfer = Transfer::init(
//                 rx_stream,
//                 I2C,
//                 static_buf,
//                 None,
//                 DmaConfig::default()
//                     .transfer_complete_interrupt(true)
//                     .memory_increment(true)
//                     .double_buffer(false),
//             );
//
//             let fut = InterruptFuture::new(&mut self.rx_int);
//
//             rx_transfer.start(|_I2C| {});
//             fut.await;
//
//             let (rx_stream, I2C, _, _) = rx_transfer.free();
//             self.rx_stream.replace(rx_stream);
//             self.I2C.replace(I2C);
//
//             Ok(())
//         }
//     }
// }

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

macro_rules! i2c {
    ($($INT:ident => ($i2c:ident),)+) => {
        $(
            impl private::Sealed for pac::$i2c {}
            impl WithInterrupt for pac::$i2c {
                type Interrupt = interrupt::$INT;
            }
        )+
    }
}

i2c! {
    I2C1 => (I2C1_EV),
    I2C2 => (I2C2_EV),
    I2C3 => (I2C3_EV),
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
