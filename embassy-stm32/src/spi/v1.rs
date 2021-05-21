#![macro_use]

use crate::gpio::{sealed::Pin, AnyPin};
use crate::pac::spi;
use crate::spi::{ByteOrder, Config, Error, Instance, MisoPin, MosiPin, SckPin, WordSize};
use crate::time::Hertz;
use core::marker::PhantomData;
use core::ptr;
use embassy::util::Unborrow;
use embassy_extras::unborrow;
pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

impl WordSize {
    fn dff(&self) -> spi::vals::Dff {
        match self {
            WordSize::EightBit => spi::vals::Dff::EIGHTBIT,
            WordSize::SixteenBit => spi::vals::Dff::SIXTEENBIT,
        }
    }
}

pub struct Spi<'d, T: Instance> {
    //peri: T,
    sck: AnyPin,
    mosi: AnyPin,
    miso: AnyPin,
    current_word_size: WordSize,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Spi<'d, T> {
    pub fn new<F>(
        pclk: Hertz,
        peri: impl Unborrow<Target = T> + 'd,
        sck: impl Unborrow<Target = impl SckPin<T>>,
        mosi: impl Unborrow<Target = impl MosiPin<T>>,
        miso: impl Unborrow<Target = impl MisoPin<T>>,
        freq: F,
        config: Config,
    ) -> Self
    where
        F: Into<Hertz>,
    {
        unborrow!(peri, sck, mosi, miso);

        unsafe {
            sck.set_as_af(sck.af_num());
            mosi.set_as_af(mosi.af_num());
            miso.set_as_af(miso.af_num());
        }

        let sck = sck.degrade();
        let mosi = mosi.degrade();
        let miso = miso.degrade();

        unsafe {
            T::regs().cr2().modify(|w| {
                w.set_ssoe(false);
            });
        }

        let br = Self::compute_baud_rate(pclk, freq.into());

        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_cpha(
                    match config.mode.phase == Phase::CaptureOnSecondTransition {
                        true => spi::vals::Cpha::SECONDEDGE,
                        false => spi::vals::Cpha::FIRSTEDGE,
                    },
                );
                w.set_cpol(match config.mode.polarity == Polarity::IdleHigh {
                    true => spi::vals::Cpol::IDLEHIGH,
                    false => spi::vals::Cpol::IDLELOW,
                });

                w.set_mstr(spi::vals::Mstr::MASTER);
                w.set_br(spi::vals::Br(br));
                w.set_spe(true);
                w.set_lsbfirst(match config.byte_order {
                    ByteOrder::LsbFirst => spi::vals::Lsbfirst::LSBFIRST,
                    ByteOrder::MsbFirst => spi::vals::Lsbfirst::MSBFIRST,
                });
                w.set_ssi(true);
                w.set_ssm(true);
                w.set_crcen(false);
                w.set_bidimode(spi::vals::Bidimode::UNIDIRECTIONAL);
                w.set_dff(WordSize::EightBit.dff())
            });
        }

        Self {
            //peri,
            sck,
            mosi,
            miso,
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
        unsafe {
            T::regs().cr1().modify(|reg| {
                reg.set_spe(false);
                reg.set_dff(word_size.dff())
            });
            T::regs().cr1().modify(|reg| {
                reg.set_spe(true);
            });
            self.current_word_size = word_size;
        }
    }
}

impl<'d, T: Instance> Drop for Spi<'d, T> {
    fn drop(&mut self) {
        unsafe {
            self.sck.set_as_analog();
            self.mosi.set_as_analog();
            self.miso.set_as_analog();
        }
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Write<u8> for Spi<'d, T> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.set_word_size(WordSize::EightBit);
        let regs = T::regs();

        for word in words.iter() {
            while unsafe { !regs.sr().read().txe() } {
                // spin
            }
            unsafe {
                let dr = regs.dr().ptr() as *mut u8;
                ptr::write_volatile(dr, *word);
            }
            loop {
                let sr = unsafe { regs.sr().read() };
                if sr.fre() {
                    return Err(Error::Framing);
                }
                if sr.ovr() {
                    return Err(Error::Overrun);
                }
                if sr.crcerr() {
                    return Err(Error::Crc);
                }
                if !sr.txe() {
                    // loop waiting for TXE
                }
            }
        }

        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Transfer<u8> for Spi<'d, T> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.set_word_size(WordSize::EightBit);
        let regs = T::regs();

        for word in words.iter_mut() {
            while unsafe { !regs.sr().read().txe() } {
                // spin
            }
            unsafe {
                let dr = regs.dr().ptr() as *mut u8;
                ptr::write_volatile(dr, *word);
            }

            while unsafe { !regs.sr().read().rxne() } {
                // spin waiting for inbound to shift in.
            }

            unsafe {
                let dr = regs.dr().ptr() as *const u8;
                *word = ptr::read_volatile(dr);
            }

            let sr = unsafe { regs.sr().read() };
            if sr.fre() {
                return Err(Error::Framing);
            }
            if sr.ovr() {
                return Err(Error::Overrun);
            }
            if sr.crcerr() {
                return Err(Error::Crc);
            }
        }

        Ok(words)
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Write<u16> for Spi<'d, T> {
    type Error = Error;

    fn write(&mut self, words: &[u16]) -> Result<(), Self::Error> {
        self.set_word_size(WordSize::SixteenBit);
        let regs = T::regs();

        for word in words.iter() {
            while unsafe { !regs.sr().read().txe() } {
                // spin
            }
            unsafe {
                let dr = regs.dr().ptr() as *mut u16;
                ptr::write_volatile(dr, *word);
            }
            loop {
                let sr = unsafe { regs.sr().read() };
                if sr.fre() {
                    return Err(Error::Framing);
                }
                if sr.ovr() {
                    return Err(Error::Overrun);
                }
                if sr.crcerr() {
                    return Err(Error::Crc);
                }
                if !sr.txe() {
                    // loop waiting for TXE
                }
            }
        }

        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Transfer<u16> for Spi<'d, T> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u16]) -> Result<&'w [u16], Self::Error> {
        self.set_word_size(WordSize::SixteenBit);
        let regs = T::regs();

        for word in words.iter_mut() {
            while unsafe { !regs.sr().read().txe() } {
                // spin
            }
            unsafe {
                let dr = regs.dr().ptr() as *mut u16;
                ptr::write_volatile(dr, *word);
            }
            while unsafe { !regs.sr().read().rxne() } {
                // spin waiting for inbound to shift in.
            }
            unsafe {
                let dr = regs.dr().ptr() as *const u16;
                *word = ptr::read_volatile(dr);
            }

            let sr = unsafe { regs.sr().read() };
            if sr.fre() {
                return Err(Error::Framing);
            }
            if sr.ovr() {
                return Err(Error::Overrun);
            }
            if sr.crcerr() {
                return Err(Error::Crc);
            }
        }

        Ok(words)
    }
}
