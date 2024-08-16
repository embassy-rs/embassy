//! Handy macros for making Binary Info entries

// Credit: Taken from https://github.com/thejpster/rp-hal-rp2350-public (also licensed Apache 2.0 + MIT).
// Copyright (c) rp-rs organization

/// Generate a static item containing the given environment variable,
/// and return its [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_env {
    ($tag:expr, $id:expr, $env_var_name:expr) => {
        $crate::binary_info_str!($tag, $id, {
            let value = concat!(env!($env_var_name), "\0");
            // # Safety
            //
            // We used `concat!` to null-terminate on the line above.
            let value_cstr = unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(value.as_bytes()) };
            value_cstr
        })
    };
}

/// Generate a static item containing the given string, and return its
/// [`EntryAddr`](super::EntryAddr).
///
/// You must pass a numeric tag, a numeric ID, and `&CStr` (which is always
/// null-terminated).
#[macro_export]
macro_rules! binary_info_str {
    ($tag:expr, $id:expr, $str:expr) => {{
        static ENTRY: $crate::binary_info::StringEntry = $crate::binary_info::StringEntry::new($tag, $id, $str);
        ENTRY.addr()
    }};
}

/// Generate a static item containing the given string, and return its
/// [`EntryAddr`](super::EntryAddr).
///
/// You must pass a numeric tag, a numeric ID, and `&CStr` (which is always
/// null-terminated).
#[macro_export]
macro_rules! binary_info_int {
    ($tag:expr, $id:expr, $int:expr) => {{
        static ENTRY: $crate::binary_info::IntegerEntry = $crate::binary_info::IntegerEntry::new($tag, $id, $int);
        ENTRY.addr()
    }};
}

/// Generate a static item containing the program name, and return its
/// [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_program_name {
    ($name:expr) => {
        $crate::binary_info_str!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PROGRAM_NAME,
            $name
        )
    };
}

/// Generate a static item containing the `CARGO_BIN_NAME` as the program name,
/// and return its [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_cargo_bin_name {
    () => {
        $crate::binary_info_env!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PROGRAM_NAME,
            "CARGO_BIN_NAME"
        )
    };
}

/// Generate a static item containing the program version, and return its
/// [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_program_version {
    ($version:expr) => {{
        $crate::binary_info_str!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PROGRAM_VERSION,
            $version
        )
    }};
}

/// Generate a static item containing the `CARGO_PKG_VERSION` as the program
/// version, and return its [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_cargo_version {
    () => {
        $crate::binary_info_env!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PROGRAM_VERSION_STRING,
            "CARGO_PKG_VERSION"
        )
    };
}

/// Generate a static item containing the program URL, and return its
/// [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_program_url {
    ($url:expr) => {
        $crate::binary_info_str!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PROGRAM_URL,
            $url
        )
    };
}

/// Generate a static item containing the `CARGO_PKG_HOMEPAGE` as the program URL,
/// and return its [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_cargo_homepage_url {
    () => {
        $crate::binary_info_env!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PROGRAM_URL,
            "CARGO_PKG_HOMEPAGE"
        )
    };
}

/// Generate a static item containing the program description, and return its
/// [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_program_description {
    ($description:expr) => {
        $crate::binary_info_str!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PROGRAM_DESCRIPTION,
            $description
        )
    };
}

/// Generate a static item containing whether this is a debug or a release
/// build, and return its [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_program_build_attribute {
    () => {
        $crate::binary_info_str!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PROGRAM_BUILD_ATTRIBUTE,
            {
                if cfg!(debug_assertions) {
                    c"debug"
                } else {
                    c"release"
                }
            }
        )
    };
}

/// Generate a static item containing the specific board this program runs on,
/// and return its [`EntryAddr`](super::EntryAddr).
#[macro_export]
macro_rules! binary_info_rp_pico_board {
    ($board:expr) => {
        $crate::binary_info_str!(
            $crate::binary_info::consts::TAG_RASPBERRY_PI,
            $crate::binary_info::consts::ID_RP_PICO_BOARD,
            $board
        )
    };
}

// End of file
