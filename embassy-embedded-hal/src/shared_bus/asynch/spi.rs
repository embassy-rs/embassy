//! Asynchronous shared SPI bus
//!
//! # Example (nrf52)
//!
//! ```rust,ignore
//! use embassy_embedded_hal::shared_bus::spi::SpiDevice;
//! use embassy_sync::mutex::Mutex;
//! use embassy_sync::blocking_mutex::raw::NoopRawMutex;
//!
//! static SPI_BUS: StaticCell<Mutex<NoopRawMutex, spim::Spim<SPI3>>> = StaticCell::new();
//! let mut config = spim::Config::default();
//! config.frequency = spim::Frequency::M32;
//! let spi = spim::Spim::new_txonly(p.SPI3, Irqs, p.P0_15, p.P0_18, config);
//! let spi_bus = Mutex::new(spi);
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

use embassy_hal_internal::drop::OnDrop;
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

impl<M, BUS, CS, Word> spi::SpiDevice<Word> for SpiDevice<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: spi::SpiBus<Word>,
    CS: OutputPin,
    Word: Copy + 'static,
{
    async fn transaction(&mut self, operations: &mut [spi::Operation<'_, Word>]) -> Result<(), Self::Error> {
        if cfg!(not(feature = "time")) && operations.iter().any(|op| matches!(op, Operation::DelayNs(_))) {
            return Err(SpiDeviceError::DelayNotSupported);
        }

        let mut bus = self.bus.lock().await;
        self.cs.set_low().map_err(SpiDeviceError::Cs)?;

        let cs_drop = OnDrop::new(|| {
            // This drop guard deasserts CS pin if the async operation is cancelled.
            // Errors are ignored in this drop handler, as there's nothing we can do about them.
            // If the async operation is completed without cancellation, this handler will not
            // be run, and the CS pin will be deasserted with proper error handling.
            let _ = self.cs.set_high();
        });

        let op_res = 'ops: {
            for op in operations {
                let res = match op {
                    Operation::Read(buf) => bus.read(buf).await,
                    Operation::Write(buf) => bus.write(buf).await,
                    Operation::Transfer(read, write) => bus.transfer(read, write).await,
                    Operation::TransferInPlace(buf) => bus.transfer_in_place(buf).await,
                    #[cfg(not(feature = "time"))]
                    Operation::DelayNs(_) => unreachable!(),
                    #[cfg(feature = "time")]
                    Operation::DelayNs(ns) => match bus.flush().await {
                        Err(e) => Err(e),
                        Ok(()) => {
                            embassy_time::Timer::after_nanos(*ns as _).await;
                            Ok(())
                        }
                    },
                };
                if let Err(e) = res {
                    break 'ops Err(e);
                }
            }
            Ok(())
        };

        // On failure, it's important to still flush and deassert CS.
        let flush_res = bus.flush().await;

        // Now that all the async operations are done, we defuse the CS guard,
        // and manually set the CS pin low (to better handle the possible errors).
        cs_drop.defuse();
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

    /// Change the device's config at runtime
    pub fn set_config(&mut self, config: BUS::Config) {
        self.config = config;
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

impl<M, BUS, CS, Word> spi::SpiDevice<Word> for SpiDeviceWithConfig<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: spi::SpiBus<Word> + SetConfig,
    CS: OutputPin,
    Word: Copy + 'static,
{
    async fn transaction(&mut self, operations: &mut [spi::Operation<'_, Word>]) -> Result<(), Self::Error> {
        if cfg!(not(feature = "time")) && operations.iter().any(|op| matches!(op, Operation::DelayNs(_))) {
            return Err(SpiDeviceError::DelayNotSupported);
        }

        let mut bus = self.bus.lock().await;
        bus.set_config(&self.config).map_err(|_| SpiDeviceError::Config)?;
        self.cs.set_low().map_err(SpiDeviceError::Cs)?;

        let cs_drop = OnDrop::new(|| {
            // Please see comment in SpiDevice for an explanation of this drop handler.
            let _ = self.cs.set_high();
        });

        let op_res = 'ops: {
            for op in operations {
                let res = match op {
                    Operation::Read(buf) => bus.read(buf).await,
                    Operation::Write(buf) => bus.write(buf).await,
                    Operation::Transfer(read, write) => bus.transfer(read, write).await,
                    Operation::TransferInPlace(buf) => bus.transfer_in_place(buf).await,
                    #[cfg(not(feature = "time"))]
                    Operation::DelayNs(_) => unreachable!(),
                    #[cfg(feature = "time")]
                    Operation::DelayNs(ns) => match bus.flush().await {
                        Err(e) => Err(e),
                        Ok(()) => {
                            embassy_time::Timer::after_nanos(*ns as _).await;
                            Ok(())
                        }
                    },
                };
                if let Err(e) = res {
                    break 'ops Err(e);
                }
            }
            Ok(())
        };

        // On failure, it's important to still flush and deassert CS.
        let flush_res = bus.flush().await;
        cs_drop.defuse();
        let cs_res = self.cs.set_high();

        let op_res = op_res.map_err(SpiDeviceError::Spi)?;
        flush_res.map_err(SpiDeviceError::Spi)?;
        cs_res.map_err(SpiDeviceError::Cs)?;

        Ok(op_res)
    }
}
