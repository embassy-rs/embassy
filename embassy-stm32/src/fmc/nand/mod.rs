//! Management of external NAND Flash through the STM32 FMC peripheral
//!
//! Commands and parameters are referenced to the Open NAND Flash Interface
//! (ONFI) Specification Revision 5.1 3 May 2022
//!
//! Addressing supports up to 64Gb / 4GByte (8-bit data) or 128Gb / 8Gbyte (16-bit data).

// Originally implemented by the `stm32-fmc` crate by Richard Meadows in 2019 under the
// MIT license, improved and rolled into Embassy by Kat Mitchell (northernpaws).

use core::array::TryFromSliceError;
use core::convert::TryInto;
use core::sync::atomic::{Ordering, fence};
use core::{cmp, fmt, ptr, str};

use embedded_hal_async::delay::DelayNs;

use crate::fmc::{self, Fmc};
pub use crate::pac::fmc::vals;

pub mod devices;

/// Specifies the data width for the Nand device.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NandDataWidth {
    /// 8-bit wide data.
    Bits8,
    // 16-bit wide data.
    Bits16,
}

impl Into<vals::Pwid> for NandDataWidth {
    fn into(self) -> vals::Pwid {
        match self {
            NandDataWidth::Bits8 => vals::Pwid::BITS8,
            NandDataWidth::Bits16 => vals::Pwid::BITS16,
        }
    }
}

/// Configuration for a NAND interface on the FMC.
///
/// These parameters need to be defined with the
/// values from the datasheet for your NAND device.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NandConfiguration {
    /// Data path width in bits
    pub data_width: NandDataWidth,

    /// Number of address bits used for the column address
    pub column_bits: u8,
}

/// NAND Timing parameters for the FMC.
///
/// These parameters need to be defined with the
/// values from the datasheet for your NAND device.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NandTiming {
    /// nCE setup time
    ///
    /// Typically defined as the symbol `tCS` in the datasheet
    pub nce_setup_time: u8,

    /// Data setup time tDS
    ///
    /// Typically defined as the symbol `tDS` in the datasheet
    pub data_setup_time: u8,

    /// ALE hold time
    pub ale_hold_time: u8,

    /// CLE hold time
    pub cle_hold_time: u8,

    /// ALE to nRE delay
    pub ale_to_nre_delay: u8,

    /// CLE to nRE delay
    pub cle_to_nre_delay: u8,

    /// nRE pulse width tRP
    ///
    /// Typically defined as the symbol `tRP` in the datasheet.
    pub nre_pulse_width_ns: u8,

    /// nWE pulse width tWP
    ///
    /// Typically defined as the symbol `tWP` in the datasheet.
    pub nwe_pulse_width_ns: u8,

    /// Read cycle time tRC
    ///
    /// Typically defined as the symbol `tRC` in the datasheet.
    pub read_cycle_time_ns: u8,

    /// Write cycle time tWC
    ///
    /// Typically defined as the symbol `tWC` in the datasheet.
    pub write_cycle_time_ns: u8,

    /// nWE high to busy tWB
    ///
    /// Typically defined as the symbol `tWB` in the datasheet.
    pub nwe_high_to_busy_ns: u8,
}

/// Provides the configuration and timing
/// parameters for a NAND chip.
pub trait NandChip {
    /// NAND controller configuration.
    const CONFIG: NandConfiguration;

    /// Timing parameters.
    const TIMING: NandTiming;
}

/// Driver for the FMC NAND controller.
#[allow(missing_debug_implementations)]
pub struct Nand<'a, 'd, T: fmc::Instance> {
    #[cfg(fmc_v1x3)]
    bank: super::FmcNandBank,

    /// Reference to the Fmc driver that was used to initialize the SDRAM.
    fmc: &'a mut Fmc<'d, T>,

    config: NandConfiguration,
    timing: NandTiming,
}

impl<'a, 'd, T: fmc::Instance> Nand<'a, 'd, T> {
    /// Creates a new NAND controller.
    fn new(
        fmc: &'a mut Fmc<'d, T>,
        #[cfg(fmc_v1x3)] bank: super::FmcNandBank,
        config: NandConfiguration,
        timing: NandTiming,
    ) -> Self {
        Self {
            fmc,
            #[cfg(fmc_v1x3)] bank,
            config,
            timing,
        }
    }

    /// Initialise the NAND driver.
    ///
    /// `delay` is used to wait 1µs after enabling the memory controller.
    pub async fn init<D>(&mut self, delay: &mut D) -> NandDevice
    where
        D: DelayNs,
    {
        // Ensure that the FMC clock is disabled
        // before adjusting the timing registers.
        self.fmc.memory_controller_disable();

        // Set the NAND registers with the configured values and timing parameters.
        self.set_features_timings();

        // Enable memory controller/FMC clock.
        self.fmc.memory_controller_enable();

        // Delay to let the FMC clock stabilize.
        delay.delay_us(1).await;

        // NOTE(unsafe): FMC controller has been initialized and enabled for this bank.
        unsafe {
            #[cfg(fmc_v1x3)]
            let ptr = self.fmc.nand_ptr(self.bank) as *mut u8;
            #[cfg(not(fmc_v1x3))]
            let ptr = FmcBank::Bank3.ptr() as *mut u8;

            NandDevice::init(ptr, self.config.column_bits as usize)
        }
    }

    /// Program the NAND registers with the configured features and timings.
    #[allow(non_snake_case)]
    fn set_features_timings<'f>(&mut self) {
        // calculate clock period, round down
        let fmc_source_ck_hz = self.fmc.source_clock_hz();
        let ker_clk_period_ns = 1_000_000_000u32 / fmc_source_ck_hz;

        let period_ns = ker_clk_period_ns as u8;
        let n_clock_periods = |time_ns: u8| {
            (time_ns + period_ns - 1) / period_ns // round up
        };

        // Extract the timing parameters into variables named after their expression
        // names in the datasheets to make reading/writing the calculations easier.
        let t_CS = self.timing.nce_setup_time;
        let t_DS = self.timing.data_setup_time;
        let t_ALH = self.timing.ale_hold_time;
        let t_CLH = self.timing.cle_hold_time;
        let t_AR = self.timing.ale_to_nre_delay;
        let t_CLR = self.timing.cle_to_nre_delay;
        let t_RP = self.timing.nre_pulse_width_ns;
        let t_WP = self.timing.nwe_pulse_width_ns;
        let t_RC = self.timing.read_cycle_time_ns;
        let t_WC = self.timing.write_cycle_time_ns;
        let t_WB = self.timing.nwe_high_to_busy_ns;

        // Now we need to convert the timings from their datasheet values
        // to the derrived values used by the FMC NAND registers.
        //
        // The FMC registers use values expressed in clock cycles, so the
        // timing in nanoseconds needs to be converted into cycle count for
        // the currently configured FMC clock speed.

        // Calculate setup time before RE/WE assertion.
        let setup_time = cmp::max(t_CS, cmp::max(t_AR, t_CLR));
        let set = cmp::max(n_clock_periods(setup_time - t_WP), 1) - 1;
        assert!(set < 255, "FMC ker clock too fast"); // 255 = reserved

        // Calculate RE/WE assertion time (minimum = 1).
        let wait = cmp::max(n_clock_periods(cmp::max(t_RP, t_WP)), 2) - 1;
        assert!(wait < 255, "FMC ker clock too fast"); // 255 = reserved

        // Calculate hold time after RE/WE deassertion (minimum = 1).
        let mut hold = cmp::max(n_clock_periods(cmp::max(t_ALH, t_CLH)), 1);
        // satisfy total cycle time
        let cycle_time = n_clock_periods(cmp::max(t_RC, t_WC));
        while wait + 1 + hold + set + 1 < cycle_time {
            hold += 1;
        }
        assert!(hold < 255, "FMC ker clock too fast"); // 255 = reserved

        // Calculate hold time to meet t_WB timing.
        let atthold = cmp::max(n_clock_periods(t_WB), 2) - 1;
        let atthold = cmp::max(atthold, hold);
        assert!(atthold < 255, "FMC ker clock too fast"); // 255 = reserved

        // Calculate CS assertion to data setup.
        let hiz = cmp::max(n_clock_periods(t_CS + t_WP - t_DS), 0);
        assert!(hiz < 255, "FMC ker clock too fast"); // 255 = reserved

        // Calculate ALE low to RE assert.
        let ale_to_nre = n_clock_periods(t_AR);
        let tar = cmp::max(ale_to_nre - set - 2, 0);
        assert!(tar < 16, "FMC ker clock too fast");

        // CLE low to RE assert
        let clr_to_nre = n_clock_periods(t_CLR);
        let tclr = cmp::max(clr_to_nre - set - 2, 0);
        assert!(tclr < 16, "FMC ker clock too fast");

        #[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1))]
        let regs = T::regs();
        #[cfg(fmc_v4)]
        let regs = T::regs().nand();

        #[cfg(not(fmc_v1x3))]
        let pcr = regs.pcr();
        #[cfg(not(fmc_v1x3))]
        let pmem = regs.pmem();
        #[cfg(not(fmc_v1x3))]
        let patt = regs.patt();

        // fmc_v1x3 supports multiple CompactFlash and NAND
        // banks, so the register structure is indexed.
        #[cfg(fmc_v1x3)]
        let pcr = regs.pcr(self.bank.into());
        #[cfg(fmc_v1x3)]
        let pmem = regs.pmem(self.bank.into());
        #[cfg(fmc_v1x3)]
        let patt = regs.patt(self.bank.into());

        // Set the NAND control register values.
        pcr.modify(|reg| {
            reg.set_tar(tar);
            reg.set_tclr(tclr);
            reg.set_eccps(vals::Eccps::BYTES512); // 0b1: 512 bytes
            reg.set_eccen(false); // 0b0: ECC computation disabled
            #[cfg(fmc_v4)]
            reg.set_ptyp(vals::Ptyp::NAND); // 0b1: NAND Flash
            reg.set_pwaiten(true) // 0b1: Wait feature enabled
        });

        // Set the memory timing parameters.
        pmem.modify(|reg| {
            reg.set_memhiz(hiz);
            reg.set_memhold(hold);
            reg.set_memwait(wait);
            reg.set_memset(set)
        });

        // Set the attribute memory timing parameters.
        patt.modify(|reg| {
            reg.set_atthiz(hiz);
            reg.set_atthold(atthold);
            reg.set_attwait(wait);
            reg.set_attset(set)
        });

        // Enable the NAND controller.
        pcr.modify(|reg| reg.set_pbken(true));
    }
}

/// NAND Commands defined in ONFI Specification 5.1.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(unused)]
enum Command {
    /// 0xFF Reset.
    ///
    /// See ONFI Section 5.3.
    Reset = 0xFF,
    /// 0x90 Read ID
    ///
    /// See ONFI Section 5.6.
    ReadID = 0x90,
    /// 0xEC Read Parameter Page
    ///
    /// See ONFI Section 5.7.
    ReadParameterPage = 0xEC,
    /// 0xED Read Unique ID
    ///
    /// See ONFI Section 5.8.
    ReadUniqueID = 0xED,
    /// Block Erase
    ///
    /// See ONFI Section 5.9.
    BlockErase = 0x60,
    /// 0x70 Read Status
    ///
    /// See ONFI Section 5.10.
    ReadStatus = 0x70,
}

/// Status returned from the Read Status (0x70) command.
///
/// See ONFI Section 5.10.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    /// Status Register indicated Pass
    Success(u8),

    /// Status Register indicates Fail
    Fail(u8),
}

impl Status {
    fn from_register(reg: u8) -> Self {
        match reg & 1 {
            1 => Self::Fail(reg),
            _ => Self::Success(reg),
        }
    }
}

/// Identifier returned from  ReadID(0x90) command.
///
/// See ONFI Section 5.6.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ID {
    /// JEDEC ID of the NAND device manufacturer.
    manufacturer_jedec: u8,

    /// JEDEC ID of the NAND device.
    device_jedec: u8,

    /// Internal chip count of the NAND.
    internal_chip_count: usize,

    /// Page sized used by the NAND device.
    page_size: usize,
}

/// Parameter Page returned from ReadParameter(0xEC) command.
///
/// See ONFI Section 5.7.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq)]
pub struct ParameterPage {
    signature: [u8; 4],
    // ONFI revision used by the NAND device.
    onfi_revision: u16,
    /// Name of the manufacturer of the NAND device.
    manufacturer: [u8; 12],
    /// Model of the NAND device.
    model: [u8; 20],
    /// Date code of the NAND device.
    date_code: u16,
    data_bytes_per_page: u32,
    spare_bytes_per_page: u16,
    pages_per_block: u32,
    blocks_per_lun: u32,
    lun_count: u8,
    /// Count of ECC bits used by the NAND device.
    ecc_bits: u8,
}

impl ParameterPage {
    /// Returns a string with the decoded manufacturer name of the device.
    pub fn manufacturer(&self) -> &str {
        str::from_utf8(&self.manufacturer).unwrap_or("<ERR>")
    }

    /// Returns a string of the decoded model number of the device.
    pub fn model(&self) -> &str {
        str::from_utf8(&self.model).unwrap_or("<ERR>")
    }
}

impl fmt::Debug for ParameterPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ONFI Parameter Page")
            .field("ONFI Revision", &self.onfi_revision)
            .field("Manufacturer", &self.manufacturer())
            .field("Model", &self.model())
            .field("Date Code", &self.date_code)
            .field("Data bytes per Page", &self.data_bytes_per_page)
            .field("Spare bytes per Page", &self.spare_bytes_per_page)
            .field("Pages per Block", &self.pages_per_block)
            .field("Blocks per LUN", &self.blocks_per_lun)
            .field("LUN Count", &self.lun_count)
            .field("ECC Bits Correctability", &self.ecc_bits)
            .finish()
    }
}

/// Wraps of the fields needed to communicate
/// with a NAND device attached to the FMC.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_copy_implementations)]
pub struct NandDevice {
    common_command: *mut u8,
    common_address: *mut u8,
    attribute_command: *mut u8,
    common_data: *mut u8,

    /// Number of address bits C that are used for the column address. The
    /// number of data bytes per page is typically 2^C
    column_bits: Option<usize>,
}

/// Writes to the specified point address and ensures
/// that the write is commited before continuing.
unsafe fn write_volatile_sync<T>(dest: *mut T, src: T) {
    ptr::write_volatile(dest, src);

    // Ensure that the write is committed before continuing. In the default
    // ARMv7-M address map the space 0x8000_0000-0x9FFF_FFFF is Normal Memory
    // with write-though cache attribute.
    fence(Ordering::SeqCst);
}

impl NandDevice {
    /// Create a `NandDevice` from a bank pointer
    ///
    /// # Safety
    ///
    /// The FMC controller must have been initialized as NAND controller and
    /// enabled for this bank, with the correct pin settings. The bank pointer
    /// must be a singleton.
    pub(crate) unsafe fn init(ptr: *mut u8, column_bits: usize) -> NandDevice {
        let mut nand = NandDevice {
            common_command: ptr.add(0x1_0000),
            common_address: ptr.add(0x2_0000),
            attribute_command: ptr.add(0x801_0000),
            common_data: ptr,
            column_bits: Some(column_bits),
        };

        // Reset Command. May be specifically required by some devices and there
        // seems to be no disadvantage of sending it always
        nand.reset();

        nand
    }

    /// 0xFF Reset: ONFI Section 5.3
    pub fn reset(&mut self) {
        unsafe {
            write_volatile_sync(self.common_command, 0xFF);
        }
    }

    /// Generic Command
    fn command(&mut self, cmd: Command, address: u8, buffer: &mut [u8]) {
        unsafe {
            write_volatile_sync(self.common_command, cmd as u8);
            write_volatile_sync(self.common_address, address);
            for x in buffer {
                *x = ptr::read_volatile(self.common_data);
            }
        }
    }

    /// Generic Address
    ///
    /// column_bits must be set first!
    fn address(&mut self, address: usize, spare: bool) {
        let column_bits = self
            .column_bits
            .expect("Number of column bits must be configured first");
        let column = (address & ((1 << column_bits) - 1)) + if spare { 1 << column_bits } else { 0 };
        let row = address >> column_bits;

        let mut addr_cycles = [0u8; 5];

        // Assuming 5-cycle address
        addr_cycles[0] = (column & 0xFF) as u8;
        addr_cycles[1] = ((column >> 8) & 0xFF) as u8;
        addr_cycles[2] = (row & 0xFF) as u8;
        addr_cycles[3] = ((row >> 8) & 0xFF) as u8;
        addr_cycles[4] = ((row >> 16) & 0xFF) as u8;

        for a in addr_cycles {
            unsafe {
                write_volatile_sync(self.common_address, a);
            }
        }
    }

    /// Executes the ReadID (0x90) command from ONFI Section 5.6.
    pub fn read_id(&mut self) -> ID {
        let mut id = [0u8; 5];
        self.command(Command::ReadID, 0, &mut id);

        let internal_chip_count = match id[2] & 3 {
            1 => 2,
            2 => 4,
            3 => 8,
            _ => 1,
        };
        let page_size = match id[3] & 3 {
            1 => 2048,
            2 => 4096,
            _ => 0,
        };
        ID {
            manufacturer_jedec: id[0],
            device_jedec: id[1],
            internal_chip_count,
            page_size,
        }
    }

    /// Executes the ReadParameterPage(0xEC) command from ONFI Section 5.7.
    pub fn read_parameter_page(&mut self) -> Result<ParameterPage, TryFromSliceError> {
        let mut page = [0u8; 115];
        self.command(Command::ReadParameterPage, 0, &mut page);

        Ok(ParameterPage {
            signature: page[0..4].try_into()?,
            onfi_revision: u16::from_le_bytes(page[4..6].try_into()?),
            manufacturer: page[32..44].try_into()?,
            model: page[44..64].try_into()?,
            date_code: u16::from_le_bytes(page[65..67].try_into()?),
            data_bytes_per_page: u32::from_le_bytes(page[80..84].try_into()?),
            spare_bytes_per_page: u16::from_le_bytes(page[84..86].try_into()?),
            pages_per_block: u32::from_le_bytes(page[92..96].try_into()?),
            blocks_per_lun: u32::from_le_bytes(page[96..100].try_into()?),
            lun_count: page[100],
            ecc_bits: page[112],
        })
    }

    /// Executes the ReadUniqueID(0xED) command from ONFI Section 5.8.
    pub fn read_unique_id(&mut self) -> u128 {
        let mut unique = [0u8; 16];
        self.command(Command::ReadUniqueID, 0, &mut unique);
        u128::from_le_bytes(unique)
    }

    /// Executes the BlockErase(0x60) command from ONFI Section 5.9.
    pub fn block_erase(&mut self, address: usize) -> Status {
        unsafe {
            write_volatile_sync(self.common_command, 0x60); // auto block erase setup
        }

        let column_bits = self
            .column_bits
            .expect("Number of column bits must be configured first!");
        let row = address >> column_bits;
        unsafe {
            // write block address
            write_volatile_sync(self.common_address, (row & 0xFF) as u8);
            write_volatile_sync(self.common_address, ((row >> 8) & 0xFF) as u8);
            write_volatile_sync(self.common_address, ((row >> 16) & 0xFF) as u8);

            // erase command
            write_volatile_sync(self.attribute_command, 0xD0); // t_WB
            write_volatile_sync(self.common_command, Command::ReadStatus as u8);
            let status_register = ptr::read_volatile(self.common_data);
            Status::from_register(status_register)
        }
    }

    /// Executes the PageRead command from ONFI Section 5.14.
    ///
    /// This method starts a Page Read operation but does not include the data
    /// phase. This method is useful when DMA is used for the data phase.
    ///
    /// For a method that completes the entire transaction see
    /// [`page_read`](Self::page_read).
    pub fn start_page_read(&mut self, address: usize, spare: bool) {
        unsafe {
            write_volatile_sync(self.common_command, 0x00);
            self.address(address, spare);
            write_volatile_sync(self.attribute_command, 0x30); // t_WB
        }
    }

    /// Executes a `PageRead` operation from the specified address. Data is
    /// copied to the slice `page`. The length of `page` determines the read
    /// length. The read length should not exceed the number of bytes between
    /// the specified address and the end of the page. Reading beyond the end of
    /// the page results in indeterminate values being returned.
    ///
    /// If `spare` is true, then the read occours from the spare area. The
    /// address offset from the start of the page plus the slice length should
    /// not exceed the spare area size.
    ///
    /// See ONFI Section 5.14.
    pub fn page_read(&mut self, address: usize, spare: bool, page: &mut [u8]) {
        self.start_page_read(address, spare);
        for x in page {
            unsafe {
                *x = ptr::read_volatile(self.common_data);
            }
        }
    }

    /// Executes a `PageProgram`` to the specified address and waits for it to
    /// complete. The length of `page` determines the write length. The write
    /// length should not exceed the number of bytes between the specified
    /// address and the end of the page. Writing beyond this length is
    /// undefined.
    ///
    /// See ONFI Section 5.16.
    pub fn page_program(&mut self, address: usize, spare: bool, page: &[u8]) -> Status {
        unsafe {
            write_volatile_sync(self.common_command, 0x80); // data input
            self.address(address, spare);
            for x in page {
                write_volatile_sync(self.common_data, *x); // write page
            }
            write_volatile_sync(self.attribute_command, 0x10); // program command, t_WB
            let mut status_register;
            while {
                write_volatile_sync(self.common_command, Command::ReadStatus as u8);
                status_register = ptr::read_volatile(self.common_data);

                status_register & 0x20 == 0 // program in progress
            } {}

            Status::from_register(status_register)
        }
    }
}

/// Methods to allow users to implement their own commands using `unsafe`.
impl NandDevice {
    /// Return a Raw Pointer to the common command space. This memory-mapped
    /// address is used to write command phase of NAND device transactions.
    ///
    /// It is recommended to use
    /// [`ptr::write_volatile`](https://doc.rust-lang.org/std/ptr/fn.write_volatile.html)
    /// to write to this pointer. Depending on the memory map in use, you may
    /// need to ensure the write is committed by using
    /// [`core::atomic::sync::fence`](https://doc.rust-lang.org/core/sync/atomic/fn.fence.html).
    pub fn common_command(&mut self) -> *mut u8 {
        self.common_command
    }

    /// Return a Raw Pointer to the attribute command space. This memory-mapped
    /// address is used to write command phase of NAND device transactions.
    ///
    /// It is recommended to use
    /// [`ptr::write_volatile`](https://doc.rust-lang.org/std/ptr/fn.write_volatile.html)
    /// to write to this pointer. Depending on the memory map in use, you may
    /// need to ensure the write is committed by using
    /// [`core::atomic::sync::fence`](https://doc.rust-lang.org/core/sync/atomic/fn.fence.html).
    pub fn attribute_command(&mut self) -> *mut u8 {
        self.attribute_command
    }

    /// Return a Raw Pointer to the common address space. This memory-mapped
    /// address is used to write the address phase of NAND device transactions.
    ///
    /// It is recommended to use
    /// [`ptr::write_volatile`](https://doc.rust-lang.org/std/ptr/fn.write_volatile.html)
    /// to write to this pointer. Depending on the memory map in use, you may
    /// need to ensure the write is committed by using
    /// [`core::atomic::sync::fence`](https://doc.rust-lang.org/core/sync/atomic/fn.fence.html).
    pub fn common_address(&mut self) -> *mut u8 {
        self.common_address
    }

    /// Return a Raw Pointer to the common data space. This memory-mapped
    /// address is used to write or read the data phase of NAND device
    /// transactions.
    ///
    /// It is recommended to use
    /// [`ptr::write_volatile`](https://doc.rust-lang.org/std/ptr/fn.write_volatile.html)
    /// to write to this pointer. Depending on the memory map in use, you may
    /// need to ensure the write is committed by using
    /// [`core::atomic::sync::fence`](https://doc.rust-lang.org/core/sync/atomic/fn.fence.html).
    pub fn common_data(&mut self) -> *mut u8 {
        self.common_data
    }
}
