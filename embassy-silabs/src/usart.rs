//! USART and UART on Series 0 (EFM32GG): blocking asynchronous UART.
//!
//! The driver implements 8N1 at a configurable baud rate, with polled
//! [`Uart::blocking_write`], [`Uart::blocking_flush`] and
//! [`Uart::blocking_read`]. UART0 and UART1 are USART blocks in
//! asynchronous mode, so this driver serves them too.
//!
//! # Pin routing
//!
//! A Series 0 peripheral routes its pins through `ROUTE.LOCATION`. One
//! location number selects the pins of all signals of the peripheral. The
//! [`TxPin`] and [`RxPin`] traits give the locations at which a pin carries
//! the signal. [`Uart::new_blocking`] uses the smallest location that both
//! pins have. A pin that is at no location of the signal does not compile.
//!
//! # Examples
//!
//! ```rust,ignore
//! use embassy_silabs::usart::{Config, Uart};
//!
//! // USART1 location 1: TX PD0, RX PD1.
//! let mut uart = Uart::new_blocking(p.USART1, p.PD1, p.PD0, Config::default()).unwrap();
//! uart.blocking_write(b"hello\r\n").unwrap();
//! ```

use embassy_hal_internal::{Peri, PeripheralType};

use crate::gpio::AnyPin;
use crate::interrupt;
use crate::pac::usart as usart_mod;
use crate::pac::usart::vals;
use crate::time::Hertz;

/// Asynchronous oversampling. The divider math assumes 16x.
const OVERSAMPLE: u32 = 16;

/// UART configuration. The frame is 8 data bits, no parity and 1 stop bit.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Baud rate in bits per second.
    pub baudrate: u32,
}

impl Default for Config {
    /// 115200 8N1.
    fn default() -> Self {
        Self { baudrate: 115_200 }
    }
}

/// Runtime UART error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Framing error: the stop bit was not high.
    Framing,
    /// The parity bit was wrong.
    Parity,
    /// The receive buffer overflowed and data was lost.
    Overrun,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Error::Framing => "framing error",
            Error::Parity => "parity error",
            Error::Overrun => "RX overrun",
        };
        write!(f, "{s}")
    }
}

impl core::error::Error for Error {}

/// Error from the constructor.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// The baud rate is too low for HFPERCLK: the divider does not fit in
    /// `CLKDIV.DIV`.
    BaudrateTooLow,
    /// The baud rate is too high for HFPERCLK.
    BaudrateTooHigh,
    /// The RX pin and the TX pin have no common location. All signals of a
    /// Series 0 peripheral share one `ROUTE.LOCATION`.
    PinsNotOnSameLocation,
}

/// The smallest location that is in both lists.
fn common_location(rx: &[u8], tx: &[u8]) -> Option<u8> {
    rx.iter().copied().filter(|l| tx.contains(l)).min()
}

/// Returns the nearest `CLKDIV.DIV` value for a baud rate.
///
/// The reference manual gives the asynchronous baud rate as
/// `baud = f / (ovs * (1 + CLKDIV / 256))`, where `CLKDIV` is the register value. `DIV` starts at
/// bit 6, so `CLKDIV = DIV * 64` and `DIV = 4 * f / (ovs * baud) - 4`.
fn compute_clkdiv(src: Hertz, baudrate: u32) -> Result<u16, ConfigError> {
    let denom = OVERSAMPLE as u64 * baudrate as u64;
    if denom == 0 {
        return Err(ConfigError::BaudrateTooLow);
    }
    let quarter_div = (4 * src.0 as u64 + denom / 2) / denom;
    if quarter_div < 4 {
        return Err(ConfigError::BaudrateTooHigh);
    }
    let div = quarter_div - 4;
    // `CLKDIV.DIV` is 15 bits wide.
    if div > 0x7FFF {
        return Err(ConfigError::BaudrateTooLow);
    }
    Ok(div as u16)
}

/// Type-erased instance data.
pub(crate) struct Info {
    pub(crate) regs: usart_mod::Usart,
    pub(crate) disable_clock: fn(),
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
}

/// A USART or UART instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::cmu::CmuPeripheral + PeripheralType + 'static {
    /// Interrupt for RX events.
    type RxInterrupt: interrupt::typelevel::Interrupt;
    /// Interrupt for TX events.
    type TxInterrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(TxPin, Instance, locations);
pin_trait!(RxPin, Instance, locations);

/// Implements [`Instance`] for one USART or UART.
///
/// `build.rs` invokes it for each `usart` peripheral in chip metadata.
macro_rules! impl_usart {
    ($inst:ident, $rxirq:ident, $txirq:ident $(,)?) => {
        impl crate::usart::SealedInstance for crate::peripherals::$inst {
            fn info() -> &'static crate::usart::Info {
                static INFO: crate::usart::Info = crate::usart::Info {
                    regs: crate::pac::$inst,
                    disable_clock: crate::cmu::disable::<crate::peripherals::$inst>,
                };
                &INFO
            }
        }
        impl crate::usart::Instance for crate::peripherals::$inst {
            type RxInterrupt = crate::interrupt::typelevel::$rxirq;
            type TxInterrupt = crate::interrupt::typelevel::$txirq;
        }
    };
}
pub(crate) use impl_usart;

/// Blocking UART driver.
pub struct Uart<'d> {
    info: &'static Info,
    tx: Peri<'d, AnyPin>,
    rx: Peri<'d, AnyPin>,
    /// A byte was written since the last flush. `STATUS.TXC` is set only
    /// after a frame is sent, so a flush with nothing sent must not wait.
    tx_pending: bool,
}

impl<'d> Uart<'d> {
    /// Create a blocking UART on `rx` and `tx` for instance `peri`.
    ///
    /// The driver uses the smallest location that both pins have. It
    /// returns [`ConfigError::PinsNotOnSameLocation`] when they have none.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let location = common_location(rx.locations(), tx.locations()).ok_or(ConfigError::PinsNotOnSameLocation)?;
        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        init::<T>(&tx, &rx, location, &config)?;
        Ok(Self {
            info: T::info(),
            tx,
            rx,
            tx_pending: false,
        })
    }

    /// Write all bytes of `buffer`. Waits for space in the transmit buffer
    /// before each byte.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for &b in buffer {
            while !r.status().read().txbl() {}
            r.txdata().write(|w| w.set_txdata(b));
            self.tx_pending = true;
        }
        Ok(())
    }

    /// Wait until the last byte is sent.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        if self.tx_pending {
            while !self.info.regs.status().read().txc() {}
            self.tx_pending = false;
        }
        Ok(())
    }

    /// Read bytes until `buffer` is full.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for slot in buffer {
            while !rx_poll(&r)? {}
            *slot = read_frame(&r)?;
        }
        Ok(())
    }

    /// Blocks until a byte is received, then reads available bytes up to `buffer.len()`.
    ///
    /// Returns the number of bytes read.
    pub fn blocking_read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let r = self.info.regs;
        while !rx_poll(&r)? {}
        let mut n = 0;
        while n < buffer.len() && r.status().read().rxdatav() {
            buffer[n] = read_frame(&r)?;
            n += 1;
        }
        Ok(n)
    }
}

/// Check for an overrun, then return whether a frame is available.
fn rx_poll(r: &usart_mod::Usart) -> Result<bool, Error> {
    if r.if_().read().rxof() {
        r.if_clr().write(|w| w.set_rxof(true));
        return Err(Error::Overrun);
    }
    Ok(r.status().read().rxdatav())
}

/// Read one frame. A frame with a framing or parity error enters the
/// receive buffer like a good frame (RM 17.3.2.4.4 and 17.3.2.4.5).
/// `RXDATAX` gives the error flags of this frame, so the read consumes the
/// frame and reports its own error.
fn read_frame(r: &usart_mod::Usart) -> Result<u8, Error> {
    let x = r.rxdatax().read();
    if x.ferr() {
        r.if_clr().write(|w| w.set_ferr(true));
        return Err(Error::Framing);
    }
    if x.perr() {
        r.if_clr().write(|w| w.set_perr(true));
        return Err(Error::Parity);
    }
    Ok(x.rxdata() as u8)
}

fn init<T: Instance>(tx: &AnyPin, rx: &AnyPin, location: u8, config: &Config) -> Result<(), ConfigError> {
    crate::cmu::enable::<T>();
    let div = match compute_clkdiv(crate::cmu::frequency::<T>(), config.baudrate) {
        Ok(div) => div,
        Err(e) => {
            crate::cmu::disable::<T>();
            return Err(e);
        }
    };
    debug!("usart: baud={} location={} clkdiv={}", config.baudrate, location, div);

    let r = T::info().regs;
    // Stop and clear the receiver and the transmitter.
    r.cmd().write(|w| {
        w.set_rxdis(true);
        w.set_txdis(true);
        w.set_masterdis(true);
        w.set_clearrx(true);
        w.set_cleartx(true);
    });
    // MSBF stays 0: LSB first.
    r.ctrl().write(|w| {
        w.set_sync(false);
        w.set_ovs(vals::Ovs::X16);
    });
    r.frame().write(|w| {
        w.set_databits(vals::Databits::Eight);
        w.set_parity(vals::Parity::None);
        w.set_stopbits(vals::Stopbits::One);
    });
    r.clkdiv().write(|w| w.set_div(div));
    r.ien().write(|_| {});
    r.if_clr().write_value(usart_mod::regs::Ifc(0xFFFF_FFFF));

    // TX is a push-pull output, idle high, and RX is an input.
    crate::gpio::set_as_alternate_output(tx);
    crate::gpio::set_as_alternate_input(rx);
    // `build.rs` checks that the TX and RX enable fields are `route.txpen` and `route.rxpen`.
    r.route().write(|w| {
        w.set_location(vals::Location::from_bits(location));
        w.set_txpen(true);
        w.set_rxpen(true);
    });

    r.cmd().write(|w| {
        w.set_rxen(true);
        w.set_txen(true);
    });
    while {
        let st = r.status().read();
        !(st.rxens() && st.txens())
    } {}
    Ok(())
}

impl Drop for Uart<'_> {
    fn drop(&mut self) {
        let r = self.info.regs;
        r.cmd().write(|w| {
            w.set_rxdis(true);
            w.set_txdis(true);
        });
        r.ien().write(|_| {});
        r.if_clr().write_value(usart_mod::regs::Ifc(0xFFFF_FFFF));
        r.route().write(|_| {});
        crate::gpio::set_as_disabled(&self.tx);
        crate::gpio::set_as_disabled(&self.rx);
        (self.info.disable_clock)();
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl embedded_io::ErrorType for Uart<'_> {
    type Error = Error;
}

impl embedded_io::Write for Uart<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), Error> {
        self.blocking_flush()
    }
}

impl embedded_io::Read for Uart<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.blocking_read_buf(buf)
    }
}
