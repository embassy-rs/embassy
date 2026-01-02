#![no_std]

use mimxrt600_fcb::FlexSPIFlashConfigurationBlock;
use {defmt_rtt as _, panic_probe as _};

// auto-generated version information from Cargo.toml
include!(concat!(env!("OUT_DIR"), "/biv.rs"));

#[unsafe(link_section = ".otfad")]
#[used]
static OTFAD: [u8; 256] = [0; 256];

#[rustfmt::skip]
#[unsafe(link_section = ".fcb")]
#[used]
static FCB: FlexSPIFlashConfigurationBlock = FlexSPIFlashConfigurationBlock::build();

#[unsafe(link_section = ".keystore")]
#[used]
static KEYSTORE: [u8; 2048] = [0; 2048];
