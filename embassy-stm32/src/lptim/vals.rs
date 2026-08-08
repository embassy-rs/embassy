//! LPTIM register value enums.
//!
//! N6 (`lptim_n6`) PAC blocks omit the `vals` module; provide compatible enums here.

#[cfg(not(lptim_n6))]
pub use crate::pac::lptim::vals::*;

#[cfg(lptim_n6)]
mod imp {
    u8_enum!(Ccsel {
        OutputCompare = 0x0,
        InputCapture = 0x01,
    });

    u8_enum!(Filter {
        Count1 = 0x0,
        Count2 = 0x01,
        Count4 = 0x02,
        Count8 = 0x03,
    });

    u8_enum!(Presc {
        Div1 = 0x0,
        Div2 = 0x01,
        Div4 = 0x02,
        Div8 = 0x03,
        Div16 = 0x04,
        Div32 = 0x05,
        Div64 = 0x06,
        Div128 = 0x07,
    });

    u8_enum!(Trigen {
        Software = 0x0,
        RisingEdge = 0x01,
        FallingEdge = 0x02,
        BothEdges = 0x03,
    });
}

#[cfg(lptim_n6)]
pub use imp::*;
