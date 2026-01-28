//! Cyclic Redundancy Check (CRC)

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use mcxa_pac::crc0::ctrl::{Fxor, Tcrc, Tot, Totr};

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
        // NoConfig? No WakeGuard!
        _ = unsafe { enable_and_reset::<CRC0>(&NoConfig) };

        Crc {
            _peri,
            _phantom: PhantomData,
        }
    }

    // Configure the underlying peripheral according to the reference manual.
    fn configure(config: Config, width: Tcrc) {
        Self::regs().ctrl().modify(|_, w| {
            w.fxor()
                .variant(config.complement_out.into())
                .totr()
                .variant(config.reflect_out.into())
                .tot()
                .variant(config.reflect_in.into())
                .was()
                .data()
                .tcrc()
                .variant(width)
        });

        Self::regs().gpoly32().write(|w| unsafe { w.bits(config.polynomial) });

        Self::regs().ctrl().modify(|_, w| w.was().seed());
        Self::regs().data32().write(|w| unsafe { w.bits(config.seed) });
        Self::regs().ctrl().modify(|_, w| w.was().data());
    }

    fn regs() -> &'static crate::pac::crc0::RegisterBlock {
        unsafe { &*crate::pac::Crc0::ptr() }
    }

    /// Read the computed CRC value
    fn finalize_inner<W: Word>(self) -> W {
        // Reference manual states:
        //
        // "After writing all the data, you must wait for at least two
        // clock cycles to read the data from CRC Data (DATA)
        // register."
        cortex_m::asm::delay(2);
        W::read(Self::regs())
    }

    fn feed_word<W: Word>(&mut self, word: W) {
        W::write(Self::regs(), word);
    }

    /// Feeds a slice of `Word`s into the CRC peripheral. Returns the computed
    /// checksum.
    ///
    /// The input is strided efficiently into as many `u32`s as possible,
    /// falling back to smaller writes for the remainder.
    fn feed_inner<W: Word>(&mut self, data: &[W]) {
        let (prefix, aligned, suffix) = unsafe { data.align_to::<u32>() };

        for w in prefix {
            self.feed_word(*w);
        }

        for w in aligned {
            self.feed_word(*w);
        }

        for w in suffix {
            self.feed_word(*w);
        }
    }
}

impl<'d> Crc<'d, Crc16> {
    /// Instantiates a new CRC peripheral driver in 16-bit mode
    pub fn new_crc16(peri: Peri<'d, CRC0>, config: Config) -> Self {
        let inst = Self::new_inner(peri);
        Self::configure(config, Tcrc::B16);
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

    /// Feeds a slice of `Word`s into the CRC peripheral.
    ///
    /// The input is strided efficiently into as many `u32`s as possible,
    /// falling back to smaller writes for the remainder.
    pub fn feed<W: Word>(&mut self, data: &[W]) {
        self.feed_inner(data);
    }

    /// Finalizes the CRC calculation and reads the resulting CRC from the
    /// hardware consuming `self`.
    pub fn finalize(self) -> u16 {
        self.finalize_inner()
    }
}

impl<'d> Crc<'d, Crc32> {
    /// Instantiates a new CRC peripheral driver in 32-bit mode
    pub fn new_crc32(peri: Peri<'d, CRC0>, config: Config) -> Self {
        let inst = Self::new_inner(peri);
        Self::configure(config, Tcrc::B32);
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

    /// Feeds a slice of `Word`s into the CRC peripheral.
    ///
    /// The input is strided efficiently into as many `u32`s as possible,
    /// falling back to smaller writes for the remainder.
    pub fn feed<W: Word>(&mut self, data: &[W]) {
        self.feed_inner(data);
    }

    /// Finalizes the CRC calculation and reads the resulting CRC from the
    /// hardware consuming `self`.
    pub fn finalize(self) -> u32 {
        self.finalize_inner()
    }
}

mod sealed {
    pub trait SealedMode {}

    pub trait SealedWord: Copy {
        fn write(regs: &'static crate::pac::crc0::RegisterBlock, word: Self);
        fn read(regs: &'static crate::pac::crc0::RegisterBlock) -> Self;
    }
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

/// Word size for the CRC.
#[allow(private_bounds)]
pub trait Word: sealed::SealedWord {}

macro_rules! impl_word {
    ($t:ty, $width:literal, $write:expr, $read:expr) => {
        impl sealed::SealedWord for $t {
            #[inline]
            fn write(regs: &'static crate::pac::crc0::RegisterBlock, word: Self) {
                $write(regs, word)
            }

            #[inline]
            fn read(regs: &'static crate::pac::crc0::RegisterBlock) -> Self {
                $read(regs)
            }
        }

        impl Word for $t {}
    };
}

impl_word!(u8, 8, write_u8, read_u8);
impl_word!(u16, 16, write_u16, read_u16);
impl_word!(u32, 32, write_u32, read_u32);

fn write_u8(regs: &'static crate::pac::crc0::RegisterBlock, word: u8) {
    regs.data8().write(|w| unsafe { w.bits(word) });
}

fn read_u8(regs: &'static crate::pac::crc0::RegisterBlock) -> u8 {
    regs.data8().read().bits()
}

fn write_u16(regs: &'static crate::pac::crc0::RegisterBlock, word: u16) {
    regs.data16().write(|w| unsafe { w.bits(word) });
}

fn read_u16(regs: &'static crate::pac::crc0::RegisterBlock) -> u16 {
    let ctrl = regs.ctrl().read();

    // if transposition is enabled, result sits in the upper 16 bits
    if ctrl.totr().is_byts_trnps() || ctrl.totr().is_byts_bts_trnps() {
        (regs.data32().read().bits() >> 16) as u16
    } else {
        regs.data16().read().bits()
    }
}

fn write_u32(regs: &'static crate::pac::crc0::RegisterBlock, word: u32) {
    regs.data32().write(|w| unsafe { w.bits(word) });
}

fn read_u32(regs: &'static crate::pac::crc0::RegisterBlock) -> u32 {
    regs.data32().read().bits()
}

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
