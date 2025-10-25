#![macro_use]

/// An IOMUXC pad.
///
/// This trait does not imply that GPIO can be used with this pad. [`Pin`](crate::gpio::Pin) must
/// also be implemented for GPIO.
#[allow(private_bounds)]
pub trait Pad: SealedPad {}

pub(crate) trait SealedPad {
    /// Address of the pad register for this pad.
    const PAD: *mut ();

    /// Address of the mux register for this pad.
    ///
    /// Some pads do not allow muxing (e.g. ONOFF).
    const MUX: Option<*mut ()>;
}

macro_rules! impl_iomuxc_pad {
    ($name: ident, $pad: expr, $mux: expr) => {
        impl crate::iomuxc::SealedPad for crate::peripherals::$name {
            const PAD: *mut () = $pad as *mut ();
            const MUX: Option<*mut ()> = Some($mux as *mut ());
        }

        impl crate::iomuxc::Pad for crate::peripherals::$name {}
    };
}
