//! CDC-ACM class implementation, aka Serial over USB.

use core::convert::Infallible;

use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pipe::Pipe;
pub use embassy_sync::pipe::{TryReadError, TryWriteError};
use embassy_usb_driver::{Driver, EndpointError};

use crate::Builder;

pub mod low_level;

/// Internal state for CDC-ACM
pub struct State<'a, const TX_BUF: usize, const RX_BUF: usize> {
    ll: low_level::State<'a>,
    rx: Pipe<NoopRawMutex, RX_BUF>,
    tx: Pipe<NoopRawMutex, TX_BUF>,
}

impl<'a, const TX_BUF: usize, const RX_BUF: usize> State<'a, TX_BUF, RX_BUF> {
    /// Create a new `State`.
    pub fn new() -> Self {
        Self {
            ll: low_level::State::new(),
            rx: Pipe::new(),
            tx: Pipe::new(),
        }
    }
}

/// USB CDC-ACM serial port.
pub struct SerialPort<'d, const TX_BUF: usize, const RX_BUF: usize> {
    rx: &'d Pipe<NoopRawMutex, RX_BUF>,
    tx: &'d Pipe<NoopRawMutex, TX_BUF>,
}

/// USB CDC-ACM serial port reader
pub struct Reader<'d, const RX_BUF: usize> {
    rx: &'d Pipe<NoopRawMutex, RX_BUF>,
}

/// USB CDC-ACM serial port writer
pub struct Writer<'d, const TX_BUF: usize> {
    tx: &'d Pipe<NoopRawMutex, TX_BUF>,
}

/// Background task runner for a CDC-ACM serial port.
///
/// You must run `run()` in the background to make the serial port work. Either spawn
/// it as a separate task, or use `join` or `select`.
pub struct Runner<'d, D: Driver<'d>, const TX_BUF: usize, const RX_BUF: usize> {
    ll: low_level::CdcAcmClass<'d, D>,
    rx: &'d Pipe<NoopRawMutex, RX_BUF>,
    tx: &'d Pipe<NoopRawMutex, TX_BUF>,
}

impl<'d, D: Driver<'d>, const TX_BUF: usize, const RX_BUF: usize> Runner<'d, D, TX_BUF, RX_BUF> {
    /// Run background processing for this serial port.
    ///
    /// You must run this in the background to make the serial port work. Either spawn
    /// it as a separate task, or use `join` or `select`.
    pub async fn run(self) -> ! {
        let (mut ll_tx, mut ll_rx) = self.ll.split();

        let rx_fut = async {
            let mut buf = [0u8; 64];
            loop {
                ll_rx.wait_connection().await;
                loop {
                    match ll_rx.read_packet(&mut buf).await {
                        Ok(n) => {
                            self.rx.write_all(&buf[..n]).await;
                        }
                        Err(EndpointError::BufferOverflow) => unreachable!(),
                        Err(EndpointError::Disabled) => break,
                    }
                }
            }
        };
        let tx_fut = async {
            let mut buf = [0u8; 64];
            loop {
                ll_tx.wait_connection().await;

                let mut needs_zlp = false;

                loop {
                    let n = if needs_zlp {
                        // USB transfer end is signaled by a not full-sized packet (less than max_packet_size).
                        // if last packet was full-sized and we have no more data, we must send
                        // a zero-length packet (ZLP). If we don't, the host might not process the data we've
                        // sent because it'll think the transfer is still not done.
                        self.tx.try_read(&mut buf).unwrap_or(0)
                    } else {
                        self.tx.read(&mut buf).await
                    };

                    match ll_tx.write_packet(&buf[..n]).await {
                        Ok(()) => {}
                        Err(EndpointError::BufferOverflow) => unreachable!(),
                        Err(EndpointError::Disabled) => break,
                    }

                    // If the packet was full-sized, record this for the next loop iteration.
                    needs_zlp = n == ll_tx.max_packet_size() as usize;
                }
            }
        };
        join(rx_fut, tx_fut).await;
        unreachable!()
    }
}

impl<'d, const TX_BUF: usize, const RX_BUF: usize> SerialPort<'d, TX_BUF, RX_BUF> {
    /// Create a new CDC ACM serial port.
    ///
    /// This returns two objects:
    /// - The `CdcAcmClass`: this is what you use to actually read/write bytes from the serial port.
    /// - A `Runner`. This contains a `run()` function that you must run in the background
    ///   to make the serial port work. Either spawn it as a separate task, or use `join` or `select`.
    pub fn new<D: Driver<'d>>(
        builder: &mut Builder<'d, D>,
        state: &'d mut State<'d, TX_BUF, RX_BUF>,
        max_packet_size: u16,
    ) -> (Self, Runner<'d, D, TX_BUF, RX_BUF>) {
        let ll = low_level::CdcAcmClass::new(builder, &mut state.ll, max_packet_size);
        (
            Self {
                tx: &state.tx,
                rx: &state.rx,
            },
            Runner {
                ll,
                tx: &state.tx,
                rx: &state.rx,
            },
        )
    }

    /// Get a writer for this pipe.
    pub fn writer(&self) -> Writer<'_, TX_BUF> {
        Writer { tx: self.tx }
    }

    /// Get a reader for this pipe.
    pub fn reader(&self) -> Reader<'_, RX_BUF> {
        Reader { rx: self.rx }
    }

    /// Write some bytes to the pipe.
    ///
    /// This method writes a nonzero amount of bytes from `buf` into the pipe, and
    /// returns the amount of bytes written.
    ///
    /// If it is not possible to write a nonzero amount of bytes because the pipe's buffer is full,
    /// this method will wait until it isn't. See [`try_write`](Self::try_write) for a variant that
    /// returns an error instead of waiting.
    ///
    /// It is not guaranteed that all bytes in the buffer are written, even if there's enough
    /// free space in the pipe buffer for all. In other words, it is possible for `write` to return
    /// without writing all of `buf` (returning a number less than `buf.len()`) and still leave
    /// free space in the pipe buffer. You should always `write` in a loop, or use helpers like
    /// `write_all` from the `embedded-io` crate.
    pub async fn write(&self, buf: &[u8]) -> usize {
        self.tx.write(buf).await
    }

    /// Write all bytes to the pipe.
    ///
    /// This method writes all bytes from `buf` into the pipe
    pub async fn write_all(&self, mut buf: &[u8]) {
        while !buf.is_empty() {
            let n = self.write(buf).await;
            buf = &buf[n..];
        }
    }

    /// Attempt to immediately write some bytes to the pipe.
    ///
    /// This method will either write a nonzero amount of bytes to the pipe immediately,
    /// or return an error if the pipe is empty. See [`write`](Self::write) for a variant
    /// that waits instead of returning an error.
    pub fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.tx.try_write(buf)
    }

    /// Read some bytes from the pipe.
    ///
    /// This method reads a nonzero amount of bytes from the pipe into `buf` and
    /// returns the amount of bytes read.
    ///
    /// If it is not possible to read a nonzero amount of bytes because the pipe's buffer is empty,
    /// this method will wait until it isn't. See [`try_read`](Self::try_read) for a variant that
    /// returns an error instead of waiting.
    ///
    /// It is not guaranteed that all bytes in the buffer are read, even if there's enough
    /// space in `buf` for all. In other words, it is possible for `read` to return
    /// without filling `buf` (returning a number less than `buf.len()`) and still leave bytes
    /// in the pipe buffer. You should always `read` in a loop, or use helpers like
    /// `read_exact` from the `embedded-io` crate.
    pub async fn read(&self, buf: &mut [u8]) -> usize {
        self.rx.read(buf).await
    }

    /// Attempt to immediately read some bytes from the pipe.
    ///
    /// This method will either read a nonzero amount of bytes from the pipe immediately,
    /// or return an error if the pipe is empty. See [`read`](Self::read) for a variant
    /// that waits instead of returning an error.
    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.rx.try_read(buf)
    }

    /// Clear the data in the receive buffer.
    pub fn clear_rx_buffer(&self) {
        self.rx.clear()
    }

    /// Clear the data in the transmit buffer.
    pub fn clear_tx_buffer(&self) {
        self.tx.clear()
    }
}

impl<'d, const TX_BUF: usize> Writer<'d, TX_BUF> {
    /// Write some bytes to the pipe.
    ///
    /// This method writes a nonzero amount of bytes from `buf` into the pipe, and
    /// returns the amount of bytes written.
    ///
    /// If it is not possible to write a nonzero amount of bytes because the pipe's buffer is full,
    /// this method will wait until it isn't. See [`try_write`](Self::try_write) for a variant that
    /// returns an error instead of waiting.
    ///
    /// It is not guaranteed that all bytes in the buffer are written, even if there's enough
    /// free space in the pipe buffer for all. In other words, it is possible for `write` to return
    /// without writing all of `buf` (returning a number less than `buf.len()`) and still leave
    /// free space in the pipe buffer. You should always `write` in a loop, or use helpers like
    /// `write_all` from the `embedded-io` crate.
    pub async fn write(&self, buf: &[u8]) -> usize {
        self.tx.write(buf).await
    }

    /// Write all bytes to the pipe.
    ///
    /// This method writes all bytes from `buf` into the pipe
    pub async fn write_all(&self, mut buf: &[u8]) {
        while !buf.is_empty() {
            let n = self.write(buf).await;
            buf = &buf[n..];
        }
    }

    /// Attempt to immediately write some bytes to the pipe.
    ///
    /// This method will either write a nonzero amount of bytes to the pipe immediately,
    /// or return an error if the pipe is empty. See [`write`](Self::write) for a variant
    /// that waits instead of returning an error.
    pub fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.tx.try_write(buf)
    }

    /// Clear the data in the transmit buffer.
    pub fn clear_tx_buffer(&self) {
        self.tx.clear()
    }
}

impl<'d, const RX_BUF: usize> Reader<'d, RX_BUF> {
    /// Read some bytes from the pipe.
    ///
    /// This method reads a nonzero amount of bytes from the pipe into `buf` and
    /// returns the amount of bytes read.
    ///
    /// If it is not possible to read a nonzero amount of bytes because the pipe's buffer is empty,
    /// this method will wait until it isn't. See [`try_read`](Self::try_read) for a variant that
    /// returns an error instead of waiting.
    ///
    /// It is not guaranteed that all bytes in the buffer are read, even if there's enough
    /// space in `buf` for all. In other words, it is possible for `read` to return
    /// without filling `buf` (returning a number less than `buf.len()`) and still leave bytes
    /// in the pipe buffer. You should always `read` in a loop, or use helpers like
    /// `read_exact` from the `embedded-io` crate.
    pub async fn read(&self, buf: &mut [u8]) -> usize {
        self.rx.read(buf).await
    }

    /// Attempt to immediately read some bytes from the pipe.
    ///
    /// This method will either read a nonzero amount of bytes from the pipe immediately,
    /// or return an error if the pipe is empty. See [`read`](Self::read) for a variant
    /// that waits instead of returning an error.
    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.rx.try_read(buf)
    }

    /// Clear the data in the receive buffer.
    pub fn clear_rx_buffer(&self) {
        self.rx.clear()
    }
}

impl<const TX_BUF: usize, const RX_BUF: usize> embedded_io::Io for SerialPort<'_, TX_BUF, RX_BUF> {
    type Error = Infallible;
}

impl<const TX_BUF: usize, const RX_BUF: usize> embedded_io::asynch::Read for SerialPort<'_, TX_BUF, RX_BUF> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(SerialPort::read(self, buf).await)
    }
}

impl<const TX_BUF: usize, const RX_BUF: usize> embedded_io::asynch::Write for SerialPort<'_, TX_BUF, RX_BUF> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(SerialPort::write(self, buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<const TX_BUF: usize, const RX_BUF: usize> embedded_io::Io for &SerialPort<'_, TX_BUF, RX_BUF> {
    type Error = Infallible;
}

impl<const TX_BUF: usize, const RX_BUF: usize> embedded_io::asynch::Read for &SerialPort<'_, TX_BUF, RX_BUF> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(SerialPort::read(self, buf).await)
    }
}

impl<const TX_BUF: usize, const RX_BUF: usize> embedded_io::asynch::Write for &SerialPort<'_, TX_BUF, RX_BUF> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(SerialPort::write(self, buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<const RX_BUF: usize> embedded_io::Io for Reader<'_, RX_BUF> {
    type Error = Infallible;
}

impl<const RX_BUF: usize> embedded_io::asynch::Read for Reader<'_, RX_BUF> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(Reader::read(self, buf).await)
    }
}

impl<const TX_BUF: usize> embedded_io::Io for Writer<'_, TX_BUF> {
    type Error = Infallible;
}

impl<const TX_BUF: usize> embedded_io::asynch::Write for Writer<'_, TX_BUF> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(Writer::write(self, buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
