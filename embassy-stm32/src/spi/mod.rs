#![macro_use]

use crate::dma;
use crate::gpio::sealed::{AFType, Pin};
use crate::gpio::{AnyPin, NoPin, OptionalPin};
use crate::pac::spi::{regs, vals};
use crate::peripherals;
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

#[cfg_attr(spi_v1, path = "v1.rs")]
#[cfg_attr(spi_f1, path = "v1.rs")]
#[cfg_attr(spi_v2, path = "v2.rs")]
#[cfg_attr(spi_v3, path = "v3.rs")]
mod _version;
pub use _version::*;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Framing,
    Crc,
    ModeFault,
    Overrun,
}

// TODO move upwards in the tree
pub enum ByteOrder {
    LsbFirst,
    MsbFirst,
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
enum WordSize {
    EightBit,
    SixteenBit,
}

impl WordSize {
    #[cfg(any(spi_v1, spi_f1))]
    fn dff(&self) -> vals::Dff {
        match self {
            WordSize::EightBit => vals::Dff::EIGHTBIT,
            WordSize::SixteenBit => vals::Dff::SIXTEENBIT,
        }
    }

    #[cfg(spi_v2)]
    fn ds(&self) -> vals::Ds {
        match self {
            WordSize::EightBit => vals::Ds::EIGHTBIT,
            WordSize::SixteenBit => vals::Ds::SIXTEENBIT,
        }
    }

    #[cfg(spi_v2)]
    fn frxth(&self) -> vals::Frxth {
        match self {
            WordSize::EightBit => vals::Frxth::QUARTER,
            WordSize::SixteenBit => vals::Frxth::HALF,
        }
    }

    #[cfg(spi_v3)]
    fn dsize(&self) -> u8 {
        match self {
            WordSize::EightBit => 0b0111,
            WordSize::SixteenBit => 0b1111,
        }
    }

    #[cfg(spi_v3)]
    fn _frxth(&self) -> vals::Fthlv {
        match self {
            WordSize::EightBit => vals::Fthlv::ONEFRAME,
            WordSize::SixteenBit => vals::Fthlv::ONEFRAME,
        }
    }
}

#[non_exhaustive]
pub struct Config {
    pub mode: Mode,
    pub byte_order: ByteOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_0,
            byte_order: ByteOrder::MsbFirst,
        }
    }
}

pub struct Spi<'d, T: Instance, Tx, Rx> {
    sck: Option<AnyPin>,
    mosi: Option<AnyPin>,
    miso: Option<AnyPin>,
    txdma: Tx,
    rxdma: Rx,
    current_word_size: WordSize,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    pub fn new<F>(
        _peri: impl Unborrow<Target = T> + 'd,
        sck: impl Unborrow<Target = impl SckPin<T>>,
        mosi: impl Unborrow<Target = impl MosiPin<T>>,
        miso: impl Unborrow<Target = impl MisoPin<T>>,
        txdma: impl Unborrow<Target = Tx>,
        rxdma: impl Unborrow<Target = Rx>,
        freq: F,
        config: Config,
    ) -> Self
    where
        F: Into<Hertz>,
    {
        unborrow!(sck, mosi, miso, txdma, rxdma);

        let sck_af = sck.af_num();
        let mosi_af = mosi.af_num();
        let miso_af = miso.af_num();
        let sck = sck.degrade_optional();
        let mosi = mosi.degrade_optional();
        let miso = miso.degrade_optional();

        unsafe {
            sck.as_ref().map(|x| {
                x.set_as_af(sck_af, AFType::OutputPushPull);
                #[cfg(any(spi_v2, spi_v3))]
                x.set_speed(crate::gpio::Speed::VeryHigh);
            });
            mosi.as_ref().map(|x| {
                x.set_as_af(mosi_af, AFType::OutputPushPull);
                #[cfg(any(spi_v2, spi_v3))]
                x.set_speed(crate::gpio::Speed::VeryHigh);
            });
            miso.as_ref().map(|x| {
                x.set_as_af(miso_af, AFType::Input);
                #[cfg(any(spi_v2, spi_v3))]
                x.set_speed(crate::gpio::Speed::VeryHigh);
            });
        }

        let pclk = T::frequency();
        let br = Self::compute_baud_rate(pclk, freq.into());

        #[cfg(any(spi_v1, spi_f1))]
        unsafe {
            T::enable();
            T::reset();
            T::regs().cr2().modify(|w| {
                w.set_ssoe(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_cpha(
                    match config.mode.phase == Phase::CaptureOnSecondTransition {
                        true => vals::Cpha::SECONDEDGE,
                        false => vals::Cpha::FIRSTEDGE,
                    },
                );
                w.set_cpol(match config.mode.polarity == Polarity::IdleHigh {
                    true => vals::Cpol::IDLEHIGH,
                    false => vals::Cpol::IDLELOW,
                });

                w.set_mstr(vals::Mstr::MASTER);
                w.set_br(vals::Br(br));
                w.set_spe(true);
                w.set_lsbfirst(match config.byte_order {
                    ByteOrder::LsbFirst => vals::Lsbfirst::LSBFIRST,
                    ByteOrder::MsbFirst => vals::Lsbfirst::MSBFIRST,
                });
                w.set_ssi(true);
                w.set_ssm(true);
                w.set_crcen(false);
                w.set_bidimode(vals::Bidimode::UNIDIRECTIONAL);
                if mosi.is_none() {
                    w.set_rxonly(vals::Rxonly::OUTPUTDISABLED);
                }
                w.set_dff(WordSize::EightBit.dff())
            });
        }
        #[cfg(spi_v2)]
        unsafe {
            T::enable();
            T::reset();
            T::regs().cr2().modify(|w| {
                w.set_frxth(WordSize::EightBit.frxth());
                w.set_ds(WordSize::EightBit.ds());
                w.set_ssoe(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_cpha(
                    match config.mode.phase == Phase::CaptureOnSecondTransition {
                        true => vals::Cpha::SECONDEDGE,
                        false => vals::Cpha::FIRSTEDGE,
                    },
                );
                w.set_cpol(match config.mode.polarity == Polarity::IdleHigh {
                    true => vals::Cpol::IDLEHIGH,
                    false => vals::Cpol::IDLELOW,
                });

                w.set_mstr(vals::Mstr::MASTER);
                w.set_br(vals::Br(br));
                w.set_lsbfirst(match config.byte_order {
                    ByteOrder::LsbFirst => vals::Lsbfirst::LSBFIRST,
                    ByteOrder::MsbFirst => vals::Lsbfirst::MSBFIRST,
                });
                w.set_ssi(true);
                w.set_ssm(true);
                w.set_crcen(false);
                w.set_bidimode(vals::Bidimode::UNIDIRECTIONAL);
                w.set_spe(true);
            });
        }
        #[cfg(spi_v3)]
        unsafe {
            T::enable();
            T::reset();
            T::regs().ifcr().write(|w| w.0 = 0xffff_ffff);
            T::regs().cfg2().modify(|w| {
                //w.set_ssoe(true);
                w.set_ssoe(false);
                w.set_cpha(
                    match config.mode.phase == Phase::CaptureOnSecondTransition {
                        true => vals::Cpha::SECONDEDGE,
                        false => vals::Cpha::FIRSTEDGE,
                    },
                );
                w.set_cpol(match config.mode.polarity == Polarity::IdleHigh {
                    true => vals::Cpol::IDLEHIGH,
                    false => vals::Cpol::IDLELOW,
                });
                w.set_lsbfrst(match config.byte_order {
                    ByteOrder::LsbFirst => vals::Lsbfrst::LSBFIRST,
                    ByteOrder::MsbFirst => vals::Lsbfrst::MSBFIRST,
                });
                w.set_ssm(true);
                w.set_master(vals::Master::MASTER);
                w.set_comm(vals::Comm::FULLDUPLEX);
                w.set_ssom(vals::Ssom::ASSERTED);
                w.set_midi(0);
                w.set_mssi(0);
                w.set_afcntr(vals::Afcntr::CONTROLLED);
                w.set_ssiop(vals::Ssiop::ACTIVEHIGH);
            });
            T::regs().cfg1().modify(|w| {
                w.set_crcen(false);
                w.set_mbr(vals::Mbr(br));
                w.set_dsize(WordSize::EightBit.dsize());
            });
            T::regs().cr2().modify(|w| {
                w.set_tsize(0);
                w.set_tser(0);
            });
            T::regs().cr1().modify(|w| {
                w.set_ssi(false);
                w.set_spe(true);
            });
        }

        Self {
            sck,
            mosi,
            miso,
            txdma,
            rxdma,
            current_word_size: WordSize::EightBit,
            phantom: PhantomData,
        }
    }

    fn compute_baud_rate(clocks: Hertz, freq: Hertz) -> u8 {
        match clocks.0 / freq.0 {
            0 => unreachable!(),
            1..=2 => 0b000,
            3..=5 => 0b001,
            6..=11 => 0b010,
            12..=23 => 0b011,
            24..=39 => 0b100,
            40..=95 => 0b101,
            96..=191 => 0b110,
            _ => 0b111,
        }
    }

    fn set_word_size(&mut self, word_size: WordSize) {
        if self.current_word_size == word_size {
            return;
        }

        #[cfg(any(spi_v1, spi_f1))]
        unsafe {
            T::regs().cr1().modify(|reg| {
                reg.set_spe(false);
                reg.set_dff(word_size.dff())
            });
            T::regs().cr1().modify(|reg| {
                reg.set_spe(true);
            });
        }
        #[cfg(spi_v2)]
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cr2().modify(|w| {
                w.set_frxth(word_size.frxth());
                w.set_ds(word_size.ds());
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }
        #[cfg(spi_v3)]
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_csusp(true);
            });
            while T::regs().sr().read().eot() {}
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cfg1().modify(|w| {
                w.set_dsize(word_size.dsize());
            });
            T::regs().cr1().modify(|w| {
                w.set_csusp(false);
                w.set_spe(true);
            });
        }

        self.current_word_size = word_size;
    }
}

impl<'d, T: Instance, Tx, Rx> Drop for Spi<'d, T, Tx, Rx> {
    fn drop(&mut self) {
        unsafe {
            self.sck.as_ref().map(|x| x.set_as_analog());
            self.mosi.as_ref().map(|x| x.set_as_analog());
            self.miso.as_ref().map(|x| x.set_as_analog());
        }
    }
}

trait RegsExt {
    fn tx_ptr<W>(&self) -> *mut W;
    fn rx_ptr<W>(&self) -> *mut W;
}

impl RegsExt for crate::pac::spi::Spi {
    fn tx_ptr<W>(&self) -> *mut W {
        #[cfg(not(spi_v3))]
        let dr = self.dr();
        #[cfg(spi_v3)]
        let dr = self.txdr();
        dr.ptr() as *mut W
    }

    fn rx_ptr<W>(&self) -> *mut W {
        #[cfg(not(spi_v3))]
        let dr = self.dr();
        #[cfg(spi_v3)]
        let dr = self.rxdr();
        dr.ptr() as *mut W
    }
}

fn check_error_flags(sr: regs::Sr) -> Result<(), Error> {
    if sr.ovr() {
        return Err(Error::Overrun);
    }
    #[cfg(not(any(spi_f1, spi_v3)))]
    if sr.fre() {
        return Err(Error::Framing);
    }
    #[cfg(spi_v3)]
    if sr.tifre() {
        return Err(Error::Framing);
    }
    if sr.modf() {
        return Err(Error::ModeFault);
    }
    #[cfg(not(spi_v3))]
    if sr.crcerr() {
        return Err(Error::Crc);
    }
    #[cfg(spi_v3)]
    if sr.crce() {
        return Err(Error::Crc);
    }

    Ok(())
}

trait Word {}

impl Word for u8 {}
impl Word for u16 {}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static crate::pac::spi::Spi;
    }

    pub trait SckPin<T: Instance>: OptionalPin {
        fn af_num(&self) -> u8;
    }

    pub trait MosiPin<T: Instance>: OptionalPin {
        fn af_num(&self) -> u8;
    }

    pub trait MisoPin<T: Instance>: OptionalPin {
        fn af_num(&self) -> u8;
    }

    pub trait TxDmaChannel<T: Instance> {
        fn request(&self) -> dma::Request;
    }

    pub trait RxDmaChannel<T: Instance> {
        fn request(&self) -> dma::Request;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {}
pub trait SckPin<T: Instance>: sealed::SckPin<T> {}
pub trait MosiPin<T: Instance>: sealed::MosiPin<T> {}
pub trait MisoPin<T: Instance>: sealed::MisoPin<T> {}
pub trait TxDmaChannel<T: Instance>: sealed::TxDmaChannel<T> + dma::Channel {}
pub trait RxDmaChannel<T: Instance>: sealed::RxDmaChannel<T> + dma::Channel {}

crate::pac::peripherals!(
    (spi, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::spi::Spi {
                &crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {}
    };
);

macro_rules! impl_pin {
    ($inst:ident, $pin:ident, $signal:ident, $af:expr) => {
        impl $signal<peripherals::$inst> for peripherals::$pin {}

        impl sealed::$signal<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
}

#[cfg(not(rcc_f1))]
crate::pac::peripheral_pins!(
    ($inst:ident, spi, SPI, $pin:ident, SCK, $af:expr) => {
        impl_pin!($inst, $pin, SckPin, $af);
    };

    ($inst:ident, spi, SPI, $pin:ident, MOSI, $af:expr) => {
        impl_pin!($inst, $pin, MosiPin, $af);
    };

    ($inst:ident, spi, SPI, $pin:ident, MISO, $af:expr) => {
        impl_pin!($inst, $pin, MisoPin, $af);
    };
);

#[cfg(rcc_f1)]
crate::pac::peripheral_pins!(
    ($inst:ident, spi, SPI, $pin:ident, SCK) => {
        impl_pin!($inst, $pin, SckPin, 0);
    };

    ($inst:ident, spi, SPI, $pin:ident, MOSI) => {
        impl_pin!($inst, $pin, MosiPin, 0);
    };

    ($inst:ident, spi, SPI, $pin:ident, MISO) => {
        impl_pin!($inst, $pin, MisoPin, 0);
    };
);

macro_rules! impl_nopin {
    ($inst:ident, $signal:ident) => {
        impl $signal<peripherals::$inst> for NoPin {}

        impl sealed::$signal<peripherals::$inst> for NoPin {
            fn af_num(&self) -> u8 {
                0
            }
        }
    };
}

crate::pac::peripherals!(
    (spi, $inst:ident) => {
        impl_nopin!($inst, SckPin);
        impl_nopin!($inst, MosiPin);
        impl_nopin!($inst, MisoPin);
    };
);

macro_rules! impl_dma {
    ($inst:ident, {dmamux: $dmamux:ident}, $signal:ident, $request:expr) => {
        impl<T> sealed::$signal<peripherals::$inst> for T
        where
            T: crate::dma::MuxChannel<Mux = crate::dma::$dmamux>,
        {
            fn request(&self) -> dma::Request {
                $request
            }
        }

        impl<T> $signal<peripherals::$inst> for T where
            T: crate::dma::MuxChannel<Mux = crate::dma::$dmamux>
        {
        }
    };
    ($inst:ident, {channel: $channel:ident}, $signal:ident, $request:expr) => {
        impl sealed::$signal<peripherals::$inst> for peripherals::$channel {
            fn request(&self) -> dma::Request {
                $request
            }
        }

        impl $signal<peripherals::$inst> for peripherals::$channel {}
    };
}

crate::pac::peripheral_dma_channels! {
    ($peri:ident, spi, $kind:ident, RX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, RxDmaChannel, $request);
    };
    ($peri:ident, spi, $kind:ident, TX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, TxDmaChannel, $request);
    };
}
