//! Functions and data from the RPI Bootrom.
//!
//! From [Section 5.4](https://rptl.io/rp2350-datasheet#section_bootrom) of the
//! RP2350 datasheet:
//!
//! > Whilst some ROM space is dedicated to the implementation of the boot
//! > sequence and USB/UART boot interfaces, the bootrom also contains public
//! > functions that provide useful RP2350 functionality that may be useful for
//! > any code or runtime running on the device

// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp235x-hal/src/rom_data.rs

/// A bootrom function table code.
pub type RomFnTableCode = [u8; 2];

/// This function searches for the tag which matches the mask.
type RomTableLookupFn = unsafe extern "C" fn(code: u32, mask: u32) -> usize;

/// Pointer to the value lookup function supplied by the ROM.
///
/// This address is described at `5.5.1. Locating the API Functions`
#[cfg(all(target_arch = "arm", target_os = "none"))]
const ROM_TABLE_LOOKUP_A2: *const u16 = 0x0000_0016 as _;

/// Pointer to the value lookup function supplied by the ROM.
///
/// This address is described at `5.5.1. Locating the API Functions`
#[cfg(all(target_arch = "arm", target_os = "none"))]
const ROM_TABLE_LOOKUP_A1: *const u32 = 0x0000_0018 as _;

/// Pointer to the data lookup function supplied by the ROM.
///
/// On Arm, the same function is used to look up code and data.
#[cfg(all(target_arch = "arm", target_os = "none"))]
const ROM_DATA_LOOKUP_A2: *const u16 = ROM_TABLE_LOOKUP_A2;

/// Pointer to the data lookup function supplied by the ROM.
///
/// On Arm, the same function is used to look up code and data.
#[cfg(all(target_arch = "arm", target_os = "none"))]
const ROM_DATA_LOOKUP_A1: *const u32 = ROM_TABLE_LOOKUP_A1;

/// Pointer to the value lookup function supplied by the ROM.
///
/// This address is described at `5.5.1. Locating the API Functions`
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
const ROM_TABLE_LOOKUP_A2: *const u16 = 0x0000_7DFA as _;

/// Pointer to the value lookup function supplied by the ROM.
///
/// This address is described at `5.5.1. Locating the API Functions`
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
const ROM_TABLE_LOOKUP_A1: *const u32 = 0x0000_7DF8 as _;

/// Pointer to the data lookup function supplied by the ROM.
///
/// On RISC-V, a different function is used to look up data.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
const ROM_DATA_LOOKUP_A2: *const u16 = 0x0000_7DF8 as _;

/// Pointer to the data lookup function supplied by the ROM.
///
/// On RISC-V, a different function is used to look up data.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
const ROM_DATA_LOOKUP_A1: *const u32 = 0x0000_7DF4 as _;

/// Address of the version number of the ROM.
const VERSION_NUMBER: *const u8 = 0x0000_0013 as _;

#[allow(unused)]
mod rt_flags {
    pub const FUNC_RISCV: u32 = 0x0001;
    pub const FUNC_RISCV_FAR: u32 = 0x0003;
    pub const FUNC_ARM_SEC: u32 = 0x0004;
    // reserved for 32-bit pointer: 0x0008
    pub const FUNC_ARM_NONSEC: u32 = 0x0010;
    // reserved for 32-bit pointer: 0x0020
    pub const DATA: u32 = 0x0040;
    // reserved for 32-bit pointer: 0x0080
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    pub const FUNC_ARM_SEC_RISCV: u32 = FUNC_ARM_SEC;
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    pub const FUNC_ARM_SEC_RISCV: u32 = FUNC_RISCV;
}

/// Retrieve rom content from a table using a code.
pub fn rom_table_lookup(tag: RomFnTableCode, mask: u32) -> usize {
    let tag = u16::from_le_bytes(tag) as u32;
    unsafe {
        let lookup_func = if rom_version_number() == 1 {
            ROM_TABLE_LOOKUP_A1.read() as usize
        } else {
            ROM_TABLE_LOOKUP_A2.read() as usize
        };
        let lookup_func: RomTableLookupFn = core::mem::transmute(lookup_func);
        lookup_func(tag, mask)
    }
}

/// Retrieve rom data content from a table using a code.
pub fn rom_data_lookup(tag: RomFnTableCode, mask: u32) -> usize {
    let tag = u16::from_le_bytes(tag) as u32;
    unsafe {
        let lookup_func = if rom_version_number() == 1 {
            ROM_DATA_LOOKUP_A1.read() as usize
        } else {
            ROM_DATA_LOOKUP_A2.read() as usize
        };
        let lookup_func: RomTableLookupFn = core::mem::transmute(lookup_func);
        lookup_func(tag, mask)
    }
}

macro_rules! declare_rom_function {
    (
        $(#[$outer:meta])*
        fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty
        $lookup:block
    ) => {
        #[doc = r"Additional access for the `"]
        #[doc = stringify!($name)]
        #[doc = r"` ROM function."]
        pub mod $name {
            /// Retrieve a function pointer.
            #[cfg(not(feature = "rom-func-cache"))]
            pub fn ptr() -> extern "C" fn( $($argname: $ty),* ) -> $ret {
                let p: usize = $lookup;
                unsafe {
                    let func : extern "C" fn( $($argname: $ty),* ) -> $ret = core::mem::transmute(p);
                    func
                }
            }

            /// Retrieve a function pointer.
            #[cfg(feature = "rom-func-cache")]
            pub fn ptr() -> extern "C" fn( $($argname: $ty),* ) -> $ret {
                use core::sync::atomic::{AtomicU16, Ordering};

                // All pointers in the ROM fit in 16 bits, so we don't need a
                // full width word to store the cached value.
                static CACHED_PTR: AtomicU16 = AtomicU16::new(0);
                // This is safe because the lookup will always resolve
                // to the same value.  So even if an interrupt or another
                // core starts at the same time, it just repeats some
                // work and eventually writes back the correct value.
                let p: usize = match CACHED_PTR.load(Ordering::Relaxed) {
                    0 => {
                        let raw: usize = $lookup;
                        CACHED_PTR.store(raw as u16, Ordering::Relaxed);
                        raw
                    },
                    val => val as usize,
                };
                unsafe {
                    let func : extern "C" fn( $($argname: $ty),* ) -> $ret = core::mem::transmute(p);
                    func
                }
            }
        }

        $(#[$outer])*
        pub extern "C" fn $name( $($argname: $ty),* ) -> $ret {
            $name::ptr()($($argname),*)
        }
    };

    (
        $(#[$outer:meta])*
        unsafe fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty
        $lookup:block
    ) => {
        #[doc = r"Additional access for the `"]
        #[doc = stringify!($name)]
        #[doc = r"` ROM function."]
        pub mod $name {
            /// Retrieve a function pointer.
            #[cfg(not(feature = "rom-func-cache"))]
            pub fn ptr() -> unsafe extern "C" fn( $($argname: $ty),* ) -> $ret {
                let p: usize = $lookup;
                unsafe {
                    let func : unsafe extern "C" fn( $($argname: $ty),* ) -> $ret = core::mem::transmute(p);
                    func
                }
            }

            /// Retrieve a function pointer.
            #[cfg(feature = "rom-func-cache")]
            pub fn ptr() -> unsafe extern "C" fn( $($argname: $ty),* ) -> $ret {
                use core::sync::atomic::{AtomicU16, Ordering};

                // All pointers in the ROM fit in 16 bits, so we don't need a
                // full width word to store the cached value.
                static CACHED_PTR: AtomicU16 = AtomicU16::new(0);
                // This is safe because the lookup will always resolve
                // to the same value.  So even if an interrupt or another
                // core starts at the same time, it just repeats some
                // work and eventually writes back the correct value.
                let p: usize = match CACHED_PTR.load(Ordering::Relaxed) {
                    0 => {
                        let raw: usize = $lookup;
                        CACHED_PTR.store(raw as u16, Ordering::Relaxed);
                        raw
                    },
                    val => val as usize,
                };
                unsafe {
                    let func : unsafe extern "C" fn( $($argname: $ty),* ) -> $ret = core::mem::transmute(p);
                    func
                }
            }
        }

        $(#[$outer])*
        /// # Safety
        ///
        /// This is a low-level C function. It may be difficult to call safely from
        /// Rust. If in doubt, check the rp235x datasheet for details and do your own
        /// safety evaluation.
        pub unsafe extern "C" fn $name( $($argname: $ty),* ) -> $ret {
            $name::ptr()($($argname),*)
        }
    };
}

// **************** 5.5.7 Low-level Flash Commands ****************

declare_rom_function! {
    /// Restore all QSPI pad controls to their default state, and connect the
    /// QMI peripheral to the QSPI pads.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn connect_internal_flash() -> () {
        crate::rom_data::rom_table_lookup(*b"IF", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Initialise the QMI for serial operations (direct mode)
    ///
    /// Also initialise a basic XIP mode, where the QMI will perform 03h serial
    /// read commands at low speed (CLKDIV=12) in response to XIP reads.
    ///
    /// Then, issue a sequence to the QSPI device on chip select 0, designed to
    /// return it from continuous read mode ("XIP mode") and/or QPI mode to a
    /// state where it will accept serial commands. This is necessary after
    /// system reset to restore the QSPI device to a known state, because
    /// resetting RP2350 does not reset attached QSPI devices. It is also
    /// necessary when user code, having already performed some
    /// continuous-read-mode or QPI-mode accesses, wishes to return the QSPI
    /// device to a state where it will accept the serial erase and programming
    /// commands issued by the bootrom’s flash access functions.
    ///
    /// If a GPIO for the secondary chip select is configured via FLASH_DEVINFO,
    /// then the XIP exit sequence is also issued to chip select 1.
    ///
    /// The QSPI device should be accessible for XIP reads after calling this
    /// function; the name flash_exit_xip refers to returning the QSPI device
    /// from its XIP state to a serial command state.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_exit_xip() -> () {
        crate::rom_data::rom_table_lookup(*b"EX", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Erase count bytes, starting at addr (offset from start of flash).
    ///
    /// Optionally, pass a block erase command e.g. D8h block erase, and the
    /// size of the block erased by this command — this function will use the
    /// larger block erase where possible, for much higher erase speed. addr
    /// must be aligned to a 4096-byte sector, and count must be a multiple of
    /// 4096 bytes.
    ///
    /// This is a low-level flash API, and no validation of the arguments is
    /// performed. See flash_op() for a higher-level API which checks alignment,
    /// flash bounds and partition permissions, and can transparently apply a
    /// runtime-to-storage address translation.
    ///
    /// The QSPI device must be in a serial command state before calling this
    /// API, which can be achieved by calling connect_internal_flash() followed
    /// by flash_exit_xip(). After the erase, the flash cache should be flushed
    /// via flash_flush_cache() to ensure the modified flash data is visible to
    /// cached XIP accesses.
    ///
    /// Finally, the original XIP mode should be restored by copying the saved
    /// XIP setup function from bootram into SRAM, and executing it: the bootrom
    /// provides a default function which restores the flash mode/clkdiv
    /// discovered during flash scanning, and user programs can override this
    /// with their own XIP setup function.
    ///
    /// For the duration of the erase operation, QMI is in direct mode (Section
    /// 12.14.5) and attempting to access XIP from DMA, the debugger or the
    /// other core will return a bus fault. XIP becomes accessible again once
    /// the function returns.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_range_erase(addr: u32, count: usize, block_size: u32, block_cmd: u8) -> () {
        crate::rom_data::rom_table_lookup(*b"RE", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Program data to a range of flash storage addresses starting at addr
    /// (offset from the start of flash) and count bytes in size.
    ///
    /// `addr` must be aligned to a 256-byte boundary, and count must be a
    /// multiple of 256.
    ///
    /// This is a low-level flash API, and no validation of the arguments is
    /// performed. See flash_op() for a higher-level API which checks alignment,
    /// flash bounds and partition permissions, and can transparently apply a
    /// runtime-to-storage address translation.
    ///
    /// The QSPI device must be in a serial command state before calling this
    /// API — see notes on flash_range_erase().
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_range_program(addr: u32, data: *const u8, count: usize) -> () {
        crate::rom_data::rom_table_lookup(*b"RP", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Flush the entire XIP cache, by issuing an invalidate by set/way
    /// maintenance operation to every cache line (Section 4.4.1).
    ///
    /// This ensures that flash program/erase operations are visible to
    /// subsequent cached XIP reads.
    ///
    /// Note that this unpins pinned cache lines, which may interfere with
    /// cache-as-SRAM use of the XIP cache.
    ///
    /// No other operations are performed.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_flush_cache() -> () {
        crate::rom_data::rom_table_lookup(*b"FC", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Configure the QMI to generate a standard 03h serial read command, with
    /// 24 address bits, upon each XIP access.
    ///
    /// This is a slow XIP configuration, but is widely supported. CLKDIV is set
    /// to 12. The debugger may call this function to ensure that flash is
    /// readable following a program/erase operation.
    ///
    /// Note that the same setup is performed by flash_exit_xip(), and the
    /// RP2350 flash program/erase functions do not leave XIP in an inaccessible
    /// state, so calls to this function are largely redundant. It is provided
    /// for compatibility with RP2040.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_enter_cmd_xip() -> () {
        crate::rom_data::rom_table_lookup(*b"CX", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Configure QMI for one of a small menu of XIP read modes supported by the
    /// bootrom. This mode is configured for both memory windows (both chip
    /// selects), and the clock divisor is also applied to direct mode.
    ///
    /// The available modes are:
    ///
    /// * 0: `03h` serial read: serial address, serial data, no wait cycles
    /// * 1: `0Bh` serial read: serial address, serial data, 8 wait cycles
    /// * 2: `BBh` dual-IO read: dual address, dual data, 4 wait cycles
    ///   (including MODE bits, which are driven to 0)
    /// * 3: `EBh` quad-IO read: quad address, quad data, 6 wait cycles
    ///   (including MODE bits, which are driven to 0)
    ///
    /// The XIP write command/format are not configured by this function. When
    /// booting from flash, the bootrom tries each of these modes in turn, from
    /// 3 down to 0. The first mode that is found to work is remembered, and a
    /// default XIP setup function is written into bootram that calls this
    /// function (flash_select_xip_read_mode) with the parameters discovered
    /// during flash scanning. This can be called at any time to restore the
    /// flash parameters discovered during flash boot.
    ///
    /// All XIP modes configured by the bootrom have an 8-bit serial command
    /// prefix, so that the flash can remain in a serial command state, meaning
    /// XIP accesses can be mixed more freely with program/erase serial
    /// operations. This has a performance penalty, so users can perform their
    /// own flash setup after flash boot using continuous read mode or QPI mode
    /// to avoid or alleviate the command prefix cost.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_select_xip_read_mode(bootrom_xip_mode: u8, clkdiv: u8) -> () {
        crate::rom_data::rom_table_lookup(*b"XM", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Restore the QMI address translation registers, ATRANS0 through ATRANS7,
    /// to their reset state. This makes the runtime- to-storage address map an
    /// identity map, i.e. the mapped and unmapped address are equal, and the
    /// entire space is fully mapped.
    ///
    /// See [Section 12.14.4](https://rptl.io/rp2350-datasheet#section_bootrom) of the RP2350
    /// datasheet.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_reset_address_trans() -> () {
        crate::rom_data::rom_table_lookup(*b"RA", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

// **************** High-level Flash Commands ****************

declare_rom_function! {
    /// Applies the address translation currently configured by QMI address
    /// translation registers, ATRANS0 through ATRANS7.
    ///
    /// See [Section 12.14.4](https://rptl.io/rp2350-datasheet#section_bootrom) of the RP2350
    /// datasheet.
    ///
    /// Translating an address outside of the XIP runtime address window, or
    /// beyond the bounds of an ATRANSx_SIZE field, returns
    /// BOOTROM_ERROR_INVALID_ADDRESS, which is not a valid flash storage
    /// address. Otherwise, return the storage address which QMI would access
    /// when presented with the runtime address addr. This is effectively a
    /// virtual-to-physical address translation for QMI.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_runtime_to_storage_addr(addr: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"FA", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Non-secure version of [flash_runtime_to_storage_addr()]
    ///
    /// Supported architectures: ARM-NS
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe fn flash_runtime_to_storage_addr_ns(addr: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"FA", crate::rom_data::inner::rt_flags::FUNC_ARM_NONSEC)
    }
}

declare_rom_function! {
    /// Perform a flash read, erase, or program operation.
    ///
    /// Erase operations must be sector-aligned (4096 bytes) and sector-
    /// multiple-sized, and program operations must be page-aligned (256 bytes)
    /// and page-multiple-sized; misaligned erase and program operations will
    /// return BOOTROM_ERROR_BAD_ALIGNMENT. The operation — erase, read, program
    /// — is selected by the CFLASH_OP_BITS bitfield of the flags argument.
    ///
    /// See datasheet section 5.5.8.2 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn flash_op(flags: u32, addr: u32, size_bytes: u32, buffer: *mut u8) -> i32 {
        crate::rom_data::rom_table_lookup(*b"FO", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Non-secure version of [flash_op()]
    ///
    /// Supported architectures: ARM-NS
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe fn flash_op_ns(flags: u32, addr: u32, size_bytes: u32, buffer: *mut u8) -> i32 {
        crate::rom_data::rom_table_lookup(*b"FO", crate::rom_data::inner::rt_flags::FUNC_ARM_NONSEC)
    }
}

// **************** Security Related Functions ****************

declare_rom_function! {
    /// Allow or disallow the specific NS API (note all NS APIs default to
    /// disabled).
    ///
    /// See datasheet section 5.5.9.1 for more details.
    ///
    /// Supported architectures: ARM-S
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe fn set_ns_api_permission(ns_api_num: u32, allowed: u8) -> i32 {
        crate::rom_data::rom_table_lookup(*b"SP", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC)
    }
}

declare_rom_function! {
    /// Utility method that can be used by secure ARM code to validate a buffer
    /// passed to it from Non-secure code.
    ///
    /// See datasheet section 5.5.9.2 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn validate_ns_buffer() -> () {
        crate::rom_data::rom_table_lookup(*b"VB", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

// **************** Miscellaneous Functions ****************

declare_rom_function! {
    /// Resets the RP2350 and uses the watchdog facility to restart.
    ///
    /// See datasheet section 5.5.10.1 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    fn reboot(flags: u32, delay_ms: u32, p0: u32, p1: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"RB", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Non-secure version of [reboot()]
    ///
    /// Supported architectures: ARM-NS
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    fn reboot_ns(flags: u32, delay_ms: u32, p0: u32, p1: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"RB", crate::rom_data::inner::rt_flags::FUNC_ARM_NONSEC)
    }
}

declare_rom_function! {
    /// Resets internal bootrom state.
    ///
    /// See datasheet section 5.5.10.2 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn bootrom_state_reset(flags: u32) -> () {
        crate::rom_data::rom_table_lookup(*b"SR", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Set a boot ROM callback.
    ///
    /// The only supported callback_number is 0 which sets the callback used for
    /// the secure_call API.
    ///
    /// See datasheet section 5.5.10.3 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn set_rom_callback(callback_number: i32, callback_fn: *const ()) -> i32 {
        crate::rom_data::rom_table_lookup(*b"RC", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

// **************** System Information Functions ****************

declare_rom_function! {
    /// Fills a buffer with various system information.
    ///
    /// Note that this API is also used to return information over the PICOBOOT
    /// interface.
    ///
    /// See datasheet section 5.5.11.1 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn get_sys_info(out_buffer: *mut u32, out_buffer_word_size: usize, flags: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"GS", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Non-secure version of [get_sys_info()]
    ///
    /// Supported architectures: ARM-NS
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe fn get_sys_info_ns(out_buffer: *mut u32, out_buffer_word_size: usize, flags: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"GS", crate::rom_data::inner::rt_flags::FUNC_ARM_NONSEC)
    }
}

declare_rom_function! {
    /// Fills a buffer with information from the partition table.
    ///
    /// Note that this API is also used to return information over the PICOBOOT
    /// interface.
    ///
    /// See datasheet section 5.5.11.2 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn get_partition_table_info(out_buffer: *mut u32, out_buffer_word_size: usize, flags_and_partition: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"GP", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Non-secure version of [get_partition_table_info()]
    ///
    /// Supported architectures: ARM-NS
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe fn get_partition_table_info_ns(out_buffer: *mut u32, out_buffer_word_size: usize, flags_and_partition: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"GP", crate::rom_data::inner::rt_flags::FUNC_ARM_NONSEC)
    }
}

declare_rom_function! {
    /// Loads the current partition table from flash, if present.
    ///
    /// See datasheet section 5.5.11.3 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn load_partition_table(workarea_base: *mut u8, workarea_size: usize, force_reload: bool) -> i32 {
        crate::rom_data::rom_table_lookup(*b"LP", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Writes data from a buffer into OTP, or reads data from OTP into a buffer.
    ///
    /// See datasheet section 5.5.11.4 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn otp_access(buf: *mut u8, buf_len: usize, row_and_flags: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"OA", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Non-secure version of [otp_access()]
    ///
    /// Supported architectures: ARM-NS
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe fn otp_access_ns(buf: *mut u8, buf_len: usize, row_and_flags: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"OA", crate::rom_data::inner::rt_flags::FUNC_ARM_NONSEC)
    }
}

// **************** Boot Related Functions ****************

declare_rom_function! {
    /// Determines which of the partitions has the "better" IMAGE_DEF. In the
    /// case of executable images, this is the one that would be booted.
    ///
    /// See datasheet section 5.5.12.1 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn pick_ab_parition(workarea_base: *mut u8, workarea_size: usize, partition_a_num: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"AB", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Searches a memory region for a launchable image, and executes it if
    /// possible.
    ///
    /// See datasheet section 5.5.12.2 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn chain_image(workarea_base: *mut u8, workarea_size: usize, region_base: i32, region_size: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"CI", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Perform an "explicit" buy of an executable launched via an IMAGE_DEF
    /// which was "explicit buy" flagged.
    ///
    /// See datasheet section 5.5.12.3 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn explicit_buy(buffer: *mut u8, buffer_size: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"EB", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Not yet documented.
    ///
    /// See datasheet section 5.5.12.4 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn get_uf2_target_partition(workarea_base: *mut u8, workarea_size: usize, family_id: u32, partition_out: *mut u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"GU", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

declare_rom_function! {
    /// Returns: The index of the B partition of partition A if a partition
    /// table is present and loaded, and there is a partition A with a B
    /// partition; otherwise returns BOOTROM_ERROR_NOT_FOUND.
    ///
    /// See datasheet section 5.5.12.5 for more details.
    ///
    /// Supported architectures: ARM-S, RISC-V
    unsafe fn get_b_partition(partition_a: u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"GB", crate::rom_data::inner::rt_flags::FUNC_ARM_SEC_RISCV)
    }
}

// **************** Non-secure-specific Functions ****************

// NB: The "secure_call" function should be here, but it doesn't have a fixed
// function signature as it is designed to let you bounce into any secure
// function from non-secure mode.

// **************** RISC-V Functions ****************

declare_rom_function! {
    /// Set stack for RISC-V bootrom functions to use.
    ///
    /// See datasheet section 5.5.14.1 for more details.
    ///
    /// Supported architectures: RISC-V
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    unsafe fn set_bootrom_stack(base_size: *mut u32) -> i32 {
        crate::rom_data::rom_table_lookup(*b"SS", crate::rom_data::inner::rt_flags::FUNC_RISCV)
    }
}

/// The version number of the rom.
pub fn rom_version_number() -> u8 {
    unsafe { *VERSION_NUMBER }
}

/// The 8 most significant hex digits of the Bootrom git revision.
pub fn git_revision() -> u32 {
    let ptr = rom_data_lookup(*b"GR", rt_flags::DATA) as *const u32;
    unsafe { ptr.read() }
}

/// A pointer to the resident partition table info.
///
/// The resident partition table is the subset of the full partition table that
/// is kept in memory, and used for flash permissions.
pub fn partition_table_pointer() -> *const u32 {
    let ptr = rom_data_lookup(*b"PT", rt_flags::DATA) as *const *const u32;
    unsafe { ptr.read() }
}

/// Determine if we are in secure mode
///
/// Returns `true` if we are in secure mode and `false` if we are in non-secure
/// mode.
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub fn is_secure_mode() -> bool {
    // Look at the start of ROM, which is always readable
    #[allow(clippy::zero_ptr)]
    let rom_base: *mut u32 = 0x0000_0000 as *mut u32;
    // Use the 'tt' instruction to check the permissions for that address
    let tt = cortex_m::asm::tt(rom_base);
    // Is the secure bit set? => secure mode
    (tt & (1 << 22)) != 0
}

/// Determine if we are in secure mode
///
/// Always returns `false` on RISC-V as it is impossible to determine if
/// you are in Machine Mode or User Mode by design.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
pub fn is_secure_mode() -> bool {
    false
}
