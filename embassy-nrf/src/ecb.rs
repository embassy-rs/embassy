//! ECB - AES electronic codebook mode encryption
//!
use embassy_hal_internal::PeripheralRef;

use crate::peripherals::ECB;

/// ECB - AES electronic codebook mode encryption.
///
/// Note: This peripheral is current unimplemented!
/// TODO: ECB shares resources with AAR and CCM, while
///       ECB will always have lowest priority.
pub struct Ecb<'d> {
    _p: PeripheralRef<'d, ECB>,
}
