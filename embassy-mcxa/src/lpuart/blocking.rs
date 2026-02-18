use embassy_hal_internal::Peri;

use super::*;
use crate::pac::lpuart::vals::{Tc, Tdre};

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

impl<'a> Lpuart<'a, Blocking> {
    /// Create a new blocking LPUART instance with RX/TX pins.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        // Configure the pins for LPUART usage
        tx_pin.as_tx();
        rx_pin.as_rx();

        let wg = Self::init::<T>(true, true, false, false, config)?;
        Ok(Self {
            tx: LpuartTx::new_inner::<T>(tx_pin.into(), None, Blocking, wg.clone()),
            rx: LpuartRx::new_inner::<T>(rx_pin.into(), None, Blocking, wg),
        })
    }

    /// Create a new blocking LPUART instance with RX, TX and RTS/CTS flow control pins.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking_with_rtscts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        // Configure the pins for LPUART usage
        rx_pin.as_rx();
        tx_pin.as_tx();
        rts_pin.as_rts();
        cts_pin.as_cts();

        let wg = Self::init::<T>(true, true, true, true, config)?;
        Ok(Self {
            rx: LpuartRx::new_inner::<T>(rx_pin.into(), Some(rts_pin.into()), Blocking, wg.clone()),
            tx: LpuartTx::new_inner::<T>(tx_pin.into(), Some(cts_pin.into()), Blocking, wg),
        })
    }

    /// Read data from LPUART RX blocking execution until the buffer is filled
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buf)
    }

    /// Read data from LPUART RX without blocking
    pub fn read(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buf)
    }

    /// Write data to LPUART TX blocking execution until all data is sent
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buf)
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.tx.write_byte(byte)
    }

    pub fn read_byte_blocking(&mut self) -> u8 {
        loop {
            if let Ok(b) = self.rx.read_byte() {
                return b;
            }
        }
    }

    pub fn write_str_blocking(&mut self, buf: &str) {
        self.tx.write_str_blocking(buf);
    }

    /// Write data to LPUART TX without blocking
    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.tx.write(buf)
    }

    /// Flush LPUART TX blocking execution until all data has been transmitted
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Flush LPUART TX without blocking
    pub fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush()
    }
}

impl<'a> LpuartTx<'a, Blocking> {
    /// Create a new blocking LPUART transmitter instance.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        // Configure the pins for LPUART usage
        tx_pin.as_tx();

        let wg = Lpuart::<Blocking>::init::<T>(true, false, false, false, config)?;
        Ok(Self::new_inner::<T>(tx_pin.into(), None, Blocking, wg))
    }

    /// Create a new blocking LPUART transmitter instance with CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking_with_cts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        cts_pin.as_cts();

        let wg = Lpuart::<Blocking>::init::<T>(true, false, true, false, config)?;
        Ok(Self::new_inner::<T>(tx_pin.into(), Some(cts_pin.into()), Blocking, wg))
    }

    fn write_byte_internal(&mut self, byte: u8) -> Result<(), Error> {
        self.info.regs().data().modify(|w| w.0 = u32::from(byte));

        Ok(())
    }

    fn blocking_write_byte(&mut self, byte: u8) -> Result<(), Error> {
        while self.info.regs().stat().read().tdre() == Tdre::TXDATA {}
        self.write_byte_internal(byte)
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        if self.info.regs().stat().read().tdre() == Tdre::TXDATA {
            Err(Error::TxFifoFull)
        } else {
            self.write_byte_internal(byte)
        }
    }

    /// Write data to LPUART TX blocking execution until all data is sent.
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<(), Error> {
        for x in buf {
            self.blocking_write_byte(*x)?;
        }

        Ok(())
    }

    pub fn write_str_blocking(&mut self, buf: &str) {
        let _ = self.blocking_write(buf.as_bytes());
    }

    /// Write data to LPUART TX without blocking.
    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        for x in buf {
            self.write_byte(*x)?;
        }

        Ok(())
    }

    /// Flush LPUART TX blocking execution until all data has been transmitted.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while self.info.regs().water().read().txcount() != 0 {
            // Wait for TX FIFO to drain
        }

        // Wait for last character to shift out
        while self.info.regs().stat().read().tc() == Tc::ACTIVE {
            // Wait for transmission to complete
        }

        Ok(())
    }

    /// Flush LPUART TX.
    pub fn flush(&mut self) -> Result<(), Error> {
        // Check if TX FIFO is empty
        if self.info.regs().water().read().txcount() != 0 {
            return Err(Error::TxBusy);
        }

        // Check if transmission is complete
        if self.info.regs().stat().read().tc() == Tc::ACTIVE {
            return Err(Error::TxBusy);
        }

        Ok(())
    }
}

impl<'a> LpuartRx<'a, Blocking> {
    /// Create a new blocking LPUART Receiver instance.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        rx_pin.as_rx();

        let wg = Lpuart::<Blocking>::init::<T>(false, true, false, false, config)?;
        Ok(Self::new_inner::<T>(rx_pin.into(), None, Blocking, wg))
    }

    /// Create a new blocking LPUART Receiver instance with RTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking_with_rts<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        rx_pin.as_rx();
        rts_pin.as_rts();

        let wg = Lpuart::<Blocking>::init::<T>(false, true, false, true, config)?;
        Ok(Self::new_inner::<T>(rx_pin.into(), Some(rts_pin.into()), Blocking, wg))
    }

    fn read_byte_internal(&mut self) -> Result<u8, Error> {
        Ok((self.info.regs().data().read().0 & 0xFF) as u8)
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        check_and_clear_rx_errors(self.info)?;

        if !has_rx_data_pending(self.info) {
            return Err(Error::RxFifoEmpty);
        }

        self.read_byte_internal()
    }

    fn blocking_read_byte(&mut self) -> Result<u8, Error> {
        loop {
            if has_rx_data_pending(self.info) {
                return self.read_byte_internal();
            }

            check_and_clear_rx_errors(self.info)?;
        }
    }

    /// Read data from LPUART RX without blocking.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        for byte in buf.iter_mut() {
            *byte = self.read_byte()?;
        }
        Ok(())
    }

    /// Read data from LPUART RX blocking execution until the buffer is filled.
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        for byte in buf.iter_mut() {
            *byte = self.blocking_read_byte()?;
        }
        Ok(())
    }
}

impl embedded_hal_02::serial::Read<u8> for LpuartRx<'_, Blocking> {
    type Error = Error;

    fn read(&mut self) -> core::result::Result<u8, nb::Error<Self::Error>> {
        let mut buf = [0; 1];
        match self.read(&mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(Error::RxFifoEmpty) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_02::serial::Write<u8> for LpuartTx<'_, Blocking> {
    type Error = Error;

    fn write(&mut self, word: u8) -> core::result::Result<(), nb::Error<Self::Error>> {
        match self.write(&[word]) {
            Ok(_) => Ok(()),
            Err(Error::TxFifoFull) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }

    fn flush(&mut self) -> core::result::Result<(), nb::Error<Self::Error>> {
        match self.flush() {
            Ok(_) => Ok(()),
            Err(Error::TxBusy) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_02::blocking::serial::Write<u8> for LpuartTx<'_, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> core::result::Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_hal_02::serial::Read<u8> for Lpuart<'_, Blocking> {
    type Error = Error;

    fn read(&mut self) -> core::result::Result<u8, nb::Error<Self::Error>> {
        embedded_hal_02::serial::Read::read(&mut self.rx)
    }
}

impl embedded_hal_02::serial::Write<u8> for Lpuart<'_, Blocking> {
    type Error = Error;

    fn write(&mut self, word: u8) -> core::result::Result<(), nb::Error<Self::Error>> {
        embedded_hal_02::serial::Write::write(&mut self.tx, word)
    }

    fn flush(&mut self) -> core::result::Result<(), nb::Error<Self::Error>> {
        embedded_hal_02::serial::Write::flush(&mut self.tx)
    }
}

impl embedded_hal_02::blocking::serial::Write<u8> for Lpuart<'_, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> core::result::Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_hal_nb::serial::Read for LpuartRx<'_, Blocking> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buf = [0; 1];
        match self.read(&mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(Error::RxFifoEmpty) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_nb::serial::Write for LpuartTx<'_, Blocking> {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        match self.write(&[word]) {
            Ok(_) => Ok(()),
            Err(Error::TxFifoFull) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        match self.flush() {
            Ok(_) => Ok(()),
            Err(Error::TxBusy) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl core::fmt::Write for LpuartTx<'_, Blocking> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.blocking_write(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

impl embedded_hal_nb::serial::Read for Lpuart<'_, Blocking> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        embedded_hal_nb::serial::Read::read(&mut self.rx)
    }
}

impl embedded_hal_nb::serial::Write for Lpuart<'_, Blocking> {
    fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
        embedded_hal_nb::serial::Write::write(&mut self.tx, char)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        embedded_hal_nb::serial::Write::flush(&mut self.tx)
    }
}

impl embedded_io::Read for LpuartRx<'_, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.blocking_read(buf).map(|_| buf.len())
    }
}

impl embedded_io::Write for LpuartTx<'_, Blocking> {
    fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.blocking_write(buf).map(|_| buf.len())
    }

    fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_io::Read for Lpuart<'_, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        embedded_io::Read::read(&mut self.rx, buf)
    }
}

impl embedded_io::Write for Lpuart<'_, Blocking> {
    fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        embedded_io::Write::write(&mut self.tx, buf)
    }

    fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        embedded_io::Write::flush(&mut self.tx)
    }
}
