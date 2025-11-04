//! CRACEN - Cryptographic Accelerator Engine driver.

#![macro_use]

use core::marker::PhantomData;

use crate::mode::{Blocking, Mode};
use crate::{Peri, interrupt, pac, peripherals};

/// A wrapper around an nRF54 CRACEN peripheral.
///
/// It has a blocking api through `rand`.
pub struct Cracen<'d, M: Mode> {
    _peri: Peri<'d, peripherals::CRACEN>,
    _p: PhantomData<M>,
}

impl<'d> Cracen<'d, Blocking> {
    /// Create a new CRACEN driver.
    pub fn new_blocking(_peri: Peri<'d, peripherals::CRACEN>) -> Self {
        let me = Self { _peri, _p: PhantomData };

        me.stop();
        me
    }
}

impl<'d, M: Mode> Cracen<'d, M> {
    fn regs() -> pac::cracen::Cracen {
        pac::CRACEN
    }

    fn core() -> pac::cracencore::Cracencore {
        pac::CRACENCORE
    }

    fn start_rng(&self) {
        let r = Self::regs();
        r.enable().write(|w| {
            w.set_rng(true);
        });

        let r = Self::core();
        r.rngcontrol().control().write(|w| {
            w.set_enable(true);
        });

        while r.rngcontrol().status().read().state() == pac::cracencore::vals::State::STARTUP {}
    }

    fn stop(&self) {
        let r = Self::regs();
        r.enable().write(|w| {
            w.set_cryptomaster(false);
            w.set_rng(false);
            w.set_pkeikg(false);
        });
    }

    /// Fill the buffer with random bytes, blocking version.
    pub fn blocking_fill_bytes(&mut self, dest: &mut [u8]) {
        self.start_rng();

        let r = Self::core();
        for chunk in dest.chunks_mut(4) {
            while r.rngcontrol().fifolevel().read() == 0 {}
            let word = r.rngcontrol().fifo(0).read().to_ne_bytes();
            let to_copy = word.len().min(chunk.len());
            chunk[..to_copy].copy_from_slice(&word[..to_copy]);
        }

        self.stop();
    }

    /// Generate a random u32
    pub fn blocking_next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.blocking_fill_bytes(&mut bytes);
        // We don't care about the endianness, so just use the native one.
        u32::from_ne_bytes(bytes)
    }

    /// Generate a random u64
    pub fn blocking_next_u64(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.blocking_fill_bytes(&mut bytes);
        u64::from_ne_bytes(bytes)
    }
}

impl<'d, M: Mode> Drop for Cracen<'d, M> {
    fn drop(&mut self) {
        let r = Self::core();
        r.rngcontrol().control().write(|w| {
            w.set_enable(false);
        });

        while r.rngcontrol().status().read().state() != pac::cracencore::vals::State::RESET {}

        let r = Self::regs();
        r.enable().write(|w| {
            w.set_cryptomaster(false);
            w.set_rng(false);
            w.set_pkeikg(false);
        });
    }
}

impl<'d, M: Mode> rand_core_06::RngCore for Cracen<'d, M> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d, M: Mode> rand_core_06::CryptoRng for Cracen<'d, M> {}

impl<'d, M: Mode> rand_core_09::RngCore for Cracen<'d, M> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }
}

impl<'d, M: Mode> rand_core_09::CryptoRng for Cracen<'d, M> {}

pub(crate) trait SealedInstance {}

/// CRACEN peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_cracen {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::cracen::SealedInstance for peripherals::$type {}
        impl crate::cracen::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
