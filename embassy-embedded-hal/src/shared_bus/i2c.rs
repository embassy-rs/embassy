//! Asynchronous shared I2C bus
//!
//! # Example (nrf52)
//!
//! ```rust
//! use embassy_embedded_hal::shared_bus::i2c::I2cBusDevice;
//! use embassy::mutex::Mutex;
//! use embassy::blocking_mutex::raw::ThreadModeRawMutex;
//!
//! static I2C_BUS: Forever<Mutex::<ThreadModeRawMutex, Twim<TWISPI0>>> = Forever::new();
//! let config = twim::Config::default();
//! let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
//! let i2c = Twim::new(p.TWISPI0, irq, p.P0_03, p.P0_04, config);
//! let i2c_bus = Mutex::<ThreadModeRawMutex, _>::new(i2c);
//! let i2c_bus = I2C_BUS.put(i2c_bus);
//!
//! // Device 1, using embedded-hal-async compatible driver for QMC5883L compass
//! let i2c_dev1 = I2cBusDevice::new(i2c_bus);
//! let compass = QMC5883L::new(i2c_dev1).await.unwrap();
//!
//! // Device 2, using embedded-hal-async compatible driver for Mpu6050 accelerometer
//! let i2c_dev2 = I2cBusDevice::new(i2c_bus);
//! let mpu = Mpu6050::new(i2c_dev2);
//! ```
use core::{fmt::Debug, future::Future};
use embassy::blocking_mutex::raw::RawMutex;
use embassy::mutex::Mutex;
use embedded_hal_async::i2c;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum I2cBusDeviceError<BUS> {
    I2c(BUS),
}

impl<BUS> i2c::Error for I2cBusDeviceError<BUS>
where
    BUS: i2c::Error + Debug,
{
    fn kind(&self) -> i2c::ErrorKind {
        match self {
            Self::I2c(e) => e.kind(),
        }
    }
}

pub struct I2cBusDevice<'a, M: RawMutex, BUS> {
    bus: &'a Mutex<M, BUS>,
}

impl<'a, M: RawMutex, BUS> I2cBusDevice<'a, M, BUS> {
    pub fn new(bus: &'a Mutex<M, BUS>) -> Self {
        Self { bus }
    }
}

impl<'a, M: RawMutex, BUS> i2c::ErrorType for I2cBusDevice<'a, M, BUS>
where
    BUS: i2c::ErrorType,
{
    type Error = I2cBusDeviceError<BUS::Error>;
}

impl<M, BUS> i2c::I2c for I2cBusDevice<'_, M, BUS>
where
    M: RawMutex + 'static,
    BUS: i2c::I2c + 'static,
{
    type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn read<'a>(&'a mut self, address: u8, buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            let mut bus = self.bus.lock().await;
            bus.read(address, buffer)
                .await
                .map_err(I2cBusDeviceError::I2c)?;
            Ok(())
        }
    }

    type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn write<'a>(&'a mut self, address: u8, bytes: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            let mut bus = self.bus.lock().await;
            bus.write(address, bytes)
                .await
                .map_err(I2cBusDeviceError::I2c)?;
            Ok(())
        }
    }

    type WriteReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

    fn write_read<'a>(
        &'a mut self,
        address: u8,
        wr_buffer: &'a [u8],
        rd_buffer: &'a mut [u8],
    ) -> Self::WriteReadFuture<'a> {
        async move {
            let mut bus = self.bus.lock().await;
            bus.write_read(address, wr_buffer, rd_buffer)
                .await
                .map_err(I2cBusDeviceError::I2c)?;
            Ok(())
        }
    }

    type TransactionFuture<'a, 'b> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a, 'b: 'a;

    fn transaction<'a, 'b>(
        &'a mut self,
        address: u8,
        operations: &'a mut [embedded_hal_async::i2c::Operation<'b>],
    ) -> Self::TransactionFuture<'a, 'b> {
        let _ = address;
        let _ = operations;
        async move { todo!() }
    }
}
