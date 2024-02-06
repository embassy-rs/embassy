//! OCTOSPI Serial Peripheral Interface
//!

#![macro_use]

pub mod enums;

use core::ops::Add;
use core::ptr;

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embedded_hal_02::blocking::i2c::Operation;
pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use enums::*;

use crate::dma::{slice_ptr_parts, word, Transfer};
use crate::gpio::sealed::{AFType, Pin as _};
use crate::gpio::{AnyPin, Pull};
use crate::pac::octospi::{regs, vals, Octospi as Regs};
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use crate::{peripherals, Peripheral};

/// OPSI driver config.
pub struct Config;

/// OSPI transfer configuration.
pub struct TransferConfig {
    /// Instruction width (IMODE)
    pub iwidth: OspiWidth,
    /// Instruction Id
    pub instruction: Option<u32>,
    /// Number of Instruction Bytes
    pub isize: AddressSize,
    /// Instruction Double Transfer rate enable
    pub idtr: bool,

    /// Address width (ADMODE)
    pub adwidth: OspiWidth,
    /// Device memory address
    pub address: Option<u32>,
    /// Number of Address Bytes
    pub adsize: AddressSize,
    /// Address Double Transfer rate enable
    pub addtr: bool,

    /// Alternate bytes width (ABMODE)
    pub abwidth: OspiWidth,
    /// Alternate Bytes
    pub alternate_bytes: Option<u32>,
    /// Number of Alternate Bytes
    pub absize: AddressSize,
    /// Alternate Bytes Double Transfer rate enable
    pub abdtr: bool,

    /// Data width (DMODE)
    pub dwidth: OspiWidth,
    /// Length of data
    pub data_len: Option<usize>,
    /// Data buffer
    pub ddtr: bool,

    /// Number of dummy cycles (DCYC)
    pub dummy: DummyCycles,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            iwidth: OspiWidth::NONE,
            instruction: None,
            isize: AddressSize::_8Bit,
            idtr: false,

            adwidth: OspiWidth::NONE,
            address: None,
            adsize: AddressSize::_8Bit,
            addtr: false,

            abwidth: OspiWidth::NONE,
            alternate_bytes: None,
            absize: AddressSize::_8Bit,
            abdtr: false,

            dwidth: OspiWidth::NONE,
            data_len: None,
            ddtr: false,

            dummy: DummyCycles::_0,
        }
    }
}

pub enum OspiError {
    Test,
}

pub trait Error {}

pub trait ErrorType {
    type Error: Error;
}

impl<T: ErrorType + ?Sized> ErrorType for &mut T {
    type Error = T::Error;
}

/// MultiSpi interface trait
pub trait MultiSpi: ErrorType {
    /// Transaction configuration for specific multispi implementation
    type Config;

    /// Command function used for a configuration operation, when no user data is
    /// supplied to or read from the target device.
    async fn command(&mut self, config: Self::Config) -> Result<(), Self::Error>;

    /// Read function used to read data from the target device following the supplied transaction
    /// configuration.
    async fn read(&mut self, data: &mut [u8], config: Self::Config) -> Result<(), Self::Error>;

    /// Write function used to send data to the target device following the supplied transaction
    /// configuration.
    async fn write(&mut self, data: &mut [u8], config: Self::Config) -> Result<(), Self::Error>;
}

/// OSPI driver.
pub struct Ospi<'d, T: Instance, Dma> {
    _peri: PeripheralRef<'d, T>,
    sck: Option<PeripheralRef<'d, AnyPin>>,
    d0: Option<PeripheralRef<'d, AnyPin>>,
    d1: Option<PeripheralRef<'d, AnyPin>>,
    d2: Option<PeripheralRef<'d, AnyPin>>,
    d3: Option<PeripheralRef<'d, AnyPin>>,
    d4: Option<PeripheralRef<'d, AnyPin>>,
    d5: Option<PeripheralRef<'d, AnyPin>>,
    d6: Option<PeripheralRef<'d, AnyPin>>,
    d7: Option<PeripheralRef<'d, AnyPin>>,
    nss: Option<PeripheralRef<'d, AnyPin>>,
    dqs: Option<PeripheralRef<'d, AnyPin>>,
    dma: PeripheralRef<'d, Dma>,
    config: Config,
}

impl Error for OspiError {}

impl<'d, T: Instance, Dma> ErrorType for Ospi<'d, T, Dma> {
    type Error = OspiError;
}

impl<'d, T: Instance, Dma: OctoDma<T>> MultiSpi for Ospi<'d, T, Dma> {
    type Config = TransferConfig;

    async fn command(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn read(&mut self, data: &mut [u8], config: Self::Config) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(&mut self, data: &mut [u8], config: Self::Config) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'d, T: Instance, Dma> Ospi<'d, T, Dma> {
    /// Create new OSPI driver for a dualspi external chip
    pub fn new_dualspi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, sck, d0, d1, nss);

        sck.set_as_af_pull(sck.af_num(), AFType::OutputPushPull, Pull::None);
        sck.set_speed(crate::gpio::Speed::VeryHigh);
        nss.set_as_af_pull(nss.af_num(), AFType::OutputPushPull, Pull::Up);
        nss.set_speed(crate::gpio::Speed::VeryHigh);
        // nss.set_as_af_pull(nss.af_num(), AFType::OutputPushPull, Pull::Down);
        // nss.set_speed(crate::gpio::Speed::VeryHigh);
        d0.set_as_af_pull(d0.af_num(), AFType::OutputPushPull, Pull::None);
        d0.set_speed(crate::gpio::Speed::VeryHigh);
        d1.set_as_af_pull(d1.af_num(), AFType::OutputPushPull, Pull::None);
        d1.set_speed(crate::gpio::Speed::VeryHigh);

        #[cfg(octospi_v1)]
        {
            T::REGS.ccr().modify(|w| {
                w.set_imode(vals::PhaseMode::TWOLINES);
                w.set_admode(vals::PhaseMode::TWOLINES);
                w.set_abmode(vals::PhaseMode::TWOLINES);
                w.set_dmode(vals::PhaseMode::TWOLINES);
            });
            T::REGS.wccr().modify(|w| {
                w.set_imode(vals::PhaseMode::TWOLINES);
                w.set_admode(vals::PhaseMode::TWOLINES);
                w.set_abmode(vals::PhaseMode::TWOLINES);
                w.set_dmode(vals::PhaseMode::TWOLINES);
            });
        }

        Self::new_inner(
            peri,
            Some(d0.map_into()),
            Some(d1.map_into()),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(sck.map_into()),
            Some(nss.map_into()),
            None,
            dma,
            config,
        )
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        d0: Option<PeripheralRef<'d, AnyPin>>,
        d1: Option<PeripheralRef<'d, AnyPin>>,
        d2: Option<PeripheralRef<'d, AnyPin>>,
        d3: Option<PeripheralRef<'d, AnyPin>>,
        d4: Option<PeripheralRef<'d, AnyPin>>,
        d5: Option<PeripheralRef<'d, AnyPin>>,
        d6: Option<PeripheralRef<'d, AnyPin>>,
        d7: Option<PeripheralRef<'d, AnyPin>>,
        sck: Option<PeripheralRef<'d, AnyPin>>,
        nss: Option<PeripheralRef<'d, AnyPin>>,
        dqs: Option<PeripheralRef<'d, AnyPin>>,
        dma: impl Peripheral<P = Dma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);

        T::enable_and_reset();
        while T::REGS.sr().read().busy() {}

        T::REGS.cr().modify(|w| {
            w.set_en(true);
        });

        T::REGS.dcr1().modify(|w| {
            w.set_devsize(23);
            w.set_mtyp(vals::MemType::MACRONIX);
            w.set_ckmode(false);
            // w.se
        });

        Self {
            _peri: peri,
            sck,
            d0,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,
            nss,
            dqs,
            dma,
            config,
        }
    }

    pub fn blocking_read(&mut self, transaction: TransferConfig) -> Result<(), ()> {
        Ok(())
    }

    fn configure_command(&mut self, command: &TransferConfig) -> Result<(), ()> {
        Ok(())
    }

    /// Poor attempt to read data from memory
    pub fn receive(&mut self, buf: &mut [u8], intruction: u8, data_len: usize) -> Result<(), ()> {
        T::REGS.cr().modify(|w| {
            w.set_fmode(vals::FunctionalMode::INDIRECTREAD);
        });

        T::REGS.ccr().modify(|w| {
            w.set_imode(vals::PhaseMode::ONELINE);
            w.set_admode(vals::PhaseMode::NONE);
            w.set_abmode(vals::PhaseMode::NONE);

            w.set_dmode(vals::PhaseMode::ONELINE);
        });

        T::REGS.dlr().modify(|w| {
            w.set_dl((data_len - 1) as u32);
        });

        // set instruction
        T::REGS.ir().modify(|w| w.set_instruction(intruction as u32));

        // read bytes
        // for idx in 0..data_len {
        //     while !T::REGS.sr().read().tcf() && !T::REGS.sr().read().ftf() {}
        //     buf[idx] = unsafe { (T::REGS.dr().as_ptr() as *mut u8).read_volatile() };
        // }
        // wait for finish
        while !T::REGS.sr().read().tcf() {}

        let fifo_count = T::REGS.sr().read().flevel();
        for idx in 0..(fifo_count as usize) {
            buf[idx] = unsafe { (T::REGS.dr().as_ptr() as *mut u8).read_volatile() };
        }

        Ok(())
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        const REGS: Regs;
    }
}

/// OSPI instance trait.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + RccPeripheral {}

pin_trait!(SckPin, Instance);
pin_trait!(NckPin, Instance);
pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);
pin_trait!(DQSPin, Instance);
pin_trait!(NSSPin, Instance);

dma_trait!(OctoDma, Instance);

foreach_peripheral!(
    (octospi, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);
