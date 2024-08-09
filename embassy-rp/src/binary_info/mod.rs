//! Code and types for creating Picotool compatible "Binary Info" metadata
//!
//! Add something like this to your program, and compile with the "binary-info"
//! and "rt" features:
//!
//! ```
//! # use rp235x_hal as hal;
//! #[link_section = ".bi_entries"]
//! #[used]
//! pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 3] = [
//!     hal::binary_info_rp_program_name!(c"Program Name Here"),
//!     hal::binary_info_rp_cargo_version!(),
//!     hal::binary_info_int!(hal::binary_info::make_tag(b"JP"), 0x0000_0001, 0x12345678),
//! ];
//! ```

pub mod consts;

mod types;
pub use types::*;

#[macro_use]
mod macros;

extern "C" {
    /// The linker script sets this symbol to have the address of the first
    /// entry in the `.bi_entries` section.
    static __bi_entries_start: EntryAddr;
    /// The linker script sets this symbol to have the address just past the
    /// last entry in the `.bi_entries` section.
    static __bi_entries_end: EntryAddr;
    /// The linker script sets this symbol to have the address of the first
    /// entry in the `.data` section.
    static __sdata: u32;
    /// The linker script sets this symbol to have the address just past the
    /// first entry in the `.data` section.
    static __edata: u32;
    /// The linker script sets this symbol to have the address of the
    /// initialisation data for the first entry in the `.data` section (i.e. a
    /// flash address, not a RAM address).
    static __sidata: u32;
}

/// Picotool can find this block in our ELF file and report interesting
/// metadata.
///
/// The data here tells picotool the start and end flash addresses of our
/// metadata.
#[cfg(feature = "binary-info")]
#[link_section = ".start_block"]
#[used]
pub static PICOTOOL_HEADER: Header = unsafe {
    Header::new(
        core::ptr::addr_of!(__bi_entries_start),
        core::ptr::addr_of!(__bi_entries_end),
        &MAPPING_TABLE,
    )
};

/// This tells picotool how to convert RAM addresses back into Flash addresses
#[cfg(feature = "binary-info")]
pub static MAPPING_TABLE: [MappingTableEntry; 2] = [
    // This is the entry for .data
    MappingTableEntry {
        source_addr_start: unsafe { core::ptr::addr_of!(__sidata) },
        dest_addr_start: unsafe { core::ptr::addr_of!(__sdata) },
        dest_addr_end: unsafe { core::ptr::addr_of!(__edata) },
    },
    // This is the terminating marker
    MappingTableEntry::null(),
];

/// Create a 'Binary Info' entry containing the program name
///
/// This is well-known to picotool, and will be displayed if you run `picotool info`.
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * ID: [`consts::ID_RP_PROGRAM_NAME`]
pub const fn rp_program_name(name: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_PROGRAM_NAME, name)
}

/// Create a 'Binary Info' entry containing the program version.
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_PROGRAM_VERSION_STRING`]
pub const fn rp_program_version(name: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_PROGRAM_VERSION_STRING, name)
}

/// Create a 'Binary Info' entry with a URL
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_PROGRAM_URL`]
pub const fn rp_program_url(url: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_PROGRAM_URL, url)
}

/// Create a 'Binary Info' with the program build date
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_PROGRAM_BUILD_DATE_STRING`]
pub const fn rp_program_build_date_string(value: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_PROGRAM_BUILD_DATE_STRING, value)
}

/// Create a 'Binary Info' with the size of the binary
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_BINARY_END`]
pub const fn rp_binary_end(value: u32) -> IntegerEntry {
    IntegerEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_BINARY_END, value)
}

/// Create a 'Binary Info' with a description of the program
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_PROGRAM_DESCRIPTION`]
pub const fn rp_program_description(value: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_PROGRAM_DESCRIPTION, value)
}

/// Create a 'Binary Info' with some feature of the program
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_PROGRAM_FEATURE`]
pub const fn rp_program_feature(value: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_PROGRAM_FEATURE, value)
}

/// Create a 'Binary Info' with some whether this was a Debug or Release build
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_PROGRAM_BUILD_ATTRIBUTE`]
pub const fn rp_program_build_attribute(value: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_PROGRAM_BUILD_ATTRIBUTE, value)
}

/// Create a 'Binary Info' with the Pico SDK version used
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_SDK_VERSION`]
pub const fn rp_sdk_version(value: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_SDK_VERSION, value)
}

/// Create a 'Binary Info' with which board this program targets
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_PICO_BOARD`]
pub const fn rp_pico_board(value: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_PICO_BOARD, value)
}

/// Create a 'Binary Info' with which `boot2` image this program uses
///
/// * Tag: [`consts::TAG_RASPBERRY_PI`]
/// * Id: [`consts::ID_RP_BOOT2_NAME`]
pub const fn rp_boot2_name(value: &'static core::ffi::CStr) -> StringEntry {
    StringEntry::new(consts::TAG_RASPBERRY_PI, consts::ID_RP_BOOT2_NAME, value)
}

/// Create a tag from two ASCII letters.
///
/// ```
/// # use rp235x_hal as hal;
/// let tag = hal::binary_info::make_tag(b"RP");
/// assert_eq!(tag, 0x5052);
/// ```
pub const fn make_tag(c: &[u8; 2]) -> u16 {
    u16::from_le_bytes(*c)
}

// End of file
