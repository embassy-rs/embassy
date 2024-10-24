//! Accelerated address resolver (AAR)
//!
use embassy_hal_internal::PeripheralRef;

use crate::peripherals::AAR;

/// AAR (Accelerated Address Resolver)
///
/// Note: This peripheral is current unimplemented!
/// TODO: AAR shares resources with CCM
pub struct Aar<'d> {
    _p: PeripheralRef<'d, AAR>,
}
