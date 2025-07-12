//! Blocking shared I2C bus
//!
//! # Example (nrf52)
//!
//! ```rust,ignore
//! use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
//! use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};
//!
//! static I2C_BUS: StaticCell<NoopMutex<RefCell<Twim<TWISPI0>>>> = StaticCell::new();
//! let i2c = Twim::new(p.TWISPI0, Irqs, p.P0_03, p.P0_04, Config::default());
//! let i2c_bus = NoopMutex::new(RefCell::new(i2c));
//! let i2c_bus = I2C_BUS.init(i2c_bus);
//!
//! let i2c_dev1 = I2cDevice::new(i2c_bus);
//! let mpu = Mpu6050::new(i2c_dev1);
//! ```

use core::cell::RefCell;

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embedded_hal_1::i2c::{ErrorType, I2c, Operation};

use crate::shared_bus::I2cDeviceError;
use crate::SetConfig;

/// I2C device on a shared bus.
pub struct I2cDevice<'a, M: RawMutex, BUS> {
    bus: &'a Mutex<M, RefCell<BUS>>,
}

impl<'a, M: RawMutex, BUS> I2cDevice<'a, M, BUS> {
    /// Create a new `I2cDevice`.
    pub fn new(bus: &'a Mutex<M, RefCell<BUS>>) -> Self {
        Self { bus }
    }
}

impl<'a, M: RawMutex, BUS> ErrorType for I2cDevice<'a, M, BUS>
where
    BUS: ErrorType,
{
    type Error = I2cDeviceError<BUS::Error>;
}

impl<M, BUS> I2c for I2cDevice<'_, M, BUS>
where
    M: RawMutex,
    BUS: I2c,
{
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.bus
            .lock(|bus| bus.borrow_mut().read(address, buffer).map_err(I2cDeviceError::I2c))
    }

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.bus
            .lock(|bus| bus.borrow_mut().write(address, bytes).map_err(I2cDeviceError::I2c))
    }

    fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            bus.borrow_mut()
                .write_read(address, wr_buffer, rd_buffer)
                .map_err(I2cDeviceError::I2c)
        })
    }

    fn transaction<'a>(&mut self, address: u8, operations: &mut [Operation<'a>]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            bus.borrow_mut()
                .transaction(address, operations)
                .map_err(I2cDeviceError::I2c)
        })
    }
}

impl<'a, M, BUS, E> embedded_hal_02::blocking::i2c::Write for I2cDevice<'_, M, BUS>
where
    M: RawMutex,
    BUS: embedded_hal_02::blocking::i2c::Write<Error = E>,
{
    type Error = I2cDeviceError<E>;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Self::Error> {
        self.bus
            .lock(|bus| bus.borrow_mut().write(addr, bytes).map_err(I2cDeviceError::I2c))
    }
}

impl<'a, M, BUS, E> embedded_hal_02::blocking::i2c::Read for I2cDevice<'_, M, BUS>
where
    M: RawMutex,
    BUS: embedded_hal_02::blocking::i2c::Read<Error = E>,
{
    type Error = I2cDeviceError<E>;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Self::Error> {
        self.bus
            .lock(|bus| bus.borrow_mut().read(addr, bytes).map_err(I2cDeviceError::I2c))
    }
}

impl<'a, M, BUS, E> embedded_hal_02::blocking::i2c::WriteRead for I2cDevice<'_, M, BUS>
where
    M: RawMutex,
    BUS: embedded_hal_02::blocking::i2c::WriteRead<Error = E>,
{
    type Error = I2cDeviceError<E>;

    fn write_read<'w>(&mut self, addr: u8, bytes: &'w [u8], buffer: &'w mut [u8]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            bus.borrow_mut()
                .write_read(addr, bytes, buffer)
                .map_err(I2cDeviceError::I2c)
        })
    }
}

/// I2C device on a shared bus, with its own configuration.
///
/// This is like [`I2cDevice`], with an additional bus configuration that's applied
/// to the bus before each use using [`SetConfig`]. This allows different
/// devices on the same bus to use different communication settings.
pub struct I2cDeviceWithConfig<'a, M: RawMutex, BUS: SetConfig> {
    bus: &'a Mutex<M, RefCell<BUS>>,
    config: BUS::Config,
}

impl<'a, M: RawMutex, BUS: SetConfig> I2cDeviceWithConfig<'a, M, BUS> {
    /// Create a new `I2cDeviceWithConfig`.
    pub fn new(bus: &'a Mutex<M, RefCell<BUS>>, config: BUS::Config) -> Self {
        Self { bus, config }
    }

    /// Change the device's config at runtime
    pub fn set_config(&mut self, config: BUS::Config) {
        self.config = config;
    }
}

impl<'a, M, BUS> ErrorType for I2cDeviceWithConfig<'a, M, BUS>
where
    M: RawMutex,
    BUS: ErrorType + SetConfig,
{
    type Error = I2cDeviceError<BUS::Error>;
}

impl<M, BUS> I2c for I2cDeviceWithConfig<'_, M, BUS>
where
    M: RawMutex,
    BUS: I2c + SetConfig,
{
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            bus.set_config(&self.config).map_err(|_| I2cDeviceError::Config)?;
            bus.read(address, buffer).map_err(I2cDeviceError::I2c)
        })
    }

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            bus.set_config(&self.config).map_err(|_| I2cDeviceError::Config)?;
            bus.write(address, bytes).map_err(I2cDeviceError::I2c)
        })
    }

    fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            bus.set_config(&self.config).map_err(|_| I2cDeviceError::Config)?;
            bus.write_read(address, wr_buffer, rd_buffer)
                .map_err(I2cDeviceError::I2c)
        })
    }

    fn transaction<'a>(&mut self, address: u8, operations: &mut [Operation<'a>]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            bus.set_config(&self.config).map_err(|_| I2cDeviceError::Config)?;
            bus.transaction(address, operations).map_err(I2cDeviceError::I2c)
        })
    }
}
