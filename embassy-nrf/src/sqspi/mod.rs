#![macro_use]

//! Soft Quad Serial Peripheral Interface (sQSPI) flash driver.
//!
//! sQSPI is a QSPI/SPI controller implemented in *firmware* running on the
//! nRF54L's FLPR RISC-V coprocessor (VPR00). Unlike a hardware peripheral its
//! register interface lives in shared RAM, not at a fixed MMIO address: the
//! host CPU and the FLPR communicate through a "virtual register block"
//! (see the [`regs`] submodule) that the firmware emulates as if it were a real
//! QSPI peripheral.
//!
//! This driver exposes the same API shape as the hardware [`crate::qspi`]
//! driver (the same `read`/`write`/`erase` methods, blocking variants,
//! `custom_instruction`, and the `embedded-storage` `NorFlash` traits), with
//! two extra constructor arguments specific to a soft peripheral:
//!
//! - `firmware`: the FLPR firmware blob (starts with a metadata header). The
//!   caller owns and provides it (e.g. via `include_bytes!`).
//! - `ram`: a RAM buffer the firmware code, its working RAM, and the virtual
//!   register block are placed into.
//!
//! # Example
//! ```ignore
//! use embassy_nrf::sqspi::{self, Config};
//! use static_cell::ConstStaticCell;
//! use core::mem::MaybeUninit;
//!
//! bind_interrupts!(struct Irqs { VPR00 => sqspi::InterruptHandler<peripherals::VPR>; });
//!
//! static FW: &[u8] = include_bytes!("sqspi_firmware.bin");
//! static RAM: ConstStaticCell<[MaybeUninit<u8>; 0x4000]> = ConstStaticCell::new([MaybeUninit::uninit(); 0x4000]);
//! let ram = RAM.take();
//!
//! let mut config = Config::default();
//! config.capacity = 8 * 1024 * 1024;
//! let mut q = sqspi::Sqspi::new(
//!     p.VPR, Irqs, FW, ram,
//!     p.P2_01, p.P2_05, p.P2_02, p.P2_04, p.P2_03, p.P2_00, config,
//! ).unwrap();
//! ```

mod regs;

use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;
// Re-exported so callers can spell `sqspi::MODE_0` etc., matching `spim`/`qspi`.
pub use embedded_hal_02::spi::{MODE_0, MODE_3, Mode, Phase, Polarity};
use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

use crate::gpio::{AnyPin, Pin as GpioPin, SealedPin};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::gpio::vals as gpiovals;
use crate::{interrupt, pac};

/// Expected sQSPI soft peripheral ID in the firmware metadata header.
const SOFTPERIPHERAL_ID_SQSPI: u16 = 0x45b1;

/// Parsed firmware metadata from the start of the sQSPI firmware blob.
///
/// The blob begins with a 32-byte (8 x u32) metadata header. Header versions 1
/// and 2 share the field layout for the fields the driver uses.
struct FirmwareMetadata {
    /// If true the VPR boots directly from the firmware address (NVM); if false
    /// the driver must copy the firmware code into RAM (the usual case).
    self_boot: bool,
    /// Firmware code-region size, in units of 16 bytes.
    fw_code_size: u16,
    /// Total RAM the firmware needs (code + working RAM + register block), in
    /// units of 16 bytes.
    fw_ram_total_size: u16,
    /// Offset (bytes) from the end of the code region to the shared register
    /// block.
    fw_shared_ram_addr_offset: u16,
}

impl FirmwareMetadata {
    fn parse(fw: &[u8]) -> Result<Self, Error> {
        if fw.len() < 32 {
            return Err(Error::FirmwareTooShort);
        }

        let w0 = u32::from_le_bytes([fw[0], fw[1], fw[2], fw[3]]);
        let w1 = u32::from_le_bytes([fw[4], fw[5], fw[6], fw[7]]);
        let w3 = u32::from_le_bytes([fw[12], fw[13], fw[14], fw[15]]);
        let w6 = u32::from_le_bytes([fw[24], fw[25], fw[26], fw[27]]);

        // w0: magic(16) | header_version(4) | comm_id(8) | reserved(3) | self_boot(1)
        let self_boot = (w0 >> 31) & 1 != 0;

        // w1: softperiph_id(16) | platform(16)
        let softperiph_id = (w1 & 0xFFFF) as u16;
        if softperiph_id != SOFTPERIPHERAL_ID_SQSPI {
            return Err(Error::InvalidFirmware);
        }

        // w3: fw_code_size(16) | fw_ram_total_size(16)
        let fw_code_size = (w3 & 0xFFFF) as u16;
        let fw_ram_total_size = ((w3 >> 16) & 0xFFFF) as u16;

        // w6: fw_shared_ram_size(16) | fw_shared_ram_addr_offset(16)
        let fw_shared_ram_addr_offset = ((w6 >> 16) & 0xFFFF) as u16;

        Ok(Self {
            self_boot,
            fw_code_size,
            fw_ram_total_size,
            fw_shared_ram_addr_offset,
        })
    }

    /// Firmware code-region size in bytes.
    fn code_size_bytes(&self) -> usize {
        (self.fw_code_size as usize) << 4
    }

    /// Total RAM needed in bytes.
    fn ram_total_bytes(&self) -> usize {
        (self.fw_ram_total_size as usize) << 4
    }

    /// Byte offset from the RAM base to the shared register block.
    fn reg_offset(&self) -> usize {
        self.code_size_bytes() + self.fw_shared_ram_addr_offset as usize
    }
}

/// sQSPI bus frequency.
///
/// The FLPR runs at a fixed 128 MHz base clock; each variant is the SCK clock
/// divider applied to it (e.g. [`Frequency::M8`] = 128 MHz / 16 = 8 MHz).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    /// 64 MHz (divider 2).
    M64 = 2,
    /// 32 MHz (divider 4).
    M32 = 4,
    /// 16 MHz (divider 8).
    M16 = 8,
    /// 8 MHz (divider 16).
    M8 = 16,
    /// 4 MHz (divider 32).
    M4 = 32,
    /// 2 MHz (divider 64).
    M2 = 64,
    /// 1 MHz (divider 128).
    M1 = 128,
}

/// Flash addressing mode.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddressMode {
    /// 24-bit addressing (3 address bytes).
    _24Bit,
    /// 32-bit addressing (4 address bytes).
    _32Bit,
}

impl AddressMode {
    fn bits(self) -> u32 {
        match self {
            AddressMode::_24Bit => 24,
            AddressMode::_32Bit => 32,
        }
    }
}

/// Multi-line SPI mode for the address and data phases.
///
/// Maps to `CTRLR0.SPI_FRF` (data-line width) and `SPICTRLR0.TRANSTYPE`
/// (whether the address is sent on the command line or the data lines). The
/// command opcode is always sent on a single line.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Lines {
    /// Single line (1-1-1): standard MOSI/MISO.
    Single,
    /// Dual output (1-1-2): single command, single address, dual data.
    Dual1_1_2,
    /// Dual I/O (1-2-2): single command, dual address, dual data.
    Dual1_2_2,
    /// Quad output (1-1-4): single command, single address, quad data.
    Quad1_1_4,
    /// Quad I/O (1-4-4): single command, quad address, quad data.
    Quad1_4_4,
}

impl Lines {
    /// `CTRLR0.SPI_FRF`: 0 = standard, 1 = dual, 2 = quad.
    fn spi_frf(self) -> u8 {
        match self {
            Lines::Single => 0,
            Lines::Dual1_1_2 | Lines::Dual1_2_2 => 1,
            Lines::Quad1_1_4 | Lines::Quad1_4_4 => 2,
        }
    }

    /// `SPICTRLR0.TRANSTYPE`: 0 = address on the command line, 1 = address in
    /// `SPI_FRF` (data-line) mode.
    fn transtype(self) -> u8 {
        match self {
            Lines::Single | Lines::Dual1_1_2 | Lines::Quad1_1_4 => 0,
            Lines::Dual1_2_2 | Lines::Quad1_4_4 => 1,
        }
    }

    /// Whether this mode drives the IO2/IO3 lines (quad only).
    fn uses_io2_io3(self) -> bool {
        matches!(self, Lines::Quad1_1_4 | Lines::Quad1_4_4)
    }
}

/// sQSPI driver configuration.
#[non_exhaustive]
pub struct Config {
    /// Bus frequency.
    pub frequency: Frequency,
    /// SPI clock polarity and phase.
    pub spi_mode: Mode,
    /// Multi-line mode used for the read/write data phases.
    pub lines: Lines,
    /// Flash addressing mode (24-bit or 32-bit).
    pub address_mode: AddressMode,
    /// Read command opcode (e.g. `0xEB` Quad I/O Read, `0x03` Read).
    pub read_opcode: u8,
    /// Page-program command opcode (e.g. `0x38` Quad I/O PP, `0x02` Page Program).
    pub write_opcode: u8,
    /// Dummy/wait cycles after the address phase of a read. Opcode- and
    /// flash-specific: MX25R `0xEB` → 6, `0x6B`/`0x0B` → 8, `0x03` → 0.
    pub read_dummy_cycles: u8,
    /// Flash page size for program operations. Programs are split so they never
    /// cross a page boundary (the flash wraps within a page otherwise).
    pub write_page_size: u32,
    /// RX sample delay, in SCK cycles.
    ///
    /// Must be at least 1 for normal-speed transfers: with 0 the sampled RX
    /// data is unreliable (RDSR returns garbage, breaking WIP polling). The
    /// default is 1, matching Nordic's Zephyr driver.
    pub sample_delay: u8,
    /// Flash capacity in bytes, used for bounds checks. 0 disables them.
    pub capacity: u32,
}

impl Default for Config {
    fn default() -> Self {
        // Quad I/O by default, matching the hardware `qspi` driver: READ4IO
        // (0xEB, 1-4-4, 6 dummy) + PP4IO (0x38, 1-4-4). Requires the flash's
        // quad-enable (QE) bit to be set.
        Self {
            frequency: Frequency::M8,
            spi_mode: MODE_0,
            lines: Lines::Quad1_4_4,
            address_mode: AddressMode::_24Bit,
            read_opcode: 0xEB,
            write_opcode: 0x38,
            read_dummy_cycles: 6,
            write_page_size: 256,
            sample_delay: 1,
            capacity: 0,
        }
    }
}

/// sQSPI error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The firmware blob is too short to contain a metadata header.
    FirmwareTooShort,
    /// The firmware blob is not a valid sQSPI soft peripheral.
    InvalidFirmware,
    /// The provided RAM buffer is too small for the firmware.
    BufferTooSmall,
    /// Operation address was out of bounds.
    OutOfBounds,
    /// The transfer was aborted by the peripheral (bus error).
    Transfer,
    /// The VPR coprocessor could not be started.
    Vpr,
}

impl From<crate::vpr::Error> for Error {
    fn from(_: crate::vpr::Error) -> Self {
        Error::Vpr
    }
}

/// `CTRLR0.TMOD` transfer direction.
#[repr(u32)]
#[derive(Copy, Clone)]
enum Dir {
    #[allow(dead_code)]
    TxRx = 0,
    Tx = 1,
    Rx = 2,
}

/// Interrupt handler for the sQSPI driver.
///
/// Bind it to the `VPR00` interrupt: the FLPR raises it (via the VPR
/// `EVENTS_TRIGGERED[20]` event) when a transfer completes.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let vpr = <T as crate::vpr::SealedInstance>::regs();
        if vpr.events_triggered(regs::SP_VPR_EVENT_IDX).read() != 0 {
            vpr.events_triggered(regs::SP_VPR_EVENT_IDX).write_value(0);
            T::state().waker.wake();
        }
    }
}

/// sQSPI flash driver.
pub struct Sqspi<'d> {
    regs: regs::Regs,
    vpr: pac::vpr::Vpr,
    state: &'static State,
    config: Config,
    task_count: u32,
    sck: Peri<'d, AnyPin>,
    csn: Peri<'d, AnyPin>,
    io0: Peri<'d, AnyPin>,
    io1: Peri<'d, AnyPin>,
    io2: Peri<'d, AnyPin>,
    io3: Peri<'d, AnyPin>,
}

impl<'d> Sqspi<'d> {
    /// Create a new sQSPI driver.
    ///
    /// Loads `firmware` into `ram`, boots the FLPR, routes the pins to it, and
    /// configures the SPI core per `config`. See the [module docs](self) for
    /// the `firmware` / `ram` requirements.
    #[allow(clippy::too_many_arguments)]
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        firmware: &[u8],
        ram: &'d mut [MaybeUninit<u8>],
        sck: Peri<'d, impl GpioPin>,
        csn: Peri<'d, impl GpioPin>,
        io0: Peri<'d, impl GpioPin>,
        io1: Peri<'d, impl GpioPin>,
        io2: Peri<'d, impl GpioPin>,
        io3: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Result<Self, Error> {
        let meta = FirmwareMetadata::parse(firmware)?;
        let vpr = <T as crate::vpr::SealedInstance>::regs();
        let state = T::state();

        // Stop a still-running FLPR before touching its RAM, in case this driver
        // is being re-created. (A cold-boot leftover is already cleared by
        // `embassy_nrf::init`; see `config::FlprReset`.)
        crate::vpr::make_secure();
        crate::vpr::stop_reset(vpr);

        // RAM layout: [ align pad | firmware code | working RAM | register block ].
        // INITPC ignores the low 7 bits, so the base must be 128-aligned.
        unsafe { ptr::write_bytes(ram.as_mut_ptr(), 0, ram.len()) };
        let raw_base = ram.as_mut_ptr() as usize;
        let base = (raw_base + 127) & !127;
        let usable = ram.len() - (base - raw_base);

        let reg_offset = meta.reg_offset();
        let needed = meta.ram_total_bytes().max(reg_offset + regs::Regs::SIZE);
        if usable < needed {
            return Err(Error::BufferTooSmall);
        }

        let reg_ptr = (base + reg_offset) as *mut ();
        let sp = unsafe { regs::Regs::from_ptr(reg_ptr) };

        // Boot the firmware through the VPR driver: `Vpr::new` sets INITPC, then
        // arm the handshake (host sets ENABLE, firmware clears it when ready),
        // load the code (the .bss tail past the blob stays zeroed), and run.
        let mut coproc = crate::vpr::Vpr::new(peri, base as *const u8)?;
        sp.enable().write_value(1);
        if !meta.self_boot {
            let copy_len = firmware.len().min(meta.code_size_bytes());
            coproc.load(&firmware[..copy_len])?;
        }
        coproc.start();
        while sp.enable().read() != 0 {}

        // IO2/IO3 double as quad data lines; outside quad mode they're held high
        // as plain GPIO so WP#/HOLD# stay released.
        let sck: Peri<'d, AnyPin> = sck.into();
        let csn: Peri<'d, AnyPin> = csn.into();
        let io0: Peri<'d, AnyPin> = io0.into();
        let io1: Peri<'d, AnyPin> = io1.into();
        let io2: Peri<'d, AnyPin> = io2.into();
        let io3: Peri<'d, AnyPin> = io3.into();
        Self::config_pin(&*sck, false, true);
        Self::config_pin(&*csn, false, true);
        Self::config_pin(&*io0, true, true);
        Self::config_pin(&*io1, true, true);
        Self::config_pin(&*io2, true, config.lines.uses_io2_io3());
        Self::config_pin(&*io3, true, config.lines.uses_io2_io3());

        // 8-bit flash frames, MSB first.
        sp.format().dfs().write_value(7);
        sp.format().bpp().write_value(8);
        sp.format().bitorder().write_value(0);
        sp.core().dr(22).write_value(32); // firmware-private "32 - padding"

        // Notify the host on DMA done/aborted/donejob.
        sp.inten().write(|w| {
            w.set_dmadone(true);
            w.set_dmaaborted(true);
            w.set_dmadonejob(true);
        });

        sp.core().baudr().write(|w| w.set_sckdv(config.frequency as u16));
        sp.core().rxsampledelay().write_value(config.sample_delay as u32);

        sp.enable().write_value(1); // activate

        let mut this = Self {
            regs: sp,
            vpr,
            state,
            config,
            task_count: 1,
            sck,
            csn,
            io0,
            io1,
            io2,
            io3,
        };
        this.asb();
        sp.events_dma().done().write_value(0);
        sp.events_dma().aborted().write_value(0);
        sp.events_dma().events_done().job().write_value(0);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Ok(this)
    }

    /// Configure a pin: `input` connects the input buffer (data lines), `to_vpr`
    /// routes it to the FLPR (else it's held high as a plain GPIO output).
    fn config_pin(pin: &impl SealedPin, input: bool, to_vpr: bool) {
        pin.set_high();
        pin.conf().write(|w| {
            w.set_dir(gpiovals::Dir::Output);
            w.set_input(if input {
                gpiovals::Input::Connect
            } else {
                gpiovals::Input::Disconnect
            });
            w.set_pull(if input {
                gpiovals::Pull::Pullup
            } else {
                gpiovals::Pull::Disabled
            });
            w.set_drive0(gpiovals::Drive::H);
            w.set_drive1(gpiovals::Drive::H);
            if to_vpr {
                w.set_ctrlsel(gpiovals::Ctrlsel::Vpr);
            }
        });
    }

    /// Extended sync barrier: publish the task counter, trigger the VPR task,
    /// and spin until the firmware echoes it back.
    fn xsb(&mut self, task: usize) {
        let aux = self.regs.spsync();
        aux.aux(0).write_value(self.task_count);
        // Order the shared-RAM (Normal) writes before the Device-memory trigger.
        cortex_m::asm::dmb();
        self.vpr.tasks_trigger(task).write_value(1);
        while aux.aux(0).read() != aux.aux(1).read() {
            cortex_m::asm::nop();
        }
        cortex_m::asm::dmb();
        self.task_count = self.task_count.wrapping_add(1);
    }

    /// Config sync barrier (`__CSB`).
    fn csb(&mut self) {
        self.xsb(regs::SP_VPR_TASK_CONFIG_IDX);
    }

    /// Action sync barrier (`__ASB`).
    fn asb(&mut self) {
        self.xsb(regs::SP_VPR_TASK_ACTION_IDX);
    }

    /// Configure and kick off one SPI transaction. Completion is awaited
    /// separately via [`wait_done`](Self::wait_done) / [`blocking_wait_done`].
    #[allow(clippy::too_many_arguments)]
    fn start(
        &mut self,
        opcode: u8,
        address: u32,
        addr_bits: u32,
        dummy: u32,
        ptr: u32,
        len: usize,
        dir: Dir,
        lines: Lines,
    ) {
        let core = self.regs.core();
        let format = self.regs.format();

        let ndf = len as u32;
        format.pixels().write_value(ndf);
        core.ctrlr1().write_value(ndf & 0xFFFF);
        core.dr(23).write_value(ndf); // firmware-private byte count
        format.cilen().write_value(1); // 8-bit command => one 32-bit word

        let scph = self.config.spi_mode.phase == Phase::CaptureOnSecondTransition;
        let scpol = self.config.spi_mode.polarity == Polarity::IdleHigh;
        core.ctrlr0().write(|w| {
            w.set_sqspiismst(true);
            w.set_dfs(7); // 8-bit frames
            w.set_cfs(7); // 8-bit control frames
            w.set_tmod(dir as u8);
            w.set_spi_frf(lines.spi_frf());
            w.set_scph(scph);
            w.set_scpol(scpol);
        });

        core.spictrlr0().write(|w| {
            w.set_transtype(lines.transtype());
            w.set_addrl((addr_bits / 4) as u8);
            w.set_instl(2); // 8-bit instruction
            w.set_waitcycles(dummy as u8);
        });

        let address = address as u64;
        core.dr(0).write_value(opcode as u32);
        core.dr(1).write_value((address & 0xFFFF_FFFF) as u32);
        core.dr(2).write_value((address >> 31) as u32);
        core.dr(3).write_value(ptr);
        core.dr(4).write_value(len as u32);

        self.csb();
        core.sqspienr().write_value(1);
        self.asb();
        cortex_m::asm::dmb();
        self.vpr.tasks_trigger(regs::SP_VPR_TASK_DPPI_0_IDX).write_value(1);
    }

    /// Tear down a finished transfer: clear the event, disable the core, ASB.
    fn finish(&mut self) {
        self.regs.core().sqspienr().write_value(0);
        self.asb();
    }

    /// Await transfer completion via the VPR00 interrupt.
    async fn wait_done(&mut self) -> Result<(), Error> {
        let regs = self.regs;
        let state = self.state;
        let res = poll_fn(|cx| {
            state.waker.register(cx.waker());
            if regs.events_dma().aborted().read() != 0 {
                regs.events_dma().aborted().write_value(0);
                return Poll::Ready(Err(Error::Transfer));
            }
            if regs.events_dma().done().read() != 0 {
                cortex_m::asm::dmb();
                regs.events_dma().done().write_value(0);
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;
        self.finish();
        res
    }

    /// Spin until transfer completion.
    fn blocking_wait_done(&mut self) -> Result<(), Error> {
        let res = loop {
            if self.regs.events_dma().aborted().read() != 0 {
                self.regs.events_dma().aborted().write_value(0);
                break Err(Error::Transfer);
            }
            if self.regs.events_dma().done().read() != 0 {
                cortex_m::asm::dmb();
                self.regs.events_dma().done().write_value(0);
                break Ok(());
            }
        };
        self.finish();
        res
    }

    // SPI NOR ignores program/erase unless WREN (0x06) sets WEL first, and stays
    // busy (WIP=1 in RDSR/0x05) for milliseconds after. The hardware `qspi`
    // peripheral automates this in CINSTRCONF; on sQSPI we do it in software.

    async fn write_enable(&mut self) -> Result<(), Error> {
        self.start(0x06, 0, 0, 0, 0, 0, Dir::Tx, Lines::Single);
        self.wait_done().await
    }

    async fn wait_wip(&mut self) -> Result<(), Error> {
        loop {
            let mut status = [0u8; 1];
            self.start(0x05, 0, 0, 0, status.as_mut_ptr() as u32, 1, Dir::Rx, Lines::Single);
            self.wait_done().await?;
            if status[0] & 0x01 == 0 {
                return Ok(());
            }
        }
    }

    fn blocking_write_enable(&mut self) -> Result<(), Error> {
        self.start(0x06, 0, 0, 0, 0, 0, Dir::Tx, Lines::Single);
        self.blocking_wait_done()
    }

    fn blocking_wait_wip(&mut self) -> Result<(), Error> {
        loop {
            let mut status = [0u8; 1];
            self.start(0x05, 0, 0, 0, status.as_mut_ptr() as u32, 1, Dir::Rx, Lines::Single);
            self.blocking_wait_done()?;
            if status[0] & 0x01 == 0 {
                return Ok(());
            }
        }
    }

    /// Do a custom SPI instruction (single line).
    ///
    /// Like the hardware `qspi` driver this issues WREN before the opcode and
    /// waits for WIP to clear afterwards, so state-changing commands (WRSR, …)
    /// work without extra orchestration. If `resp` is non-empty the command
    /// reads into it; otherwise `req` is transmitted.
    pub async fn custom_instruction(&mut self, opcode: u8, req: &[u8], resp: &mut [u8]) -> Result<(), Error> {
        self.write_enable().await?;
        if !resp.is_empty() {
            self.start(
                opcode,
                0,
                0,
                0,
                resp.as_mut_ptr() as u32,
                resp.len(),
                Dir::Rx,
                Lines::Single,
            );
        } else {
            self.start(opcode, 0, 0, 0, req.as_ptr() as u32, req.len(), Dir::Tx, Lines::Single);
        }
        self.wait_done().await?;
        self.wait_wip().await
    }

    /// Do a custom SPI instruction, blocking version.
    pub fn blocking_custom_instruction(&mut self, opcode: u8, req: &[u8], resp: &mut [u8]) -> Result<(), Error> {
        self.blocking_write_enable()?;
        if !resp.is_empty() {
            self.start(
                opcode,
                0,
                0,
                0,
                resp.as_mut_ptr() as u32,
                resp.len(),
                Dir::Rx,
                Lines::Single,
            );
        } else {
            self.start(opcode, 0, 0, 0, req.as_ptr() as u32, req.len(), Dir::Tx, Lines::Single);
        }
        self.blocking_wait_done()?;
        self.blocking_wait_wip()
    }

    /// Send a single bare opcode (no address, no data) on a single line,
    /// *without* issuing WREN first or polling WIP afterwards.
    ///
    /// Unlike [`custom_instruction`](Self::custom_instruction) this does not
    /// touch the status register, which is what makes it usable for power-state
    /// commands such as deep power-down (`0xB9`): once the flash is in DPD it
    /// stops answering RDSR, so the usual trailing WIP poll would hang forever.
    pub fn blocking_custom_opcode(&mut self, opcode: u8) -> Result<(), Error> {
        self.start(opcode, 0, 0, 0, 0, 0, Dir::Tx, Lines::Single);
        self.blocking_wait_done()
    }

    /// Raw read: no bounds check against the configured capacity.
    pub async fn read_raw(&mut self, address: u32, data: &mut [u8]) -> Result<(), Error> {
        if data.is_empty() {
            return Ok(());
        }
        let (op, addr_bits, dummy, lines) = self.read_params();
        self.start(
            op,
            address,
            addr_bits,
            dummy,
            data.as_mut_ptr() as u32,
            data.len(),
            Dir::Rx,
            lines,
        );
        self.wait_done().await
    }

    /// Raw write: no bounds check against the configured capacity.
    pub async fn write_raw(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        let mut addr = address;
        let mut rest = data;
        let (op, addr_bits, lines) = self.write_params();
        while !rest.is_empty() {
            let n = self.page_chunk(addr, rest.len());
            self.write_enable().await?;
            self.start(op, addr, addr_bits, 0, rest.as_ptr() as u32, n, Dir::Tx, lines);
            self.wait_done().await?;
            self.wait_wip().await?;
            addr += n as u32;
            rest = &rest[n..];
        }
        Ok(())
    }

    /// Raw read, blocking version.
    pub fn blocking_read_raw(&mut self, address: u32, data: &mut [u8]) -> Result<(), Error> {
        if data.is_empty() {
            return Ok(());
        }
        let (op, addr_bits, dummy, lines) = self.read_params();
        self.start(
            op,
            address,
            addr_bits,
            dummy,
            data.as_mut_ptr() as u32,
            data.len(),
            Dir::Rx,
            lines,
        );
        self.blocking_wait_done()
    }

    /// Raw write, blocking version.
    pub fn blocking_write_raw(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        let mut addr = address;
        let mut rest = data;
        let (op, addr_bits, lines) = self.write_params();
        while !rest.is_empty() {
            let n = self.page_chunk(addr, rest.len());
            self.blocking_write_enable()?;
            self.start(op, addr, addr_bits, 0, rest.as_ptr() as u32, n, Dir::Tx, lines);
            self.blocking_wait_done()?;
            self.blocking_wait_wip()?;
            addr += n as u32;
            rest = &rest[n..];
        }
        Ok(())
    }

    /// Read from flash, bounds-checked against the configured capacity.
    pub async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Error> {
        self.bounds_check(address, data.len())?;
        self.read_raw(address, data).await
    }

    /// Write to flash, bounds-checked against the configured capacity.
    pub async fn write(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        self.bounds_check(address, data.len())?;
        self.write_raw(address, data).await
    }

    /// Erase a 4 KiB sector (opcode `0x20`).
    pub async fn erase(&mut self, address: u32) -> Result<(), Error> {
        self.erase_bounds(address)?;
        self.write_enable().await?;
        self.start(
            0x20,
            address,
            self.config.address_mode.bits(),
            0,
            0,
            0,
            Dir::Tx,
            Lines::Single,
        );
        self.wait_done().await?;
        self.wait_wip().await
    }

    /// Read from flash, blocking + bounds-checked.
    pub fn blocking_read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Error> {
        self.bounds_check(address, data.len())?;
        self.blocking_read_raw(address, data)
    }

    /// Write to flash, blocking + bounds-checked.
    pub fn blocking_write(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        self.bounds_check(address, data.len())?;
        self.blocking_write_raw(address, data)
    }

    /// Erase a 4 KiB sector, blocking version.
    pub fn blocking_erase(&mut self, address: u32) -> Result<(), Error> {
        self.erase_bounds(address)?;
        self.blocking_write_enable()?;
        self.start(
            0x20,
            address,
            self.config.address_mode.bits(),
            0,
            0,
            0,
            Dir::Tx,
            Lines::Single,
        );
        self.blocking_wait_done()?;
        self.blocking_wait_wip()
    }

    fn read_params(&self) -> (u8, u32, u32, Lines) {
        (
            self.config.read_opcode,
            self.config.address_mode.bits(),
            self.config.read_dummy_cycles as u32,
            self.config.lines,
        )
    }

    fn write_params(&self) -> (u8, u32, Lines) {
        (
            self.config.write_opcode,
            self.config.address_mode.bits(),
            self.config.lines,
        )
    }

    /// Bytes that can be programmed at `addr` without crossing a page boundary.
    fn page_chunk(&self, addr: u32, len: usize) -> usize {
        let page = self.config.write_page_size;
        let room = (page - (addr % page)) as usize;
        room.min(len)
    }

    fn bounds_check(&self, address: u32, len: usize) -> Result<(), Error> {
        if self.config.capacity == 0 {
            return Ok(());
        }
        let len: u32 = len.try_into().map_err(|_| Error::OutOfBounds)?;
        let end = address.checked_add(len).ok_or(Error::OutOfBounds)?;
        if end > self.config.capacity {
            return Err(Error::OutOfBounds);
        }
        Ok(())
    }

    fn erase_bounds(&self, address: u32) -> Result<(), Error> {
        if self.config.capacity != 0 && address >= self.config.capacity {
            return Err(Error::OutOfBounds);
        }
        Ok(())
    }
}

impl<'d> Drop for Sqspi<'d> {
    fn drop(&mut self) {
        self.regs.core().sqspienr().write_value(0);
        self.regs.enable().write_value(0);
        self.asb();

        // Hand the pins back from the FLPR to plain GPIO before stopping it
        self.csn.conf().write(|w| {
            w.set_dir(gpiovals::Dir::Output);
            w.set_input(gpiovals::Input::Disconnect);
        });
        for pin in [&self.sck, &self.io0, &self.io1, &self.io2, &self.io3] {
            pin.conf().write(|w| w.set_input(gpiovals::Input::Disconnect));
        }

        crate::vpr::stop_reset(self.vpr);
    }
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Error::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            _ => NorFlashErrorKind::Other,
        }
    }
}

impl<'d> ErrorType for Sqspi<'d> {
    type Error = Error;
}

impl<'d> ReadNorFlash for Sqspi<'d> {
    const READ_SIZE: usize = 4;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.config.capacity as usize
    }
}

impl<'d> NorFlash for Sqspi<'d> {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        for address in (from..to).step_by(<Self as NorFlash>::ERASE_SIZE) {
            self.blocking_erase(address)?;
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }
}

mod _eh1 {
    use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

    use super::*;

    impl<'d> AsyncReadNorFlash for Sqspi<'d> {
        const READ_SIZE: usize = 4;

        async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Self::Error> {
            self.read(address, data).await
        }

        fn capacity(&self) -> usize {
            self.config.capacity as usize
        }
    }

    impl<'d> AsyncNorFlash for Sqspi<'d> {
        const WRITE_SIZE: usize = <Self as NorFlash>::WRITE_SIZE;
        const ERASE_SIZE: usize = <Self as NorFlash>::ERASE_SIZE;

        async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
            self.write(offset, data).await
        }

        async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            for address in (from..to).step_by(<Self as AsyncNorFlash>::ERASE_SIZE) {
                self.erase(address).await?;
            }
            Ok(())
        }
    }
}

/// Peripheral static state.
pub(crate) struct State {
    waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

pub(crate) trait SealedInstance {
    fn state() -> &'static State;
}

/// sQSPI peripheral instance (the VPR coprocessor running the firmware).
///
/// Builds on [`vpr::Instance`](crate::vpr::Instance), which provides the VPR
/// registers and the completion interrupt.
#[allow(private_bounds)]
pub trait Instance: crate::vpr::Instance + SealedInstance {}

macro_rules! impl_sqspi {
    ($type:ident) => {
        impl crate::sqspi::SealedInstance for peripherals::$type {
            fn state() -> &'static crate::sqspi::State {
                static STATE: crate::sqspi::State = crate::sqspi::State::new();
                &STATE
            }
        }
        impl crate::sqspi::Instance for peripherals::$type {}
    };
}
