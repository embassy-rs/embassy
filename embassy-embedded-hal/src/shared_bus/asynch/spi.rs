//! Asynchronous shared SPI bus
//!
//! # Example (nrf52)
//!
//! ```rust
//! use embassy_embedded_hal::shared_bus::spi::SpiDevice;
//! use embassy_sync::mutex::Mutex;
//! use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
//!
//! static SPI_BUS: StaticCell<Mutex<ThreadModeRawMutex, spim::Spim<SPI3>>> = StaticCell::new();
//! let mut config = spim::Config::default();
//! config.frequency = spim::Frequency::M32;
//! let irq = interrupt::take!(SPIM3);
//! let spi = spim::Spim::new_txonly(p.SPI3, irq, p.P0_15, p.P0_18, config);
//! let spi_bus = Mutex::<ThreadModeRawMutex, _>::new(spi);
//! let spi_bus = SPI_BUS.init(spi_bus);
//!
//! // Device 1, using embedded-hal-async compatible driver for ST7735 LCD display
//! let cs_pin1 = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
//! let spi_dev1 = SpiDevice::new(spi_bus, cs_pin1);
//! let display1 = ST7735::new(spi_dev1, dc1, rst1, Default::default(), 160, 128);
//!
//! // Device 2
//! let cs_pin2 = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
//! let spi_dev2 = SpiDevice::new(spi_bus, cs_pin2);
//! let display2 = ST7735::new(spi_dev2, dc2, rst2, Default::default(), 160, 128);
//! ```
use core::future::Future;

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;
use embedded_hal_1::digital::OutputPin;
use embedded_hal_1::spi::ErrorType;
use embedded_hal_async::spi;

use crate::shared_bus::SpiDeviceError;
use crate::SetConfig;

/// SPI device on a shared bus.
pub struct SpiDevice<'a, M: RawMutex, BUS, CS> {
    bus: &'a Mutex<M, BUS>,
    cs: CS,
}

impl<'a, M: RawMutex, BUS, CS> SpiDevice<'a, M, BUS, CS> {
    /// Create a new `SpiDevice`.
    pub fn new(bus: &'a Mutex<M, BUS>, cs: CS) -> Self {
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

unsafe impl<M, BUS, CS> spi::SpiDevice for SpiDevice<'_, M, BUS, CS>
where
    M: RawMutex + 'static,
    BUS: spi::SpiBusFlush + 'static,
    CS: OutputPin,
{
    type Bus = BUS;

    type TransactionFuture<'a, R, F, Fut> = impl Future<Output = Result<R, Self::Error>> + 'a
    where
        Self: 'a, R: 'a, F: FnOnce(*mut Self::Bus) -> Fut + 'a,
        Fut: Future<Output =  Result<R, <Self::Bus as ErrorType>::Error>> + 'a;

    fn transaction<'a, R, F, Fut>(&'a mut self, f: F) -> Self::TransactionFuture<'a, R, F, Fut>
    where
        R: 'a,
        F: FnOnce(*mut Self::Bus) -> Fut + 'a,
        Fut: Future<Output = Result<R, <Self::Bus as ErrorType>::Error>> + 'a,
    {
        async move {
            let mut bus = self.bus.lock().await;
            self.cs.set_low().map_err(SpiDeviceError::Cs)?;

            let f_res = f(&mut *bus).await;

            // On failure, it's important to still flush and deassert CS.
            let flush_res = bus.flush().await;
            let cs_res = self.cs.set_high();

            let f_res = f_res.map_err(SpiDeviceError::Spi)?;
            flush_res.map_err(SpiDeviceError::Spi)?;
            cs_res.map_err(SpiDeviceError::Cs)?;

            Ok(f_res)
        }
    }
}

/// SPI device on a shared bus, with its own configuration.
///
/// This is like [`SpiDevice`], with an additional bus configuration that's applied
/// to the bus before each use using [`SetConfig`]. This allows different
/// devices on the same bus to use different communication settings.
pub struct SpiDeviceWithConfig<'a, M: RawMutex, BUS: SetConfig, CS> {
    bus: &'a Mutex<M, BUS>,
    cs: CS,
    config: BUS::Config,
}

impl<'a, M: RawMutex, BUS: SetConfig, CS> SpiDeviceWithConfig<'a, M, BUS, CS> {
    /// Create a new `SpiDeviceWithConfig`.
    pub fn new(bus: &'a Mutex<M, BUS>, cs: CS, config: BUS::Config) -> Self {
        Self { bus, cs, config }
    }
}

impl<'a, M, BUS, CS> spi::ErrorType for SpiDeviceWithConfig<'a, M, BUS, CS>
where
    BUS: spi::ErrorType + SetConfig,
    CS: OutputPin,
    M: RawMutex,
{
    type Error = SpiDeviceError<BUS::Error, CS::Error>;
}

unsafe impl<M, BUS, CS> spi::SpiDevice for SpiDeviceWithConfig<'_, M, BUS, CS>
where
    M: RawMutex + 'static,
    BUS: spi::SpiBusFlush + SetConfig + 'static,
    CS: OutputPin,
{
    type Bus = BUS;

    type TransactionFuture<'a, R, F, Fut> = impl Future<Output = Result<R, Self::Error>> + 'a
    where
        Self: 'a, R: 'a, F: FnOnce(*mut Self::Bus) -> Fut + 'a,
        Fut: Future<Output =  Result<R, <Self::Bus as ErrorType>::Error>> + 'a;

    fn transaction<'a, R, F, Fut>(&'a mut self, f: F) -> Self::TransactionFuture<'a, R, F, Fut>
    where
        R: 'a,
        F: FnOnce(*mut Self::Bus) -> Fut + 'a,
        Fut: Future<Output = Result<R, <Self::Bus as ErrorType>::Error>> + 'a,
    {
        async move {
            let mut bus = self.bus.lock().await;
            bus.set_config(&self.config);
            self.cs.set_low().map_err(SpiDeviceError::Cs)?;

            let f_res = f(&mut *bus).await;

            // On failure, it's important to still flush and deassert CS.
            let flush_res = bus.flush().await;
            let cs_res = self.cs.set_high();

            let f_res = f_res.map_err(SpiDeviceError::Spi)?;
            flush_res.map_err(SpiDeviceError::Spi)?;
            cs_res.map_err(SpiDeviceError::Cs)?;

            Ok(f_res)
        }
    }
}
