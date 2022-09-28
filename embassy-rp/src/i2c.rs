use core::marker::PhantomData;

use embassy_hal_common::{into_ref, PeripheralRef};
use pac::i2c;

use crate::gpio::sealed::Pin;
use crate::gpio::AnyPin;
use crate::{pac, peripherals, Peripheral};

/// I2C error abort reason
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AbortReason {
    /// A bus operation was not acknowledged, e.g. due to the addressed device
    /// not being available on the bus or the device not being ready to process
    /// requests at the moment
    NoAcknowledge,
    /// The arbitration was lost, e.g. electrical problems with the clock signal
    ArbitrationLoss,
    Other(u32),
}

/// I2C error
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// I2C abort with error
    Abort(AbortReason),
    /// User passed in a read buffer that was 0 length
    InvalidReadBufferLength,
    /// User passed in a write buffer that was 0 length
    InvalidWriteBufferLength,
    /// Target i2c address is out of range
    AddressOutOfRange(u16),
    /// Target i2c address is reserved
    AddressReserved(u16),
}

#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    pub frequency: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { frequency: 100_000 }
    }
}

const FIFO_SIZE: u8 = 16;

pub struct I2c<'d, T: Instance, M: Mode> {
    phantom: PhantomData<(&'d mut T, M)>,
}

impl<'d, T: Instance> I2c<'d, T, Blocking> {
    pub fn new_blocking(
        peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(scl, sda);
        Self::new_inner(peri, scl.map_into(), sda.map_into(), config)
    }

    fn read_blocking_internal(&mut self, buffer: &mut [u8], restart: bool, send_stop: bool) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        let p = T::regs();
        let lastindex = buffer.len() - 1;
        for (i, byte) in buffer.iter_mut().enumerate() {
            let first = i == 0;
            let last = i == lastindex;

            // NOTE(unsafe) We have &mut self
            unsafe {
                // wait until there is space in the FIFO to write the next byte
                while p.ic_txflr().read().txflr() == FIFO_SIZE {}

                p.ic_data_cmd().write(|w| {
                    w.set_restart(restart && first);
                    w.set_stop(send_stop && last);

                    w.set_cmd(true);
                });

                while p.ic_rxflr().read().rxflr() == 0 {
                    self.read_and_clear_abort_reason()?;
                }

                *byte = p.ic_data_cmd().read().dat();
            }
        }

        Ok(())
    }

    fn write_blocking_internal(&mut self, bytes: &[u8], send_stop: bool) -> Result<(), Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidWriteBufferLength);
        }

        let p = T::regs();

        for (i, byte) in bytes.iter().enumerate() {
            let last = i == bytes.len() - 1;

            // NOTE(unsafe) We have &mut self
            unsafe {
                p.ic_data_cmd().write(|w| {
                    w.set_stop(send_stop && last);
                    w.set_dat(*byte);
                });

                // Wait until the transmission of the address/data from the
                // internal shift register has completed. For this to function
                // correctly, the TX_EMPTY_CTRL flag in IC_CON must be set. The
                // TX_EMPTY_CTRL flag was set in i2c_init.
                while !p.ic_raw_intr_stat().read().tx_empty() {}

                let abort_reason = self.read_and_clear_abort_reason();

                if abort_reason.is_err() || (send_stop && last) {
                    // If the transaction was aborted or if it completed
                    // successfully wait until the STOP condition has occured.

                    while !p.ic_raw_intr_stat().read().stop_det() {}

                    p.ic_clr_stop_det().read().clr_stop_det();
                }

                // Note the hardware issues a STOP automatically on an abort
                // condition. Note also the hardware clears RX FIFO as well as
                // TX on abort, ecause we set hwparam
                // IC_AVOID_RX_FIFO_FLUSH_ON_TX_ABRT to 0.
                abort_reason?;
            }
        }
        Ok(())
    }

    // =========================
    // Blocking public API
    // =========================

    pub fn blocking_read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        Self::setup(address.into())?;
        self.read_blocking_internal(buffer, true, true)
        // Automatic Stop
    }

    pub fn blocking_write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        Self::setup(address.into())?;
        self.write_blocking_internal(bytes, true)
    }

    pub fn blocking_write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        Self::setup(address.into())?;
        self.write_blocking_internal(bytes, false)?;
        self.read_blocking_internal(buffer, true, true)
        // Automatic Stop
    }
}
}

impl<'d, T: Instance, M: Mode> I2c<'d, T, M> {
    fn new_inner(
        _peri: impl Peripheral<P = T> + 'd,
        scl: PeripheralRef<'d, AnyPin>,
        sda: PeripheralRef<'d, AnyPin>,
        config: Config,
    ) -> Self {
        into_ref!(_peri);

        assert!(config.frequency <= 1_000_000);
        assert!(config.frequency > 0);

        let p = T::regs();

        unsafe {
            p.ic_enable().write(|w| w.set_enable(false));

            // Select controller mode & speed
            p.ic_con().modify(|w| {
                // Always use "fast" mode (<= 400 kHz, works fine for standard
                // mode too)
                w.set_speed(i2c::vals::Speed::FAST);
                w.set_master_mode(true);
                w.set_ic_slave_disable(true);
                w.set_ic_restart_en(true);
                w.set_tx_empty_ctrl(true);
            });

            // Set FIFO watermarks to 1 to make things simpler. This is encoded
            // by a register value of 0.
            p.ic_tx_tl().write(|w| w.set_tx_tl(0));
            p.ic_rx_tl().write(|w| w.set_rx_tl(0));

            // Configure SCL & SDA pins
            scl.io().ctrl().write(|w| w.set_funcsel(3));
            sda.io().ctrl().write(|w| w.set_funcsel(3));

            scl.pad_ctrl().write(|w| {
                w.set_schmitt(true);
                w.set_ie(true);
                w.set_od(false);
                w.set_pue(true);
                w.set_pde(false);
            });
            sda.pad_ctrl().write(|w| {
                w.set_schmitt(true);
                w.set_ie(true);
                w.set_od(false);
                w.set_pue(true);
                w.set_pde(false);
            });

            // Configure baudrate

            // There are some subtleties to I2C timing which we are completely
            // ignoring here See:
            // https://github.com/raspberrypi/pico-sdk/blob/bfcbefafc5d2a210551a4d9d80b4303d4ae0adf7/src/rp2_common/hardware_i2c/i2c.c#L69
            let clk_base = crate::clocks::clk_peri_freq();

            let period = (clk_base + config.frequency / 2) / config.frequency;
            let lcnt = period * 3 / 5; // spend 3/5 (60%) of the period low
            let hcnt = period - lcnt; // and 2/5 (40%) of the period high

            // Check for out-of-range divisors:
            assert!(hcnt <= 0xffff);
            assert!(lcnt <= 0xffff);
            assert!(hcnt >= 8);
            assert!(lcnt >= 8);

            // Per I2C-bus specification a device in standard or fast mode must
            // internally provide a hold time of at least 300ns for the SDA
            // signal to bridge the undefined region of the falling edge of SCL.
            // A smaller hold time of 120ns is used for fast mode plus.
            let sda_tx_hold_count = if config.frequency < 1_000_000 {
                // sda_tx_hold_count = clk_base [cycles/s] * 300ns * (1s /
                // 1e9ns) Reduce 300/1e9 to 3/1e7 to avoid numbers that don't
                // fit in uint. Add 1 to avoid division truncation.
                ((clk_base * 3) / 10_000_000) + 1
            } else {
                // fast mode plus requires a clk_base > 32MHz
                assert!(clk_base >= 32_000_000);

                // sda_tx_hold_count = clk_base [cycles/s] * 120ns * (1s /
                // 1e9ns) Reduce 120/1e9 to 3/25e6 to avoid numbers that don't
                // fit in uint. Add 1 to avoid division truncation.
                ((clk_base * 3) / 25_000_000) + 1
            };
            assert!(sda_tx_hold_count <= lcnt - 2);

            p.ic_fs_scl_hcnt().write(|w| w.set_ic_fs_scl_hcnt(hcnt as u16));
            p.ic_fs_scl_lcnt().write(|w| w.set_ic_fs_scl_lcnt(lcnt as u16));
            p.ic_fs_spklen()
                .write(|w| w.set_ic_fs_spklen(if lcnt < 16 { 1 } else { (lcnt / 16) as u8 }));
            p.ic_sda_hold()
                .modify(|w| w.set_ic_sda_tx_hold(sda_tx_hold_count as u16));

            // Enable I2C block
            p.ic_enable().write(|w| w.set_enable(true));
        }

        Self {
            phantom: PhantomData,
        }
    }

    fn setup(addr: u16) -> Result<(), Error> {
        if addr >= 0x80 {
            return Err(Error::AddressOutOfRange(addr));
        }

        if i2c_reserved_addr(addr) {
            return Err(Error::AddressReserved(addr));
        }

        let p = T::regs();
        unsafe {
            p.ic_enable().write(|w| w.set_enable(false));
            p.ic_tar().write(|w| w.set_ic_tar(addr));
            p.ic_enable().write(|w| w.set_enable(true));
        }
        Ok(())
    }

    fn read_and_clear_abort_reason(&mut self) -> Result<(), Error> {
        let p = T::regs();
        unsafe {
            let abort_reason = p.ic_tx_abrt_source().read();
            if abort_reason.0 != 0 {
                // Note clearing the abort flag also clears the reason, and this
                // instance of flag is clear-on-read! Note also the
                // IC_CLR_TX_ABRT register always reads as 0.
                p.ic_clr_tx_abrt().read();

                let reason = if abort_reason.abrt_7b_addr_noack()
                    | abort_reason.abrt_10addr1_noack()
                    | abort_reason.abrt_10addr2_noack()
                {
                    AbortReason::NoAcknowledge
                } else if abort_reason.arb_lost() {
                    AbortReason::ArbitrationLoss
                } else {
                    AbortReason::Other(abort_reason.0)
                };

                Err(Error::Abort(reason))
            } else {
                Ok(())
            }
        }
    }

}

mod eh02 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_02::blocking::i2c::Read for I2c<'d, T, Blocking> {
        type Error = Error;

        fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_read(address, buffer)
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::i2c::Write for I2c<'d, T, Blocking> {
        type Error = Error;

        fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(address, bytes)
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, T, Blocking> {
        type Error = Error;

        fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_write_read(address, bytes, buffer)
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::i2c::Error for Error {
        fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
            match *self {
                Self::Abort(AbortReason::ArbitrationLoss) => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
                Self::Abort(AbortReason::NoAcknowledge) => {
                    embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Address)
                }
                Self::Abort(AbortReason::Other(_)) => embedded_hal_1::i2c::ErrorKind::Other,
                Self::InvalidReadBufferLength => embedded_hal_1::i2c::ErrorKind::Other,
                Self::InvalidWriteBufferLength => embedded_hal_1::i2c::ErrorKind::Other,
                Self::AddressOutOfRange(_) => embedded_hal_1::i2c::ErrorKind::Other,
                Self::AddressReserved(_) => embedded_hal_1::i2c::ErrorKind::Other,
            }
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::ErrorType for I2c<'d, T, M> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::i2c::I2c for I2c<'d, T, Blocking> {
        fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_read(address, buffer)
        }

        fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(address, buffer)
        }

        fn write_iter<B>(&mut self, address: u8, bytes: B) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            let mut peekable = bytes.into_iter().peekable();
            Self::setup(address.into())?;

            while let Some(tx) = peekable.next() {
                self.write_blocking_internal(&[tx], peekable.peek().is_none())?;
            }
            Ok(())
        }

        fn write_iter_read<B>(&mut self, address: u8, bytes: B, buffer: &mut [u8]) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            let peekable = bytes.into_iter().peekable();
            Self::setup(address.into())?;

            for tx in peekable {
                self.write_blocking_internal(&[tx], false)?
            }
            self.read_blocking_internal(buffer, true, true)
        }

        fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_write_read(address, wr_buffer, rd_buffer)
        }

        fn transaction<'a>(
            &mut self,
            address: u8,
            operations: &mut [embedded_hal_1::i2c::Operation<'a>],
        ) -> Result<(), Self::Error> {
            Self::setup(address.into())?;
            for i in 0..operations.len() {
                let last = i == operations.len() - 1;
                match &mut operations[i] {
                    embedded_hal_1::i2c::Operation::Read(buf) => self.read_blocking_internal(buf, false, last)?,
                    embedded_hal_1::i2c::Operation::Write(buf) => self.write_blocking_internal(buf, last)?,
                }
            }
            Ok(())
        }

        fn transaction_iter<'a, O>(&mut self, address: u8, operations: O) -> Result<(), Self::Error>
        where
            O: IntoIterator<Item = embedded_hal_1::i2c::Operation<'a>>,
        {
            Self::setup(address.into())?;
            let mut peekable = operations.into_iter().peekable();
            while let Some(operation) = peekable.next() {
                let last = peekable.peek().is_none();
                match operation {
                    embedded_hal_1::i2c::Operation::Read(buf) => self.read_blocking_internal(buf, false, last)?,
                    embedded_hal_1::i2c::Operation::Write(buf) => self.write_blocking_internal(buf, last)?,
                }
            }
            Ok(())
        }
    }
}

fn i2c_reserved_addr(addr: u16) -> bool {
    (addr & 0x78) == 0 || (addr & 0x78) == 0x78
}

mod sealed {
    use embassy_cortex_m::interrupt::Interrupt;

    pub trait Instance {
        const TX_DREQ: u8;
        const RX_DREQ: u8;

        type Interrupt: Interrupt;

        fn regs() -> crate::pac::i2c::I2c;
    }

    pub trait Mode {}

    pub trait SdaPin<T: Instance> {}
    pub trait SclPin<T: Instance> {}
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
    ($type:ident, $irq:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl sealed::Instance for peripherals::$type {
            const TX_DREQ: u8 = $tx_dreq;
            const RX_DREQ: u8 = $rx_dreq;

            type Interrupt = crate::interrupt::$irq;

            #[inline]
            fn regs() -> pac::i2c::I2c {
                pac::$type
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(I2C0, I2C0_IRQ, 32, 33);
impl_instance!(I2C1, I2C1_IRQ, 34, 35);

pub trait SdaPin<T: Instance>: sealed::SdaPin<T> + crate::gpio::Pin {}
pub trait SclPin<T: Instance>: sealed::SclPin<T> + crate::gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl sealed::$function<peripherals::$instance> for peripherals::$pin {}
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, I2C0, SdaPin);
impl_pin!(PIN_1, I2C0, SclPin);
impl_pin!(PIN_2, I2C1, SdaPin);
impl_pin!(PIN_3, I2C1, SclPin);
impl_pin!(PIN_4, I2C0, SdaPin);
impl_pin!(PIN_5, I2C0, SclPin);
impl_pin!(PIN_6, I2C1, SdaPin);
impl_pin!(PIN_7, I2C1, SclPin);
impl_pin!(PIN_8, I2C0, SdaPin);
impl_pin!(PIN_9, I2C0, SclPin);
impl_pin!(PIN_10, I2C1, SdaPin);
impl_pin!(PIN_11, I2C1, SclPin);
impl_pin!(PIN_12, I2C0, SdaPin);
impl_pin!(PIN_13, I2C0, SclPin);
impl_pin!(PIN_14, I2C1, SdaPin);
impl_pin!(PIN_15, I2C1, SclPin);
impl_pin!(PIN_16, I2C0, SdaPin);
impl_pin!(PIN_17, I2C0, SclPin);
impl_pin!(PIN_18, I2C1, SdaPin);
impl_pin!(PIN_19, I2C1, SclPin);
impl_pin!(PIN_20, I2C0, SdaPin);
impl_pin!(PIN_21, I2C0, SclPin);
impl_pin!(PIN_22, I2C1, SdaPin);
impl_pin!(PIN_23, I2C1, SclPin);
impl_pin!(PIN_24, I2C0, SdaPin);
impl_pin!(PIN_25, I2C0, SclPin);
impl_pin!(PIN_26, I2C1, SdaPin);
impl_pin!(PIN_27, I2C1, SclPin);
impl_pin!(PIN_28, I2C0, SdaPin);
impl_pin!(PIN_29, I2C0, SclPin);
