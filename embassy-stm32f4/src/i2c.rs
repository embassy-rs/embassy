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
    I2C: PeriAddress<MemSize = u8> + WithTwoInterrupts,
    TSTREAM: Stream + WithInterrupt,
    RSTREAM: Stream + WithInterrupt,
    CHANNEL: Channel,
> {
    tx_stream: Option<TSTREAM>,
    rx_stream: Option<RSTREAM>,
    i2c: Option<I2C>,
    tx_int: TSTREAM::Interrupt,
    rx_int: RSTREAM::Interrupt,
    i2c_tint: I2C::TInterrupt,
    i2c_eint: I2C::EInterrupt,
    channel: PhantomData<CHANNEL>,
}

// static mut INSTANCE: *const Serial<I2C1, Stream7<DMA2>, Stream2<DMA2>> = ptr::null_mut();

impl<TI2C, TSTREAM, RSTREAM, CHANNEL> I2C<TI2C, TSTREAM, RSTREAM, CHANNEL>
where
    TI2C: i2c::Instance
        + PeriAddress<MemSize = u8>
        + DMASet<TSTREAM, CHANNEL, MemoryToPeripheral>
        + DMASet<RSTREAM, CHANNEL, PeripheralToMemory>
        + WithTwoInterrupts,
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
        i2c_tint: TI2C::TInterrupt,
        i2c_eint: TI2C::EInterrupt,
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
            i2c_tint: i2c_tint,
            i2c_eint: i2c_eint,
            channel: PhantomData,
        }
    }
}

impl<TI2C, TSTREAM, RSTREAM, CHANNEL> I2c for I2C<TI2C, TSTREAM, RSTREAM, CHANNEL>
where
    TI2C: i2c::Instance
        + PeriAddress<MemSize = u8>
        + DMASet<TSTREAM, CHANNEL, MemoryToPeripheral>
        + DMASet<RSTREAM, CHANNEL, PeripheralToMemory>
        + WithTwoInterrupts
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

        let static_buf: &'static mut [u8] = unsafe { core::mem::transmute(buffer) };

        let rx_stream = s.rx_stream.take().unwrap();
        let i2c = s.i2c.take().unwrap();
        async move {
            // Send a START condition and set ACK bit
            i2c.cr1.modify(|_, w| w.start().set_bit().ack().set_bit());

            // Wait until START condition was generated
            loop {
                let fut = InterruptFuture::new(&mut s.i2c_tint);
                if !i2c.sr1.read().sb().bit_is_clear() {
                    break;
                }
                fut.await;
            }

            // Also wait until signalled we're master and everything is waiting for us
            loop {
                let fut = InterruptFuture::new(&mut s.i2c_tint);
                let sr2 = i2c.sr2.read();
                if !(sr2.msl().bit_is_clear() && sr2.busy().bit_is_clear()) {
                    break;
                }
                fut.await;
            }

            // Set up current address, we're trying to talk to
            i2c.dr
                .write(|w| unsafe { w.bits((u32::from(address) << 1) + 1) });

            loop {
                let fut = InterruptFuture::new(&mut s.i2c_tint);
                let sr2 = i2c.sr2.read();
                // self.check_and_clear_error_flags()?;

                if !(i2c.sr1.read().addr().bit_is_clear()) {
                    break;
                }
                fut.await;
            }

            // Clear condition by reading SR2
            i2c.sr2.read();

            let mut rx_transfer = Transfer::init(
                rx_stream,
                i2c,
                static_buf,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );

            let fut = InterruptFuture::new(&mut s.rx_int);

            rx_transfer.start(|_i2c| {});
            fut.await;
            let (rx_stream, i2c, _, _) = rx_transfer.free();

            // Prepare to send NACK then STOP after next byte
            i2c.cr1.modify(|_, w| w.ack().clear_bit().stop().set_bit());

            loop {
                let fut = InterruptFuture::new(&mut s.i2c_tint);
                if !i2c.cr1.read().stop().bit_is_set() {
                    break;
                }
                fut.await;
            }

            s.rx_stream.replace(rx_stream);
            s.i2c.replace(i2c);

            Ok(())
        }
    }

    fn write<'a>(
        self: Pin<&'a mut Self>,
        address: SevenBitAddress,
        buffer: &[u8],
    ) -> Self::WriteFuture<'a> {
        let s = unsafe { self.get_unchecked_mut() };

        #[allow(mutable_transmutes)]
        let static_buf: &'static mut [u8] = unsafe { core::mem::transmute(buffer) };

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

            let (tx_stream, i2c, _buf, _) = tx_transfer.free();

            // Send a STOP condition
            i2c.cr1.modify(|_, w| w.stop().set_bit());

            // Wait for STOP condition to transmit.
            loop {
                let fut = InterruptFuture::new(&mut s.i2c_tint);
                if !i2c.cr1.read().stop().bit_is_set() {
                    break;
                }
                fut.await;
            }

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

mod private {
    pub trait Sealed {}
}

pub trait WithInterrupt: private::Sealed {
    type Interrupt: Interrupt;
}

pub trait WithTwoInterrupts: private::Sealed {
    type TInterrupt: Interrupt;
    type EInterrupt: Interrupt;
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
    ($(($TINT:ident, $EINT:ident) => ($i2c:ident),)+) => {
        $(
            impl private::Sealed for pac::$i2c {}
            impl WithTwoInterrupts for pac::$i2c {
                type TInterrupt = interrupt::$TINT;
                type EInterrupt = interrupt::$EINT;
            }
        )+
    }
}

i2c! {
    (I2C1_EV, I2C1_ER) => (I2C1),
    (I2C2_EV, I2C2_ER) => (I2C2),
    (I2C3_EV, I2C3_ER) => (I2C3),
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
