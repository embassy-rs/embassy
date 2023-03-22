#![macro_use]

use embassy_hal_common::{into_ref, PeripheralRef};

use crate::dma::TransferOptions;
use crate::gpio::sealed::AFType;
use crate::gpio::AnyPin;
use crate::pac::quadspi::Quadspi as Regs;
use crate::rcc::RccPeripheral;
use crate::{peripherals, Peripheral};

pub struct QspiWidth;

#[allow(dead_code)]
impl QspiWidth {
    pub const NONE: u8 = 0b00;
    pub const SING: u8 = 0b01;
    pub const DUAL: u8 = 0b10;
    pub const QUAD: u8 = 0b11;
}

struct QspiMode;

#[allow(dead_code)]
impl QspiMode {
    pub const INDIRECT_WRITE: u8 = 0b00;
    pub const INDIRECT_READ: u8 = 0b01;
    pub const AUTO_POLLING: u8 = 0b10;
    pub const MEMORY_MAPPED: u8 = 0b11;
}

pub struct QspiTransaction {
    pub iwidth: u8,
    pub awidth: u8,
    pub dwidth: u8,
    pub instruction: u8,
    pub address: Option<u32>,
    pub dummy: u8,
    pub data_len: Option<usize>,
}

impl Default for QspiTransaction {
    fn default() -> Self {
        Self {
            iwidth: QspiWidth::NONE,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: 0,
            address: None,
            dummy: 0,
            data_len: None,
        }
    }
}

pub struct Config {
    pub memory_size: u8,
    pub address_size: u8,
    pub prescaler: u8,
    pub fifo_threshold: u8,
    pub cs_high_time: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            memory_size: 0,
            address_size: 2,
            prescaler: 128,
            fifo_threshold: 16,
            cs_high_time: 4,
        }
    }
}

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
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, d0, d1, d2, d3, sck, nss);

        unsafe {
            sck.set_as_af(sck.af_num(), AFType::OutputPushPull);
            sck.set_speed(crate::gpio::Speed::VeryHigh);
            nss.set_as_af(nss.af_num(), AFType::OutputPushPull);
            nss.set_speed(crate::gpio::Speed::VeryHigh);
            d0.set_as_af(d0.af_num(), AFType::OutputPushPull);
            d0.set_speed(crate::gpio::Speed::VeryHigh);
            d1.set_as_af(d1.af_num(), AFType::OutputPushPull);
            d1.set_speed(crate::gpio::Speed::VeryHigh);
            d2.set_as_af(d2.af_num(), AFType::OutputPushPull);
            d2.set_speed(crate::gpio::Speed::VeryHigh);
            d3.set_as_af(d3.af_num(), AFType::OutputPushPull);
            d3.set_speed(crate::gpio::Speed::VeryHigh);
        }

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
    ) -> Self {
        into_ref!(peri, dma);

        T::enable();
        unsafe {
            T::REGS.cr().write(|w| w.set_fthres(config.fifo_threshold));

            while T::REGS.sr().read().busy() {}

            T::REGS.cr().write(|w| {
                w.set_prescaler(config.prescaler);
                w.set_en(true);
            });
            T::REGS.dcr().write(|w| {
                w.set_fsize(config.memory_size);
                w.set_csht(config.cs_high_time);
                w.set_ckmode(false);
            });
        }

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

    pub fn command(&mut self, transaction: QspiTransaction) {
        unsafe {
            T::REGS.cr().modify(|v| v.set_dmaen(false));
            self.setup_transaction(QspiMode::INDIRECT_WRITE, &transaction);

            while !T::REGS.sr().read().tcf() {}
            T::REGS.fcr().modify(|v| v.set_ctcf(true));
        }
    }

    pub fn read(&mut self, buf: &mut [u8], transaction: QspiTransaction) {
        unsafe {
            T::REGS.cr().modify(|v| v.set_dmaen(false));
            self.setup_transaction(QspiMode::INDIRECT_WRITE, &transaction);

            if let Some(len) = transaction.data_len {
                let current_ar = T::REGS.ar().read().address();
                T::REGS.ccr().modify(|v| {
                    v.set_fmode(QspiMode::INDIRECT_READ);
                });
                T::REGS.ar().write(|v| {
                    v.set_address(current_ar);
                });

                for idx in 0..len {
                    while !T::REGS.sr().read().tcf() && !T::REGS.sr().read().ftf() {}
                    buf[idx] = *(T::REGS.dr().ptr() as *mut u8);
                }
            }

            while !T::REGS.sr().read().tcf() {}
            T::REGS.fcr().modify(|v| v.set_ctcf(true));
        }
    }

    pub fn write(&mut self, buf: &[u8], transaction: QspiTransaction) {
        unsafe {
            T::REGS.cr().modify(|v| v.set_dmaen(false));
            self.setup_transaction(QspiMode::INDIRECT_WRITE, &transaction);

            if let Some(len) = transaction.data_len {
                T::REGS.ccr().modify(|v| {
                    v.set_fmode(QspiMode::INDIRECT_WRITE);
                });

                for idx in 0..len {
                    while !T::REGS.sr().read().ftf() {}
                    *(T::REGS.dr().ptr() as *mut u8) = buf[idx];
                }
            }

            while !T::REGS.sr().read().tcf() {}
            T::REGS.fcr().modify(|v| v.set_ctcf(true));
        }
    }

    pub fn read_dma(&mut self, buf: &mut [u8], transaction: QspiTransaction)
    where
        Dma: QuadDma<T>,
    {
        unsafe {
            self.setup_transaction(QspiMode::INDIRECT_WRITE, &transaction);

            let request = self.dma.request();
            let options = TransferOptions::default();

            T::REGS.ccr().modify(|v| {
                v.set_fmode(QspiMode::INDIRECT_READ);
            });
            let current_ar = T::REGS.ar().read().address();
            T::REGS.ar().write(|v| {
                v.set_address(current_ar);
            });

            self.dma
                .start_read(request, T::REGS.dr().ptr() as *mut u8, buf, options);

            T::REGS.cr().modify(|v| v.set_dmaen(true));

            while self.dma.is_running() {}
        }
    }

    pub fn write_dma(&mut self, buf: &[u8], transaction: QspiTransaction)
    where
        Dma: QuadDma<T>,
    {
        unsafe {
            self.setup_transaction(QspiMode::INDIRECT_WRITE, &transaction);

            let request = self.dma.request();
            let options = TransferOptions::default();

            T::REGS.ccr().modify(|v| {
                v.set_fmode(QspiMode::INDIRECT_WRITE);
            });

            self.dma
                .start_write(request, buf, T::REGS.dr().ptr() as *mut u8, options);

            T::REGS.cr().modify(|v| v.set_dmaen(true));

            while self.dma.is_running() {}
        }
    }

    fn setup_transaction(&mut self, fmode: u8, transaction: &QspiTransaction) {
        unsafe {
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
                v.set_fmode(fmode);
                v.set_imode(transaction.iwidth);
                v.set_instruction(transaction.instruction);
                v.set_admode(transaction.awidth);
                v.set_adsize(self.config.address_size);
                v.set_dmode(transaction.dwidth);
                v.set_abmode(QspiWidth::NONE);
                v.set_dcyc(transaction.dummy);
            });

            if let Some(addr) = transaction.address {
                T::REGS.ar().write(|v| {
                    v.set_address(addr);
                });
            }
        }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        const REGS: Regs;
    }
}

pub trait Instance: Peripheral<P = Self> + sealed::Instance + RccPeripheral {}

pin_trait!(SckPin, Instance);
pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(NSSPin, Instance);

dma_trait!(QuadDma, Instance);

foreach_peripheral!(
    (quadspi, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);
