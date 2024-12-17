#![cfg_attr(
    feature = "rp2040",
    doc = r"
//! Functions and data from the RPI Bootrom.
//!
//! From the [RP2040 datasheet](https://datasheets.raspberrypi.org/rp2040/rp2040-datasheet.pdf), Section 2.8.2.1:
//!
//! > The Bootrom contains a number of public functions that provide useful
//! > RP2040 functionality that might be needed in the absence of any other code
//! > on the device, as well as highly optimized versions of certain key
//! > functionality that would otherwise have to take up space in most user
//! > binaries.
"
)]
#![cfg_attr(
    feature = "_rp235x",
    doc = r"
//! Functions and data from the RPI Bootrom.
//!
//! From [Section 5.4](https://rptl.io/rp2350-datasheet#section_bootrom) of the
//! RP2350 datasheet:
//!
//! > Whilst some ROM space is dedicated to the implementation of the boot
//! > sequence and USB/UART boot interfaces, the bootrom also contains public
//! > functions that provide useful RP2350 functionality that may be useful for
//! > any code or runtime running on the device
"
)]

#[cfg_attr(feature = "rp2040", path = "rp2040.rs")]
#[cfg_attr(feature = "_rp235x", path = "rp235x.rs")]
mod inner;
pub use inner::*;
