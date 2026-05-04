//! Cortex M7 MPU

use cortex_m::peripheral::MPU;

//https://developer.arm.com/documentation/dui0646/c/Cortex-M7-Peripherals/Optional-Memory-Protection-Unit/MPU-access-permission-attributes

/// Normal memory, write-through, no write-allocate:
/// TEX=0, C=1, B=0
pub const ATTR_WRITE_THROUGH: u32 = (0b000 << 19) | (1 << 17) | (0 << 16) | (0 << 18);

/// Normal memory, write-back, write/read allocate:
/// Common Cortex-M setting: TEX=0, C=1, B=1
pub const ATTR_WRITE_BACK: u32 = (0b000 << 19) | (1 << 17) | (1 << 16) | (0 << 18);

// https://developer.arm.com/documentation/dui0646/c/Cortex-M7-Peripherals/Optional-Memory-Protection-Unit/MPU-Region-Attribute-and-Size-Register
const MPU_XN: u32 = 1 << 28;
const MPU_ENABLE: u32 = 1;
const MPU_AP_FULL_ACCESS: u32 = (0b011) << 24;

const fn mpu_size_field(size: usize) -> u32 {
    size.ilog2() - 1
}

pub(super) unsafe fn mpu_region(mpu: &mut MPU, region: u32, base: usize, size: usize, attrs: u32) {
    debug_assert!(size.is_power_of_two());
    debug_assert_eq!(base & (size - 1), 0);

    let rasr: u32 = attrs | MPU_AP_FULL_ACCESS | MPU_XN | (mpu_size_field(size) << 1) | MPU_ENABLE;

    defmt::debug!(
        "MPU region: {} base: {:x} size: {} rasr: {:b}",
        region,
        base,
        size,
        rasr
    );

    unsafe {
        mpu.rnr.write(region);
        mpu.rbar.write(base as u32);
        mpu.rasr.write(rasr);
    }
}
