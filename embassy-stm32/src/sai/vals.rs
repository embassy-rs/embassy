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
    pub enum Mckdiv {
        Div1 = 0x01,
        Div2 = 0x02,
        Div3 = 0x03,
        Div4 = 0x04,
        Div5 = 0x05,
        Div6 = 0x06,
        Div7 = 0x07,
        Div8 = 0x08,
        Div9 = 0x09,
        Div10 = 0x0a,
        Div11 = 0x0b,
        Div12 = 0x0c,
        Div13 = 0x0d,
        Div14 = 0x0e,
        Div15 = 0x0f,
        Div16 = 0x10,
        Div17 = 0x11,
        Div18 = 0x12,
        Div19 = 0x13,
        Div20 = 0x14,
        Div21 = 0x15,
        Div22 = 0x16,
        Div23 = 0x17,
        Div24 = 0x18,
        Div25 = 0x19,
        Div26 = 0x1a,
        Div27 = 0x1b,
        Div28 = 0x1c,
        Div29 = 0x1d,
        Div30 = 0x1e,
        Div31 = 0x1f,
        Div32 = 0x20,
        Div33 = 0x21,
        Div34 = 0x22,
        Div35 = 0x23,
        Div36 = 0x24,
        Div37 = 0x25,
        Div38 = 0x26,
        Div39 = 0x27,
        Div40 = 0x28,
        Div41 = 0x29,
        Div42 = 0x2a,
        Div43 = 0x2b,
        Div44 = 0x2c,
        Div45 = 0x2d,
        Div46 = 0x2e,
        Div47 = 0x2f,
        Div48 = 0x30,
        Div49 = 0x31,
        Div50 = 0x32,
        Div51 = 0x33,
        Div52 = 0x34,
        Div53 = 0x35,
        Div54 = 0x36,
        Div55 = 0x37,
        Div56 = 0x38,
        Div57 = 0x39,
        Div58 = 0x3a,
        Div59 = 0x3b,
        Div60 = 0x3c,
        Div61 = 0x3d,
        Div62 = 0x3e,
        Div63 = 0x3f,
    }

    impl Mckdiv {
        #[inline(always)]
        pub const fn from_bits(val: u8) -> Self {
            unsafe { core::mem::transmute(val & 0x3f) }
        }

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
