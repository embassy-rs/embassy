//! CRACEN - Cryptographic Accelerator Engine driver.

#![macro_use]

use core::marker::PhantomData;

use crate::mode::{Blocking, Mode};
use crate::{Peri, interrupt, pac, peripherals};

/// TRNG health-test cut-offs for CRACEN Lite, from NCS `RNG_REPEATTHRESHOLD_VAL` and
/// `RNG_PROPTESTCUTOFF_VAL`.
///
/// These registers reset to 4 and 13, which a working noise source fails, and they revert
/// whenever CRACEN is powered down — which `stop_rng` does after every call. NCS programs
/// them on every power-up in `cracen_acquire`, calling it a workaround for incorrect
/// hardware defaults on CRACEN Lite.
///
/// Only CRACEN Lite is affected, hence the cfg: nRF54L15/L10/L05 reset the same registers
/// to 41 and 793, which are sane, and NCS does not apply the workaround to them either.
#[cfg(feature = "_nrf54lm20")]
const RNG_REPEATTHRESHOLD_VAL: u8 = 21;
#[cfg(feature = "_nrf54lm20")]
const RNG_PROPTESTCUTOFF_VAL: u16 = 311;

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
        Self { _peri, _p: PhantomData }
    }
}

impl<'d, M: Mode> Cracen<'d, M> {
    fn regs() -> pac::cracen::Cracen {
        pac::CRACEN
    }

    fn core() -> pac::cracencore::Cracencore {
        pac::CRACENCORE
    }

    /// Program the health-test cut-offs. Must happen after every power-up, since the
    /// registers revert with the block.
    #[cfg(feature = "_nrf54lm20")]
    fn configure_health_tests() {
        let r = Self::core().rngcontrol();
        r.repeatthreshold()
            .write(|w| w.set_repeatthreshold(RNG_REPEATTHRESHOLD_VAL));
        r.proptestcutoff()
            .write(|w| w.set_proptestcutoff(RNG_PROPTESTCUTOFF_VAL));
    }

    fn start_rng(&self) {
        let r = Self::regs();
        r.enable().write(|w| {
            w.set_rng(true);
        });

        // Before the RNG is started, and after the power-up that cleared them.
        #[cfg(feature = "_nrf54lm20")]
        Self::configure_health_tests();

        let r = Self::core();

        #[cfg(feature = "_nrf54lm20")]
        r.rngcontrol().cooldownperiod().write(|w| {
            w.set_cooldownperiod(0);
        });

        // Modify, not write: NB128BITBLOCKS defaults to 4 and zero is not a legal
        // value, so the other fields have to survive the start.
        r.rngcontrol().control().modify(|w| {
            w.set_enable(true);
        });

        while r.rngcontrol().status().read().state() == pac::cracencore::vals::State::Startup {}
    }

    fn stop_rng(&self) {
        #[cfg(not(feature = "_nrf54lm20"))]
        {
            let r = Self::core();
            r.rngcontrol().control().write(|w| {
                w.set_enable(false);
            });

            while r.rngcontrol().status().read().state() != pac::cracencore::vals::State::Reset {}
        }

        let r = Self::regs();
        r.enable().write(|w| {
            w.set_cryptomaster(false);
            w.set_rng(false);
            w.set_pkeikg(false);
        });
    }

    /// Fill the buffer with random bytes, blocking version.
    pub fn blocking_fill_bytes(&mut self, dest: &mut [u8]) {
        if dest.is_empty() {
            return;
        }

        self.start_rng();

        let r = Self::core();
        for chunk in dest.chunks_mut(4) {
            // A failed health test parks the FSM in `Error`, where the FIFO never fills
            // again — so without this check the poll never returns, and being a blocking
            // loop in a sync fn it takes the whole executor with it. There is nothing to
            // do but fail loudly: measured on an nRF54LM20A, neither a SoftRst nor
            // reprogramming the cut-offs revives a TRNG that has already reached `Error`,
            // only a reset of the part does, and an infallible API cannot report. The
            // cut-offs programmed in `start_rng` are what keeps this unreached.
            let word = loop {
                if r.rngcontrol().fifolevel().read() != 0 {
                    break r.rngcontrol().fifo(0).read();
                }
                let status = r.rngcontrol().status().read();
                if status.state() == pac::cracencore::vals::State::Error {
                    panic!(
                        "CRACEN RNG health test failed (rep={} prop={} startup={}); it needs a reset to produce entropy again",
                        status.repfail(),
                        status.propfail(),
                        status.startupfail()
                    );
                }
            };

            let word = word.to_ne_bytes();
            let to_copy = word.len().min(chunk.len());
            chunk[..to_copy].copy_from_slice(&word[..to_copy]);
        }

        self.stop_rng();
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
        // nothing to do here, since we stop+disable rng for each operation.
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

impl<'d, M: Mode> rand_core_10::TryRng for Cracen<'d, M> {
    type Error = core::convert::Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.blocking_next_u32())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(self.blocking_next_u64())
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d, M: Mode> rand_core_10::TryCryptoRng for Cracen<'d, M> {}

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
