//! Pure counter arithmetic, independently testable without hardware.

pub(crate) fn from_micros(micros: u64, freq: u32) -> Option<u32> {
    let ticks = micros.checked_mul(freq as u64)?.div_ceil(1_000_000);
    ticks.try_into().ok()
}

#[cfg(any(feature = "time-driver-lptmr", test))]
pub(crate) fn extend(previous: u64, counter: u32) -> u64 {
    previous + counter.wrapping_sub(previous as u32) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_rounds_up_and_rejects_overflow() {
        assert_eq!(from_micros(0, 1_000_000), Some(0));
        assert_eq!(from_micros(1, 16_384), Some(1));
        assert_eq!(from_micros(1_000, 16_384), Some(17));
        assert_eq!(from_micros(20_000, 1_000_000), Some(20_000));
        assert_eq!(from_micros(u32::MAX as u64, 1_000_000), Some(u32::MAX));
        assert_eq!(from_micros(u32::MAX as u64 + 1, 1_000_000), None);
        assert_eq!(from_micros(u64::MAX, 25_000_000), None);
    }

    #[test]
    fn counter_wrap_extends_without_losing_elapsed_ticks() {
        assert_eq!(extend(0, 17), 17);
        assert_eq!(extend(100, 150), 150);
        assert_eq!(extend(0xffff_fff0, 0x10), 0x1_0000_0010);
        assert_eq!(extend(0x1_ffff_fffe, 2), 0x2_0000_0002);
        assert_eq!(extend(0x2_0000_0002, 2), 0x2_0000_0002);
    }
}
