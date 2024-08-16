//! Constants for binary info

// Credit: Taken from https://github.com/thejpster/rp-hal-rp2350-public (also licensed Apache 2.0 + MIT).
// Copyright (c) rp-rs organization

/// All Raspberry Pi specified IDs have this tag.
///
/// You can create your own for custom fields.
pub const TAG_RASPBERRY_PI: u16 = super::make_tag(b"RP");

/// Used to note the program name - use with StringEntry
pub const ID_RP_PROGRAM_NAME: u32 = 0x02031c86;
/// Used to note the program version - use with StringEntry
pub const ID_RP_PROGRAM_VERSION_STRING: u32 = 0x11a9bc3a;
/// Used to note the program build date - use with StringEntry
pub const ID_RP_PROGRAM_BUILD_DATE_STRING: u32 = 0x9da22254;
/// Used to note the size of the binary - use with IntegerEntry
pub const ID_RP_BINARY_END: u32 = 0x68f465de;
/// Used to note a URL for the program - use with StringEntry
pub const ID_RP_PROGRAM_URL: u32 = 0x1856239a;
/// Used to note a description of the program - use with StringEntry
pub const ID_RP_PROGRAM_DESCRIPTION: u32 = 0xb6a07c19;
/// Used to note some feature of the program - use with StringEntry
pub const ID_RP_PROGRAM_FEATURE: u32 = 0xa1f4b453;
/// Used to note some whether this was a Debug or Release build - use with StringEntry
pub const ID_RP_PROGRAM_BUILD_ATTRIBUTE: u32 = 0x4275f0d3;
/// Used to note the Pico SDK version used - use with StringEntry
pub const ID_RP_SDK_VERSION: u32 = 0x5360b3ab;
/// Used to note which board this program targets - use with StringEntry
pub const ID_RP_PICO_BOARD: u32 = 0xb63cffbb;
/// Used to note which `boot2` image this program uses - use with StringEntry
pub const ID_RP_BOOT2_NAME: u32 = 0x7f8882e1;

// End of file
