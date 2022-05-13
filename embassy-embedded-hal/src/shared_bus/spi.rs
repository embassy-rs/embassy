//! Asynchronous shared SPI bus
//!
//! # Example (nrf52)
//!
//! ```rust
//! use embassy_embedded_hal::shared_bus::spi::SpiBusDevice;
//! use embassy::mutex::Mutex;
//! use embassy::blocking_mutex::raw::ThreadModeRawMutex;
//!
//! static SPI_BUS: Forever<Mutex<ThreadModeRawMutex, spim::Spim<SPI3>>> = Forever::new();
//! let mut config = spim::Config::default();
//! config.frequency = spim::Frequency::M32;
//! let irq = interrupt::take!(SPIM3);
//! let spi = spim::Spim::new_txonly(p.SPI3, irq, p.P0_15, p.P0_18, config);
//! let spi_bus = Mutex::<ThreadModeRawMutex, _>::new(spi);
//! let spi_bus = SPI_BUS.put(spi_bus);
//!
//! // Device 1, using embedded-hal-async compatible driver for ST7735 LCD display
//! let cs_pin1 = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
//! let spi_dev1 = SpiBusDevice::new(spi_bus, cs_pin1);
//! let display1 = ST7735::new(spi_dev1, dc1, rst1, Default::default(), 160, 128);
//!
//! // Device 2
//! let cs_pin2 = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
//! let spi_dev2 = SpiBusDevice::new(spi_bus, cs_pin2);
//! let display2 = ST7735::new(spi_dev2, dc2, rst2, Default::default(), 160, 128);
//! ```
use core::{fmt::Debug, future::Future};
use embassy::blocking_mutex::raw::RawMutex;
use embassy::mutex::Mutex;

use embedded_hal_1::digital::blocking::OutputPin;
use embedded_hal_1::spi::ErrorType;
use embedded_hal_async::spi;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SpiBusDeviceError<BUS, CS> {
    Spi(BUS),
    Cs(CS),
}

impl<BUS, CS> spi::Error for SpiBusDeviceError<BUS, CS>
where
    BUS: spi::Error + Debug,
    CS: Debug,
{
    fn kind(&self) -> spi::ErrorKind {
        match self {
            Self::Spi(e) => e.kind(),
            Self::Cs(_) => spi::ErrorKind::Other,
        }
    }
}

pub struct SpiBusDevice<'a, M: RawMutex, BUS, CS> {
    bus: &'a Mutex<M, BUS>,
    cs: CS,
}

impl<'a, M: RawMutex, BUS, CS> SpiBusDevice<'a, M, BUS, CS> {
    pub fn new(bus: &'a Mutex<M, BUS>, cs: CS) -> Self {
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

impl<M, BUS, CS> spi::SpiDevice for SpiBusDevice<'_, M, BUS, CS>
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
            self.cs.set_low().map_err(SpiBusDeviceError::Cs)?;

            let f_res = f(&mut *bus).await;

            // On failure, it's important to still flush and deassert CS.
            let flush_res = bus.flush().await;
            let cs_res = self.cs.set_high();

            let f_res = f_res.map_err(SpiBusDeviceError::Spi)?;
            flush_res.map_err(SpiBusDeviceError::Spi)?;
            cs_res.map_err(SpiBusDeviceError::Cs)?;

            Ok(f_res)
        }
    }
}
