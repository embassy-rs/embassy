//! Shared firmware support for the LPC55 USB HIL tests.
//!
//! The crate keeps its `src/bin/*.rs` targets; this library only holds code two
//! or more of them share. It deliberately does not link `defmt-rtt` or
//! `panic-probe` — every binary already does.

#![no_std]

pub mod conformance;
