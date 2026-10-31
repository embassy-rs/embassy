#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| embassy_boot_fuzz::check(data));
