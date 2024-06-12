//! Asynchronous shared I2C bus
//!
//! # Example (nrf52)
//!
//! ```rust,ignore
//! use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
//! use embassy_sync::mutex::Mutex;
//! use embassy_sync::blocking_mutex::raw::NoopRawMutex;
//!
//! static I2C_BUS: StaticCell<Mutex<NoopRawMutex, Twim<TWISPI0>>> = StaticCell::new();
//! let config = twim::Config::default();
//! let i2c = Twim::new(p.TWISPI0, Irqs, p.P0_03, p.P0_04, config);
//! let i2c_bus = Mutex::new(i2c);
//! let i2c_bus = I2C_BUS.init(i2c_bus);
//!
//! // Device 1, using embedded-hal-async compatible driver for QMC5883L compass
//! let i2c_dev1 = I2cDevice::new(i2c_bus);
//! let compass = QMC5883L::new(i2c_dev1).await.unwrap();
//!
//! // Device 2, using embedded-hal-async compatible driver for Mpu6050 accelerometer
//! let i2c_dev2 = I2cDevice::new(i2c_bus);
//! let mpu = Mpu6050::new(i2c_dev2);
//! ```

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;
use embedded_hal_async::i2c;

use crate::shared_bus::I2cDeviceError;
use crate::SetConfig;

/// I2C device on a shared bus.
pub struct I2cDevice<'a, M: RawMutex, BUS> {
    bus: &'a Mutex<M, BUS>,
}

impl<'a, M: RawMutex, BUS> I2cDevice<'a, M, BUS> {
    /// Create a new `I2cDevice`.
    pub fn new(bus: &'a Mutex<M, BUS>) -> Self {
        Self { bus }
    }
}

impl<'a, M: RawMutex, BUS> i2c::ErrorType for I2cDevice<'a, M, BUS>
where
    BUS: i2c::ErrorType,
{
    type Error = I2cDeviceError<BUS::Error>;
}

impl<M, BUS> i2c::I2c for I2cDevice<'_, M, BUS>
where
    M: RawMutex + 'static,
    BUS: i2c::I2c + 'static,
{
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), I2cDeviceError<BUS::Error>> {
        let mut bus = self.bus.lock().await;
        bus.read(address, read).await.map_err(I2cDeviceError::I2c)?;
        Ok(())
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), I2cDeviceError<BUS::Error>> {
        let mut bus = self.bus.lock().await;
        bus.write(address, write).await.map_err(I2cDeviceError::I2c)?;
        Ok(())
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), I2cDeviceError<BUS::Error>> {
        let mut bus = self.bus.lock().await;
        bus.write_read(address, write, read)
            .await
            .map_err(I2cDeviceError::I2c)?;
        Ok(())
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), I2cDeviceError<BUS::Error>> {
        let mut bus = self.bus.lock().await;
        bus.transaction(address, operations)
            .await
            .map_err(I2cDeviceError::I2c)?;
        Ok(())
    }
}

/// I2C device on a shared bus, with its own configuration.
///
/// This is like [`I2cDevice`], with an additional bus configuration that's applied
/// to the bus before each use using [`SetConfig`]. This allows different
/// devices on the same bus to use different communication settings.
pub struct I2cDeviceWithConfig<'a, M: RawMutex, BUS: SetConfig> {
    bus: &'a Mutex<M, BUS>,
    config: BUS::Config,
}

impl<'a, M: RawMutex, BUS: SetConfig> I2cDeviceWithConfig<'a, M, BUS> {
    /// Create a new `I2cDeviceWithConfig`.
    pub fn new(bus: &'a Mutex<M, BUS>, config: BUS::Config) -> Self {
        Self { bus, config }
    }

    /// Change the device's config at runtime
    pub fn set_config(&mut self, config: BUS::Config) {
        self.config = config;
    }
}

impl<'a, M, BUS> i2c::ErrorType for I2cDeviceWithConfig<'a, M, BUS>
where
    BUS: i2c::ErrorType,
    M: RawMutex,
    BUS: SetConfig,
{
    type Error = I2cDeviceError<BUS::Error>;
}

impl<M, BUS> i2c::I2c for I2cDeviceWithConfig<'_, M, BUS>
where
    M: RawMutex + 'static,
    BUS: i2c::I2c + SetConfig + 'static,
{
    async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), I2cDeviceError<BUS::Error>> {
        let mut bus = self.bus.lock().await;
        bus.set_config(&self.config).map_err(|_| I2cDeviceError::Config)?;
        bus.read(address, buffer).await.map_err(I2cDeviceError::I2c)?;
        Ok(())
    }

    async fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), I2cDeviceError<BUS::Error>> {
        let mut bus = self.bus.lock().await;
        bus.set_config(&self.config).map_err(|_| I2cDeviceError::Config)?;
        bus.write(address, bytes).await.map_err(I2cDeviceError::I2c)?;
        Ok(())
    }

    async fn write_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), I2cDeviceError<BUS::Error>> {
        let mut bus = self.bus.lock().await;
        bus.set_config(&self.config).map_err(|_| I2cDeviceError::Config)?;
        bus.write_read(address, wr_buffer, rd_buffer)
            .await
            .map_err(I2cDeviceError::I2c)?;
        Ok(())
    }

    async fn transaction(&mut self, address: u8, operations: &mut [i2c::Operation<'_>]) -> Result<(), Self::Error> {
        let mut bus = self.bus.lock().await;
        bus.set_config(&self.config).map_err(|_| I2cDeviceError::Config)?;
        bus.transaction(address, operations)
            .await
            .map_err(I2cDeviceError::I2c)?;
        Ok(())
    }
}
