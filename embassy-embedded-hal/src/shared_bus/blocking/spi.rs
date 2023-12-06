//! Blocking shared SPI bus
//!
//! # Example (nrf52)
//!
//! ```rust,ignore
//! use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
//! use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};
//!
//! static SPI_BUS: StaticCell<NoopMutex<RefCell<Spim<SPI3>>>> = StaticCell::new();
//! let spi = Spim::new_txonly(p.SPI3, Irqs, p.P0_15, p.P0_18, Config::default());
//! let spi_bus = NoopMutex::new(RefCell::new(spi));
//! let spi_bus = SPI_BUS.init(spi_bus);
//!
//! // Device 1, using embedded-hal compatible driver for ST7735 LCD display
//! let cs_pin1 = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
//! let spi_dev1 = SpiDevice::new(spi_bus, cs_pin1);
//! let display1 = ST7735::new(spi_dev1, dc1, rst1, Default::default(), false, 160, 128);
//! ```

use core::cell::RefCell;

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embedded_hal_1::digital::OutputPin;
use embedded_hal_1::spi::{self, Operation, SpiBus};

use crate::shared_bus::SpiDeviceError;
use crate::SetConfig;

/// SPI device on a shared bus.
pub struct SpiDevice<'a, M: RawMutex, BUS, CS> {
    bus: &'a Mutex<M, RefCell<BUS>>,
    cs: CS,
}

impl<'a, M: RawMutex, BUS, CS> SpiDevice<'a, M, BUS, CS> {
    /// Create a new `SpiDevice`.
    pub fn new(bus: &'a Mutex<M, RefCell<BUS>>, cs: CS) -> Self {
        Self { bus, cs }
    }
}

impl<'a, M: RawMutex, BUS, CS> spi::ErrorType for SpiDevice<'a, M, BUS, CS>
where
    BUS: spi::ErrorType,
    CS: OutputPin,
{
    type Error = SpiDeviceError<BUS::Error, CS::Error>;
}

impl<BUS, M, CS> embedded_hal_1::spi::SpiDevice for SpiDevice<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: SpiBus,
    CS: OutputPin,
{
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        if cfg!(not(feature = "time")) && operations.iter().any(|op| matches!(op, Operation::DelayNs(_))) {
            return Err(SpiDeviceError::DelayNotSupported);
        }

        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            self.cs.set_low().map_err(SpiDeviceError::Cs)?;

            let op_res = operations.iter_mut().try_for_each(|op| match op {
                Operation::Read(buf) => bus.read(buf),
                Operation::Write(buf) => bus.write(buf),
                Operation::Transfer(read, write) => bus.transfer(read, write),
                Operation::TransferInPlace(buf) => bus.transfer_in_place(buf),
                #[cfg(not(feature = "time"))]
                Operation::DelayNs(_) => unreachable!(),
                #[cfg(feature = "time")]
                Operation::DelayNs(ns) => {
                    embassy_time::block_for(embassy_time::Duration::from_nanos(*ns as _));
                    Ok(())
                }
            });

            // On failure, it's important to still flush and deassert CS.
            let flush_res = bus.flush();
            let cs_res = self.cs.set_high();

            let op_res = op_res.map_err(SpiDeviceError::Spi)?;
            flush_res.map_err(SpiDeviceError::Spi)?;
            cs_res.map_err(SpiDeviceError::Cs)?;

            Ok(op_res)
        })
    }
}

impl<'d, M, BUS, CS, BusErr, CsErr> embedded_hal_02::blocking::spi::Transfer<u8> for SpiDevice<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: embedded_hal_02::blocking::spi::Transfer<u8, Error = BusErr>,
    CS: OutputPin<Error = CsErr>,
{
    type Error = SpiDeviceError<BusErr, CsErr>;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            self.cs.set_low().map_err(SpiDeviceError::Cs)?;
            let op_res = bus.transfer(words);
            let cs_res = self.cs.set_high();
            let op_res = op_res.map_err(SpiDeviceError::Spi)?;
            cs_res.map_err(SpiDeviceError::Cs)?;
            Ok(op_res)
        })
    }
}

impl<'d, M, BUS, CS, BusErr, CsErr> embedded_hal_02::blocking::spi::Write<u8> for SpiDevice<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: embedded_hal_02::blocking::spi::Write<u8, Error = BusErr>,
    CS: OutputPin<Error = CsErr>,
{
    type Error = SpiDeviceError<BusErr, CsErr>;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            self.cs.set_low().map_err(SpiDeviceError::Cs)?;
            let op_res = bus.write(words);
            let cs_res = self.cs.set_high();
            let op_res = op_res.map_err(SpiDeviceError::Spi)?;
            cs_res.map_err(SpiDeviceError::Cs)?;
            Ok(op_res)
        })
    }
}

/// SPI device on a shared bus, with its own configuration.
///
/// This is like [`SpiDevice`], with an additional bus configuration that's applied
/// to the bus before each use using [`SetConfig`]. This allows different
/// devices on the same bus to use different communication settings.
pub struct SpiDeviceWithConfig<'a, M: RawMutex, BUS: SetConfig, CS> {
    bus: &'a Mutex<M, RefCell<BUS>>,
    cs: CS,
    config: BUS::Config,
}

impl<'a, M: RawMutex, BUS: SetConfig, CS> SpiDeviceWithConfig<'a, M, BUS, CS> {
    /// Create a new `SpiDeviceWithConfig`.
    pub fn new(bus: &'a Mutex<M, RefCell<BUS>>, cs: CS, config: BUS::Config) -> Self {
        Self { bus, cs, config }
    }
}

impl<'a, M, BUS, CS> spi::ErrorType for SpiDeviceWithConfig<'a, M, BUS, CS>
where
    M: RawMutex,
    BUS: spi::ErrorType + SetConfig,
    CS: OutputPin,
{
    type Error = SpiDeviceError<BUS::Error, CS::Error>;
}

impl<BUS, M, CS> embedded_hal_1::spi::SpiDevice for SpiDeviceWithConfig<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: SpiBus + SetConfig,
    CS: OutputPin,
{
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        if cfg!(not(feature = "time")) && operations.iter().any(|op| matches!(op, Operation::DelayNs(_))) {
            return Err(SpiDeviceError::DelayNotSupported);
        }

        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            bus.set_config(&self.config).map_err(|_| SpiDeviceError::Config)?;
            self.cs.set_low().map_err(SpiDeviceError::Cs)?;

            let op_res = operations.iter_mut().try_for_each(|op| match op {
                Operation::Read(buf) => bus.read(buf),
                Operation::Write(buf) => bus.write(buf),
                Operation::Transfer(read, write) => bus.transfer(read, write),
                Operation::TransferInPlace(buf) => bus.transfer_in_place(buf),
                #[cfg(not(feature = "time"))]
                Operation::DelayNs(_) => unreachable!(),
                #[cfg(feature = "time")]
                Operation::DelayNs(ns) => {
                    embassy_time::block_for(embassy_time::Duration::from_nanos(*ns as _));
                    Ok(())
                }
            });

            // On failure, it's important to still flush and deassert CS.
            let flush_res = bus.flush();
            let cs_res = self.cs.set_high();

            let op_res = op_res.map_err(SpiDeviceError::Spi)?;
            flush_res.map_err(SpiDeviceError::Spi)?;
            cs_res.map_err(SpiDeviceError::Cs)?;
            Ok(op_res)
        })
    }
}
