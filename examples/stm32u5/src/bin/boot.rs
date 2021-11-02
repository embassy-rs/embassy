#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    loop {}
}
