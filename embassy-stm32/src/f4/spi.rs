//! Async SPI

use embassy::time;

use core::{future::Future, marker::PhantomData, mem, ops::Deref, pin::Pin, ptr};
use embassy::{interrupt::Interrupt, traits::spi::FullDuplex, util::InterruptFuture};
use nb;

pub use crate::hal::spi::{Mode, Phase, Polarity};
use crate::hal::{
    bb, dma,
    dma::config::DmaConfig,
    dma::traits::{Channel, DMASet, PeriAddress, Stream},
    dma::{MemoryToPeripheral, PeripheralToMemory, Transfer},
    rcc::Clocks,
    spi::Pins,
    time::Hertz,
};
use crate::interrupt;
use crate::pac;
use futures::future;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    Overrun,
    ModeFault,
    Crc,
}

fn read_sr<T: Instance>(spi: &T) -> nb::Result<u8, Error> {
    let sr = spi.sr.read();
    Err(if sr.ovr().bit_is_set() {
        nb::Error::Other(Error::Overrun)
    } else if sr.modf().bit_is_set() {
        nb::Error::Other(Error::ModeFault)
    } else if sr.crcerr().bit_is_set() {
        nb::Error::Other(Error::Crc)
    } else if sr.rxne().bit_is_set() {
        // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
        // reading a half-word)
        return Ok(unsafe { ptr::read_volatile(&spi.dr as *const _ as *const u8) });
    } else {
        nb::Error::WouldBlock
    })
}

fn write_sr<T: Instance>(spi: &T, byte: u8) -> nb::Result<(), Error> {
    let sr = spi.sr.read();
    Err(if sr.ovr().bit_is_set() {
        // Read from the DR to clear the OVR bit
        let _ = spi.dr.read();
        nb::Error::Other(Error::Overrun)
    } else if sr.modf().bit_is_set() {
        // Write to CR1 to clear MODF
        spi.cr1.modify(|_r, w| w);
        nb::Error::Other(Error::ModeFault)
    } else if sr.crcerr().bit_is_set() {
        // Clear the CRCERR bit
        spi.sr.modify(|_r, w| {
            w.crcerr().clear_bit();
            w
        });
        nb::Error::Other(Error::Crc)
    } else if sr.txe().bit_is_set() {
        // NOTE(write_volatile) see note above
        unsafe { ptr::write_volatile(&spi.dr as *const _ as *mut u8, byte) }
        return Ok(());
    } else {
        nb::Error::WouldBlock
    })
}

/// Interface to the Serial peripheral
pub struct Spi<
    SPI: PeriAddress<MemSize = u8> + WithInterrupt,
    TSTREAM: Stream + WithInterrupt,
    RSTREAM: Stream + WithInterrupt,
    CHANNEL: Channel,
> {
    tx_stream: Option<TSTREAM>,
    rx_stream: Option<RSTREAM>,
    spi: Option<SPI>,
    tx_int: TSTREAM::Interrupt,
    rx_int: RSTREAM::Interrupt,
    spi_int: SPI::Interrupt,
    channel: PhantomData<CHANNEL>,
}

impl<SPI, TSTREAM, RSTREAM, CHANNEL> Spi<SPI, TSTREAM, RSTREAM, CHANNEL>
where
    SPI: Instance
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
        spi: SPI,
        streams: (TSTREAM, RSTREAM),
        pins: PINS,
        tx_int: TSTREAM::Interrupt,
        rx_int: RSTREAM::Interrupt,
        spi_int: SPI::Interrupt,
        mode: Mode,
        freq: Hertz,
        clocks: Clocks,
    ) -> Self
    where
        PINS: Pins<SPI>,
    {
        let (tx_stream, rx_stream) = streams;

        //        let spi1: crate::pac::SPI1 = unsafe { mem::transmute(()) };
        //        let mut hspi = crate::hal::spi::Spi::spi1(
        //            spi1,
        //            (
        //                crate::hal::spi::NoSck,
        //                crate::hal::spi::NoMiso,
        //                crate::hal::spi::NoMosi,
        //            ),
        //            mode,
        //            freq,
        //            clocks,
        //        );

        unsafe { SPI::enable_clock() };

        let clock = SPI::clock_speed(clocks);

        // disable SS output
        // spi.cr2
        //     .write(|w| w.ssoe().clear_bit().rxdmaen().set_bit().txdmaen().set_bit());
        spi.cr2.write(|w| w.ssoe().clear_bit());

        let br = match clock.0 / freq.0 {
            0 => unreachable!(),
            1..=2 => 0b000,
            3..=5 => 0b001,
            6..=11 => 0b010,
            12..=23 => 0b011,
            24..=47 => 0b100,
            48..=95 => 0b101,
            96..=191 => 0b110,
            _ => 0b111,
        };

        // mstr: master configuration
        // lsbfirst: MSB first
        // ssm: enable software slave management (NSS pin free for other uses)
        // ssi: set nss high = master mode
        // dff: 8 bit frames
        // bidimode: 2-line unidirectional
        // spe: enable the SPI bus
        spi.cr1.write(|w| {
            w.cpha()
                .bit(mode.phase == Phase::CaptureOnSecondTransition)
                .cpol()
                .bit(mode.polarity == Polarity::IdleHigh)
                .mstr()
                .set_bit()
                .br()
                .bits(br)
                .lsbfirst()
                .clear_bit()
                .ssm()
                .set_bit()
                .ssi()
                .set_bit()
                .rxonly()
                .clear_bit()
                .dff()
                .clear_bit()
                .bidimode()
                .clear_bit()
                .spe()
                .set_bit()
        });

        Self {
            tx_stream: Some(tx_stream),
            rx_stream: Some(rx_stream),
            spi: Some(spi),
            tx_int: tx_int,
            rx_int: rx_int,
            spi_int: spi_int,
            channel: PhantomData,
        }
    }
}

impl<SPI, TSTREAM, RSTREAM, CHANNEL> FullDuplex<u8> for Spi<SPI, TSTREAM, RSTREAM, CHANNEL>
where
    SPI: Instance
        + PeriAddress<MemSize = u8>
        + DMASet<TSTREAM, CHANNEL, MemoryToPeripheral>
        + DMASet<RSTREAM, CHANNEL, PeripheralToMemory>
        + WithInterrupt
        + 'static,
    TSTREAM: Stream + WithInterrupt + 'static,
    RSTREAM: Stream + WithInterrupt + 'static,
    CHANNEL: Channel + 'static,
{
    type Error = Error;

    type WriteFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
    type ReadFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
    type WriteReadFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;

    fn read<'a>(self: Pin<&'a mut Self>, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        let this = unsafe { self.get_unchecked_mut() };
        #[allow(mutable_transmutes)]
        let static_buf: &'static mut [u8] = unsafe { mem::transmute(buf) };

        async move {
            let rx_stream = this.rx_stream.take().unwrap();
            let spi = this.spi.take().unwrap();

            spi.cr2.modify(|_, w| w.errie().set_bit());

            let mut rx_transfer = Transfer::init(
                rx_stream,
                spi,
                static_buf,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(true)
                    .memory_increment(true)
                    .double_buffer(false),
            );

            let fut = InterruptFuture::new(&mut this.rx_int);
            let fut_err = InterruptFuture::new(&mut this.spi_int);

            rx_transfer.start(|_spi| {});
            future::select(fut, fut_err).await;

            let (rx_stream, spi, _buf, _) = rx_transfer.free();

            spi.cr2.modify(|_, w| w.errie().clear_bit());
            this.rx_stream.replace(rx_stream);
            this.spi.replace(spi);

            Ok(())
        }
    }

    fn write<'a>(self: Pin<&'a mut Self>, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        let this = unsafe { self.get_unchecked_mut() };
        #[allow(mutable_transmutes)]
        let static_buf: &'static mut [u8] = unsafe { mem::transmute(buf) };

        async move {
            let tx_stream = this.tx_stream.take().unwrap();
            let spi = this.spi.take().unwrap();

            //            let mut tx_transfer = Transfer::init(
            //                tx_stream,
            //                spi,
            //                static_buf,
            //                None,
            //                DmaConfig::default()
            //                    .transfer_complete_interrupt(true)
            //                    .memory_increment(true)
            //                    .double_buffer(false),
            //            );
            //
            //            let fut = InterruptFuture::new(&mut this.tx_int);
            //
            //            tx_transfer.start(|_spi| {});
            //            fut.await;

            // let (tx_stream, spi, _buf, _) = tx_transfer.free();

            for i in 0..(static_buf.len() - 1) {
                let byte = static_buf[i];
                nb::block!(write_sr(&spi, byte));
            }

            this.tx_stream.replace(tx_stream);
            this.spi.replace(spi);

            Ok(())
        }
    }

    fn read_write<'a>(
        self: Pin<&'a mut Self>,
        read_buf: &'a mut [u8],
        write_buf: &'a [u8],
    ) -> Self::WriteReadFuture<'a> {
        let this = unsafe { self.get_unchecked_mut() };

        #[allow(mutable_transmutes)]
        let write_static_buf: &'static mut [u8] = unsafe { mem::transmute(write_buf) };
        let read_static_buf: &'static mut [u8] = unsafe { mem::transmute(read_buf) };

        async move {
            let tx_stream = this.tx_stream.take().unwrap();
            let rx_stream = this.rx_stream.take().unwrap();
            let spi_tx = this.spi.take().unwrap();
            let spi_rx: SPI = unsafe { mem::transmute_copy(&spi_tx) };

            spi_rx
                .cr2
                .modify(|_, w| w.errie().set_bit().txeie().set_bit().rxneie().set_bit());

            //            let mut tx_transfer = Transfer::init(
            //                tx_stream,
            //                spi_tx,
            //                write_static_buf,
            //                None,
            //                DmaConfig::default()
            //                    .transfer_complete_interrupt(true)
            //                    .memory_increment(true)
            //                    .double_buffer(false),
            //            );
            //
            //            let mut rx_transfer = Transfer::init(
            //                rx_stream,
            //                spi_rx,
            //                read_static_buf,
            //                None,
            //                DmaConfig::default()
            //                    .transfer_complete_interrupt(true)
            //                    .memory_increment(true)
            //                    .double_buffer(false),
            //            );
            //
            //            let tx_fut = InterruptFuture::new(&mut this.tx_int);
            //            let rx_fut = InterruptFuture::new(&mut this.rx_int);
            //            let rx_fut_err = InterruptFuture::new(&mut this.spi_int);
            //
            //            rx_transfer.start(|_spi| {});
            //            tx_transfer.start(|_spi| {});
            //
            //            time::Timer::after(time::Duration::from_millis(500)).await;
            //
            //            // tx_fut.await;
            //            // future::select(rx_fut, rx_fut_err).await;
            //
            //            let (rx_stream, spi_rx, _buf, _) = rx_transfer.free();
            //            let (tx_stream, _, _buf, _) = tx_transfer.free();

            for i in 0..(read_static_buf.len() - 1) {
                let byte = write_static_buf[i];
                loop {
                    let fut = InterruptFuture::new(&mut this.spi_int);
                    match write_sr(&spi_tx, byte) {
                        Ok(()) => break,
                        _ => {}
                    }
                    fut.await;
                }

                loop {
                    let fut = InterruptFuture::new(&mut this.spi_int);
                    match read_sr(&spi_tx) {
                        Ok(byte) => {
                            read_static_buf[i] = byte;
                            break;
                        }
                        _ => {}
                    }
                    fut.await;
                }
            }

            spi_rx.cr2.modify(|_, w| {
                w.errie()
                    .clear_bit()
                    .txeie()
                    .clear_bit()
                    .rxneie()
                    .clear_bit()
            });
            this.rx_stream.replace(rx_stream);
            this.tx_stream.replace(tx_stream);
            this.spi.replace(spi_rx);

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

pub trait Instance: Deref<Target = pac::spi1::RegisterBlock> + private::Sealed {
    unsafe fn enable_clock();
    fn clock_speed(clocks: Clocks) -> Hertz;
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

macro_rules! spi {
    ($($PER:ident => ($SPI:ident, $pclkX:ident,  $apbXenr:ident, $en:expr),)+) => {
        $(
            impl private::Sealed for pac::$SPI {}
            impl Instance for pac::$SPI {
                unsafe fn enable_clock() {
                    const EN_BIT: u8 = $en;
                    // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
                    let rcc = &(*pac::RCC::ptr());
                    // Enable clock.
                    bb::set(&rcc.$apbXenr, EN_BIT);
                    // Stall the pipeline to work around erratum 2.1.13 (DM00037591)
                    cortex_m::asm::dsb();
                }

                fn clock_speed(clocks: Clocks) -> Hertz {
                    clocks.$pclkX()
                }
            }
            impl WithInterrupt for pac::$SPI {
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

#[cfg(any(
    feature = "stm32f401",
    feature = "stm32f410",
    feature = "stm32f411",
    feature = "stm32f446",
))]
spi! {
    SPI1 => (SPI1, pclk2, apb2enr, 12),
    SPI2 => (SPI2, pclk1, apb2enr, 14),
//    SPI6 => (SPI6, pclk2, apb2enr, 21),
    SPI4 => (SPI3, pclk2, apb2enr, 13),
//    SPI5 => (SPI3, pclk2, apb2enr, 20),
}

#[cfg(any(feature = "stm32f405", feature = "stm32f407"))]
spi! {
    SPI1 => (SPI1, pclk2, apb2enr, 12),
    SPI3 => (SPI3, pclk1, apb2enr, 15),
}
