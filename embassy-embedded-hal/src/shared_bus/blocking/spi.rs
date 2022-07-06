//! Blocking shared SPI bus
use core::cell::RefCell;

use embassy::blocking_mutex::raw::RawMutex;
use embassy::blocking_mutex::Mutex;
use embedded_hal_1::digital::blocking::OutputPin;
use embedded_hal_1::spi;
use embedded_hal_1::spi::blocking::{SpiBusFlush, SpiDevice};

use crate::shared_bus::spi::SpiBusDeviceError;

pub struct SpiBusDevice<'a, M: RawMutex, BUS, CS> {
    bus: &'a Mutex<M, RefCell<BUS>>,
    cs: CS,
}

impl<'a, M: RawMutex, BUS, CS> SpiBusDevice<'a, M, BUS, CS> {
    pub fn new(bus: &'a Mutex<M, RefCell<BUS>>, cs: CS) -> Self {
        Self { bus, cs }
    }
}

impl<'a, M: RawMutex, BUS, CS> spi::ErrorType for SpiBusDevice<'a, M, BUS, CS>
where
    BUS: spi::ErrorType,
    CS: OutputPin,
{
    type Error = SpiBusDeviceError<BUS::Error, CS::Error>;
}

impl<BUS, M, CS> SpiDevice for SpiBusDevice<'_, M, BUS, CS>
where
    M: RawMutex,
    BUS: SpiBusFlush,
    CS: OutputPin,
{
    type Bus = BUS;

    fn transaction<R>(&mut self, f: impl FnOnce(&mut Self::Bus) -> Result<R, BUS::Error>) -> Result<R, Self::Error> {
        self.bus.lock(|bus| {
            let mut bus = bus.borrow_mut();
            self.cs.set_low().map_err(SpiBusDeviceError::Cs)?;

            let f_res = f(&mut bus);

            // On failure, it's important to still flush and deassert CS.
            let flush_res = bus.flush();
            let cs_res = self.cs.set_high();

            let f_res = f_res.map_err(SpiBusDeviceError::Spi)?;
            flush_res.map_err(SpiBusDeviceError::Spi)?;
            cs_res.map_err(SpiBusDeviceError::Cs)?;

            Ok(f_res)
        })
    }
}
