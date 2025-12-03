//! Cyclic Redundandy Check (CRC)

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use mcxa_pac::crc0::ctrl::{Fxor, Tot, Totr};

use crate::clocks::enable_and_reset;
use crate::clocks::periph_helpers::NoConfig;
use crate::peripherals::CRC0;

/// CRC driver.
pub struct Crc<'d, M> {
    _peri: Peri<'d, CRC0>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> Crc<'d, M> {
    fn new_inner(_peri: Peri<'d, CRC0>) -> Self {
        _ = unsafe { enable_and_reset::<CRC0>(&NoConfig) };

        Crc {
            _peri,
            _phantom: PhantomData,
        }
    }

    // Configure the underlying peripheral. `f` is expected to set the
    // operating mode to either 16- or 32-bits.
    fn configure<F>(config: Config, f: F)
    where
        F: FnOnce(),
    {
        f();

        Self::regs().ctrl().modify(|_, w| {
            w.fxor()
                .variant(config.complement_out.into())
                .totr()
                .variant(config.reflect_out.into())
                .tot()
                .variant(config.reflect_in.into())
                .was()
                .data()
        });

        Self::regs().gpoly32().write(|w| unsafe { w.bits(config.polynomial) });

        Self::regs().ctrl().modify(|_, w| w.was().seed());
        Self::regs().data32().write(|w| unsafe { w.bits(config.seed) });
        Self::regs().ctrl().modify(|_, w| w.was().data());
    }

    fn regs() -> &'static crate::pac::crc0::RegisterBlock {
        unsafe { &*crate::pac::Crc0::ptr() }
    }

    /// Feeds a byte into the CRC peripheral.
    fn feed_byte(&mut self, byte: u8) {
        Self::regs().data8().write(|w| unsafe { w.bits(byte) });
    }

    /// Feeds a halfword into the CRC peripheral.
    fn feed_halfword(&mut self, halfword: u16) {
        Self::regs().data16().write(|w| unsafe { w.bits(halfword) });
    }

    /// Feeds a word into the CRC peripheral.
    fn feed_word(&mut self, word: u32) {
        Self::regs().data32().write(|w| unsafe { w.bits(word) });
    }
}

impl<'d> Crc<'d, Crc16> {
    /// Instantiates a new CRC peripheral driver in 16-bit mode
    pub fn new_crc16(peri: Peri<'d, CRC0>, config: Config) -> Self {
        let inst = Self::new_inner(peri);

        Self::configure(config, || {
            Self::regs().ctrl().modify(|_, w| w.tcrc().b16());
        });

        inst
    }

    /// Instantiates a new CRC peripheral driver for the given `Algorithm16`.
    pub fn new_algorithm16(peri: Peri<'d, CRC0>, algorithm: Algorithm16) -> Self {
        Self::new_crc16(peri, algorithm.into_config())
    }

    /// Instantiates a new CRC peripheral for the `A` algorithm.
    pub fn new_a(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::A)
    }

    /// Instantiates a new CRC peripheral for the `AugCcitt` algorithm.
    pub fn new_aug_ccitt(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::AugCcitt)
    }

    /// Instantiates a new CRC peripheral for the `Arc` algorithm.
    pub fn new_arc(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Arc)
    }

    /// Instantiates a new CRC peripheral for the `Buypass` algorithm.
    pub fn new_buypass(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Buypass)
    }

    /// Instantiates a new CRC peripheral for the `CcittFalse` algorithm.
    pub fn new_ccitt_false(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::CcittFalse)
    }

    /// Instantiates a new CRC peripheral for the `CcittZero` algorithm.
    pub fn new_ccitt_zero(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::CcittZero)
    }

    /// Instantiates a new CRC peripheral for the `Cdma2000` algorithm.
    pub fn new_cdma_2000(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Cdma2000)
    }

    /// Instantiates a new CRC peripheral for the `Dds110` algorithm.
    pub fn new_dds_110(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Dds110)
    }

    /// Instantiates a new CRC peripheral for the `DectX` algorithm.
    pub fn new_dect_x(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::DectX)
    }

    /// Instantiates a new CRC peripheral for the `Dnp` algorithm.
    pub fn new_dnp(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Dnp)
    }

    /// Instantiates a new CRC peripheral for the `En13757` algorithm.
    pub fn new_en13757(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::En13757)
    }

    /// Instantiates a new CRC peripheral for the `Genibus` algorithm.
    pub fn new_genibus(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Genibus)
    }

    /// Instantiates a new CRC peripheral for the `Kermit` algorithm.
    pub fn new_kermit(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Kermit)
    }

    /// Instantiates a new CRC peripheral for the `Maxim` algorithm.
    pub fn new_maxim(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Maxim)
    }

    /// Instantiates a new CRC peripheral for the `Mcrf4xx` algorithm.
    pub fn new_mcrf4xx(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Mcrf4xx)
    }

    /// Instantiates a new CRC peripheral for the `Modbus` algorithm.
    pub fn new_modbus(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Modbus)
    }

    /// Instantiates a new CRC peripheral for the `Riello` algorithm.
    pub fn new_riello(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Riello)
    }

    /// Instantiates a new CRC peripheral for the `T10Dif` algorithm.
    pub fn new_t10_dif(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::T10Dif)
    }

    /// Instantiates a new CRC peripheral for the `Teledisk` algorithm.
    pub fn new_teledisk(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Teledisk)
    }

    /// Instantiates a new CRC peripheral for the `Tms37157` algorithm.
    pub fn new_tms_37157(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Tms37157)
    }

    /// Instantiates a new CRC peripheral for the `Usb` algorithm.
    pub fn new_usb(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Usb)
    }

    /// Instantiates a new CRC peripheral for the `X25` algorithm.
    pub fn new_x25(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::X25)
    }

    /// Instantiates a new CRC peripheral for the `Xmodem` algorithm.
    pub fn new_xmodem(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm16(peri, Algorithm16::Xmodem)
    }

    fn read_crc(&mut self) -> u16 {
        // Reference manual states:
        //
        // "After writing all the data, you must wait for at least two
        // clock cycles to read the data from CRC Data (DATA)
        // register."
        cortex_m::asm::delay(2);

        let ctrl = Self::regs().ctrl().read();

        // if transposition is enabled, result sits in the upper 16 bits
        if ctrl.totr().is_byts_trnps() || ctrl.totr().is_byts_bts_trnps() {
            (Self::regs().data32().read().bits() >> 16) as u16
        } else {
            Self::regs().data16().read().bits()
        }
    }

    /// Feeds a slice of bytes into the CRC peripheral. Returns the computed checksum.
    pub fn feed_bytes(&mut self, bytes: &[u8]) -> u16 {
        let (prefix, data, suffix) = unsafe { bytes.align_to::<u32>() };

        for b in prefix {
            self.feed_byte(*b);
        }

        // use 32-bit writes as long as possible
        for w in data {
            self.feed_word(*w);
        }

        for b in suffix {
            self.feed_byte(*b);
        }

        // read back result.
        self.read_crc()
    }

    /// Feeds a slice of halfwords into the CRC peripheral. Returns the computed checksum.
    pub fn feed_halfwords(&mut self, halfwords: &[u16]) -> u16 {
        for halfword in halfwords {
            self.feed_halfword(*halfword);
        }

        self.read_crc()
    }
}

impl<'d> Crc<'d, Crc32> {
    /// Instantiates a new CRC peripheral driver in 32-bit mode
    pub fn new_crc32(peri: Peri<'d, CRC0>, config: Config) -> Self {
        let inst = Self::new_inner(peri);

        Self::configure(config, || {
            Self::regs().ctrl().modify(|_, w| w.tcrc().b32());
        });

        inst
    }

    /// Instantiates a new CRC peripheral driver for the given `Algorithm32`.
    pub fn new_algorithm32(peri: Peri<'d, CRC0>, algorithm: Algorithm32) -> Self {
        Self::new_crc32(peri, algorithm.into_config())
    }

    /// Instantiates a new CRC peripheral for the `Bzip2` algorithm.
    pub fn new_bzip2(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::Bzip2)
    }

    /// Instantiates a new CRC peripheral for the `C` algorithm.
    pub fn new_c(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::C)
    }

    /// Instantiates a new CRC peripheral for the `D` algorithm.
    pub fn new_d(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::D)
    }

    /// Instantiates a new CRC peripheral for the `IsoHdlc` algorithm.
    pub fn new_iso_hdlc(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::IsoHdlc)
    }

    /// Instantiates a new CRC peripheral for the `JamCrc` algorithm.
    pub fn new_jam_crc(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::JamCrc)
    }

    /// Instantiates a new CRC peripheral for the `Mpeg2` algorithm.
    pub fn new_mpeg2(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::Mpeg2)
    }

    /// Instantiates a new CRC peripheral for the `Posix` algorithm.
    pub fn new_posix(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::Posix)
    }

    /// Instantiates a new CRC peripheral for the `Q` algorithm.
    pub fn new_q(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::Q)
    }

    /// Instantiates a new CRC peripheral for the `Xfer` algorithm.
    pub fn new_xfer(peri: Peri<'d, CRC0>) -> Self {
        Self::new_algorithm32(peri, Algorithm32::Xfer)
    }

    fn read_crc(&mut self) -> u32 {
        // Reference manual states:
        //
        // "After writing all the data, you must wait for at least two
        // clock cycles to read the data from CRC Data (DATA)
        // register."
        cortex_m::asm::delay(2);
        Self::regs().data32().read().bits()
    }

    /// Feeds a slice of bytes into the CRC peripheral. Returns the computed checksum.
    pub fn feed_bytes(&mut self, bytes: &[u8]) -> u32 {
        let (prefix, data, suffix) = unsafe { bytes.align_to::<u32>() };

        for b in prefix {
            self.feed_byte(*b);
        }

        // use 32-bit writes as long as possible
        for w in data {
            self.feed_word(*w);
        }

        for b in suffix {
            self.feed_byte(*b);
        }

        // read back result.
        self.read_crc()
    }

    /// Feeds a slice of halfwords into the CRC peripheral. Returns the computed checksum.
    pub fn feed_halfwords(&mut self, halfwords: &[u16]) -> u32 {
        for halfword in halfwords {
            self.feed_halfword(*halfword);
        }

        self.read_crc()
    }

    /// Feeds a slice of words into the CRC peripheral. Returns the computed checksum.
    pub fn feed_words(&mut self, words: &[u32]) -> u32 {
        for word in words {
            self.feed_word(*word);
        }

        self.read_crc()
    }
}

mod sealed {
    pub trait SealedMode {}
}

/// Mode of operation: 32 or 16-bit CRC.
#[allow(private_bounds)]
pub trait Mode: sealed::SealedMode {}

/// 16-bit CRC.
pub struct Crc16;
impl sealed::SealedMode for Crc16 {}
impl Mode for Crc16 {}

/// 32-bit CRC.
pub struct Crc32;
impl sealed::SealedMode for Crc32 {}
impl Mode for Crc32 {}

/// CRC configuration.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct Config {
    /// The CRC polynomial to be used.
    pub polynomial: u32,

    /// Reflect bit order of input?
    pub reflect_in: Reflect,

    /// Reflect CRC bit order?
    pub reflect_out: Reflect,

    /// 1's complement CRC?
    pub complement_out: Complement,

    /// CRC Seed
    pub seed: u32,
}

impl Config {
    /// Create a new CRC config.
    #[must_use]
    pub fn new(
        polynomial: u32,
        reflect_in: Reflect,
        reflect_out: Reflect,
        complement_out: Complement,
        seed: u32,
    ) -> Self {
        Config {
            polynomial,
            reflect_in,
            reflect_out,
            complement_out,
            seed,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            polynomial: 0,
            reflect_in: Reflect::No,
            reflect_out: Reflect::No,
            complement_out: Complement::No,
            seed: 0xffff_ffff,
        }
    }
}

/// Supported standard CRC16 algorithms.
#[derive(Copy, Clone, Debug)]
pub enum Algorithm16 {
    A,
    Arc,
    AugCcitt,
    Buypass,
    CcittFalse,
    CcittZero,
    Cdma2000,
    Dds110,
    DectX,
    Dnp,
    En13757,
    Genibus,
    Kermit,
    Maxim,
    Mcrf4xx,
    Modbus,
    Riello,
    T10Dif,
    Teledisk,
    Tms37157,
    Usb,
    X25,
    Xmodem,
}

impl Algorithm16 {
    fn into_config(self) -> Config {
        match self {
            Algorithm16::A => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0xc6c6,
            },
            Algorithm16::Arc => Config {
                polynomial: 0x8005,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0,
            },
            Algorithm16::AugCcitt => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0x1d0f,
            },
            Algorithm16::Buypass => Config {
                polynomial: 0x8005,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0,
            },
            Algorithm16::CcittFalse => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0xffff,
            },
            Algorithm16::CcittZero => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0,
            },
            Algorithm16::Cdma2000 => Config {
                polynomial: 0xc867,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0xffff,
            },
            Algorithm16::Dds110 => Config {
                polynomial: 0x8005,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0x800d,
            },
            Algorithm16::DectX => Config {
                polynomial: 0x0589,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0,
            },
            Algorithm16::Dnp => Config {
                polynomial: 0x3d65,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::Yes,
                seed: 0,
            },
            Algorithm16::En13757 => Config {
                polynomial: 0x3d65,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::Yes,
                seed: 0,
            },
            Algorithm16::Genibus => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::Yes,
                seed: 0xffff,
            },
            Algorithm16::Kermit => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0,
            },
            Algorithm16::Maxim => Config {
                polynomial: 0x8005,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::Yes,
                seed: 0,
            },
            Algorithm16::Mcrf4xx => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0xffff,
            },
            Algorithm16::Modbus => Config {
                polynomial: 0x8005,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0xffff,
            },
            Algorithm16::Riello => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0xb2aa,
            },
            Algorithm16::T10Dif => Config {
                polynomial: 0x8bb7,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0,
            },
            Algorithm16::Teledisk => Config {
                polynomial: 0xa097,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0,
            },
            Algorithm16::Tms37157 => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0x89ec,
            },
            Algorithm16::Usb => Config {
                polynomial: 0x8005,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0xffff,
            },
            Algorithm16::X25 => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::Yes,
                seed: 0xffff,
            },
            Algorithm16::Xmodem => Config {
                polynomial: 0x1021,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0,
            },
        }
    }
}

/// Supported standard CRC32 algorithms.
#[derive(Copy, Clone, Debug)]
pub enum Algorithm32 {
    Bzip2,
    C,
    D,
    IsoHdlc,
    JamCrc,
    Mpeg2,
    Posix,
    Q,
    Xfer,
}

impl Algorithm32 {
    fn into_config(self) -> Config {
        match self {
            Algorithm32::Bzip2 => Config {
                polynomial: 0x04c1_1db7,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::Yes,
                seed: 0xffff_ffff,
            },
            Algorithm32::C => Config {
                polynomial: 0x1edc_6f41,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::Yes,
                seed: 0xffff_ffff,
            },
            Algorithm32::D => Config {
                polynomial: 0xa833_982b,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::Yes,
                seed: 0xffff_ffff,
            },
            Algorithm32::IsoHdlc => Config {
                polynomial: 0x04c1_1db7,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::Yes,
                seed: 0xffff_ffff,
            },
            Algorithm32::JamCrc => Config {
                polynomial: 0x04c1_1db7,
                reflect_in: Reflect::Yes,
                reflect_out: Reflect::Yes,
                complement_out: Complement::No,
                seed: 0xffff_ffff,
            },
            Algorithm32::Mpeg2 => Config {
                polynomial: 0x04c1_1db7,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0xffff_ffff,
            },
            Algorithm32::Posix => Config {
                polynomial: 0x04c1_1db7,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::Yes,
                seed: 0,
            },
            Algorithm32::Q => Config {
                polynomial: 0x8141_41ab,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0,
            },
            Algorithm32::Xfer => Config {
                polynomial: 0x0000_00af,
                reflect_in: Reflect::No,
                reflect_out: Reflect::No,
                complement_out: Complement::No,
                seed: 0,
            },
        }
    }
}

/// Reflect bit order.
#[derive(Copy, Clone, Debug)]
pub enum Reflect {
    No,
    Yes,
}

impl From<Reflect> for Tot {
    fn from(value: Reflect) -> Tot {
        match value {
            Reflect::No => Tot::BytsTrnps,
            Reflect::Yes => Tot::BytsBtsTrnps,
        }
    }
}

impl From<Reflect> for Totr {
    fn from(value: Reflect) -> Totr {
        match value {
            Reflect::No => Totr::Notrnps,
            Reflect::Yes => Totr::BytsBtsTrnps,
        }
    }
}

/// 1's complement output.
#[derive(Copy, Clone, Debug)]
pub enum Complement {
    No,
    Yes,
}

impl From<Complement> for Fxor {
    fn from(value: Complement) -> Fxor {
        match value {
            Complement::No => Fxor::Noxor,
            Complement::Yes => Fxor::Invert,
        }
    }
}
