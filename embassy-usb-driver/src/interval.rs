//! Endpoint polling interval.

use crate::{EndpointType, Speed};

/// Number of microframes (125 µs) in a frame (1 ms).
const MICROFRAMES_PER_FRAME: u32 = 8;

/// Polling interval of an endpoint.
///
/// This is the *period* the endpoint is serviced with, independent of the bus speed, so it means
/// the same thing wherever it is stored: in an [`EndpointConfig`](crate::EndpointConfig), an
/// [`EndpointInfo`](crate::EndpointInfo), or a host-side pipe. It is kept as a whole number of
/// microframes (125 µs, the high-speed bus interval), and one frame (1 ms) is 8 microframes, so
/// equal periods are always equal values: `from_ms(1) == from_microframes(8)`.
///
/// The `bInterval` byte of an endpoint descriptor is a speed- and transfer-type-specific encoding
/// of this period, which [`b_interval`](Self::b_interval) and
/// [`from_b_interval`](Self::from_b_interval) convert to and from.
///
/// Bulk and control endpoints have no polling period, they use [`EndpointInterval::NONE`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointInterval {
    microframes: u32,
}

impl EndpointInterval {
    /// No polling period, for bulk and control endpoints.
    ///
    /// For a periodic (interrupt or isochronous) endpoint this is the shortest period, every
    /// frame or microframe.
    pub const NONE: Self = Self { microframes: 0 };

    /// A period of `ms` milliseconds (frames).
    pub const fn from_ms(ms: u8) -> Self {
        Self {
            microframes: ms as u32 * MICROFRAMES_PER_FRAME,
        }
    }

    /// A period of `microframes` microframes (125 µs units).
    pub const fn from_microframes(microframes: u32) -> Self {
        Self { microframes }
    }

    /// The period in microframes (125 µs units).
    pub const fn microframes(self) -> u32 {
        self.microframes
    }

    /// The period in whole frames (1 ms), rounded up and at least 1.
    pub const fn frames(self) -> u32 {
        let frames = self.microframes.div_ceil(MICROFRAMES_PER_FRAME);
        if frames == 0 { 1 } else { frames }
    }

    /// The `bInterval` of a full-speed (or low-speed) endpoint descriptor for this period.
    ///
    /// Interrupt endpoints use a linear period of 1..=255 frames, isochronous endpoints
    /// the exponential period `2^(bInterval-1)` frames, rounded up to a power of two. Bulk and control
    /// endpoints use 0.
    pub const fn full_speed_b_interval(self, ep_type: EndpointType) -> u8 {
        match ep_type {
            EndpointType::Interrupt => {
                let frames = self.frames();
                if frames > 255 { 255 } else { frames as u8 }
            }
            EndpointType::Isochronous => exponent(self.frames()),
            EndpointType::Control | EndpointType::Bulk => 0,
        }
    }

    /// The `bInterval` of a high-speed (or SuperSpeed) endpoint descriptor for this period.
    ///
    /// Interrupt and isochronous endpoints use the exponential period `2^(bInterval-1)`
    /// microframes, rounded up to a power of two. Bulk and control endpoints use 0.
    pub const fn high_speed_b_interval(self, ep_type: EndpointType) -> u8 {
        match ep_type {
            EndpointType::Interrupt | EndpointType::Isochronous => exponent(self.microframes),
            EndpointType::Control | EndpointType::Bulk => 0,
        }
    }

    /// The `bInterval` of an endpoint descriptor at `speed` for this period.
    pub const fn b_interval(self, speed: Speed, ep_type: EndpointType) -> u8 {
        match speed {
            Speed::Low | Speed::Full => self.full_speed_b_interval(ep_type),
            _ => self.high_speed_b_interval(ep_type),
        }
    }

    /// The period of an endpoint with the descriptor `bInterval` at `speed`.
    ///
    /// The inverse of [`b_interval`](Self::b_interval) for the values it produces.
    /// Bulk and control endpoints have no period, which gives [`EndpointInterval::NONE`].
    pub const fn from_b_interval(speed: Speed, ep_type: EndpointType, b_interval: u8) -> Self {
        match (ep_type, speed) {
            (EndpointType::Control | EndpointType::Bulk, _) => Self::NONE,
            (EndpointType::Interrupt, Speed::Low | Speed::Full) => {
                let frames = if b_interval == 0 { 1 } else { b_interval as u32 };
                Self::from_microframes(frames * MICROFRAMES_PER_FRAME)
            }
            (EndpointType::Isochronous, Speed::Low | Speed::Full) => {
                Self::from_microframes(pow2(b_interval) * MICROFRAMES_PER_FRAME)
            }
            (_, _) => Self::from_microframes(pow2(b_interval)),
        }
    }
}

/// `bInterval` for the exponential period `2^(bInterval-1)` that is at least `period`, in 1..=16.
const fn exponent(period: u32) -> u8 {
    if period <= 1 {
        return 1;
    }
    // ceil(log2(period)) + 1, where 2^15 is the longest period.
    let bits = u32::BITS - (period - 1).leading_zeros();
    let bits = if bits > 15 { 15 } else { bits };
    bits as u8 + 1
}

/// The period `2^(b_interval-1)`, with `b_interval` clamped to 1..=16.
const fn pow2(b_interval: u8) -> u32 {
    let b = if b_interval == 0 {
        1
    } else if b_interval > 16 {
        16
    } else {
        b_interval
    };
    1 << (b - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_periods_are_equal() {
        assert_eq!(EndpointInterval::from_ms(1), EndpointInterval::from_microframes(8));
        assert_eq!(EndpointInterval::from_ms(0), EndpointInterval::NONE);
    }

    #[test]
    fn frames_round_up() {
        assert_eq!(EndpointInterval::NONE.frames(), 1);
        assert_eq!(EndpointInterval::from_microframes(1).frames(), 1);
        assert_eq!(EndpointInterval::from_microframes(8).frames(), 1);
        assert_eq!(EndpointInterval::from_microframes(9).frames(), 2);
        assert_eq!(EndpointInterval::from_microframes(u32::MAX).frames(), u32::MAX / 8 + 1);
    }

    #[test]
    fn exponent_rounds_up_to_power_of_two() {
        assert_eq!(exponent(0), 1);
        assert_eq!(exponent(1), 1);
        assert_eq!(exponent(2), 2);
        assert_eq!(exponent(3), 3);
        assert_eq!(exponent(4), 3);
        assert_eq!(exponent(5), 4);
        assert_eq!(exponent(8), 4);
        assert_eq!(exponent(32768), 16);
        assert_eq!(exponent(32769), 16);
        assert_eq!(exponent(u32::MAX), 16);
    }

    #[test]
    fn high_speed_b_interval() {
        use EndpointType::*;
        let hs = |i: EndpointInterval, t| i.high_speed_b_interval(t);
        assert_eq!(hs(EndpointInterval::NONE, Interrupt), 1); // 125 us
        assert_eq!(hs(EndpointInterval::from_ms(1), Interrupt), 4); // 8 microframes
        assert_eq!(hs(EndpointInterval::from_ms(2), Interrupt), 5);
        assert_eq!(hs(EndpointInterval::from_ms(3), Isochronous), 6); // rounded up to 32 microframes
        assert_eq!(hs(EndpointInterval::from_ms(255), Interrupt), 12); // 2048 microframes
        assert_eq!(hs(EndpointInterval::from_microframes(2), Isochronous), 2); // 250 us
        assert_eq!(hs(EndpointInterval::from_microframes(3), Isochronous), 3); // rounded up to 500 us
        assert_eq!(hs(EndpointInterval::from_microframes(65536), Interrupt), 16); // clamped
        assert_eq!(hs(EndpointInterval::from_ms(4), Bulk), 0);
        assert_eq!(hs(EndpointInterval::from_ms(4), Control), 0);
    }

    #[test]
    fn full_speed_b_interval() {
        use EndpointType::*;
        let fs = |i: EndpointInterval, t| i.full_speed_b_interval(t);
        // Interrupt is linear in frames.
        assert_eq!(fs(EndpointInterval::NONE, Interrupt), 1);
        assert_eq!(fs(EndpointInterval::from_ms(4), Interrupt), 4);
        assert_eq!(fs(EndpointInterval::from_ms(255), Interrupt), 255);
        assert_eq!(fs(EndpointInterval::from_microframes(2), Interrupt), 1); // sub-frame is 1 frame
        assert_eq!(fs(EndpointInterval::from_microframes(9), Interrupt), 2);
        assert_eq!(fs(EndpointInterval::from_microframes(u32::MAX), Interrupt), 255);
        // Isochronous is exponential in frames.
        assert_eq!(fs(EndpointInterval::NONE, Isochronous), 1);
        assert_eq!(fs(EndpointInterval::from_ms(1), Isochronous), 1);
        assert_eq!(fs(EndpointInterval::from_ms(2), Isochronous), 2);
        assert_eq!(fs(EndpointInterval::from_ms(3), Isochronous), 3); // rounded up to 4 frames
        assert_eq!(fs(EndpointInterval::from_ms(4), Isochronous), 3);
        assert_eq!(fs(EndpointInterval::from_ms(255), Isochronous), 9); // 256 frames
        assert_eq!(fs(EndpointInterval::from_microframes(2), Isochronous), 1);
        assert_eq!(fs(EndpointInterval::from_ms(4), Bulk), 0);
    }

    #[test]
    fn same_period_both_speeds() {
        // 4 ms: bInterval 4 (linear frames) vs 6 (2^5 = 32 microframes) for interrupt.
        let four = EndpointInterval::from_ms(4);
        assert_eq!(four.b_interval(Speed::Full, EndpointType::Interrupt), 4);
        assert_eq!(four.b_interval(Speed::High, EndpointType::Interrupt), 6);
        assert_eq!(four.b_interval(Speed::Super, EndpointType::Interrupt), 6);
        assert_eq!(four.b_interval(Speed::Full, EndpointType::Isochronous), 3);
        assert_eq!(four.b_interval(Speed::High, EndpointType::Isochronous), 6);
    }

    #[test]
    fn b_interval_roundtrip() {
        for ep_type in [EndpointType::Interrupt, EndpointType::Isochronous] {
            for speed in [Speed::Full, Speed::High, Speed::Super] {
                for b in 1..=16u8 {
                    let interval = EndpointInterval::from_b_interval(speed, ep_type, b);
                    assert_eq!(interval.b_interval(speed, ep_type), b, "{ep_type:?} {speed:?} {b}");
                }
            }
        }
        // Full-speed interrupt is linear up to 255 frames.
        let i = EndpointInterval::from_b_interval(Speed::Full, EndpointType::Interrupt, 255);
        assert_eq!(i, EndpointInterval::from_ms(255));
        assert_eq!(i.b_interval(Speed::Full, EndpointType::Interrupt), 255);
        // bInterval 0 is the shortest period.
        let z = EndpointInterval::from_b_interval(Speed::Full, EndpointType::Interrupt, 0);
        assert_eq!(z, EndpointInterval::from_ms(1));
        assert_eq!(
            EndpointInterval::from_b_interval(Speed::High, EndpointType::Interrupt, 0),
            EndpointInterval::from_microframes(1)
        );
        // Out-of-range exponents are clamped.
        assert_eq!(
            EndpointInterval::from_b_interval(Speed::High, EndpointType::Interrupt, 255),
            EndpointInterval::from_microframes(1 << 15)
        );
    }

    #[test]
    fn bulk_and_control_have_no_interval() {
        for ep_type in [EndpointType::Bulk, EndpointType::Control] {
            assert_eq!(
                EndpointInterval::from_b_interval(Speed::High, ep_type, 5),
                EndpointInterval::NONE
            );
        }
    }
}
