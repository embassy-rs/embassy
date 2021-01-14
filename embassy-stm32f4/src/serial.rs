//! Async low power Serial.
//!
//! The peripheral is autmatically enabled and disabled as required to save power.
//! Lowest power consumption can only be guaranteed if the send receive futures
//! are dropped correctly (e.g. not using `mem::forget()`).

use core::future::Future;
use core::ptr;
use core::sync::atomic::{self, Ordering};
use paste::paste;

use embassy::interrupt::OwnedInterrupt;
use embassy::uart::{Error, Uart};
use embassy::util::Signal;

use crate::hal::dma::config::DmaConfig;
use crate::hal::dma::traits::{PeriAddress, Stream};
use crate::hal::dma::{Stream2, Stream5, Stream6, Stream7, StreamsTuple, Transfer};
use crate::hal::gpio::gpioa::{PA10, PA2, PA3, PA9};
use crate::hal::gpio::{Alternate, AF7};
use crate::hal::rcc::Clocks;
use crate::hal::serial::config::{
    Config as SerialConfig, DmaConfig as SerialDmaConfig, Parity, StopBits, WordLength,
};
use crate::hal::serial::{Event as SerialEvent, Serial as HalSerial};
use crate::hal::serial::{PinRx, PinTx};
use crate::hal::time::Bps;
use crate::interrupt::{
    DMA1_STREAM5Interrupt, DMA1_STREAM6Interrupt, DMA2_STREAM2Interrupt, DMA2_STREAM7Interrupt,
    USART1Interrupt, USART2Interrupt,
};
use crate::pac::{DMA1, DMA2, USART1, USART2};

/// Interface to the Serial peripheral
pub struct Serial<
    USART: PeriAddress<MemSize = u8>,
    TSTREAM: Stream,
    RSTREAM: Stream,
    TINT: OwnedInterrupt,
    RINT: OwnedInterrupt,
    UINT: OwnedInterrupt,
> {
    tx_stream: Option<TSTREAM>,
    rx_stream: Option<RSTREAM>,
    usart: Option<USART>,
    tx_int: TINT,
    rx_int: RINT,
    usart_int: UINT,
    tx: Signal<()>,
    rx: Signal<()>,
}

macro_rules! usart {
    ($($USART:ident => ($DMA:ident, $TSTREAM:expr, $RSTREAM:expr, $TPIN:ident, $RPIN:ident),)+) => {
        $(
            paste! {
                static mut [<INSTANCE _ $USART _ $TSTREAM _ $RSTREAM>]: *const Serial<
                    $USART,
                    [<Stream $TSTREAM>]<$DMA>,
                    [<Stream $RSTREAM>]<$DMA>,
                    [<$DMA _ STREAM $TSTREAM Interrupt>],
                    [<$DMA _ STREAM $RSTREAM Interrupt>],
                    [<$USART Interrupt>],
                > = ptr::null_mut();

                impl Serial<
                    $USART,
                    [<Stream $TSTREAM>]<$DMA>,
                    [<Stream $RSTREAM>]<$DMA>,
                    [<$DMA _ STREAM $TSTREAM Interrupt>],
                    [<$DMA _ STREAM $RSTREAM Interrupt>],
                    [<$USART Interrupt>]
                > {
                    pub unsafe fn new(
                        txd: $TPIN<Alternate<AF7>>,
                        rxd: $RPIN<Alternate<AF7>>,
                        tx_int: [<$DMA _ STREAM $TSTREAM Interrupt>],
                        rx_int: [<$DMA _ STREAM $RSTREAM Interrupt>],
                        usart_int: [<$USART Interrupt>],
                        dma: $DMA,
                        usart: $USART,
                        parity: Parity,
                        stopbits: StopBits,
                        baudrate: Bps,
                        clocks: Clocks,
                    ) -> Self {
                        let mut serial = HalSerial::[<$USART:lower>](
                            usart,
                            (txd, rxd),
                            SerialConfig {
                                baudrate: baudrate,
                                wordlength: WordLength::DataBits8,
                                parity: parity,
                                stopbits: stopbits,
                                dma: SerialDmaConfig::TxRx,
                            },
                            clocks,
                        )
                        .unwrap();
                        serial.listen(SerialEvent::Idle);
                        serial.listen(SerialEvent::Txe);
                        let (usart, _) = serial.release();
                        tx_int.set_handler(Self::on_tx_irq, core::ptr::null_mut());
                        rx_int.set_handler(Self::on_rx_irq, core::ptr::null_mut());
                        usart_int.set_handler(Self::on_usart_irq, core::ptr::null_mut());
                        let streams = StreamsTuple::new(dma);
                        Serial {
                            tx_stream: Some(streams.$TSTREAM),
                            rx_stream: Some(streams.$RSTREAM),
                            usart: Some(usart),
                            tx_int: tx_int,
                            rx_int: rx_int,
                            usart_int: usart_int,
                            tx: Signal::new(),
                            rx: Signal::new(),
                        }
                    }

                    unsafe fn on_tx_irq(_ctx: *mut ()) {
                        let s = &(*[<INSTANCE _ $USART _ $TSTREAM _ $RSTREAM>]);
                        atomic::compiler_fence(Ordering::Acquire);
                        s.tx_int.disable();
                        s.usart_int.unpend();
                        s.usart_int.enable();
                        atomic::compiler_fence(Ordering::Release);
                    }
                    unsafe fn on_rx_irq(_ctx: *mut ()) {
                        let s = &(*[<INSTANCE _ $USART _ $TSTREAM _ $RSTREAM>]);
                        atomic::compiler_fence(Ordering::Acquire);
                        s.rx_int.disable();
                        s.usart_int.disable();
                        atomic::compiler_fence(Ordering::Release);
                        s.rx.signal(());
                    }
                    unsafe fn on_usart_irq(_ctx: *mut ()) {
                        let s = &(*[<INSTANCE _ $USART _ $TSTREAM _ $RSTREAM>]);
                        let status = &(*$USART::ptr()).sr.read();
                        let is_txe = status.txe().bit_is_set();
                        let is_idle = status.idle().bit_is_set();
                        atomic::compiler_fence(Ordering::Acquire);
                        s.rx_int.disable();
                        s.usart_int.disable();
                        atomic::compiler_fence(Ordering::Release);
                        if is_txe {
                            s.tx.signal(());
                        }
                        if is_idle {
                            s.rx.signal(());
                        }
                    }
                }

                impl Uart for Serial<
                    $USART,
                    [<Stream $TSTREAM>]<$DMA>,
                    [<Stream $RSTREAM>]<$DMA>,
                    [<$DMA _ STREAM $TSTREAM Interrupt>],
                    [<$DMA _ STREAM $RSTREAM Interrupt>],
                    [<$USART Interrupt>]
                > {
                    type SendFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
                    type ReceiveFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
                    /// Sends serial data.
                    fn send<'a>(&'a mut self, buf: &'a [u8]) -> Self::SendFuture<'a> {
                        unsafe { [<INSTANCE _ $USART _ $TSTREAM _ $RSTREAM>] = self };
                        #[allow(mutable_transmutes)]
                        let static_buf = unsafe { core::mem::transmute::<&'a [u8], &'static mut [u8]>(buf) };
                        let tx_stream = self.tx_stream.take().unwrap();
                        let usart = self.usart.take().unwrap();
                        self.tx.reset();
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
                            self.tx_int.unpend();
                            self.tx_int.enable();
                            tx_transfer.start(|_usart| {});
                            self.tx.wait().await;
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
                        unsafe { [<INSTANCE _ $USART _ $TSTREAM _ $RSTREAM>] = self };
                        let static_buf = unsafe { core::mem::transmute::<&'a mut [u8], &'static mut [u8]>(buf) };
                        let rx_stream = self.rx_stream.take().unwrap();
                        let usart = self.usart.take().unwrap();
                        self.rx.reset();
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
                            self.rx_int.unpend();
                            self.rx_int.enable();
                            rx_transfer.start(|_usart| {});
                            self.rx.wait().await;
                            let (rx_stream, usart, _, _) = rx_transfer.free();
                            self.rx_stream.replace(rx_stream);
                            self.usart.replace(usart);
                            Ok(())
                        }
                    }
                }
            }
        )+
    }
}

#[cfg(any(
    feature = "stm32f401",
    feature = "stm32f417",
    feature = "stm32f415",
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f410",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f439",
    feature = "stm32f437",
    feature = "stm32f429",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479",
))]
usart! {
    USART1 => (DMA2, 7, 2, PA9, PA10),
    USART2 => (DMA1, 6, 5, PA2, PA3),
}

// ($($USART:ident => ($TSTREAM:ident,$TDMA:ident,  $RSTREAM:ident, $RDMA:ident, $TINT:ident, $RINT:ident, $UINT:ident, $TPIN:ident, $RPIN:ident),)+) => {

//        (Stream7<DMA2>, Channel4, pac::USART1, MemoryToPeripheral),     //USART1_TX
//        (Stream2<DMA2>, Channel4, pac::USART1, PeripheralToMemory),     //USART1_RX
//
//        (Stream6<DMA1>, Channel4, pac::USART2, MemoryToPeripheral),     //USART2_TX
//        (Stream5<DMA1>, Channel4, pac::USART2, PeripheralToMemory),     //USART2_RX
//
//        (Stream6<DMA2>, Channel5, pac::USART6, MemoryToPeripheral),     //USART6_TX
//        (Stream1<DMA2>, Channel5, pac::USART6, PeripheralToMemory),     //USART6_RX
