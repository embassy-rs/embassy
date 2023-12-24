//! Quad Serial Peripheral Interface (QSPI)

#![macro_use]

pub mod enums;

use embassy_hal_internal::{into_ref, PeripheralRef};
use enums::*;

use crate::dma::Transfer;
use crate::gpio::sealed::AFType;
use crate::gpio::{AnyPin, Pull};
use crate::pac::quadspi::Quadspi as Regs;
use crate::rcc::RccPeripheral;
use crate::{peripherals, Peripheral};

/// QSPI transfer configuration.
pub struct TransferConfig {
    /// Instraction width (IMODE)
    pub iwidth: QspiWidth,
    /// Address width (ADMODE)
    pub awidth: QspiWidth,
    /// Data width (DMODE)
    pub dwidth: QspiWidth,
    /// Instruction Id
    pub instruction: u8,
    /// Flash memory address
    pub address: Option<u32>,
    /// Number of dummy cycles (DCYC)
    pub dummy: DummyCycles,
    /// Length of data
    pub data_len: Option<usize>,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            iwidth: QspiWidth::NONE,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: 0,
            address: None,
            dummy: DummyCycles::_0,
            data_len: None,
        }
    }
}

/// QSPI driver configuration.
pub struct Config {
    /// Flash memory size representend as 2^[0-32], as reasonable minimum 1KiB(9) was chosen.
    /// If you need other value the whose predefined use `Other` variant.
    pub memory_size: MemorySize,
    /// Address size (8/16/24/32-bit)
    pub address_size: AddressSize,
    /// Scalar factor for generating CLK [0-255]
    pub prescaler: u8,
    /// Number of bytes to trigger FIFO threshold flag.
    pub fifo_threshold: FIFOThresholdLevel,
    /// Minimum number of cycles that chip select must be high between issued commands
    pub cs_high_time: ChipSelectHighTime,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            memory_size: MemorySize::Other(0),
            address_size: AddressSize::_24bit,
            prescaler: 128,
            fifo_threshold: FIFOThresholdLevel::_17Bytes,
            cs_high_time: ChipSelectHighTime::_5Cycle,
        }
    }
}

/// QSPI driver.
#[allow(dead_code)]
pub struct Qspi<'d, T: Instance, Dma> {
    _peri: PeripheralRef<'d, T>,
    sck: Option<PeripheralRef<'d, AnyPin>>,
    d0: Option<PeripheralRef<'d, AnyPin>>,
    d1: Option<PeripheralRef<'d, AnyPin>>,
    d2: Option<PeripheralRef<'d, AnyPin>>,
    d3: Option<PeripheralRef<'d, AnyPin>>,
    nss: Option<PeripheralRef<'d, AnyPin>>,
    dma: PeripheralRef<'d, Dma>,
    config: Config,
}

impl<'d, T: Instance, Dma> Qspi<'d, T, Dma> {
    /// Create a new QSPI driver for bank 1.
    pub fn new_bk1(
        peri: impl Peripheral<P = T> + 'd,
        d0: impl Peripheral<P = impl BK1D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl BK1D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl BK1D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl BK1D3Pin<T>> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        nss: impl Peripheral<P = impl BK1NSSPin<T>> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, d0, d1, d2, d3, sck, nss);

        sck.set_as_af_pull(sck.af_num(), AFType::OutputPushPull, Pull::None);
        sck.set_speed(crate::gpio::Speed::VeryHigh);
        nss.set_as_af_pull(nss.af_num(), AFType::OutputPushPull, Pull::Up);
        nss.set_speed(crate::gpio::Speed::VeryHigh);
        d0.set_as_af_pull(d0.af_num(), AFType::OutputPushPull, Pull::None);
        d0.set_speed(crate::gpio::Speed::VeryHigh);
        d1.set_as_af_pull(d1.af_num(), AFType::OutputPushPull, Pull::None);
        d1.set_speed(crate::gpio::Speed::VeryHigh);
        d2.set_as_af_pull(d2.af_num(), AFType::OutputPushPull, Pull::None);
        d2.set_speed(crate::gpio::Speed::VeryHigh);
        d3.set_as_af_pull(d3.af_num(), AFType::OutputPushPull, Pull::None);
        d3.set_speed(crate::gpio::Speed::VeryHigh);

        Self::new_inner(
            peri,
            Some(d0.map_into()),
            Some(d1.map_into()),
            Some(d2.map_into()),
            Some(d3.map_into()),
            Some(sck.map_into()),
            Some(nss.map_into()),
            dma,
            config,
            FlashSelection::Flash1,
        )
    }

    /// Create a new QSPI driver for bank 2.
    pub fn new_bk2(
        peri: impl Peripheral<P = T> + 'd,
        d0: impl Peripheral<P = impl BK2D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl BK2D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl BK2D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl BK2D3Pin<T>> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        nss: impl Peripheral<P = impl BK2NSSPin<T>> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, d0, d1, d2, d3, sck, nss);

        sck.set_as_af_pull(sck.af_num(), AFType::OutputPushPull, Pull::None);
        sck.set_speed(crate::gpio::Speed::VeryHigh);
        nss.set_as_af_pull(nss.af_num(), AFType::OutputPushPull, Pull::Up);
        nss.set_speed(crate::gpio::Speed::VeryHigh);
        d0.set_as_af_pull(d0.af_num(), AFType::OutputPushPull, Pull::None);
        d0.set_speed(crate::gpio::Speed::VeryHigh);
        d1.set_as_af_pull(d1.af_num(), AFType::OutputPushPull, Pull::None);
        d1.set_speed(crate::gpio::Speed::VeryHigh);
        d2.set_as_af_pull(d2.af_num(), AFType::OutputPushPull, Pull::None);
        d2.set_speed(crate::gpio::Speed::VeryHigh);
        d3.set_as_af_pull(d3.af_num(), AFType::OutputPushPull, Pull::None);
        d3.set_speed(crate::gpio::Speed::VeryHigh);

        Self::new_inner(
            peri,
            Some(d0.map_into()),
            Some(d1.map_into()),
            Some(d2.map_into()),
            Some(d3.map_into()),
            Some(sck.map_into()),
            Some(nss.map_into()),
            dma,
            config,
            FlashSelection::Flash2,
        )
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        d0: Option<PeripheralRef<'d, AnyPin>>,
        d1: Option<PeripheralRef<'d, AnyPin>>,
        d2: Option<PeripheralRef<'d, AnyPin>>,
        d3: Option<PeripheralRef<'d, AnyPin>>,
        sck: Option<PeripheralRef<'d, AnyPin>>,
        nss: Option<PeripheralRef<'d, AnyPin>>,
        dma: impl Peripheral<P = Dma> + 'd,
        config: Config,
        fsel: FlashSelection,
    ) -> Self {
        into_ref!(peri, dma);

        T::enable_and_reset();

        while T::REGS.sr().read().busy() {}

        #[cfg(stm32h7)]
        {
            use stm32_metapac::quadspi::regs::Cr;
            // Apply precautionary steps according to the errata...
            T::REGS.cr().write_value(Cr(0));
            while T::REGS.sr().read().busy() {}
            T::REGS.cr().write_value(Cr(0xFF000001));
            T::REGS.ccr().write(|w| w.set_frcm(true));
            T::REGS.ccr().write(|w| w.set_frcm(true));
            T::REGS.cr().write_value(Cr(0));
            while T::REGS.sr().read().busy() {}
        }

        T::REGS.cr().modify(|w| {
            w.set_en(true);
            //w.set_tcen(false);
            w.set_sshift(false);
            w.set_fthres(config.fifo_threshold.into());
            w.set_prescaler(config.prescaler);
            w.set_fsel(fsel.into());
        });
        T::REGS.dcr().modify(|w| {
            w.set_fsize(config.memory_size.into());
            w.set_csht(config.cs_high_time.into());
            w.set_ckmode(true);
        });

        Self {
            _peri: peri,
            sck,
            d0,
            d1,
            d2,
            d3,
            nss,
            dma,
            config,
        }
    }

    /// Do a QSPI command.
    pub fn command(&mut self, transaction: TransferConfig) {
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(false));
        self.setup_transaction(QspiMode::IndirectWrite, &transaction);

        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().modify(|v| v.set_ctcf(true));
    }

    /// Blocking read data.
    pub fn blocking_read(&mut self, buf: &mut [u8], transaction: TransferConfig) {
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(false));
        self.setup_transaction(QspiMode::IndirectWrite, &transaction);

        if let Some(len) = transaction.data_len {
            let current_ar = T::REGS.ar().read().address();
            T::REGS.ccr().modify(|v| {
                v.set_fmode(QspiMode::IndirectRead.into());
            });
            T::REGS.ar().write(|v| {
                v.set_address(current_ar);
            });

            for idx in 0..len {
                while !T::REGS.sr().read().tcf() && !T::REGS.sr().read().ftf() {}
                buf[idx] = unsafe { (T::REGS.dr().as_ptr() as *mut u8).read_volatile() };
            }
        }

        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().modify(|v| v.set_ctcf(true));
    }

    /// Blocking write data.
    pub fn blocking_write(&mut self, buf: &[u8], transaction: TransferConfig) {
        // STM32H7 does not have dmaen
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(false));

        self.setup_transaction(QspiMode::IndirectWrite, &transaction);

        if let Some(len) = transaction.data_len {
            T::REGS.ccr().modify(|v| {
                v.set_fmode(QspiMode::IndirectWrite.into());
            });

            for idx in 0..len {
                while !T::REGS.sr().read().ftf() {}
                unsafe { (T::REGS.dr().as_ptr() as *mut u8).write_volatile(buf[idx]) };
            }
        }

        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().modify(|v| v.set_ctcf(true));
    }

    /// Blocking read data, using DMA.
    pub fn blocking_read_dma(&mut self, buf: &mut [u8], transaction: TransferConfig)
    where
        Dma: QuadDma<T>,
    {
        self.setup_transaction(QspiMode::IndirectWrite, &transaction);

        T::REGS.ccr().modify(|v| {
            v.set_fmode(QspiMode::IndirectRead.into());
        });
        let current_ar = T::REGS.ar().read().address();
        T::REGS.ar().write(|v| {
            v.set_address(current_ar);
        });

        let request = self.dma.request();
        let transfer = unsafe {
            Transfer::new_read(
                &mut self.dma,
                request,
                T::REGS.dr().as_ptr() as *mut u8,
                buf,
                Default::default(),
            )
        };

        // STM32H7 does not have dmaen
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(true));

        transfer.blocking_wait();
    }

    /// Blocking write data, using DMA.
    pub fn blocking_write_dma(&mut self, buf: &[u8], transaction: TransferConfig)
    where
        Dma: QuadDma<T>,
    {
        self.setup_transaction(QspiMode::IndirectWrite, &transaction);

        T::REGS.ccr().modify(|v| {
            v.set_fmode(QspiMode::IndirectWrite.into());
        });

        let request = self.dma.request();
        let transfer = unsafe {
            Transfer::new_write(
                &mut self.dma,
                request,
                buf,
                T::REGS.dr().as_ptr() as *mut u8,
                Default::default(),
            )
        };

        // STM32H7 does not have dmaen
        #[cfg(not(stm32h7))]
        T::REGS.cr().modify(|v| v.set_dmaen(true));

        transfer.blocking_wait();
    }

    fn setup_transaction(&mut self, fmode: QspiMode, transaction: &TransferConfig) {
        T::REGS.fcr().modify(|v| {
            v.set_csmf(true);
            v.set_ctcf(true);
            v.set_ctef(true);
            v.set_ctof(true);
        });

        while T::REGS.sr().read().busy() {}

        if let Some(len) = transaction.data_len {
            T::REGS.dlr().write(|v| v.set_dl(len as u32 - 1));
        }

        T::REGS.ccr().write(|v| {
            v.set_fmode(fmode.into());
            v.set_imode(transaction.iwidth.into());
            v.set_instruction(transaction.instruction);
            v.set_admode(transaction.awidth.into());
            v.set_adsize(self.config.address_size.into());
            v.set_dmode(transaction.dwidth.into());
            v.set_abmode(QspiWidth::NONE.into());
            v.set_dcyc(transaction.dummy.into());
        });

        if let Some(addr) = transaction.address {
            T::REGS.ar().write(|v| {
                v.set_address(addr);
            });
        }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        const REGS: Regs;
    }
}

/// QSPI instance trait.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + RccPeripheral {}

pin_trait!(SckPin, Instance);
pin_trait!(BK1D0Pin, Instance);
pin_trait!(BK1D1Pin, Instance);
pin_trait!(BK1D2Pin, Instance);
pin_trait!(BK1D3Pin, Instance);
pin_trait!(BK1NSSPin, Instance);

pin_trait!(BK2D0Pin, Instance);
pin_trait!(BK2D1Pin, Instance);
pin_trait!(BK2D2Pin, Instance);
pin_trait!(BK2D3Pin, Instance);
pin_trait!(BK2NSSPin, Instance);

dma_trait!(QuadDma, Instance);

foreach_peripheral!(
    (quadspi, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);
