//! Blocking shared I2C bus
use core::cell::RefCell;
use core::fmt::Debug;
use core::future::Future;

use embedded_hal_1::i2c;

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

pub struct I2cBusDevice<'a, BUS> {
    bus: &'a RefCell<BUS>,
}

impl<'a, BUS> I2cBusDevice<'a, BUS> {
    pub fn new(bus: &'a RefCell<BUS>) -> Self {
        Self { bus }
    }
}

impl<'a, BUS> i2c::ErrorType for I2cBusDevice<'a, BUS>
where
    BUS: i2c::ErrorType,
{
    type Error = I2cBusDeviceError<BUS::Error>;
}

impl<M, BUS> i2c::I2c for I2cBusDevice<'_, BUS>
where
    BUS: i2c::I2c,
{
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let mut bus = self.bus.borrow_mut();
        bus.read(address, buffer).map_err(I2cBusDeviceError::I2c)?;
        Ok(())
    }

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        let mut bus = self.bus.borrow_mut();
        bus.write(address, bytes).map_err(I2cBusDeviceError::I2c)?;
        Ok(())
    }

    fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
        let mut bus = self.bus.borrow_mut();
        bus.write_read(address, wr_buffer, rd_buffer)
            .map_err(I2cBusDeviceError::I2c)?;
        Ok(())
    }

    fn transaction<'a>(&mut self, address: u8, operations: &mut [i2c::Operation<'a>]) -> Result<(), Self::Error> {
        let _ = address;
        let _ = operations;
        todo!()
    }
}
