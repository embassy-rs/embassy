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

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;
use embedded_hal_1::digital::OutputPin;
use embedded_hal_1::spi::Operation;
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

impl<M, BUS, CS> spi::SpiDevice for SpiDevice<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: spi::SpiBus,
    CS: OutputPin,
{
    async fn transaction(&mut self, operations: &mut [spi::Operation<'_, u8>]) -> Result<(), Self::Error> {
        let mut bus = self.bus.lock().await;
        self.cs.set_low().map_err(SpiDeviceError::Cs)?;

        let op_res: Result<(), BUS::Error> = try {
            for op in operations {
                match op {
                    Operation::Read(buf) => bus.read(buf).await?,
                    Operation::Write(buf) => bus.write(buf).await?,
                    Operation::Transfer(read, write) => bus.transfer(read, write).await?,
                    Operation::TransferInPlace(buf) => bus.transfer_in_place(buf).await?,
                    #[cfg(not(feature = "time"))]
                    Operation::DelayUs(_) => return Err(SpiDeviceError::DelayUsNotSupported),
                    #[cfg(feature = "time")]
                    Operation::DelayUs(us) => {
                        embassy_time::Timer::after(embassy_time::Duration::from_micros(*us as _)).await
                    }
                }
            }
        };

        // On failure, it's important to still flush and deassert CS.
        let flush_res = bus.flush().await;
        let cs_res = self.cs.set_high();

        let op_res = op_res.map_err(SpiDeviceError::Spi)?;
        flush_res.map_err(SpiDeviceError::Spi)?;
        cs_res.map_err(SpiDeviceError::Cs)?;

        Ok(op_res)
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

impl<M, BUS, CS> spi::SpiDevice for SpiDeviceWithConfig<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: spi::SpiBus + SetConfig,
    CS: OutputPin,
{
    async fn transaction(&mut self, operations: &mut [spi::Operation<'_, u8>]) -> Result<(), Self::Error> {
        let mut bus = self.bus.lock().await;
        bus.set_config(&self.config);
        self.cs.set_low().map_err(SpiDeviceError::Cs)?;

        let op_res: Result<(), BUS::Error> = try {
            for op in operations {
                match op {
                    Operation::Read(buf) => bus.read(buf).await?,
                    Operation::Write(buf) => bus.write(buf).await?,
                    Operation::Transfer(read, write) => bus.transfer(read, write).await?,
                    Operation::TransferInPlace(buf) => bus.transfer_in_place(buf).await?,
                    #[cfg(not(feature = "time"))]
                    Operation::DelayUs(_) => return Err(SpiDeviceError::DelayUsNotSupported),
                    #[cfg(feature = "time")]
                    Operation::DelayUs(us) => {
                        embassy_time::Timer::after(embassy_time::Duration::from_micros(*us as _)).await
                    }
                }
            }
        };

        // On failure, it's important to still flush and deassert CS.
        let flush_res = bus.flush().await;
        let cs_res = self.cs.set_high();

        let op_res = op_res.map_err(SpiDeviceError::Spi)?;
        flush_res.map_err(SpiDeviceError::Spi)?;
        cs_res.map_err(SpiDeviceError::Cs)?;

        Ok(op_res)
    }
}
