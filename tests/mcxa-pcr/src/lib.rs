//! Host tests for GPIO PCR operations using the actual PAC backed by RAM.
//!
//! This separate harness avoids the HAL's Cortex-M executor dev-dependency.

#![no_std]

#[cfg(test)]
use nxp_pac as pac;

#[cfg(test)]
#[path = "../../../embassy-mcxa/src/gpio/pcr.rs"]
mod pcr;
