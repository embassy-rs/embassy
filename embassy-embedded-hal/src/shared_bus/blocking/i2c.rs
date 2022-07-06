//! Blocking shared I2C bus
use core::cell::RefCell;

use embassy::blocking_mutex::raw::RawMutex;
use embassy::blocking_mutex::Mutex;
use embedded_hal_1::i2c::blocking::{I2c, Operation};
use embedded_hal_1::i2c::ErrorType;

use crate::shared_bus::i2c::I2cBusDeviceError;

pub struct I2cBusDevice<'a, M: RawMutex, BUS> {
    bus: &'a Mutex<M, RefCell<BUS>>,
}

impl<'a, M: RawMutex, BUS> I2cBusDevice<'a, M, BUS> {
    pub fn new(bus: &'a Mutex<M, RefCell<BUS>>) -> Self {
        Self { bus }
    }
}

impl<'a, M: RawMutex, BUS> ErrorType for I2cBusDevice<'a, M, BUS>
where
    BUS: ErrorType,
{
    type Error = I2cBusDeviceError<BUS::Error>;
}

impl<M, BUS> I2c for I2cBusDevice<'_, M, BUS>
where
    M: RawMutex,
    BUS: I2c,
{
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.bus
            .lock(|bus| bus.borrow_mut().read(address, buffer).map_err(I2cBusDeviceError::I2c))
    }

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.bus
            .lock(|bus| bus.borrow_mut().write(address, bytes).map_err(I2cBusDeviceError::I2c))
    }

    fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.bus.lock(|bus| {
            bus.borrow_mut()
                .write_read(address, wr_buffer, rd_buffer)
                .map_err(I2cBusDeviceError::I2c)
        })
    }

    fn transaction<'a>(&mut self, address: u8, operations: &mut [Operation<'a>]) -> Result<(), Self::Error> {
        let _ = address;
        let _ = operations;
        todo!()
    }

    fn write_iter<B: IntoIterator<Item = u8>>(&mut self, addr: u8, bytes: B) -> Result<(), Self::Error> {
        let _ = addr;
        let _ = bytes;
        todo!()
    }

    fn write_iter_read<B: IntoIterator<Item = u8>>(
        &mut self,
        addr: u8,
        bytes: B,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let _ = addr;
        let _ = bytes;
        let _ = buffer;
        todo!()
    }

    fn transaction_iter<'a, O: IntoIterator<Item = Operation<'a>>>(
        &mut self,
        address: u8,
        operations: O,
    ) -> Result<(), Self::Error> {
        let _ = address;
        let _ = operations;
        todo!()
    }
}
