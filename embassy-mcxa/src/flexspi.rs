//! Experimental FLEXSPI NOR flash driver for MCXA5xx.
//!
//! This module currently targets the FRDM-MCXA577 board wiring used by the
//! FlexSPI SDK examples. It provides a blocking mode and a DMA-assisted mode
//! while keeping the example binaries thin and consistent with the other MCXA
//! examples in this repository.

use embassy_hal_internal::{Peri, PeripheralType};

use crate::dma::{Channel, DmaChannel, DmaRequest, TransferOptions};
use crate::gpio::{AnyPin, DriveStrength, GpioPin, Pull, SlewRate};
use crate::pac;
use crate::pac::edma_0_tcd::regs::{TcdAttr, TcdBiterElinkno, TcdCiterElinkno, TcdCsr};
use crate::pac::flexspi::Flexspi as Regs;
use crate::pac::flexspi::regs::{
    Ahbcr, Ahbrxbuf0cr0, Flsha1cr0, Flshcr1, Flshcr2, Flshcr4, Intr, Ipcmd, Ipcr0, Ipcr1, Iprxfcr,
    Iptxfcr, Lut, Lutcr, Lutkey, Mcr0, Tfdr,
};
use crate::pac::mrcc::regs::{FlexspiClkdiv, FlexspiClksel, GlbCcSet, GlbRstSet};
use crate::pac::syscon::regs::Clkunlock;

const FLASH_BUSY_STATUS_POL: bool = true;
const FLASH_BUSY_STATUS_OFFSET: u8 = 0;
const CUSTOM_LUT_LENGTH: usize = 60;
const FLASH_SIZE_KBYTE: u32 = 0x10000;
const READ_CHUNK_SIZE: usize = 16;
const FLEXSPI_LUT_KEY_VALUE: u32 = 0x5AF0_5AF0;
const FLEXSPI_CLKSRC_FRO_HF: pac::mrcc::vals::FlexspiClkselMux =
    pac::mrcc::vals::FlexspiClkselMux::I1_CLKROOT_FIRC_GATED;
const PORT3_MRCC_BIT: u32 = 1 << 13;
const FLEXSPI0_MRCC_BIT: u32 = 1 << 12;
const DMA_ATTR_SIZE_32BIT: u16 = 2;
const DMA_FIFO_WINDOW_BYTES: usize = 8;
const DMA_FIFO_WINDOW_MODULO: u16 = 3;

const LUT_READ: usize = 0;
const LUT_READSTATUS: usize = 1;
const LUT_WRITEENABLE: usize = 2;
const LUT_READID_OPI: usize = 3;
const LUT_WRITEENABLE_OPI: usize = 4;
const LUT_ERASESECTOR: usize = 5;
const LUT_CHIPERASE: usize = 6;
const LUT_PAGEPROGRAM: usize = 7;
const LUT_ENTEROPI: usize = 8;
const LUT_WRITE: usize = 9;
const LUT_READSTATUS_OPI: usize = 10;

const LUT_CMD_STOP: u8 = 0x00;
const LUT_CMD_SDR: u8 = 0x01;
const LUT_CMD_RADDR_SDR: u8 = 0x02;
const LUT_CMD_MODE8_SDR: u8 = 0x07;
const LUT_CMD_WRITE_SDR: u8 = 0x08;
const LUT_CMD_READ_SDR: u8 = 0x09;
const LUT_CMD_DUMMY_SDR: u8 = 0x0c;

const LUT_1PAD: u8 = 0x00;
const LUT_4PAD: u8 = 0x02;

/// NOR flash page size used by the SDK example flash part.
pub const PAGE_SIZE: usize = 256;
const PAGE_WORDS: usize = PAGE_SIZE / 4;

mod sealed {
    pub trait Sealed {}
}

/// Marker for blocking transfers.
pub struct Blocking;
impl sealed::Sealed for Blocking {}

/// Marker for DMA-assisted transfers.
pub struct Dma<'d> {
    tx_dma: DmaChannel<'d>,
    rx_dma: DmaChannel<'d>,
    tx_request: DmaRequest,
    rx_request: DmaRequest,
}
impl sealed::Sealed for Dma<'_> {}

trait Mode: sealed::Sealed {}
impl Mode for Blocking {}
impl Mode for Dma<'_> {}

trait FlexspiPin: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self) {
        self.set_pull(Pull::Disabled);
        self.set_slew_rate(SlewRate::Fast.into());
        self.set_drive_strength(DriveStrength::Double.into());
        self.set_function(crate::pac::port::vals::Mux::MUX9);
        self.set_enable_input_buffer(true);
    }
}

macro_rules! impl_pin {
    ($pin:ident) => {
        impl sealed::Sealed for crate::peripherals::$pin {}
        impl FlexspiPin for crate::peripherals::$pin {}
    };
}

impl_pin!(P3_0);
impl_pin!(P3_1);
impl_pin!(P3_6);
impl_pin!(P3_7);
impl_pin!(P3_8);
impl_pin!(P3_9);
impl_pin!(P3_10);
impl_pin!(P3_11);

/// FRDM-MCXA577 port 3 pin bundle for the FlexSPI demo flash.
pub struct Port3Pins<'d> {
    _p3_0: Peri<'d, AnyPin>,
    _p3_1: Peri<'d, AnyPin>,
    _p3_6: Peri<'d, AnyPin>,
    _p3_7: Peri<'d, AnyPin>,
    _p3_8: Peri<'d, AnyPin>,
    _p3_9: Peri<'d, AnyPin>,
    _p3_10: Peri<'d, AnyPin>,
    _p3_11: Peri<'d, AnyPin>,
}

impl<'d> Port3Pins<'d> {
    /// Create the FRDM-MCXA577 FlexSPI pin bundle.
    pub fn new(
        p3_0: Peri<'d, crate::peripherals::P3_0>,
        p3_1: Peri<'d, crate::peripherals::P3_1>,
        p3_6: Peri<'d, crate::peripherals::P3_6>,
        p3_7: Peri<'d, crate::peripherals::P3_7>,
        p3_8: Peri<'d, crate::peripherals::P3_8>,
        p3_9: Peri<'d, crate::peripherals::P3_9>,
        p3_10: Peri<'d, crate::peripherals::P3_10>,
        p3_11: Peri<'d, crate::peripherals::P3_11>,
    ) -> Self {
        p3_0.mux();
        p3_1.mux();
        p3_6.mux();
        p3_7.mux();
        p3_8.mux();
        p3_9.mux();
        p3_10.mux();
        p3_11.mux();

        Self {
            _p3_0: p3_0.into(),
            _p3_1: p3_1.into(),
            _p3_6: p3_6.into(),
            _p3_7: p3_7.into(),
            _p3_8: p3_8.into(),
            _p3_9: p3_9.into(),
            _p3_10: p3_10.into(),
            _p3_11: p3_11.into(),
        }
    }
}

/// FlexSPI configuration.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Config {
    /// Divider applied to the FlexSPI clock root. Valid values are 1..=16.
    pub clock_divider: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self { clock_divider: 4 }
    }
}

/// Errors related to hardware setup.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Invalid clock divider.
    InvalidClockDivider,
}

/// Errors that can occur during FlexSPI flash operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IoError {
    /// FlexSPI reported an IPCMD error.
    Command {
        /// The command error source identifier.
        error_id: u8,
        /// The reported command error code.
        error_code: pac::flexspi::vals::Ipcmderrcode,
    },
    /// DMA configuration error.
    InvalidDmaParameters,
}

impl From<crate::dma::InvalidParameters> for IoError {
    fn from(_: crate::dma::InvalidParameters) -> Self {
        IoError::InvalidDmaParameters
    }
}

/// FlexSPI NOR flash driver.
pub struct FlexSpi<'d, M> {
    regs: Regs,
    _pins: Port3Pins<'d>,
    mode: M,
}

impl<'d> FlexSpi<'d, Blocking> {
    /// Create a blocking FlexSPI NOR flash driver.
    pub fn new_blocking(pins: Port3Pins<'d>, config: Config) -> Result<Self, SetupError> {
        let mut flash = Self {
            regs: pac::FLEXSPI0,
            _pins: pins,
            mode: Blocking,
        };
        flash.initialize(config)?;
        Ok(flash)
    }

    /// Read flash contents using the FlexSPI IP read path.
    pub fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), IoError> {
        let mut offset = 0;

        while offset < buffer.len() {
            let end = (offset + READ_CHUNK_SIZE).min(buffer.len());
            self.issue_ip_read_command(address + offset as u32, LUT_READ, &mut buffer[offset..end])?;
            offset = end;
        }

        Ok(())
    }

    /// Program a single page.
    pub fn page_program(&mut self, address: u32, data: &[u8; PAGE_SIZE]) -> Result<(), IoError> {
        self.write_enable(true)?;
        self.issue_ip_write_command(address, LUT_PAGEPROGRAM, data)?;
        self.wait_bus_busy(true)?;
        self.software_reset();
        Ok(())
    }
}

impl<'d> FlexSpi<'d, Dma<'d>> {
    /// Create a DMA-assisted FlexSPI NOR flash driver.
    pub fn new_with_dma(
        pins: Port3Pins<'d>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Result<Self, SetupError> {
        let mut flash = Self {
            regs: pac::FLEXSPI0,
            _pins: pins,
            mode: Dma {
                tx_dma: DmaChannel::new(tx_dma),
                rx_dma: DmaChannel::new(rx_dma),
                tx_request: DmaRequest::FlexSPI0Tx,
                rx_request: DmaRequest::FlexSPI0Rx,
            },
        };
        flash.initialize(config)?;
        Ok(flash)
    }

    /// Read flash contents using DMA for full-page chunks and polling for any tail.
    pub fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), IoError> {
        let mut offset = 0;

        while offset + PAGE_SIZE <= buffer.len() {
            let mut words = [0u32; PAGE_WORDS];
            self.issue_ip_read_dma(address + offset as u32, LUT_READ, &mut words)?;
            self.words_to_bytes(&words, &mut buffer[offset..offset + PAGE_SIZE]);
            self.software_reset();
            offset += PAGE_SIZE;
        }

        if offset < buffer.len() {
            self.issue_ip_read_command(address + offset as u32, LUT_READ, &mut buffer[offset..])?;
        }

        Ok(())
    }

    /// Program a single page using DMA.
    pub fn page_program(&mut self, address: u32, data: &[u8; PAGE_SIZE]) -> Result<(), IoError> {
        let words = self.bytes_to_words(data);
        self.write_enable(true)?;
        self.issue_ip_write_dma(address, LUT_PAGEPROGRAM, &words)?;
        self.wait_bus_busy(true)?;
        self.software_reset();
        Ok(())
    }

    fn issue_ip_write_dma(&mut self, address: u32, seq_index: usize, data: &[u32; PAGE_WORDS]) -> Result<(), IoError> {
        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(PAGE_SIZE as u16);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        let tx_dma = &self.mode.tx_dma;
        self.configure_dma_fifo_transfer(
            tx_dma,
            self.mode.tx_request,
            data.as_ptr() as u32,
            4,
            0,
            self.regs.tfdr(0).as_ptr() as u32,
            4,
            DMA_FIFO_WINDOW_MODULO,
            PAGE_SIZE,
        );

        self.regs
            .iptxfcr()
            .modify(|r: &mut Iptxfcr| r.set_txdmaen(pac::flexspi::vals::Txdmaen::VAL1));
        self.regs.ipcmd().write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::vals::Trg::VALUE1));

        self.wait_dma_complete(tx_dma);
        self.regs
            .iptxfcr()
            .modify(|r: &mut Iptxfcr| r.set_txdmaen(pac::flexspi::vals::Txdmaen::VAL0));

        self.wait_ip_command_done();
        self.wait_idle();
        self.wait_no_ip_error()
    }

    fn issue_ip_read_dma(
        &mut self,
        address: u32,
        seq_index: usize,
        buffer: &mut [u32; PAGE_WORDS],
    ) -> Result<(), IoError> {
        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(PAGE_SIZE as u16);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        let rx_dma = &self.mode.rx_dma;
        self.configure_dma_fifo_transfer(
            rx_dma,
            self.mode.rx_request,
            self.regs.rfdr(0).as_ptr() as u32,
            4,
            DMA_FIFO_WINDOW_MODULO,
            buffer.as_mut_ptr() as u32,
            4,
            0,
            PAGE_SIZE,
        );

        self.regs
            .iprxfcr()
            .modify(|r: &mut Iprxfcr| r.set_rxdmaen(pac::flexspi::vals::Rxdmaen::VAL1));
        self.regs.ipcmd().write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::vals::Trg::VALUE1));

        self.wait_dma_complete(rx_dma);
        self.regs
            .iprxfcr()
            .modify(|r: &mut Iprxfcr| r.set_rxdmaen(pac::flexspi::vals::Rxdmaen::VAL0));

        self.wait_ip_command_done();
        self.wait_idle();
        self.wait_no_ip_error()
    }
}

#[allow(private_bounds)]
impl<'d, M: Mode> FlexSpi<'d, M> {
    /// Read the connected flash vendor ID.
    pub fn read_vendor_id(&mut self) -> Result<u8, IoError> {
        self.issue_ip_command(0, LUT_READID_OPI, 10, None)?;

        for index in 0..3 {
            let word = self.regs.rfdr(index).read().rxdata();
            for byte in word.to_le_bytes() {
                if byte != 0x7f {
                    return Ok(byte);
                }
            }
        }

        Ok(0)
    }

    /// Erase a sector and wait for it to complete.
    pub fn erase_sector(&mut self, address: u32) -> Result<(), IoError> {
        self.write_enable(true)?;
        self.issue_ip_command(address, LUT_ERASESECTOR, 0, None)?;
        self.wait_bus_busy(true)?;
        self.software_reset();
        Ok(())
    }

    fn initialize(&mut self, config: Config) -> Result<(), SetupError> {
        if config.clock_divider == 0 {
            return Err(SetupError::InvalidClockDivider);
        }

        self.configure_clocks(config.clock_divider);
        self.configure_controller();
        self.flash_reset().ok();
        self.enable_octal_mode().ok();

        Ok(())
    }

    fn configure_clocks(&self, divider: u8) {
        let mrcc = pac::MRCC0;
        let encoded_divider = divider.saturating_sub(1);

        self.unlock_clock_config();
        mrcc.mrcc_glb_cc1_set().write_value(GlbCcSet(PORT3_MRCC_BIT));
        mrcc.mrcc_glb_rst1_set().write_value(GlbRstSet(PORT3_MRCC_BIT));
        mrcc.mrcc_glb_cc2_set().write_value(GlbCcSet(FLEXSPI0_MRCC_BIT));
        mrcc.mrcc_glb_rst2_set().write_value(GlbRstSet(FLEXSPI0_MRCC_BIT));
        mrcc.mrcc_flexspi0_clksel()
            .write(|r: &mut FlexspiClksel| r.set_mux(FLEXSPI_CLKSRC_FRO_HF));
        mrcc
            .mrcc_flexspi0_clkdiv()
            .write_value(FlexspiClkdiv((0x3u32 << 29) | encoded_divider as u32));
        mrcc
            .mrcc_flexspi0_clkdiv()
            .write_value(FlexspiClkdiv(encoded_divider as u32));
        self.freeze_clock_config();
    }

    fn configure_controller(&mut self) {
        self.regs.mcr0().write(|r: &mut Mcr0| {
            r.set_mdis(pac::flexspi::vals::Mdis::VAL0);
            r.set_rxclksrc(pac::flexspi::vals::Rxclksrc::VAL1);
        });
        self.regs.ahbcr().write(|r: &mut Ahbcr| {
            r.set_aparen(pac::flexspi::vals::Aparen::INDIVIDUAL);
            r.set_clrahbrxbuf(pac::flexspi::vals::Clrahbrxbuf::VAL0);
            r.set_clrahbtxbuf(pac::flexspi::vals::Clrahbtxbuf::VAL0);
            r.set_cachableen(pac::flexspi::vals::Cachableen::VAL1);
            r.set_bufferableen(pac::flexspi::vals::Bufferableen::VAL1);
            r.set_prefetchen(pac::flexspi::vals::AhbcrPrefetchen::VALUE1);
            r.set_readaddropt(pac::flexspi::vals::Readaddropt::VAL0);
            r.set_resumedisable(pac::flexspi::vals::Resumedisable::VAL0);
            r.set_readszalign(pac::flexspi::vals::Readszalign::VAL0);
            r.set_aflashbase(0x8);
        });
        self.regs.ahbrxbuf0cr0().write(|r: &mut Ahbrxbuf0cr0| {
            r.set_bufsz(0xff);
            r.set_mstrid(0);
            r.set_priority(0);
            r.set_prefetchen(pac::flexspi::vals::Ahbrxbuf0cr0Prefetchen::VALUE1);
        });
        self.regs.flsha1cr0().write(|r: &mut Flsha1cr0| {
            r.set_flshsz(FLASH_SIZE_KBYTE);
            r.set_splitwren(false);
            r.set_splitrden(false);
        });
        self.regs.flshcr1(0).write(|r: &mut Flshcr1| {
            r.set_tcss(3);
            r.set_tcsh(3);
            r.set_wa(pac::flexspi::vals::Wa::VALUE0);
            r.set_cas(0);
            r.set_csintervalunit(pac::flexspi::vals::Csintervalunit::VAL0);
            r.set_csinterval(2);
        });
        self.regs.flshcr2(0).write(|r: &mut Flshcr2| {
            r.set_ardseqid(LUT_READ as u8);
            r.set_ardseqnum(1);
            r.set_awrseqid(LUT_PAGEPROGRAM as u8);
            r.set_awrseqnum(1);
            r.set_awrwait(0);
            r.set_awrwaitunit(pac::flexspi::vals::Awrwaitunit::VAL0);
            r.set_clrinstrptr(false);
        });
        self.regs.flshcr4().write(|r: &mut Flshcr4| {
            r.set_wmopt1(false);
            r.set_wmopt2(pac::flexspi::vals::Wmopt2::VAL0);
            r.set_wmena(pac::flexspi::vals::Wmena::VAL0);
            r.set_wmenb(pac::flexspi::vals::Wmenb::VAL0);
        });
        self.regs.iptxfcr().modify(|r: &mut Iptxfcr| r.set_txwmrk(0));
        self.regs.iprxfcr().modify(|r: &mut Iprxfcr| r.set_rxwmrk(0));

        self.load_lut(&w25q64_lut());
        self.software_reset();
    }

    fn write_enable(&mut self, enable_octal: bool) -> Result<(), IoError> {
        let seq = if enable_octal { LUT_WRITEENABLE_OPI } else { LUT_WRITEENABLE };
        self.issue_ip_command(0, seq, 0, None)
    }

    fn read_status(&mut self, enable_octal: bool) -> Result<u8, IoError> {
        let seq = if enable_octal { LUT_READSTATUS_OPI } else { LUT_READSTATUS };
        self.issue_ip_command(0, seq, 1, None)?;
        Ok((self.regs.rfdr(0).read().rxdata() & 0xff) as u8)
    }

    fn enable_octal_mode(&mut self) -> Result<(), IoError> {
        self.write_enable(false)?;
        self.issue_ip_write_command(0, LUT_ENTEROPI, &[0xE7])?;
        self.wait_bus_busy(true)?;
        self.software_reset();
        Ok(())
    }

    fn flash_reset(&mut self) -> Result<(), IoError> {
        self.load_lut(&flash_reset_lut());
        self.issue_ip_command(0, LUT_READ, 0, None)?;
        self.load_lut(&w25q64_lut());
        self.software_reset();
        Ok(())
    }

    fn wait_bus_busy(&mut self, enable_octal: bool) -> Result<(), IoError> {
        loop {
            let read_value = self.read_status(enable_octal)?;
            let is_busy = if FLASH_BUSY_STATUS_POL {
                (read_value & (1 << FLASH_BUSY_STATUS_OFFSET)) != 0
            } else {
                (read_value & (1 << FLASH_BUSY_STATUS_OFFSET)) == 0
            };

            if !is_busy {
                return Ok(());
            }
        }
    }

    fn load_lut(&mut self, entries: &[(usize, u32)]) {
        self.wait_idle();
        self.regs.lutkey().write_value(Lutkey(FLEXSPI_LUT_KEY_VALUE));
        self.regs.lutcr().write_value(Lutcr(0x02));

        for index in 0..CUSTOM_LUT_LENGTH {
            self.write_lut(index, 0);
        }

        for (index, value) in entries {
            self.write_lut(*index, *value);
        }

        self.regs.lutkey().write_value(Lutkey(FLEXSPI_LUT_KEY_VALUE));
        self.regs.lutcr().write_value(Lutcr(0x01));
    }

    fn write_lut(&mut self, index: usize, value: u32) {
        self.regs.lut(index).write_value(Lut(value));
    }

    fn software_reset(&mut self) {
        self.regs
            .mcr0()
            .modify(|r: &mut Mcr0| r.set_swreset(pac::flexspi::vals::Swreset::VAL1));
        while self.regs.mcr0().read().swreset() == pac::flexspi::vals::Swreset::VAL1 {}
    }

    fn wait_idle(&self) {
        while self.regs.sts0().read().seqidle() != pac::flexspi::vals::Seqidle::VALUE1 {}
    }

    fn prepare_ip_transfer(&mut self) {
        self.wait_idle();

        self.regs.flshcr2(0).modify(|r: &mut Flshcr2| r.set_clrinstrptr(true));
        self.regs.intr().write(|r: &mut Intr| {
            r.set_ahbcmderr(true);
            r.set_ipcmderr(true);
            r.set_ahbcmdge(true);
            r.set_ipcmdge(true);
            r.set_ipcmddone(true);
        });
        self.regs
            .iptxfcr()
            .modify(|r: &mut Iptxfcr| r.set_clriptxf(pac::flexspi::vals::Clriptxf::VALUE1));
        self.regs
            .iprxfcr()
            .modify(|r: &mut Iprxfcr| r.set_clriprxf(pac::flexspi::vals::Clriprxf::VALUE1));
    }

    fn wait_ip_command_done(&self) {
        while !self.regs.intr().read().ipcmddone() {}
    }

    fn wait_no_ip_error(&self) -> Result<(), IoError> {
        let status = self.regs.sts1().read();
        if status.ipcmderrcode() == pac::flexspi::vals::Ipcmderrcode::VAL0 {
            Ok(())
        } else {
            Err(IoError::Command {
                error_id: status.ipcmderrid() as u8,
                error_code: status.ipcmderrcode(),
            })
        }
    }

    fn issue_ip_command(
        &mut self,
        address: u32,
        seq_index: usize,
        data_size: u16,
        data: Option<u32>,
    ) -> Result<(), IoError> {
        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(data_size);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        if let Some(word) = data {
            self.regs.tfdr(0).write_value(Tfdr(word));
        }

        self.regs.ipcmd().write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::vals::Trg::VALUE1));
        self.wait_ip_command_done();
        self.wait_idle();
        self.wait_no_ip_error()
    }

    fn issue_ip_write_command(&mut self, address: u32, seq_index: usize, data: &[u8]) -> Result<(), IoError> {
        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(data.len() as u16);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        self.regs.ipcmd().write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::vals::Trg::VALUE1));

        let tx_watermark = self.regs.iptxfcr().read().txwmrk() as usize + 1;
        let mut offset = 0;

        while offset < data.len() {
            while !self.regs.intr().read().iptxwe() {}

            let chunk_len = (8 * tx_watermark).min(data.len() - offset);
            for (index, chunk) in data[offset..offset + chunk_len].chunks(4).enumerate() {
                let mut word = [0u8; 4];
                word[..chunk.len()].copy_from_slice(chunk);
                self.regs.tfdr(index).write_value(Tfdr(u32::from_le_bytes(word)));
            }

            offset += chunk_len;
            self.regs.intr().write(|r: &mut Intr| r.set_iptxwe(true));
        }

        self.wait_ip_command_done();
        self.wait_idle();
        self.wait_no_ip_error()
    }

    fn issue_ip_read_command(&mut self, address: u32, seq_index: usize, buffer: &mut [u8]) -> Result<(), IoError> {
        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(buffer.len() as u16);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        self.regs.ipcmd().write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::vals::Trg::VALUE1));
        self.wait_ip_command_done();
        self.wait_idle();
        self.wait_no_ip_error()?;

        for (index, chunk) in buffer.chunks_mut(4).enumerate() {
            let word = self.regs.rfdr(index).read().rxdata().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }

        Ok(())
    }

    fn configure_dma_fifo_transfer(
        &self,
        channel: &DmaChannel<'_>,
        request_source: DmaRequest,
        src_addr: u32,
        src_offset: u16,
        src_modulo: u16,
        dst_addr: u32,
        dst_offset: u16,
        dst_modulo: u16,
        data_len: usize,
    ) {
        let tcd = channel.tcd();
        let major_count = (data_len / DMA_FIFO_WINDOW_BYTES) as u16;
        let attr = (DMA_ATTR_SIZE_32BIT & 0x7)
            | ((dst_modulo & 0x1f) << 3)
            | ((DMA_ATTR_SIZE_32BIT & 0x7) << 8)
            | ((src_modulo & 0x1f) << 11);

        tcd.ch_csr().write(|w| w.set_done(true));
        tcd.ch_es().write(|w| w.set_err(true));
        tcd.ch_int().write(|w| w.set_int(true));
        tcd.tcd_saddr().write(|w| w.set_saddr(0));
        tcd.tcd_soff().write(|w| w.set_soff(0));
        tcd.tcd_attr().write(|w| *w = TcdAttr(0));
        tcd.tcd_nbytes_mloffno().write(|w| w.set_nbytes(0));
        tcd.tcd_daddr().write(|w| w.set_daddr(0));
        tcd.tcd_doff().write(|w| w.set_doff(0));
        tcd.tcd_citer_elinkno().write(|w| *w = TcdCiterElinkno(0));
        tcd.tcd_slast_sda().write(|w| w.set_slast_sda(0));
        tcd.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));
        tcd.tcd_csr().write(|w| *w = TcdCsr(0));
        tcd.tcd_biter_elinkno().write(|w| *w = TcdBiterElinkno(0));

        tcd.tcd_saddr().write(|w| w.set_saddr(src_addr));
        tcd.tcd_soff().write(|w| w.set_soff(src_offset));
        tcd.tcd_daddr().write(|w| w.set_daddr(dst_addr));
        tcd.tcd_doff().write(|w| w.set_doff(dst_offset));
        tcd.tcd_attr().write(|w| *w = TcdAttr(attr));
        tcd.tcd_nbytes_mloffno().write(|w| {
            w.set_smloe(pac::edma_0_tcd::vals::TcdNbytesMloffnoSmloe::OFFSET_NOT_APPLIED);
            w.set_dmloe(pac::edma_0_tcd::vals::TcdNbytesMloffnoDmloe::OFFSET_NOT_APPLIED);
            w.set_nbytes(DMA_FIFO_WINDOW_BYTES as u32);
        });
        tcd.tcd_citer_elinkno().write(|w| w.set_citer(major_count));
        tcd.tcd_biter_elinkno().write(|w| w.set_biter(major_count));
        tcd.tcd_slast_sda().write(|w| w.set_slast_sda(0));
        tcd.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));
        tcd.ch_pri().write(|w| {
            w.set_dpa(pac::edma_0_tcd::vals::Dpa::SUSPEND);
            w.set_ecp(pac::edma_0_tcd::vals::Ecp::SUSPEND);
            w.set_apl(TransferOptions::NO_INTERRUPTS.priority as u8);
        });
        tcd.tcd_csr().write(|w| {
            w.set_intmajor(false);
            w.set_inthalf(false);
            w.set_start(pac::edma_0_tcd::vals::Start::CHANNEL_NOT_STARTED);
            w.set_esg(pac::edma_0_tcd::vals::Esg::NORMAL_FORMAT);
            w.set_majorelink(false);
            w.set_eeop(false);
            w.set_esda(false);
            w.set_bwc(pac::edma_0_tcd::vals::Bwc::NO_STALL);
            w.set_dreq(pac::edma_0_tcd::vals::Dreq::ERQ_FIELD_CLEAR);
        });

        unsafe {
            channel.set_request_source(request_source);
            channel.clear_done();
            channel.enable_request();
        }
    }

    fn wait_dma_complete(&self, channel: &DmaChannel<'_>) {
        while !channel.is_done() {}
        unsafe {
            channel.disable_request();
            channel.clear_done();
            channel.clear_interrupt();
        }
    }

    fn bytes_to_words(&self, data: &[u8; PAGE_SIZE]) -> [u32; PAGE_WORDS] {
        let mut words = [0u32; PAGE_WORDS];

        for (index, chunk) in data.chunks_exact(4).enumerate() {
            words[index] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }

        words
    }

    fn words_to_bytes(&self, words: &[u32; PAGE_WORDS], buffer: &mut [u8]) {
        for (chunk, word) in buffer.chunks_mut(4).zip(words.iter()) {
            chunk.copy_from_slice(&word.to_le_bytes()[..chunk.len()]);
        }
    }

    fn unlock_clock_config(&self) {
        pac::SYSCON.clkunlock().write_value(Clkunlock(0));
    }

    fn freeze_clock_config(&self) {
        pac::SYSCON.clkunlock().write_value(Clkunlock(1));
    }
}

fn lut_seq(cmd0: u8, pads0: u8, operand0: u8, cmd1: u8, pads1: u8, operand1: u8) -> u32 {
    (operand0 as u32)
        | ((pads0 as u32) << 8)
        | ((cmd0 as u32) << 10)
        | ((operand1 as u32) << 16)
        | ((pads1 as u32) << 24)
        | ((cmd1 as u32) << 26)
}

fn w25q64_lut() -> [(usize, u32); 15] {
    [
        (
            4 * LUT_READ,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0xEB, LUT_CMD_RADDR_SDR, LUT_4PAD, 0x18),
        ),
        (
            4 * LUT_READ + 1,
            lut_seq(LUT_CMD_MODE8_SDR, LUT_4PAD, 0xF0, LUT_CMD_DUMMY_SDR, LUT_4PAD, 0x04),
        ),
        (
            4 * LUT_READ + 2,
            lut_seq(LUT_CMD_READ_SDR, LUT_4PAD, 0x00, LUT_CMD_STOP, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_READSTATUS,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x05, LUT_CMD_READ_SDR, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_WRITEENABLE,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x06, LUT_CMD_STOP, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_READID_OPI,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x9F, LUT_CMD_READ_SDR, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_WRITEENABLE_OPI,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x06, LUT_CMD_STOP, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_ERASESECTOR,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x20, LUT_CMD_RADDR_SDR, LUT_1PAD, 0x18),
        ),
        (
            4 * LUT_CHIPERASE,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x60, LUT_CMD_STOP, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_PAGEPROGRAM,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x02, LUT_CMD_RADDR_SDR, LUT_1PAD, 0x18),
        ),
        (
            4 * LUT_PAGEPROGRAM + 1,
            lut_seq(LUT_CMD_WRITE_SDR, LUT_1PAD, 0x00, LUT_CMD_STOP, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_ENTEROPI,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x05, LUT_CMD_READ_SDR, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_WRITE,
            lut_seq(LUT_CMD_STOP, LUT_1PAD, 0x00, LUT_CMD_STOP, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_READSTATUS_OPI,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x05, LUT_CMD_READ_SDR, LUT_1PAD, 0x00),
        ),
        (
            4 * LUT_READSTATUS_OPI + 1,
            lut_seq(LUT_CMD_STOP, LUT_1PAD, 0x00, LUT_CMD_STOP, LUT_1PAD, 0x00),
        ),
    ]
}

fn flash_reset_lut() -> [(usize, u32); 2] {
    [
        (
            4 * LUT_READ,
            lut_seq(LUT_CMD_SDR, LUT_1PAD, 0x66, LUT_CMD_SDR, LUT_1PAD, 0x99),
        ),
        (
            4 * LUT_READ + 1,
            lut_seq(LUT_CMD_STOP, LUT_1PAD, 0x00, LUT_CMD_STOP, LUT_1PAD, 0x00),
        ),
    ]
}