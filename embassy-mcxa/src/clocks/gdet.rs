//! Read-only access prerequisites for adopting the ROM-configured digital detector.

use crate::pac;

/// Checks the physical access gates and reset without changing retained setup.
///
/// CPU/system clocks and security/privilege access remain platform prerequisites.
/// This does not validate the detector's reference clock or calibration.
pub(crate) fn gdet_access_ready() -> bool {
    critical_section::with(|_| {
        let clocks = pac::MRCC0.mrcc_glb_cc4().read();
        let reset = pac::MRCC0.mrcc_glb_rst4().read();
        clocks.dgdet0() && clocks.seccon() && reset.dgdet0()
    })
}
