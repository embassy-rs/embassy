//! Types and code shared by the UART drivers.
//!
//! `eusart` (Series 2) and `usart` (Series 0) re-export these types, so one `Config` serves each
//! UART driver on a chip. [`impl_uart_common!`] gives each driver the `Uart` wrapper.

/// Number of data bits per frame.
///
/// A driver returns [`ConfigError::DataBitsNotSupported`] for a frame size that its hardware lacks.
/// The Series 2 EUSART has no 5-bit or 6-bit frames.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataBits {
    /// 5 data bits.
    DataBits5,
    /// 6 data bits.
    DataBits6,
    /// 7 data bits.
    DataBits7,
    /// 8 data bits.
    DataBits8,
    /// 9 data bits. The byte API sends the 9th bit as 0, and drops it on receive.
    DataBits9,
}

/// Parity bit.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Parity {
    /// No parity bit.
    ParityNone,
    /// Even parity.
    ParityEven,
    /// Odd parity.
    ParityOdd,
}

/// Number of stop bits.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopBits {
    /// 0.5 stop bits.
    STOP0P5,
    /// 1 stop bit.
    STOP1,
    /// 1.5 stop bits.
    STOP1P5,
    /// 2 stop bits.
    STOP2,
}

/// UART configuration.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Baud rate in bits per second.
    pub baudrate: u32,
    /// Number of data bits.
    pub data_bits: DataBits,
    /// Number of stop bits.
    pub stop_bits: StopBits,
    /// Parity.
    pub parity: Parity,
    /// Inverts the TX line: it idles low instead of high.
    pub invert_tx: bool,
    /// Inverts the RX line.
    pub invert_rx: bool,
    /// Internal loopback: the receiver gets the transmitter output inside the peripheral.
    pub loopback: bool,
}

impl Default for Config {
    /// 115200 8N1, no inversion, no loopback.
    fn default() -> Self {
        Self {
            baudrate: 115_200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
            invert_tx: false,
            invert_rx: false,
            loopback: false,
        }
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

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

/// Error from a constructor, `set_config` or `set_baudrate`.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// The baud rate is 0, or so low that the divider overflows `CLKDIV.DIV`.
    BaudrateTooLow,
    /// The baud rate is too high for the peripheral clock.
    BaudrateTooHigh,
    /// The peripheral cannot send frames with this number of data bits.
    DataBitsNotSupported,
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            ConfigError::BaudrateTooLow => "baud rate too low",
            ConfigError::BaudrateTooHigh => "baud rate too high",
            ConfigError::DataBitsNotSupported => "data bits not supported",
        };
        write!(f, "{s}")
    }
}

impl core::error::Error for ConfigError {}

/// Emits the code that each UART driver shares.
///
/// It emits the `Uart` wrapper, its forwarding methods, `split`, `split_ref` and the trait impls.
///
/// Call it once in the driver module, which must define `UartTx` and `UartRx` with these methods:
///
/// - `UartTx`, both modes: `blocking_write`, `blocking_flush`, `busy`, `send_break`, `write_ready`.
/// - `UartRx`, both modes: `blocking_read`, `blocking_read_buf`, `read_ready`.
/// - `Async`: `write` and `flush` on `UartTx`, `read` and `read_buf` on `UartRx`.
/// - Both halves: `set_config`, `set_baudrate`, and `borrowed(&mut self)`, which returns a half
///   that does not turn the peripheral off when it drops.
macro_rules! impl_uart_common {
    () => {
        /// Bidirectional UART.
        pub struct Uart<'d, M: crate::mode::Mode> {
            tx: UartTx<'d, M>,
            rx: UartRx<'d, M>,
        }

        impl<'d, M: crate::mode::Mode> Uart<'d, M> {
            /// Splits the UART into transmitter and receiver.
            ///
            /// Each half keeps the peripheral on, and it turns off when both halves drop.
            pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
                (self.tx, self.rx)
            }

            /// Borrows the transmitter and receiver.
            ///
            /// Borrowed halves do not turn the peripheral off when they drop. The `Uart` does.
            pub fn split_ref(&mut self) -> (UartTx<'_, M>, UartRx<'_, M>) {
                (self.tx.borrowed(), self.rx.borrowed())
            }

            /// Changes the frame format and baud rate. See [`UartTx::set_config`].
            pub fn set_config(
                &mut self,
                config: &crate::uart_common::Config,
            ) -> Result<(), crate::uart_common::ConfigError> {
                self.tx.set_config(config)
            }

            /// Changes the baud rate. See [`UartTx::set_baudrate`].
            pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), crate::uart_common::ConfigError> {
                self.tx.set_baudrate(baudrate)
            }

            /// Sends a break. See [`UartTx::send_break`].
            pub fn send_break(&mut self) {
                self.tx.send_break()
            }

            /// Returns `true` if the transmitter has unsent data.
            pub fn busy(&self) -> bool {
                self.tx.busy()
            }

            /// Writes all bytes. Blocks until each byte is in the transmit buffer.
            pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), crate::uart_common::Error> {
                self.tx.blocking_write(buffer)
            }

            /// Blocks until all written bytes are sent.
            pub fn blocking_flush(&mut self) -> Result<(), crate::uart_common::Error> {
                self.tx.blocking_flush()
            }

            /// Fills `buffer`. Blocks until each byte is received.
            pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), crate::uart_common::Error> {
                self.rx.blocking_read(buffer)
            }

            /// Blocks until a byte is received, then reads available bytes up to `buffer.len()`.
            ///
            /// Returns the number of bytes read.
            pub fn blocking_read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, crate::uart_common::Error> {
                self.rx.blocking_read_buf(buffer)
            }
        }

        impl<'d> Uart<'d, crate::mode::Async> {
            /// Writes all bytes. Yields while the transmit buffer is full.
            pub async fn write(&mut self, buffer: &[u8]) -> Result<(), crate::uart_common::Error> {
                self.tx.write(buffer).await
            }

            /// Waits until all written bytes are sent.
            pub async fn flush(&mut self) -> Result<(), crate::uart_common::Error> {
                self.tx.flush().await
            }

            /// Fills `buffer`. Yields while no data is available.
            pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), crate::uart_common::Error> {
                self.rx.read(buffer).await
            }

            /// Waits for at least one byte, then reads available bytes up to `buffer.len()`.
            ///
            /// Returns the number of bytes read.
            pub async fn read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, crate::uart_common::Error> {
                self.rx.read_buf(buffer).await
            }
        }

        impl<M: crate::mode::Mode> embedded_io::ErrorType for Uart<'_, M> {
            type Error = crate::uart_common::Error;
        }
        impl<M: crate::mode::Mode> embedded_io::ErrorType for UartTx<'_, M> {
            type Error = crate::uart_common::Error;
        }
        impl<M: crate::mode::Mode> embedded_io::ErrorType for UartRx<'_, M> {
            type Error = crate::uart_common::Error;
        }

        impl<M: crate::mode::Mode> embedded_io::Write for UartTx<'_, M> {
            fn write(&mut self, buf: &[u8]) -> Result<usize, crate::uart_common::Error> {
                self.blocking_write(buf)?;
                Ok(buf.len())
            }
            fn flush(&mut self) -> Result<(), crate::uart_common::Error> {
                self.blocking_flush()
            }
        }

        impl<M: crate::mode::Mode> embedded_io::Write for Uart<'_, M> {
            fn write(&mut self, buf: &[u8]) -> Result<usize, crate::uart_common::Error> {
                self.blocking_write(buf)?;
                Ok(buf.len())
            }
            fn flush(&mut self) -> Result<(), crate::uart_common::Error> {
                self.blocking_flush()
            }
        }

        impl<M: crate::mode::Mode> embedded_io::Read for UartRx<'_, M> {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, crate::uart_common::Error> {
                self.blocking_read_buf(buf)
            }
        }

        impl<M: crate::mode::Mode> embedded_io::Read for Uart<'_, M> {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, crate::uart_common::Error> {
                self.blocking_read_buf(buf)
            }
        }

        impl<M: crate::mode::Mode> embedded_io::ReadReady for UartRx<'_, M> {
            fn read_ready(&mut self) -> Result<bool, crate::uart_common::Error> {
                UartRx::read_ready(self)
            }
        }

        impl<M: crate::mode::Mode> embedded_io::ReadReady for Uart<'_, M> {
            fn read_ready(&mut self) -> Result<bool, crate::uart_common::Error> {
                self.rx.read_ready()
            }
        }

        impl<M: crate::mode::Mode> embedded_io::WriteReady for UartTx<'_, M> {
            fn write_ready(&mut self) -> Result<bool, crate::uart_common::Error> {
                Ok(UartTx::write_ready(self))
            }
        }

        impl<M: crate::mode::Mode> embedded_io::WriteReady for Uart<'_, M> {
            fn write_ready(&mut self) -> Result<bool, crate::uart_common::Error> {
                Ok(self.tx.write_ready())
            }
        }

        impl embedded_io_async::Write for UartTx<'_, crate::mode::Async> {
            async fn write(&mut self, buf: &[u8]) -> Result<usize, crate::uart_common::Error> {
                UartTx::write(self, buf).await?;
                Ok(buf.len())
            }
            async fn flush(&mut self) -> Result<(), crate::uart_common::Error> {
                UartTx::flush(self).await
            }
        }

        impl embedded_io_async::Write for Uart<'_, crate::mode::Async> {
            async fn write(&mut self, buf: &[u8]) -> Result<usize, crate::uart_common::Error> {
                Uart::write(self, buf).await?;
                Ok(buf.len())
            }
            async fn flush(&mut self) -> Result<(), crate::uart_common::Error> {
                Uart::flush(self).await
            }
        }

        impl embedded_io_async::Read for UartRx<'_, crate::mode::Async> {
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, crate::uart_common::Error> {
                UartRx::read_buf(self, buf).await
            }
        }

        impl embedded_io_async::Read for Uart<'_, crate::mode::Async> {
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, crate::uart_common::Error> {
                Uart::read_buf(self, buf).await
            }
        }

        impl<M: crate::mode::Mode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'_, M> {
            type Error = crate::uart_common::Error;
            fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), crate::uart_common::Error> {
                self.blocking_write(buffer)
            }
            fn bflush(&mut self) -> Result<(), crate::uart_common::Error> {
                self.blocking_flush()
            }
        }

        impl<M: crate::mode::Mode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'_, M> {
            type Error = crate::uart_common::Error;
            fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), crate::uart_common::Error> {
                self.blocking_write(buffer)
            }
            fn bflush(&mut self) -> Result<(), crate::uart_common::Error> {
                self.blocking_flush()
            }
        }

        impl<M: crate::mode::Mode> embassy_embedded_hal::SetConfig for Uart<'_, M> {
            type Config = crate::uart_common::Config;
            type ConfigError = crate::uart_common::ConfigError;
            fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
                Uart::set_config(self, config)
            }
        }

        impl<M: crate::mode::Mode> embassy_embedded_hal::SetConfig for UartTx<'_, M> {
            type Config = crate::uart_common::Config;
            type ConfigError = crate::uart_common::ConfigError;
            fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
                UartTx::set_config(self, config)
            }
        }

        impl<M: crate::mode::Mode> embassy_embedded_hal::SetConfig for UartRx<'_, M> {
            type Config = crate::uart_common::Config;
            type ConfigError = crate::uart_common::ConfigError;
            fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
                UartRx::set_config(self, config)
            }
        }
    };
}
pub(crate) use impl_uart_common;
