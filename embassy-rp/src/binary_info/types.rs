//! Types for the Binary Info system

/// This is the 'Binary Info' header block that `picotool` looks for in your UF2
/// file/ELF file/Pico in Bootloader Mode to give you useful metadata about your
/// program.
///
/// It should be placed in the first 4096 bytes of flash, so use your `memory.x`
/// to insert a section between `.text` and `.vector_table` and put a static
/// value of this type in that section.
#[repr(C)]
pub struct Header {
    /// Must be equal to Picotool::MARKER_START
    marker_start: u32,
    /// The first in our table of pointers to Entries
    entries_start: *const EntryAddr,
    /// The last in our table of pointers to Entries
    entries_end: *const EntryAddr,
    /// The first entry in a null-terminated RAM/Flash mapping table
    mapping_table: *const MappingTableEntry,
    /// Must be equal to Picotool::MARKER_END
    marker_end: u32,
}

impl Header {
    /// This is the `BINARY_INFO_MARKER_START` magic value from `picotool`
    const MARKER_START: u32 = 0x7188ebf2;
    /// This is the `BINARY_INFO_MARKER_END` magic value from `picotool`
    const MARKER_END: u32 = 0xe71aa390;

    /// Create a new `picotool` compatible header.
    ///
    /// * `entries_start` - the first [`EntryAddr`] in the table
    /// * `entries_end` - the last [`EntryAddr`] in the table
    /// * `mapping_table` - the RAM/Flash address mapping table
    pub const fn new(
        entries_start: *const EntryAddr,
        entries_end: *const EntryAddr,
        mapping_table: &'static [MappingTableEntry],
    ) -> Self {
        let mapping_table = mapping_table.as_ptr();
        Self {
            marker_start: Self::MARKER_START,
            entries_start,
            entries_end,
            mapping_table,
            marker_end: Self::MARKER_END,
        }
    }
}

// We need this as rustc complains that is is unsafe to share `*const u32`
// pointers between threads. We only allow these to be created with static
// data, so this is OK.
unsafe impl Sync for Header {}

/// This is a reference to an entry. It's like a `&dyn` ref to some type `T:
/// Entry`, except that the run-time type information is encoded into the
/// Entry itself in very specific way.
#[repr(transparent)]
pub struct EntryAddr(*const u32);

// We need this as rustc complains that is is unsafe to share `*const u32`
// pointers between threads. We only allow these to be created with static
// data, so this is OK.
unsafe impl Sync for EntryAddr {}

/// Allows us to tell picotool where values are in the UF2 given their run-time
/// address.
///
/// The most obvious example is RAM variables, which must be found in the
/// `.data` section of the UF2.
#[repr(C)]
pub struct MappingTableEntry {
    /// The start address in RAM (or wherever the address picotool finds will
    /// point)
    pub source_addr_start: *const u32,
    /// The start address in flash (or whever the data actually lives in the
    /// ELF)
    pub dest_addr_start: *const u32,
    /// The end address in flash
    pub dest_addr_end: *const u32,
}

impl MappingTableEntry {
    /// Generate a null entry to mark the end of the list
    pub const fn null() -> MappingTableEntry {
        MappingTableEntry {
            source_addr_start: core::ptr::null(),
            dest_addr_start: core::ptr::null(),
            dest_addr_end: core::ptr::null(),
        }
    }
}

// We need this as rustc complains that is is unsafe to share `*const u32`
// pointers between threads. We only allow these to be created with static
// data, so this is OK.
unsafe impl Sync for MappingTableEntry {}

/// This is the set of data types that `picotool` supports.
#[repr(u16)]
pub enum DataType {
    /// Raw data
    Raw = 1,
    /// Data with a size
    SizedData = 2,
    /// A list of binary data
    BinaryInfoListZeroTerminated = 3,
    /// A BSON encoded blob
    Bson = 4,
    /// An Integer with an ID
    IdAndInt = 5,
    /// A string with an Id
    IdAndString = 6,
    /// A block device
    BlockDevice = 7,
    /// GPIO pins, with their function
    PinsWithFunction = 8,
    /// GPIO pins, with their name
    PinsWithName = 9,
    /// GPIO pins, with multiple names?
    PinsWithNames = 10,
}

/// All Entries start with this common header
#[repr(C)]
struct EntryCommon {
    data_type: DataType,
    tag: u16,
}

/// An entry which contains both an ID (e.g. `ID_RP_PROGRAM_NAME`) and a pointer
/// to a null-terminated string.
#[repr(C)]
pub struct StringEntry {
    header: EntryCommon,
    id: u32,
    value: *const core::ffi::c_char,
}

impl StringEntry {
    /// Create a new `StringEntry`
    pub const fn new(tag: u16, id: u32, value: &'static core::ffi::CStr) -> StringEntry {
        StringEntry {
            header: EntryCommon {
                data_type: DataType::IdAndString,
                tag,
            },
            id,
            value: value.as_ptr(),
        }
    }

    /// Get this entry's address
    pub const fn addr(&self) -> EntryAddr {
        EntryAddr(self as *const Self as *const u32)
    }
}

// We need this as rustc complains that is is unsafe to share `*const
// core::ffi::c_char` pointers between threads. We only allow these to be
// created with static string slices, so it's OK.
unsafe impl Sync for StringEntry {}

/// An entry which contains both an ID (e.g. `ID_RP_BINARY_END`) and an integer.
#[repr(C)]
pub struct IntegerEntry {
    header: EntryCommon,
    id: u32,
    value: u32,
}

impl IntegerEntry {
    /// Create a new `StringEntry`
    pub const fn new(tag: u16, id: u32, value: u32) -> IntegerEntry {
        IntegerEntry {
            header: EntryCommon {
                data_type: DataType::IdAndInt,
                tag,
            },
            id,
            value,
        }
    }

    /// Get this entry's address
    pub const fn addr(&self) -> EntryAddr {
        EntryAddr(self as *const Self as *const u32)
    }
}

// End of file
