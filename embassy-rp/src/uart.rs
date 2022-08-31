use core::marker::PhantomData;

use embassy_hal_common::{into_ref, PeripheralRef};

use crate::dma::{AnyChannel, Channel};
use crate::gpio::sealed::Pin;
use crate::gpio::AnyPin;
use crate::{pac, peripherals, Peripheral};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    DataBits5,
    DataBits6,
    DataBits7,
    DataBits8,
}

impl DataBits {
    fn bits(&self) -> u8 {
        match self {
            Self::DataBits5 => 0b00,
            Self::DataBits6 => 0b01,
            Self::DataBits7 => 0b10,
            Self::DataBits8 => 0b11,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone,
    ParityEven,
    ParityOdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "2 stop bits"]
    STOP2,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
        }
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Triggered when the FIFO (or shift-register) is overflowed.
    Overrun,
    /// Triggered when a break is received
    Break,
    /// Triggered when there is a parity mismatch between what's received and
    /// our settings.
    Parity,
    /// Triggered when the received character didn't have a valid stop bit.
    Framing,
}

pub struct Uart<'d, T: Instance, M: Mode> {
    tx: UartTx<'d, T, M>,
    rx: UartRx<'d, T, M>,
}

pub struct UartTx<'d, T: Instance, M: Mode> {
    tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    phantom: PhantomData<(&'d mut T, M)>,
}

pub struct UartRx<'d, T: Instance, M: Mode> {
    rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    phantom: PhantomData<(&'d mut T, M)>,
}

impl<'d, T: Instance, M: Mode> UartTx<'d, T, M> {
    fn new(tx_dma: Option<PeripheralRef<'d, AnyChannel>>) -> Self {
        Self {
            tx_dma,
            phantom: PhantomData,
        }
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = T::regs();
        unsafe {
            for &b in buffer {
                while r.uartfr().read().txff() {}
                r.uartdr().write(|w| w.set_data(b));
            }
        }
        Ok(())
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let r = T::regs();
        unsafe { while !r.uartfr().read().txfe() {} }
        Ok(())
    }
}

impl<'d, T: Instance> UartTx<'d, T, Async> {
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let (from_ptr, len) = crate::dma::slice_ptr_parts(buffer);
        let ch = self.tx_dma.as_mut().unwrap();
        let transfer = unsafe {
            T::regs().uartdmacr().modify(|reg| {
                reg.set_txdmae(true);
            });
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::write(ch, from_ptr as *const u32, T::regs().uartdr().ptr() as *mut _, len, T::TX_DREQ)
        };
        transfer.await;
        Ok(())
    }
}

impl<'d, T: Instance, M: Mode> UartRx<'d, T, M> {
    fn new(rx_dma: Option<PeripheralRef<'d, AnyChannel>>) -> Self {
        Self {
            rx_dma,
            phantom: PhantomData,
        }
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = T::regs();
        unsafe {
            for b in buffer {
                *b = loop {
                    if r.uartfr().read().rxfe() {
                        continue;
                    }

                    let dr = r.uartdr().read();

                    if dr.oe() {
                        return Err(Error::Overrun);
                    } else if dr.be() {
                        return Err(Error::Break);
                    } else if dr.pe() {
                        return Err(Error::Parity);
                    } else if dr.fe() {
                        return Err(Error::Framing);
                    } else {
                        break dr.data();
                    }
                };
            }
        }
        Ok(())
    }
}

impl<'d, T: Instance> UartRx<'d, T, Async> {
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let (to_ptr, len) = crate::dma::slice_ptr_parts_mut(buffer);
        let ch = self.rx_dma.as_mut().unwrap();
        let transfer = unsafe {
            T::regs().uartdmacr().modify(|reg| {
                reg.set_rxdmae(true);
            });
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(ch, T::regs().uartdr().ptr() as *const _, to_ptr as *mut u32, len, T::RX_DREQ)
        };
        transfer.await;
        Ok(())
    }
}

impl<'d, T: Instance> Uart<'d, T, Blocking> {
    /// Create a new UART without hardware flow control
    pub fn new_blocking(
        uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx);
        Self::new_inner(uart, rx.map_into(), tx.map_into(), None, None, None, None, config)
    }

    /// Create a new UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts_blocking(
        uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, cts, rts);
        Self::new_inner(
            uart,
            rx.map_into(),
            tx.map_into(),
            Some(rts.map_into()),
            Some(cts.map_into()),
            None,
            None,
            config,
        )
    }
}

impl<'d, T: Instance> Uart<'d, T, Async> {
    /// Create a new DMA enabled UART without hardware flow control
    pub fn new(
        uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx_dma: impl Peripheral<P = impl Channel> + 'd,
        rx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, tx_dma, rx_dma);
        Self::new_inner(
            uart,
            rx.map_into(),
            tx.map_into(),
            None,
            None,
            Some(tx_dma.map_into()),
            Some(rx_dma.map_into()),
            config,
        )
    }

    /// Create a new DMA enabled UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_dma: impl Peripheral<P = impl Channel> + 'd,
        rx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, cts, rts, tx_dma, rx_dma);
        Self::new_inner(
            uart,
            rx.map_into(),
            tx.map_into(),
            Some(rts.map_into()),
            Some(cts.map_into()),
            Some(tx_dma.map_into()),
            Some(rx_dma.map_into()),
            config,
        )
    }
}

impl<'d, T: Instance, M: Mode> Uart<'d, T, M> {
    fn new_inner(
        _uart: impl Peripheral<P = T> + 'd,
        tx: PeripheralRef<'d, AnyPin>,
        rx: PeripheralRef<'d, AnyPin>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
        rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
        config: Config,
    ) -> Self {
        into_ref!(_uart);

        unsafe {
            let r = T::regs();

            tx.io().ctrl().write(|w| w.set_funcsel(2));
            rx.io().ctrl().write(|w| w.set_funcsel(2));

            tx.pad_ctrl().write(|w| {
                w.set_ie(true);
            });

            rx.pad_ctrl().write(|w| {
                w.set_ie(true);
            });

            if let Some(pin) = &cts {
                pin.io().ctrl().write(|w| w.set_funcsel(2));
                pin.pad_ctrl().write(|w| {
                    w.set_ie(true);
                });
            }
            if let Some(pin) = &rts {
                pin.io().ctrl().write(|w| w.set_funcsel(2));
                pin.pad_ctrl().write(|w| {
                    w.set_ie(true);
                });
            }

            let clk_base = crate::clocks::clk_peri_freq();

            let baud_rate_div = (8 * clk_base) / config.baudrate;
            let mut baud_ibrd = baud_rate_div >> 7;
            let mut baud_fbrd = ((baud_rate_div & 0x7f) + 1) / 2;

            if baud_ibrd == 0 {
                baud_ibrd = 1;
                baud_fbrd = 0;
            } else if baud_ibrd >= 65535 {
                baud_ibrd = 65535;
                baud_fbrd = 0;
            }

            // Load PL011's baud divisor registers
            r.uartibrd().write_value(pac::uart::regs::Uartibrd(baud_ibrd));
            r.uartfbrd().write_value(pac::uart::regs::Uartfbrd(baud_fbrd));

            let (pen, eps) = match config.parity {
                Parity::ParityNone => (false, false),
                Parity::ParityOdd => (true, false),
                Parity::ParityEven => (true, true),
            };

            // PL011 needs a (dummy) line control register write to latch in the
            // divisors. We don't want to actually change LCR contents here.
            r.uartlcr_h().modify(|_| {});

            r.uartlcr_h().write(|w| {
                w.set_wlen(config.data_bits.bits());
                w.set_stp2(config.stop_bits == StopBits::STOP2);
                w.set_pen(pen);
                w.set_eps(eps);
                w.set_fen(true);
            });

            r.uartcr().write(|w| {
                w.set_uarten(true);
                w.set_rxe(true);
                w.set_txe(true);
                w.set_ctsen(cts.is_some());
                w.set_rtsen(rts.is_some());
            });
        }

        Self {
            tx: UartTx::new(tx_dma),
            rx: UartRx::new(rx_dma),
        }
    }
}

impl<'d, T: Instance, M: Mode> Uart<'d, T, M> {
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Split the Uart into a transmitter and receiver, which is particuarly
    /// useful when having two tasks correlating to transmitting and receiving.
    pub fn split(self) -> (UartTx<'d, T, M>, UartRx<'d, T, M>) {
        (self.tx, self.rx)
    }
}

impl<'d, T: Instance> Uart<'d, T, Async> {
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }
}

mod eh02 {
    use super::*;

    impl<'d, T: Instance, M: Mode> embedded_hal_02::serial::Read<u8> for UartRx<'d, T, M> {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            let r = T::regs();
            unsafe {
                if r.uartfr().read().rxfe() {
                    return Err(nb::Error::WouldBlock);
                }

                let dr = r.uartdr().read();

                if dr.oe() {
                    Err(nb::Error::Other(Error::Overrun))
                } else if dr.be() {
                    Err(nb::Error::Other(Error::Break))
                } else if dr.pe() {
                    Err(nb::Error::Other(Error::Parity))
                } else if dr.fe() {
                    Err(nb::Error::Other(Error::Framing))
                } else {
                    Ok(dr.data())
                }
            }
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, T, M> {
        type Error = Error;
        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }
        fn bflush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_02::serial::Read<u8> for Uart<'d, T, M> {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, T, M> {
        type Error = Error;
        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }
        fn bflush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::serial::Error for Error {
        fn kind(&self) -> embedded_hal_1::serial::ErrorKind {
            match *self {
                Self::Framing => embedded_hal_1::serial::ErrorKind::FrameFormat,
                Self::Break => embedded_hal_1::serial::ErrorKind::Other,
                Self::Overrun => embedded_hal_1::serial::ErrorKind::Overrun,
                Self::Parity => embedded_hal_1::serial::ErrorKind::Parity,
            }
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_1::serial::ErrorType for Uart<'d, T, M> {
        type Error = Error;
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_1::serial::ErrorType for UartTx<'d, T, M> {
        type Error = Error;
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_1::serial::ErrorType for UartRx<'d, T, M> {
        type Error = Error;
    }
}

#[cfg(all(
    feature = "unstable-traits",
    feature = "nightly",
    feature = "_todo_embedded_hal_serial"
))]
mod eha {
    use core::future::Future;

    use super::*;

    impl<'d, T: Instance, M: Mode> embedded_hal_async::serial::Write for UartTx<'d, T, M> {
        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            self.write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            async move { Ok(()) }
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_async::serial::Read for UartRx<'d, T, M> {
        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.read(buf)
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_async::serial::Write for Uart<'d, T, M> {
        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            self.write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            async move { Ok(()) }
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_async::serial::Read for Uart<'d, T, M> {
        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.read(buf)
        }
    }
}

mod sealed {
    use super::*;

    pub trait Mode {}

    pub trait Instance {
        const TX_DREQ: u8;
        const RX_DREQ: u8;

        fn regs() -> pac::uart::Uart;
    }
    pub trait TxPin<T: Instance> {}
    pub trait RxPin<T: Instance> {}
    pub trait CtsPin<T: Instance> {}
    pub trait RtsPin<T: Instance> {}
}

pub trait Mode: sealed::Mode {}

macro_rules! impl_mode {
    ($name:ident) => {
        impl sealed::Mode for $name {}
        impl Mode for $name {}
    };
}

pub struct Blocking;
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

pub trait Instance: sealed::Instance {}

macro_rules! impl_instance {
    ($inst:ident, $irq:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl sealed::Instance for peripherals::$inst {
            const TX_DREQ: u8 = $tx_dreq;
            const RX_DREQ: u8 = $rx_dreq;

            fn regs() -> pac::uart::Uart {
                pac::$inst
            }
        }
        impl Instance for peripherals::$inst {}
    };
}

impl_instance!(UART0, UART0, 20, 21);
impl_instance!(UART1, UART1, 22, 23);

pub trait TxPin<T: Instance>: sealed::TxPin<T> + crate::gpio::Pin {}
pub trait RxPin<T: Instance>: sealed::RxPin<T> + crate::gpio::Pin {}
pub trait CtsPin<T: Instance>: sealed::CtsPin<T> + crate::gpio::Pin {}
pub trait RtsPin<T: Instance>: sealed::RtsPin<T> + crate::gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl sealed::$function<peripherals::$instance> for peripherals::$pin {}
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, UART0, TxPin);
impl_pin!(PIN_1, UART0, RxPin);
impl_pin!(PIN_2, UART0, CtsPin);
impl_pin!(PIN_3, UART0, RtsPin);
impl_pin!(PIN_4, UART1, TxPin);
impl_pin!(PIN_5, UART1, RxPin);
impl_pin!(PIN_6, UART1, CtsPin);
impl_pin!(PIN_7, UART1, RtsPin);
impl_pin!(PIN_8, UART1, TxPin);
impl_pin!(PIN_9, UART1, RxPin);
impl_pin!(PIN_10, UART1, CtsPin);
impl_pin!(PIN_11, UART1, RtsPin);
impl_pin!(PIN_12, UART0, TxPin);
impl_pin!(PIN_13, UART0, RxPin);
impl_pin!(PIN_14, UART0, CtsPin);
impl_pin!(PIN_15, UART0, RtsPin);
impl_pin!(PIN_16, UART0, TxPin);
impl_pin!(PIN_17, UART0, RxPin);
impl_pin!(PIN_18, UART0, CtsPin);
impl_pin!(PIN_19, UART0, RtsPin);
impl_pin!(PIN_20, UART1, TxPin);
impl_pin!(PIN_21, UART1, RxPin);
impl_pin!(PIN_22, UART1, CtsPin);
impl_pin!(PIN_23, UART1, RtsPin);
impl_pin!(PIN_24, UART1, TxPin);
impl_pin!(PIN_25, UART1, RxPin);
impl_pin!(PIN_26, UART1, CtsPin);
impl_pin!(PIN_27, UART1, RtsPin);
impl_pin!(PIN_28, UART0, TxPin);
impl_pin!(PIN_29, UART0, RxPin);
