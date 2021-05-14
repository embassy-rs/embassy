//! Time units

/// Bits per second
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Bps(pub u32);

/// Hertz
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Hertz(pub u32);

/// KiloHertz
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct KiloHertz(pub u32);

/// MegaHertz
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct MegaHertz(pub u32);

/// MilliSeconds
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct MilliSeconds(pub u32);

/// MicroSeconds
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct MicroSeconds(pub u32);

/// NanoSeconds
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct NanoSeconds(pub u32);

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `Bps`
    fn bps(self) -> Bps;

    /// Wrap in `Hertz`
    fn hz(self) -> Hertz;

    /// Wrap in `KiloHertz`
    fn khz(self) -> KiloHertz;

    /// Wrap in `MegaHertz`
    fn mhz(self) -> MegaHertz;

    /// Wrap in "MilliSeconds"
    fn ms(self) -> MilliSeconds;

    /// Wrap in "MicroSeconds"
    fn us(self) -> MicroSeconds;

    /// Wrap in "NanoSeconds"
    fn ns(self) -> NanoSeconds;
}

impl U32Ext for u32 {
    fn bps(self) -> Bps {
        Bps(self)
    }

    fn hz(self) -> Hertz {
        Hertz(self)
    }

    fn khz(self) -> KiloHertz {
        KiloHertz(self)
    }

    fn mhz(self) -> MegaHertz {
        MegaHertz(self)
    }

    fn ms(self) -> MilliSeconds {
        MilliSeconds(self)
    }

    fn us(self) -> MicroSeconds {
        MicroSeconds(self)
    }

    fn ns(self) -> NanoSeconds {
        NanoSeconds(self)
    }
}

// Unit conversions
impl Into<Hertz> for Bps {
    fn into(self) -> Hertz {
        Hertz(self.0)
    }
}

impl Into<Hertz> for KiloHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000)
    }
}

impl Into<Hertz> for MegaHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000_000)
    }
}

impl Into<KiloHertz> for MegaHertz {
    fn into(self) -> KiloHertz {
        KiloHertz(self.0 * 1_000)
    }
}

impl Into<NanoSeconds> for MicroSeconds {
    fn into(self) -> NanoSeconds {
        NanoSeconds(self.0 * 1_000)
    }
}

impl Into<NanoSeconds> for MilliSeconds {
    fn into(self) -> NanoSeconds {
        NanoSeconds(self.0 * 1_000_000)
    }
}

impl Into<MicroSeconds> for MilliSeconds {
    fn into(self) -> MicroSeconds {
        MicroSeconds(self.0 * 1_000)
    }
}
