//! SAI register value enums.
//!
//! N6 (`sai_n6`) PAC blocks omit the `vals` module; provide compatible enums here.

#[cfg(not(sai_n6))]
pub use crate::pac::sai::vals::*;

#[cfg(sai_n6)]
mod imp {
    u8_enum!(Ckstr {
        FallingEdge = 0x0,
        RisingEdge = 0x01,
    });

    u8_enum!(Comp {
        NoCompanding = 0x0,
        MuLaw = 0x02,
        ALaw = 0x03,
    });

    u8_enum!(Cpl {
        OnesComplement = 0x0,
        TwosComplement = 0x01,
    });

    u8_enum!(Ds {
        Bit8 = 0x02,
        Bit10 = 0x03,
        Bit16 = 0x04,
        Bit20 = 0x05,
        Bit24 = 0x06,
        Bit32 = 0x07,
    });

    u8_enum!(Fsoff {
        OnFirst = 0x0,
        BeforeFirst = 0x01,
    });

    u8_enum!(Fspol {
        FallingEdge = 0x0,
        RisingEdge = 0x01,
    });

    u8_enum!(Fth {
        Empty = 0x0,
        Quarter1 = 0x01,
        Quarter2 = 0x02,
        Quarter3 = 0x03,
        Full = 0x05,
    });

    u8_enum!(Lsbfirst {
        LsbFirst = 0x0,
        MsbFirst = 0x01,
    });

    // Full divider range uses the same bit layout as SAI v3.
    #[repr(u8)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    /// SAI master clock divider (`MCKDIV` field), full range (1..63).
    pub enum Mckdiv {
        /// Master clock divider = 1.
        Div1 = 0x01,
        /// Master clock divider = 2.
        Div2 = 0x02,
        /// Master clock divider = 3.
        Div3 = 0x03,
        /// Master clock divider = 4.
        Div4 = 0x04,
        /// Master clock divider = 5.
        Div5 = 0x05,
        /// Master clock divider = 6.
        Div6 = 0x06,
        /// Master clock divider = 7.
        Div7 = 0x07,
        /// Master clock divider = 8.
        Div8 = 0x08,
        /// Master clock divider = 9.
        Div9 = 0x09,
        /// Master clock divider = 10.
        Div10 = 0x0a,
        /// Master clock divider = 11.
        Div11 = 0x0b,
        /// Master clock divider = 12.
        Div12 = 0x0c,
        /// Master clock divider = 13.
        Div13 = 0x0d,
        /// Master clock divider = 14.
        Div14 = 0x0e,
        /// Master clock divider = 15.
        Div15 = 0x0f,
        /// Master clock divider = 16.
        Div16 = 0x10,
        /// Master clock divider = 17.
        Div17 = 0x11,
        /// Master clock divider = 18.
        Div18 = 0x12,
        /// Master clock divider = 19.
        Div19 = 0x13,
        /// Master clock divider = 20.
        Div20 = 0x14,
        /// Master clock divider = 21.
        Div21 = 0x15,
        /// Master clock divider = 22.
        Div22 = 0x16,
        /// Master clock divider = 23.
        Div23 = 0x17,
        /// Master clock divider = 24.
        Div24 = 0x18,
        /// Master clock divider = 25.
        Div25 = 0x19,
        /// Master clock divider = 26.
        Div26 = 0x1a,
        /// Master clock divider = 27.
        Div27 = 0x1b,
        /// Master clock divider = 28.
        Div28 = 0x1c,
        /// Master clock divider = 29.
        Div29 = 0x1d,
        /// Master clock divider = 30.
        Div30 = 0x1e,
        /// Master clock divider = 31.
        Div31 = 0x1f,
        /// Master clock divider = 32.
        Div32 = 0x20,
        /// Master clock divider = 33.
        Div33 = 0x21,
        /// Master clock divider = 34.
        Div34 = 0x22,
        /// Master clock divider = 35.
        Div35 = 0x23,
        /// Master clock divider = 36.
        Div36 = 0x24,
        /// Master clock divider = 37.
        Div37 = 0x25,
        /// Master clock divider = 38.
        Div38 = 0x26,
        /// Master clock divider = 39.
        Div39 = 0x27,
        /// Master clock divider = 40.
        Div40 = 0x28,
        /// Master clock divider = 41.
        Div41 = 0x29,
        /// Master clock divider = 42.
        Div42 = 0x2a,
        /// Master clock divider = 43.
        Div43 = 0x2b,
        /// Master clock divider = 44.
        Div44 = 0x2c,
        /// Master clock divider = 45.
        Div45 = 0x2d,
        /// Master clock divider = 46.
        Div46 = 0x2e,
        /// Master clock divider = 47.
        Div47 = 0x2f,
        /// Master clock divider = 48.
        Div48 = 0x30,
        /// Master clock divider = 49.
        Div49 = 0x31,
        /// Master clock divider = 50.
        Div50 = 0x32,
        /// Master clock divider = 51.
        Div51 = 0x33,
        /// Master clock divider = 52.
        Div52 = 0x34,
        /// Master clock divider = 53.
        Div53 = 0x35,
        /// Master clock divider = 54.
        Div54 = 0x36,
        /// Master clock divider = 55.
        Div55 = 0x37,
        /// Master clock divider = 56.
        Div56 = 0x38,
        /// Master clock divider = 57.
        Div57 = 0x39,
        /// Master clock divider = 58.
        Div58 = 0x3a,
        /// Master clock divider = 59.
        Div59 = 0x3b,
        /// Master clock divider = 60.
        Div60 = 0x3c,
        /// Master clock divider = 61.
        Div61 = 0x3d,
        /// Master clock divider = 62.
        Div62 = 0x3e,
        /// Master clock divider = 63.
        Div63 = 0x3f,
    }

    impl Mckdiv {
        /// Construct from the raw `MCKDIV` field value (masked to 6 bits).
        #[inline(always)]
        pub const fn from_bits(val: u8) -> Self {
            unsafe { core::mem::transmute(val & 0x3f) }
        }

        /// Returns the raw `MCKDIV` field value.
        #[inline(always)]
        pub const fn to_bits(self) -> u8 {
            unsafe { core::mem::transmute(self) }
        }
    }

    impl From<u8> for Mckdiv {
        #[inline(always)]
        fn from(val: u8) -> Self {
            Mckdiv::from_bits(val)
        }
    }

    impl From<Mckdiv> for u8 {
        #[inline(always)]
        fn from(val: Mckdiv) -> u8 {
            Mckdiv::to_bits(val)
        }
    }

    u8_enum!(Mode {
        MasterTx = 0x0,
        MasterRx = 0x01,
        SlaveTx = 0x02,
        SlaveRx = 0x03,
    });

    u8_enum!(Mono {
        Stereo = 0x0,
        Mono = 0x01,
    });

    u8_enum!(Muteval {
        SendZero = 0x0,
        SendLast = 0x01,
    });

    u8_enum!(Outdriv {
        OnStart = 0x0,
        Immediately = 0x01,
    });

    u8_enum!(Prtcfg {
        Free = 0x0,
        Spdif = 0x01,
        Ac97 = 0x02,
    });

    u8_enum!(Slotsz {
        DataSize = 0x0,
        Bit16 = 0x01,
        Bit32 = 0x02,
    });

    u8_enum!(Syncen {
        Asynchronous = 0x0,
        Internal = 0x01,
        External = 0x02,
    });
}

#[cfg(sai_n6)]
pub use imp::*;
