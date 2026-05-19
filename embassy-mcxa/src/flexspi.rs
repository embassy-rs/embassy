//! FlexSPI NOR flash driver for MCXA5xx.
//!
//! This module provides blocking, interrupt-driven async, and DMA-backed
//! async access to `FLEXSPI0` on MCXA5xx devices.
//!
//! The driver is configured with a user-provided [`FlashConfig`] that
//! describes the flash geometry and LUT command sequences, allowing the
//! transport layer to be reused across compatible NOR flash devices.
//!
//! Current scope:
//! - MCXA5xx only
//! - `FLEXSPI0` only
//! - NOR flash style IP-command workflows, as used by the MCXA577 examples

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use embassy_futures::yield_now;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::periph_helpers::{Div4, FlexspiClockSel, FlexspiConfig as FlexspiClockConfig};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::dma::{Channel, DmaChannel, TransferOptions};
use crate::gpio::{AnyPin, DriveStrength, GpioPin, Pull, SlewRate};
use crate::interrupt::typelevel::{Handler, Interrupt};
pub use crate::pac::flexspi::Flexspi as Regs;
use crate::pac::flexspi::{
    Ahbcr, Ahbrxbuf0cr0, Flsha1cr0, Flshcr1, Flshcr2, Flshcr4, Intr, Ipcmd, Ipcr0, Ipcr1, Iprxfcr, Iptxfcr, Lut, Lutcr,
    Lutkey, Mcr0, Tfdr,
};
use crate::{interrupt, pac};

const MAX_PAGE_SIZE: usize = 256;
const MAX_PAGE_WORDS: usize = MAX_PAGE_SIZE / 4;
const LUT_KEY_VALUE: u32 = 0x5AF0_5AF0;
const LUT_WORD_COUNT: usize = 64;
const DMA_FIFO_WINDOW_BYTES: usize = 8;
const TEMP_SEQUENCE_INDEX: u8 = 15;
const IP_FIFO_DEPTH_WORDS: usize = 32;
const IP_FIFO_CAPACITY_BYTES: usize = IP_FIFO_DEPTH_WORDS * 4;
const DMA_TRANSFER_OPTIONS: TransferOptions = TransferOptions::COMPLETE_INTERRUPT;
const IRQ_EVENT_COMMAND_DONE: u32 = 1 << 0;
const IRQ_EVENT_COMMAND_GRANT: u32 = 1 << 1;
const IRQ_EVENT_COMMAND_ERROR: u32 = 1 << 2;
const IRQ_EVENT_TX_WATERMARK: u32 = 1 << 3;

pub mod lookup {
    const INSTRUCTIONS_PER_SEQUENCE: usize = 8;
    const SEQUENCE_COUNT: usize = 16;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum Command {
        Read = 0,
        ReadStatus = 1,
        WriteEnable = 3,
        WriteStatus = 4,
        EraseSector = 5,
        ReadId = 8,
        PageProgram = 9,
        ChipErase = 11,
        ReadSfdp = 13,
        RestoreNocmd = 14,
        Dummy = 15,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Opcode(pub u8);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum Pads {
        One = 0x00,
        Two = 0x01,
        Four = 0x02,
        Eight = 0x03,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Instr(u16);

    impl Instr {
        pub const fn new(opcode: Opcode, pads: Pads, operand: u8) -> Self {
            Self((operand as u16) | ((((opcode.0 << 2) | pads as u8) as u16) << 8))
        }

        pub const fn stop() -> Self {
            Self::new(opcodes::STOP, Pads::One, 0)
        }

        pub const fn jump_on_cs() -> Self {
            Self::new(opcodes::JUMP_ON_CS, Pads::One, 0)
        }

        const fn raw(self) -> u16 {
            self.0
        }
    }

    pub const STOP: Instr = Instr::stop();
    pub const JUMP_ON_CS: Instr = Instr::jump_on_cs();

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Sequence([Instr; INSTRUCTIONS_PER_SEQUENCE]);

    impl Sequence {
        pub const fn stopped() -> Self {
            Self([Instr::stop(); INSTRUCTIONS_PER_SEQUENCE])
        }

        pub const fn into_words(self) -> [u32; 4] {
            let raw = self.0;
            [
                (raw[0].raw() as u32) | ((raw[1].raw() as u32) << 16),
                (raw[2].raw() as u32) | ((raw[3].raw() as u32) << 16),
                (raw[4].raw() as u32) | ((raw[5].raw() as u32) << 16),
                (raw[6].raw() as u32) | ((raw[7].raw() as u32) << 16),
            ]
        }
    }

    pub struct SequenceBuilder {
        sequence: Sequence,
        offset: usize,
    }

    impl SequenceBuilder {
        pub const fn new() -> Self {
            Self {
                sequence: Sequence::stopped(),
                offset: 0,
            }
        }

        pub const fn instr(self, instr: Instr) -> Self {
            let mut raw = self.sequence.0;
            raw[self.offset] = instr;
            Self {
                sequence: Sequence(raw),
                offset: self.offset + 1,
            }
        }

        pub const fn build(self) -> Sequence {
            self.sequence
        }
    }

    #[derive(Clone, Copy, Debug)]
    #[repr(transparent)]
    pub struct LookupTable([Sequence; SEQUENCE_COUNT]);

    impl LookupTable {
        pub const fn new() -> Self {
            Self([Sequence::stopped(); SEQUENCE_COUNT])
        }

        pub const fn command(mut self, cmd: Command, sequence: Sequence) -> Self {
            self.0[cmd as usize] = sequence;
            self
        }

        pub const fn sequence(self, cmd: Command) -> Sequence {
            self.0[cmd as usize]
        }

        pub const fn custom_command(mut self, index: u8, sequence: Sequence) -> Self {
            self.0[index as usize] = sequence;
            self
        }

        pub const fn custom_sequence(self, index: u8) -> Sequence {
            self.0[index as usize]
        }
    }

    pub mod opcodes {
        use super::Opcode;

        pub const STOP: Opcode = Opcode(0x00);
        pub const JUMP_ON_CS: Opcode = Opcode(0x1F);

        pub mod sdr {
            use super::Opcode;

            pub const CMD: Opcode = Opcode(0x01);
            pub const RADDR: Opcode = Opcode(0x02);
            pub const CADDR: Opcode = Opcode(0x03);
            pub const MODE1: Opcode = Opcode(0x04);
            pub const MODE2: Opcode = Opcode(0x05);
            pub const MODE4: Opcode = Opcode(0x06);
            pub const MODE8: Opcode = Opcode(0x07);
            pub const WRITE: Opcode = Opcode(0x08);
            pub const READ: Opcode = Opcode(0x09);
            pub const LEARN: Opcode = Opcode(0x0A);
            pub const DATASZ: Opcode = Opcode(0x0B);
            pub const DUMMY: Opcode = Opcode(0x0C);
            pub const DUMMY_RWDS: Opcode = Opcode(0x0D);
        }

        pub mod ddr {
            use super::{Opcode, sdr};

            const fn to_ddr(opcode: Opcode) -> Opcode {
                Opcode(opcode.0 + 0x20)
            }

            pub const CMD: Opcode = to_ddr(sdr::CMD);
            pub const RADDR: Opcode = to_ddr(sdr::RADDR);
            pub const CADDR: Opcode = to_ddr(sdr::CADDR);
            pub const MODE1: Opcode = to_ddr(sdr::MODE1);
            pub const MODE2: Opcode = to_ddr(sdr::MODE2);
            pub const MODE4: Opcode = to_ddr(sdr::MODE4);
            pub const MODE8: Opcode = to_ddr(sdr::MODE8);
            pub const WRITE: Opcode = to_ddr(sdr::WRITE);
            pub const READ: Opcode = to_ddr(sdr::READ);
            pub const LEARN: Opcode = to_ddr(sdr::LEARN);
            pub const DATASZ: Opcode = to_ddr(sdr::DATASZ);
            pub const DUMMY: Opcode = to_ddr(sdr::DUMMY);
            pub const DUMMY_RWDS: Opcode = to_ddr(sdr::DUMMY_RWDS);
        }
    }
}

pub struct Info {
    pub regs: Regs,
    pub pending_events: AtomicU32,
    pub waker: AtomicWaker,
}

unsafe impl Sync for Info {}

impl Info {
    #[inline(always)]
    fn regs(&self) -> Regs {
        self.regs
    }

    #[inline(always)]
    fn pending_events(&self) -> &AtomicU32 {
        &self.pending_events
    }

    #[inline(always)]
    fn waker(&self) -> &AtomicWaker {
        &self.waker
    }
}

pub mod sealed {
    pub trait Sealed {}

    pub trait Instance: crate::clocks::Gate<MrccPeriphConfig = crate::clocks::periph_helpers::FlexspiConfig> {
        fn info() -> &'static super::Info;
        fn regs() -> super::Regs;
        const CLOCK_INSTANCE: crate::clocks::periph_helpers::FlexspiInstance;
        const TX_DMA_REQUEST: crate::dma::DmaRequest;
        const RX_DMA_REQUEST: crate::dma::DmaRequest;
    }
}

#[allow(private_bounds)]
pub trait Instance: sealed::Instance + PeripheralType + 'static + Send {
    type Interrupt: interrupt::typelevel::Interrupt;
}

pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::info().regs();
        let status = regs.intr().read();
        let inten = regs.inten().read();

        let mut pending_events = 0;
        let mut clear_command_flags = false;

        if status.ipcmddone() && inten.ipcmddoneen() == pac::flexspi::Ipcmddoneen::Value1 {
            pending_events |= IRQ_EVENT_COMMAND_DONE;
            clear_command_flags = true;
        }
        if status.ipcmdge() && inten.ipcmdgeen() == pac::flexspi::Ipcmdgeen::Value1 {
            pending_events |= IRQ_EVENT_COMMAND_GRANT;
            clear_command_flags = true;
        }
        if status.ipcmderr() && inten.ipcmderren() == pac::flexspi::Ipcmderren::Value1 {
            pending_events |= IRQ_EVENT_COMMAND_ERROR;
            clear_command_flags = true;
        }
        if status.iptxwe() && inten.iptxween() == pac::flexspi::Iptxween::Value1 {
            pending_events |= IRQ_EVENT_TX_WATERMARK;
        }

        if pending_events != 0 {
            T::info().pending_events().fetch_or(pending_events, Ordering::Release);
            T::info().waker().wake();
        }

        regs.inten().write(|_| {});

        if clear_command_flags {
            regs.intr().write(|w| {
                w.set_ipcmddone(status.ipcmddone());
                w.set_ipcmdge(status.ipcmdge());
                w.set_ipcmderr(status.ipcmderr());
            });
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_flexspi_instance {
    ($n:expr) => {
        paste::paste! {
            impl crate::flexspi::sealed::Instance for crate::peripherals::[<FLEXSPI $n>] {
                fn info() -> &'static crate::flexspi::Info {
                    static INFO: crate::flexspi::Info = crate::flexspi::Info {
                        regs: crate::pac::[<FLEXSPI $n>],
                        pending_events: core::sync::atomic::AtomicU32::new(0),
                        waker: embassy_sync::waitqueue::AtomicWaker::new(),
                    };
                    &INFO
                }

                fn regs() -> crate::flexspi::Regs {
                    crate::pac::[<FLEXSPI $n>]
                }

                const CLOCK_INSTANCE: crate::clocks::periph_helpers::FlexspiInstance =
                    crate::clocks::periph_helpers::FlexspiInstance::[<Flexspi $n>];
                const TX_DMA_REQUEST: crate::dma::DmaRequest =
                    crate::dma::DmaRequest::[<Flexspi $n Tx>];
                const RX_DMA_REQUEST: crate::dma::DmaRequest =
                    crate::dma::DmaRequest::[<Flexspi $n Rx>];
            }

            impl crate::flexspi::Instance for crate::peripherals::[<FLEXSPI $n>] {
                type Interrupt = crate::interrupt::typelevel::[<FLEXSPI $n>];
            }
        }
    };
}

pub trait Pin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self) {
        self.set_pull(Pull::Disabled);
        self.set_slew_rate(SlewRate::Fast.into());
        self.set_drive_strength(DriveStrength::Double.into());
        self.set_function(crate::pac::port::Mux::Mux9);
        self.set_enable_input_buffer(true);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_flexspi_pin {
    ($pin:ident, $peri:ident) => {
        impl crate::flexspi::sealed::Sealed for crate::peripherals::$pin {}
        impl crate::flexspi::Pin<crate::peripherals::$peri> for crate::peripherals::$pin {}
    };
}

#[derive(Clone, Copy)]
pub struct ClockConfig {
    pub power: PoweredClock,
    pub source: FlexspiClockSel,
    pub div: Div4,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: FlexspiClockSel::FroHf,
            div: Div4::from_divisor(4).unwrap(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct DeviceCommand {
    pub seq: u8,
    pub payload: [u8; 4],
    pub len: usize,
    pub requires_write_enable: bool,
}

impl DeviceCommand {
    pub const fn new(seq: u8, payload: [u8; 4], len: usize, requires_write_enable: bool) -> Self {
        Self {
            seq,
            payload,
            len,
            requires_write_enable,
        }
    }
}

#[derive(Clone, Copy)]
pub struct FlashConfig {
    pub flash_size_kbytes: u32,
    pub page_size: usize,
    pub busy_status_polarity: bool,
    pub busy_status_offset: u8,
    pub lookup_table: lookup::LookupTable,
    pub read_seq: u8,
    pub read_status_seq: u8,
    pub write_enable_seq: u8,
    pub read_id_seq: u8,
    pub erase_sector_seq: u8,
    pub page_program_seq: u8,
    pub reset_sequence: Option<lookup::Sequence>,
    pub device_mode_command: Option<DeviceCommand>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    InvalidPageSize,
    ClockSetup(ClockError),
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IoError {
    Command { error_code: pac::flexspi::Ipcmderrcode },
    CommandGrantTimeout,
    Dma(crate::dma::TransferErrors),
    InterruptWait,
    InvalidDmaParameters,
    InvalidTransferLength,
}

impl From<crate::dma::InvalidParameters> for IoError {
    fn from(_: crate::dma::InvalidParameters) -> Self {
        IoError::InvalidDmaParameters
    }
}

impl From<crate::dma::TransferErrors> for IoError {
    fn from(err: crate::dma::TransferErrors) -> Self {
        IoError::Dma(err)
    }
}

struct DmaState<'d> {
    tx_dma: DmaChannel<'d>,
    rx_dma: DmaChannel<'d>,
}

struct InnerFlexSpi<'d, T: Instance> {
    _peri: Peri<'d, T>,
    regs: Regs,
    _pins: [Peri<'d, AnyPin>; 8],
    dma: Option<DmaState<'d>>,
    interrupt_mode: bool,
    flash: FlashConfig,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<T>,
}

impl<'d, T: Instance> InnerFlexSpi<'d, T> {
    fn use_interrupt_waits(&self) -> bool {
        self.interrupt_mode
    }

    pub fn new_blocking(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        pin0.mux();
        pin1.mux();
        pin2.mux();
        pin3.mux();
        pin4.mux();
        pin5.mux();
        pin6.mux();
        pin7.mux();

        Self::new_inner(
            peri,
            [
                pin0.into(),
                pin1.into(),
                pin2.into(),
                pin3.into(),
                pin4.into(),
                pin5.into(),
                pin6.into(),
                pin7.into(),
            ],
            None,
            false,
            clock,
            flash,
        )
    }

    pub fn new_async(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        pin0.mux();
        pin1.mux();
        pin2.mux();
        pin3.mux();
        pin4.mux();
        pin5.mux();
        pin6.mux();
        pin7.mux();

        let driver = Self::new_inner(
            peri,
            [
                pin0.into(),
                pin1.into(),
                pin2.into(),
                pin3.into(),
                pin4.into(),
                pin5.into(),
                pin6.into(),
                pin7.into(),
            ],
            None,
            true,
            clock,
            flash,
        )?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Ok(driver)
    }

    pub fn new_with_dma(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        pin0.mux();
        pin1.mux();
        pin2.mux();
        pin3.mux();
        pin4.mux();
        pin5.mux();
        pin6.mux();
        pin7.mux();

        let driver = Self::new_inner(
            peri,
            [
                pin0.into(),
                pin1.into(),
                pin2.into(),
                pin3.into(),
                pin4.into(),
                pin5.into(),
                pin6.into(),
                pin7.into(),
            ],
            Some(DmaState {
                tx_dma: DmaChannel::new(tx_dma),
                rx_dma: DmaChannel::new(rx_dma),
            }),
            true,
            clock,
            flash,
        )?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Ok(driver)
    }

    fn new_inner(
        peri: Peri<'d, T>,
        pins: [Peri<'d, AnyPin>; 8],
        dma: Option<DmaState<'d>>,
        interrupt_mode: bool,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        if flash.page_size == 0 || flash.page_size > MAX_PAGE_SIZE {
            return Err(SetupError::InvalidPageSize);
        }

        let clock_cfg = FlexspiClockConfig {
            power: clock.power,
            source: clock.source,
            div: clock.div,
            instance: T::CLOCK_INSTANCE,
        };
        let parts = unsafe { enable_and_reset::<T>(&clock_cfg).map_err(SetupError::ClockSetup)? };

        let mut flash_driver = Self {
            _peri: peri,
            regs: T::regs(),
            _pins: pins,
            dma,
            interrupt_mode,
            flash,
            _wg: parts.wake_guard,
            _phantom: PhantomData,
        };

        flash_driver.initialize()?;
        Ok(flash_driver)
    }

    pub fn page_size(&self) -> usize {
        self.flash.page_size
    }

    pub fn read_vendor_id(&mut self) -> Result<u8, IoError> {
        self.issue_ip_command(0, self.flash.read_id_seq as usize, 10, None)?;

        self.extract_vendor_id()
    }

    pub async fn read_vendor_id_async(&mut self) -> Result<u8, IoError> {
        self.issue_ip_command_async(0, self.flash.read_id_seq as usize, 10, None)
            .await?;

        self.extract_vendor_id()
    }

    fn extract_vendor_id(&self) -> Result<u8, IoError> {
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

    pub fn erase_sector(&mut self, address: u32) -> Result<(), IoError> {
        self.write_enable()?;
        self.issue_ip_command(address, self.flash.erase_sector_seq as usize, 0, None)?;
        self.wait_bus_busy()?;
        Ok(())
    }

    pub async fn erase_sector_async(&mut self, address: u32) -> Result<(), IoError> {
        self.write_enable_async().await?;
        self.issue_ip_command_async(address, self.flash.erase_sector_seq as usize, 0, None)
            .await?;
        self.wait_bus_busy_async().await?;

        Ok(())
    }

    pub fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), IoError> {
        let mut offset = 0;

        while offset < buffer.len() {
            let remaining = buffer.len() - offset;
            let chunk = remaining.min(IP_FIFO_CAPACITY_BYTES);

            self.issue_ip_read_command(
                address + offset as u32,
                self.flash.read_seq as usize,
                &mut buffer[offset..offset + chunk],
            )?;

            offset += chunk;
        }

        Ok(())
    }

    pub async fn read_async(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), IoError> {
        let mut offset = 0;

        while offset < buffer.len() {
            let remaining = buffer.len() - offset;
            let dma_chunk = remaining.min(IP_FIFO_CAPACITY_BYTES);

            if self.dma.is_some() && dma_chunk >= 4 {
                let mut words = [0u32; MAX_PAGE_WORDS];
                let word_len = dma_chunk.div_ceil(4);
                self.issue_ip_read_dma(
                    address + offset as u32,
                    self.flash.read_seq as usize,
                    &mut words[..word_len],
                    dma_chunk,
                )
                .await?;
                self.words_to_bytes(&words[..word_len], &mut buffer[offset..offset + dma_chunk]);
                offset += dma_chunk;
                continue;
            }

            let chunk = remaining.min(IP_FIFO_CAPACITY_BYTES);
            self.issue_ip_read_command_async(
                address + offset as u32,
                self.flash.read_seq as usize,
                &mut buffer[offset..offset + chunk],
            )
            .await?;
            offset += chunk;
        }

        Ok(())
    }

    pub fn page_program(&mut self, address: u32, data: &[u8]) -> Result<(), IoError> {
        if data.is_empty() || data.len() > self.flash.page_size {
            return Err(IoError::InvalidTransferLength);
        }

        self.write_enable()?;

        self.issue_ip_write_command(address, self.flash.page_program_seq as usize, data)?;

        self.wait_bus_busy()?;
        Ok(())
    }

    pub async fn page_program_async(&mut self, address: u32, data: &[u8]) -> Result<(), IoError> {
        if data.is_empty() || data.len() > self.flash.page_size {
            return Err(IoError::InvalidTransferLength);
        }

        self.write_enable_async().await?;

        if self.dma.is_some() && data.len() >= DMA_FIFO_WINDOW_BYTES && data.len() % DMA_FIFO_WINDOW_BYTES == 0 {
            let mut words = [0u32; MAX_PAGE_WORDS];
            let word_len = data.len().div_ceil(4);
            self.bytes_to_words(data, &mut words[..word_len]);
            self.issue_ip_write_dma(
                address,
                self.flash.page_program_seq as usize,
                &words[..word_len],
                data.len(),
            )
            .await?;
        } else {
            self.issue_ip_write_command_async(address, self.flash.page_program_seq as usize, data)
                .await?;
        }

        self.wait_bus_busy_async().await?;
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), SetupError> {
        self.configure_controller();
        self.flash_reset().ok();
        self.apply_device_mode().ok();
        Ok(())
    }

    fn configure_controller(&mut self) {
        self.regs.mcr0().write(|r: &mut Mcr0| {
            r.set_mdis(pac::flexspi::Mdis::Val0);
            r.set_rxclksrc(pac::flexspi::Rxclksrc::Val1);
        });
        self.regs.ahbcr().write(|r: &mut Ahbcr| {
            r.set_aparen(pac::flexspi::Aparen::Individual);
            r.set_clrahbrxbuf(pac::flexspi::Clrahbrxbuf::Val0);
            r.set_clrahbtxbuf(pac::flexspi::Clrahbtxbuf::Val0);
            r.set_cachableen(pac::flexspi::Cachableen::Val1);
            r.set_bufferableen(pac::flexspi::Bufferableen::Val1);
            r.set_prefetchen(pac::flexspi::AhbcrPrefetchen::Value1);
            r.set_readaddropt(pac::flexspi::Readaddropt::Val0);
            r.set_resumedisable(pac::flexspi::Resumedisable::Val0);
            r.set_readszalign(pac::flexspi::Readszalign::Val0);
            r.set_aflashbase(0x8);
        });
        self.regs.ahbrxbuf0cr0().write(|r: &mut Ahbrxbuf0cr0| {
            r.set_bufsz(0xff);
            r.set_mstrid(0);
            r.set_priority(0);
            r.set_prefetchen(pac::flexspi::Ahbrxbuf0cr0Prefetchen::Value1);
        });
        self.regs.flsha1cr0().write(|r: &mut Flsha1cr0| {
            r.set_flshsz(self.flash.flash_size_kbytes);
        });
        self.regs.flshcr1(0).write(|r: &mut Flshcr1| {
            r.set_tcss(3);
            r.set_tcsh(3);
            r.set_wa(pac::flexspi::Wa::Value0);
            r.set_cas(0);
            r.set_csintervalunit(pac::flexspi::Csintervalunit::Val0);
            r.set_csinterval(2);
        });
        self.regs.flshcr2(0).write(|r: &mut Flshcr2| {
            r.set_ardseqid(self.flash.read_seq);
            r.set_ardseqnum(1);
            r.set_awrseqid(self.flash.page_program_seq);
            r.set_awrseqnum(1);
            r.set_awrwait(0);
            r.set_awrwaitunit(pac::flexspi::Awrwaitunit::Val0);
            r.set_clrinstrptr(false);
        });
        self.regs.flshcr4().write(|r: &mut Flshcr4| {
            r.set_wmopt1(false);
            r.set_wmopt2b(pac::flexspi::Wmopt2b::Val0);
            r.set_wmena(pac::flexspi::Wmena::Val0);
            r.set_wmenb(pac::flexspi::Wmenb::Val0);
        });
        self.regs.iptxfcr().modify(|r: &mut Iptxfcr| r.set_txwmrk(0));
        self.regs.iprxfcr().modify(|r: &mut Iprxfcr| r.set_rxwmrk(0));

        self.load_lut(self.flash.lookup_table);
        self.software_reset();
    }

    fn load_lut(&mut self, table: lookup::LookupTable) {
        self.blocking_wait_bus_idle();
        self.set_lut_lock(false);

        for index in 0..LUT_WORD_COUNT {
            self.regs.lut(index).write_value(Lut(0));
        }

        for seq_index in 0..16 {
            let words = table.custom_sequence(seq_index as u8).into_words();
            for (word_index, word) in words.iter().enumerate() {
                self.regs.lut(4 * seq_index + word_index).write_value(Lut(*word));
            }
        }

        self.set_lut_lock(true);
    }

    fn flash_reset(&mut self) -> Result<(), IoError> {
        let Some(sequence) = self.flash.reset_sequence else {
            return Ok(());
        };

        let table = lookup::LookupTable::new().custom_command(TEMP_SEQUENCE_INDEX, sequence);
        self.load_lut(table);
        self.issue_ip_command(0, TEMP_SEQUENCE_INDEX as usize, 0, None)?;
        self.load_lut(self.flash.lookup_table);
        self.software_reset();
        Ok(())
    }

    fn apply_device_mode(&mut self) -> Result<(), IoError> {
        let Some(command) = self.flash.device_mode_command else {
            return Ok(());
        };

        if command.requires_write_enable {
            self.write_enable()?;
        }

        self.issue_ip_write_command(0, command.seq as usize, &command.payload[..command.len])?;
        self.wait_bus_busy()?;
        self.software_reset();
        Ok(())
    }

    fn write_enable(&mut self) -> Result<(), IoError> {
        self.issue_ip_command(0, self.flash.write_enable_seq as usize, 0, None)
    }

    async fn write_enable_async(&mut self) -> Result<(), IoError> {
        self.issue_ip_command_async(0, self.flash.write_enable_seq as usize, 0, None)
            .await
    }

    fn read_status(&mut self) -> Result<u8, IoError> {
        self.issue_ip_command(0, self.flash.read_status_seq as usize, 1, None)?;
        Ok((self.regs.rfdr(0).read().rxdata() & 0xff) as u8)
    }

    async fn read_status_async(&mut self) -> Result<u8, IoError> {
        self.issue_ip_command_async(0, self.flash.read_status_seq as usize, 1, None)
            .await?;
        Ok((self.regs.rfdr(0).read().rxdata() & 0xff) as u8)
    }

    fn wait_bus_busy(&mut self) -> Result<(), IoError> {
        loop {
            let read_value = self.read_status()?;
            let is_busy = if self.flash.busy_status_polarity {
                (read_value & (1 << self.flash.busy_status_offset)) != 0
            } else {
                (read_value & (1 << self.flash.busy_status_offset)) == 0
            };

            if !is_busy {
                return Ok(());
            }
        }
    }

    async fn wait_bus_busy_async(&mut self) -> Result<(), IoError> {
        loop {
            let read_value = self.read_status_async().await?;
            let is_busy = if self.flash.busy_status_polarity {
                (read_value & (1 << self.flash.busy_status_offset)) != 0
            } else {
                (read_value & (1 << self.flash.busy_status_offset)) == 0
            };

            if !is_busy {
                return Ok(());
            }

            yield_now().await;
        }
    }

    fn software_reset(&mut self) {
        self.regs
            .mcr0()
            .modify(|r: &mut Mcr0| r.set_swreset(pac::flexspi::Swreset::Val1));
        while self.regs.mcr0().read().swreset() == pac::flexspi::Swreset::Val1 {}
    }

    fn set_lut_lock(&mut self, lock: bool) {
        self.regs.lutkey().write_value(Lutkey(LUT_KEY_VALUE));
        self.regs.lutcr().write_value(Lutcr(if lock { 0x01 } else { 0x02 }));
    }

    fn blocking_wait_bus_idle(&self) {
        self.wait_idle();
    }

    fn wait_idle(&self) {
        while self.regs.sts0().read().seqidle() != pac::flexspi::Seqidle::Value1 {}
    }

    async fn wait_idle_async(&self) {
        while self.regs.sts0().read().seqidle() != pac::flexspi::Seqidle::Value1 {
            yield_now().await;
        }
    }

    fn prepare_ip_transfer(&mut self) {
        self.wait_idle();
        T::info().pending_events().store(0, Ordering::Release);
        T::Interrupt::unpend();

        self.regs.inten().write(|_| {});
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
            .modify(|r: &mut Iptxfcr| r.set_clriptxf(pac::flexspi::Clriptxf::Value1));
        self.regs
            .iprxfcr()
            .modify(|r: &mut Iprxfcr| r.set_clriprxf(pac::flexspi::Clriprxf::Value1));
    }

    fn wait_ip_command_done(&self) {
        while !self.regs.intr().read().ipcmddone() {}
    }

    fn wait_no_ip_error(&self) -> Result<(), IoError> {
        let status = self.regs.sts1().read();
        if status.ipcmderrcode() == pac::flexspi::Ipcmderrcode::Val0 {
            Ok(())
        } else {
            Err(IoError::Command {
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

        self.regs
            .ipcmd()
            .write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::Trg::Value1));
        self.wait_ip_command_done();
        self.wait_idle();
        self.wait_no_ip_error()
    }

    async fn issue_ip_command_async(
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

        self.regs
            .ipcmd()
            .write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::Trg::Value1));
        self.wait_for_command_completion_async().await
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

        self.regs
            .ipcmd()
            .write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::Trg::Value1));

        let tx_watermark = self.regs.iptxfcr().read().txwmrk() as usize + 1;
        let mut offset = 0;

        while offset < data.len() {
            while !self.regs.intr().read().iptxwe() {}

            let chunk_len = (8 * tx_watermark).min(data.len() - offset);
            for (index, chunk) in data[offset..offset + chunk_len].chunks(4).enumerate() {
                // Pad the trailing partial word with 0xFF.
                let mut word = [0xFFu8; 4];
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

    async fn issue_ip_write_command_async(
        &mut self,
        address: u32,
        seq_index: usize,
        data: &[u8],
    ) -> Result<(), IoError> {
        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(data.len() as u16);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        self.regs
            .ipcmd()
            .write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::Trg::Value1));

        let tx_watermark = self.regs.iptxfcr().read().txwmrk() as usize + 1;
        let mut offset = 0;

        while offset < data.len() {
            self.wait_for_tx_watermark_async().await?;

            let chunk_len = (8 * tx_watermark).min(data.len() - offset);
            for (index, chunk) in data[offset..offset + chunk_len].chunks(4).enumerate() {
                // Pad the trailing partial word with 0xFF (see the blocking
                // sibling above for why).
                let mut word = [0xFFu8; 4];
                word[..chunk.len()].copy_from_slice(chunk);
                self.regs.tfdr(index).write_value(Tfdr(u32::from_le_bytes(word)));
            }

            offset += chunk_len;
            self.regs.intr().write(|r: &mut Intr| r.set_iptxwe(true));
        }

        self.wait_for_command_completion_async().await
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

        self.regs
            .ipcmd()
            .write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::Trg::Value1));
        self.wait_ip_command_done();
        self.wait_idle();
        self.wait_no_ip_error()?;

        for (index, chunk) in buffer.chunks_mut(4).enumerate() {
            let word = self.regs.rfdr(index).read().rxdata().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }

        Ok(())
    }

    async fn issue_ip_read_command_async(
        &mut self,
        address: u32,
        seq_index: usize,
        buffer: &mut [u8],
    ) -> Result<(), IoError> {
        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(buffer.len() as u16);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        self.regs
            .ipcmd()
            .write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::Trg::Value1));
        self.wait_for_command_completion_async().await?;

        for (index, chunk) in buffer.chunks_mut(4).enumerate() {
            let word = self.regs.rfdr(index).read().rxdata().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }

        Ok(())
    }

    async fn issue_ip_write_dma(
        &mut self,
        address: u32,
        seq_index: usize,
        data: &[u32],
        data_len: usize,
    ) -> Result<(), IoError> {
        if data_len % DMA_FIFO_WINDOW_BYTES != 0 {
            return Err(IoError::InvalidTransferLength);
        }

        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(data_len as u16);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        let regs = self.regs;
        regs.ipcmd().write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::Trg::Value1));

        let tx_words_per_watermark = 2 * (regs.iptxfcr().read().txwmrk() as usize + 1);
        let mut offset = 0;

        while offset < data.len() {
            self.wait_for_tx_watermark_async().await?;

            let chunk_words = tx_words_per_watermark.min(data.len() - offset);
            let fifo_window =
                unsafe { core::slice::from_raw_parts_mut(regs.tfdr(0).as_ptr() as *mut u32, chunk_words) };
            {
                let dma = self.dma.as_mut().ok_or(IoError::InvalidTransferLength)?;
                let transfer =
                    dma.tx_dma
                        .mem_to_mem(&data[offset..offset + chunk_words], fifo_window, DMA_TRANSFER_OPTIONS)?;
                transfer.await?;
            }

            offset += chunk_words;
            regs.intr().write(|r: &mut Intr| r.set_iptxwe(true));
        }

        self.wait_for_command_completion_async().await
    }

    async fn issue_ip_read_dma(
        &mut self,
        address: u32,
        seq_index: usize,
        buffer: &mut [u32],
        data_len: usize,
    ) -> Result<(), IoError> {
        if data_len == 0 || data_len > IP_FIFO_CAPACITY_BYTES {
            return Err(IoError::InvalidTransferLength);
        }

        self.prepare_ip_transfer();

        self.regs.ipcr0().write(|r: &mut Ipcr0| r.set_sfar(address));
        self.regs.ipcr1().write(|r: &mut Ipcr1| {
            r.set_idatsz(data_len as u16);
            r.set_iseqid(seq_index as u8);
            r.set_iseqnum(0);
            r.set_iparen(false);
        });

        let regs = self.regs;
        regs.ipcmd().write(|r: &mut Ipcmd| r.set_trg(pac::flexspi::Trg::Value1));

        self.wait_for_command_completion_async().await?;

        let fifo_words = data_len.div_ceil(4);
        let fifo_window = unsafe { core::slice::from_raw_parts(regs.rfdr(0).as_ptr() as *const u32, fifo_words) };
        {
            let dma = self.dma.as_mut().ok_or(IoError::InvalidTransferLength)?;
            let transfer = dma
                .rx_dma
                .mem_to_mem(fifo_window, &mut buffer[..fifo_words], DMA_TRANSFER_OPTIONS)?;
            transfer.await?;
        }

        Ok(())
    }

    async fn wait_for_command_completion_async(&mut self) -> Result<(), IoError> {
        if !self.use_interrupt_waits() {
            loop {
                let status = self.regs.intr().read();

                if status.ipcmdge() {
                    self.wait_idle_async().await;
                    return Err(IoError::CommandGrantTimeout);
                }

                if status.ipcmddone() || status.ipcmderr() {
                    self.wait_idle_async().await;
                    return self.wait_no_ip_error();
                }

                yield_now().await;
            }
        }

        let timed_out = poll_fn(|cx| {
            T::info().waker().register(cx.waker());

            self.regs.inten().write(|w| {
                w.set_ipcmddoneen(pac::flexspi::Ipcmddoneen::Value1);
                w.set_ipcmdgeen(pac::flexspi::Ipcmdgeen::Value1);
                w.set_ipcmderren(pac::flexspi::Ipcmderren::Value1);
            });

            let pending_events = T::info().pending_events().load(Ordering::Acquire);
            let status = self.regs.intr().read();

            if (pending_events & IRQ_EVENT_COMMAND_GRANT) != 0 || status.ipcmdge() {
                T::info()
                    .pending_events()
                    .fetch_and(!IRQ_EVENT_COMMAND_GRANT, Ordering::AcqRel);
                return Poll::Ready(true);
            }

            if (pending_events & (IRQ_EVENT_COMMAND_DONE | IRQ_EVENT_COMMAND_ERROR)) != 0
                || status.ipcmddone()
                || status.ipcmderr()
            {
                T::info()
                    .pending_events()
                    .fetch_and(!(IRQ_EVENT_COMMAND_DONE | IRQ_EVENT_COMMAND_ERROR), Ordering::AcqRel);
                return Poll::Ready(false);
            }

            Poll::Pending
        })
        .await;

        self.regs.inten().write(|_| {});
        self.wait_idle_async().await;

        if timed_out {
            return Err(IoError::CommandGrantTimeout);
        }

        self.wait_no_ip_error()
    }

    async fn wait_for_tx_watermark_async(&mut self) -> Result<(), IoError> {
        if self.use_interrupt_waits() {
            return poll_fn(|cx| {
                T::info().waker().register(cx.waker());

                self.regs.inten().write(|w| {
                    w.set_iptxween(pac::flexspi::Iptxween::Value1);
                    w.set_ipcmdgeen(pac::flexspi::Ipcmdgeen::Value1);
                    w.set_ipcmderren(pac::flexspi::Ipcmderren::Value1);
                });

                let pending_events = T::info().pending_events().load(Ordering::Acquire);
                let status = self.regs.intr().read();

                if (pending_events & IRQ_EVENT_COMMAND_GRANT) != 0 || status.ipcmdge() {
                    T::info()
                        .pending_events()
                        .fetch_and(!IRQ_EVENT_COMMAND_GRANT, Ordering::AcqRel);
                    if status.ipcmdge() {
                        self.regs.intr().write(|w| w.set_ipcmdge(true));
                    }
                    return Poll::Ready(Err(IoError::CommandGrantTimeout));
                }

                if (pending_events & IRQ_EVENT_COMMAND_ERROR) != 0 || status.ipcmderr() {
                    T::info()
                        .pending_events()
                        .fetch_and(!IRQ_EVENT_COMMAND_ERROR, Ordering::AcqRel);
                    if status.ipcmderr() {
                        self.regs.intr().write(|w| w.set_ipcmderr(true));
                    }
                    return Poll::Ready(self.wait_no_ip_error());
                }

                if (pending_events & IRQ_EVENT_TX_WATERMARK) != 0 || status.iptxwe() {
                    T::info()
                        .pending_events()
                        .fetch_and(!IRQ_EVENT_TX_WATERMARK, Ordering::AcqRel);
                    return Poll::Ready(Ok(()));
                }

                Poll::Pending
            })
            .await;
        }

        loop {
            let status = self.regs.intr().read();

            if status.ipcmdge() {
                self.regs.intr().write(|w| w.set_ipcmdge(true));
                return Err(IoError::CommandGrantTimeout);
            }

            if status.ipcmderr() {
                self.regs.intr().write(|w| w.set_ipcmderr(true));
                return self.wait_no_ip_error();
            }

            if status.iptxwe() {
                return Ok(());
            }

            yield_now().await;
        }
    }

    fn bytes_to_words(&self, data: &[u8], words: &mut [u32]) {
        for word in words.iter_mut() {
            *word = 0;
        }

        for (index, chunk) in data.chunks(4).enumerate() {
            let mut raw = [0u8; 4];
            raw[..chunk.len()].copy_from_slice(chunk);
            words[index] = u32::from_le_bytes(raw);
        }
    }

    fn words_to_bytes(&self, words: &[u32], buffer: &mut [u8]) {
        for (chunk, word) in buffer.chunks_mut(4).zip(words.iter()) {
            chunk.copy_from_slice(&word.to_le_bytes()[..chunk.len()]);
        }
    }
}

pub struct Flexspi<'d, T: Instance> {
    inner: InnerFlexSpi<'d, T>,
}

impl<'d, T: Instance> Flexspi<'d, T> {
    pub fn new_blocking(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        Ok(Self {
            inner: InnerFlexSpi::new_blocking(peri, pin0, pin1, pin2, pin3, pin4, pin5, pin6, pin7, clock, flash)?,
        })
    }

    pub fn new_async(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        Ok(Self {
            inner: InnerFlexSpi::new_async(peri, pin0, pin1, pin2, pin3, pin4, pin5, pin6, pin7, irq, clock, flash)?,
        })
    }

    pub fn new_with_dma(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        Ok(Self {
            inner: InnerFlexSpi::new_with_dma(
                peri, pin0, pin1, pin2, pin3, pin4, pin5, pin6, pin7, tx_dma, rx_dma, irq, clock, flash,
            )?,
        })
    }
}

pub struct NorFlash<'d, T: Instance> {
    flexspi: Flexspi<'d, T>,
}

impl<'d, T: Instance> NorFlash<'d, T> {
    pub fn new_blocking(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        Ok(Self::from_flexspi(Flexspi::new_blocking(
            peri, pin0, pin1, pin2, pin3, pin4, pin5, pin6, pin7, clock, flash,
        )?))
    }

    pub fn new_async(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        Ok(Self::from_flexspi(Flexspi::new_async(
            peri, pin0, pin1, pin2, pin3, pin4, pin5, pin6, pin7, irq, clock, flash,
        )?))
    }

    pub fn new_with_dma(
        peri: Peri<'d, T>,
        pin0: Peri<'d, impl Pin<T> + 'd>,
        pin1: Peri<'d, impl Pin<T> + 'd>,
        pin2: Peri<'d, impl Pin<T> + 'd>,
        pin3: Peri<'d, impl Pin<T> + 'd>,
        pin4: Peri<'d, impl Pin<T> + 'd>,
        pin5: Peri<'d, impl Pin<T> + 'd>,
        pin6: Peri<'d, impl Pin<T> + 'd>,
        pin7: Peri<'d, impl Pin<T> + 'd>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clock: ClockConfig,
        flash: FlashConfig,
    ) -> Result<Self, SetupError> {
        Ok(Self::from_flexspi(Flexspi::new_with_dma(
            peri, pin0, pin1, pin2, pin3, pin4, pin5, pin6, pin7, tx_dma, rx_dma, irq, clock, flash,
        )?))
    }

    pub fn from_flexspi(flexspi: Flexspi<'d, T>) -> Self {
        Self { flexspi }
    }

    pub fn page_size(&self) -> usize {
        self.flexspi.inner.page_size()
    }

    pub fn blocking_vendor_id(&mut self) -> Result<u8, IoError> {
        self.flexspi.inner.read_vendor_id()
    }

    pub async fn read_vendor_id_async(&mut self) -> Result<u8, IoError> {
        self.flexspi.inner.read_vendor_id_async().await
    }

    pub fn blocking_erase_sector(&mut self, address: u32) -> Result<(), IoError> {
        self.flexspi.inner.erase_sector(address)
    }

    pub async fn erase_sector_async(&mut self, address: u32) -> Result<(), IoError> {
        self.flexspi.inner.erase_sector_async(address).await
    }

    pub fn blocking_read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), IoError> {
        self.flexspi.inner.read(address, buffer)
    }

    pub fn blocking_page_program(&mut self, address: u32, data: &[u8]) -> Result<(), IoError> {
        self.flexspi.inner.page_program(address, data)
    }

    pub async fn read_async(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), IoError> {
        self.flexspi.inner.read_async(address, buffer).await
    }

    pub async fn page_program_async(&mut self, address: u32, data: &[u8]) -> Result<(), IoError> {
        self.flexspi.inner.page_program_async(address, data).await
    }
}
