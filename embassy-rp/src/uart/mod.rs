use core::marker::PhantomData;

use embassy_hal_common::{into_ref, PeripheralRef};

use crate::dma::{AnyChannel, Channel};
use crate::gpio::sealed::Pin;
use crate::gpio::AnyPin;
use crate::pac::uart as pac;
use crate::{peripherals, Peripheral};

#[cfg(feature = "nightly")]
mod buffered;
#[cfg(feature = "nightly")]
pub use buffered::*;

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

pub struct Uart<'d, M: Mode> {
    tx: UartTx<'d, M>,
    rx: UartRx<'d, M>,
}

pub struct UartTx<'d, M: Mode> {
    info: &'static Info,
    tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    phantom: PhantomData<M>,
}

pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    phantom: PhantomData<M>,
}

impl<'d, M: Mode> UartTx<'d, M> {
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;
        unsafe {
            for &b in buffer {
                while r.uartfr().read().txff() {}
                r.uartdr().write(|w| w.set_data(b));
            }
        }
        Ok(())
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let r = self.info.regs;
        unsafe { while !r.uartfr().read().txfe() {} }
        Ok(())
    }
}

impl<'d> UartTx<'d, Async> {
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let ch = self.tx_dma.as_mut().unwrap();
        let transfer = unsafe {
            self.info.regs.uartdmacr().modify(|reg| {
                reg.set_txdmae(true);
            });
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::write(ch, buffer, self.info.regs.uartdr().ptr() as *mut _, self.info.tx_dreq)
        };
        transfer.await;
        Ok(())
    }
}

impl<'d, M: Mode> UartRx<'d, M> {
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info.regs;
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

impl<'d> UartRx<'d, Async> {
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let ch = self.rx_dma.as_mut().unwrap();
        let transfer = unsafe {
            self.info.regs.uartdmacr().modify(|reg| {
                reg.set_rxdmae(true);
            });
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(ch, self.info.regs.uartdr().ptr() as *const _, buffer, self.info.rx_dreq)
        };
        transfer.await;
        Ok(())
    }
}

impl<'d> Uart<'d, Blocking> {
    /// Create a new UART without hardware flow control
    pub fn new_blocking<T: Instance>(
        _uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx);
        Self::new_inner(T::info(), tx.map_into(), rx.map_into(), None, None, None, None, config)
    }

    /// Create a new UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts_blocking<T: Instance>(
        _uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, cts, rts);
        Self::new_inner(
            T::info(),
            tx.map_into(),
            rx.map_into(),
            Some(rts.map_into()),
            Some(cts.map_into()),
            None,
            None,
            config,
        )
    }
}

impl<'d> Uart<'d, Async> {
    /// Create a new DMA enabled UART without hardware flow control
    pub fn new<T: Instance>(
        _uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx_dma: impl Peripheral<P = impl Channel> + 'd,
        rx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, tx_dma, rx_dma);
        Self::new_inner(
            T::info(),
            tx.map_into(),
            rx.map_into(),
            None,
            None,
            Some(tx_dma.map_into()),
            Some(rx_dma.map_into()),
            config,
        )
    }

    /// Create a new DMA enabled UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts<T: Instance>(
        _uart: impl Peripheral<P = T> + 'd,
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
            T::info(),
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

impl<'d, M: Mode> Uart<'d, M> {
    fn new_inner(
        info: &'static Info,
        mut tx: PeripheralRef<'d, AnyPin>,
        mut rx: PeripheralRef<'d, AnyPin>,
        mut rts: Option<PeripheralRef<'d, AnyPin>>,
        mut cts: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
        rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
        config: Config,
    ) -> Self {
        init(
            info,
            Some(tx.reborrow()),
            Some(rx.reborrow()),
            rts.as_mut().map(|x| x.reborrow()),
            cts.as_mut().map(|x| x.reborrow()),
            config,
        );

        Self {
            tx: UartTx {
                info,
                tx_dma,
                phantom: PhantomData,
            },
            rx: UartRx {
                info,
                rx_dma,
                phantom: PhantomData,
            },
        }
    }
}

fn init(
    info: &'static Info,
    tx: Option<PeripheralRef<'_, AnyPin>>,
    rx: Option<PeripheralRef<'_, AnyPin>>,
    rts: Option<PeripheralRef<'_, AnyPin>>,
    cts: Option<PeripheralRef<'_, AnyPin>>,
    config: Config,
) {
    let r = info.regs;
    unsafe {
        if let Some(pin) = &tx {
            pin.io().ctrl().write(|w| w.set_funcsel(2));
            pin.pad_ctrl().write(|w| w.set_ie(true));
        }
        if let Some(pin) = &rx {
            pin.io().ctrl().write(|w| w.set_funcsel(2));
            pin.pad_ctrl().write(|w| w.set_ie(true));
        }
        if let Some(pin) = &cts {
            pin.io().ctrl().write(|w| w.set_funcsel(2));
            pin.pad_ctrl().write(|w| w.set_ie(true));
        }
        if let Some(pin) = &rts {
            pin.io().ctrl().write(|w| w.set_funcsel(2));
            pin.pad_ctrl().write(|w| w.set_ie(true));
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
        r.uartibrd().write_value(pac::regs::Uartibrd(baud_ibrd));
        r.uartfbrd().write_value(pac::regs::Uartfbrd(baud_fbrd));

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

        r.uartifls().write(|w| {
            w.set_rxiflsel(0b000);
            w.set_txiflsel(0b000);
        });

        r.uartcr().write(|w| {
            w.set_uarten(true);
            w.set_rxe(true);
            w.set_txe(true);
            w.set_ctsen(cts.is_some());
            w.set_rtsen(rts.is_some());
        });
    }
}

impl<'d, M: Mode> Uart<'d, M> {
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
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
        (self.tx, self.rx)
    }
}

impl<'d> Uart<'d, Async> {
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }
}

mod eh02 {
    use super::*;

    impl<'d, M: Mode> embedded_hal_02::serial::Read<u8> for UartRx<'d, M> {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            let r = self.info.regs;
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

    impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, M> {
        type Error = Error;

        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn bflush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, M: Mode> embedded_hal_02::serial::Read<u8> for Uart<'d, M> {
        type Error = Error;

        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, M> {
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

    impl<'d, M: Mode> embedded_hal_1::serial::ErrorType for Uart<'d, M> {
        type Error = Error;
    }

    impl<'d, M: Mode> embedded_hal_1::serial::ErrorType for UartTx<'d, M> {
        type Error = Error;
    }

    impl<'d, M: Mode> embedded_hal_1::serial::ErrorType for UartRx<'d, M> {
        type Error = Error;
    }

    impl<'d, M: Mode> embedded_hal_nb::serial::Read for UartRx<'d, M> {
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            let r = self.info.regs;
            unsafe {
                let dr = r.uartdr().read();

                if dr.oe() {
                    Err(nb::Error::Other(Error::Overrun))
                } else if dr.be() {
                    Err(nb::Error::Other(Error::Break))
                } else if dr.pe() {
                    Err(nb::Error::Other(Error::Parity))
                } else if dr.fe() {
                    Err(nb::Error::Other(Error::Framing))
                } else if dr.fe() {
                    Ok(dr.data())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
        }
    }

    impl<'d, M: Mode> embedded_hal_1::serial::Write for UartTx<'d, M> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, M: Mode> embedded_hal_nb::serial::Write for UartTx<'d, M> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
        }
    }

    impl<'d, M: Mode> embedded_hal_nb::serial::Read for Uart<'d, M> {
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, M: Mode> embedded_hal_1::serial::Write for Uart<'d, M> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, M: Mode> embedded_hal_nb::serial::Write for Uart<'d, M> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
        }
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

    impl<'d, M: Mode> embedded_hal_async::serial::Write for UartTx<'d, M> {
        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            self.write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            async move { Ok(()) }
        }
    }

    impl<'d, M: Mode> embedded_hal_async::serial::Read for UartRx<'d, M> {
        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.read(buf)
        }
    }

    impl<'d, M: Mode> embedded_hal_async::serial::Write for Uart<'d, M> {
        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            self.write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            async move { Ok(()) }
        }
    }

    impl<'d, M: Mode> embedded_hal_async::serial::Read for Uart<'d, M> {
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
        type Interrupt: crate::interrupt::Interrupt;
        fn info() -> &'static Info;
    }
    pub trait TxPin<T: Instance> {}
    pub trait RxPin<T: Instance> {}
    pub trait CtsPin<T: Instance> {}
    pub trait RtsPin<T: Instance> {}

    /// Info about one concrete peripheral instance.
    pub struct Info {
        pub(crate) regs: pac::Uart,
        pub(crate) tx_dreq: u8,
        pub(crate) rx_dreq: u8,
        pub(crate) irq: crate::pac::Interrupt,
        pub(crate) state: &'static super::buffered::State,
    }
}

use sealed::Info;

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
            type Interrupt = crate::interrupt::$irq;

            fn info() -> &'static Info {
                static STATE: buffered::State = buffered::State::new();
                static INFO: Info = Info {
                    regs: crate::pac::$inst,
                    tx_dreq: $tx_dreq,
                    rx_dreq: $rx_dreq,
                    irq: crate::pac::Interrupt::$irq,
                    state: &STATE,
                };
                &INFO
            }
        }
        impl Instance for peripherals::$inst {}

        impl crate::interrupt::InterruptFunction for crate::interrupt::$irq {
            fn on_interrupt() {
                buffered::on_interrupt(<peripherals::$inst as sealed::Instance>::info())
            }
        }
    };
}

impl_instance!(UART0, UART0_IRQ, 20, 21);
impl_instance!(UART1, UART1_IRQ, 22, 23);

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
