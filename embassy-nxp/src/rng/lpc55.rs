#![macro_use]

use core::convert::Infallible;

use embassy_futures::yield_now;
use embassy_hal_internal::{Peri, PeripheralType};
use rand_core_10::{TryCryptoRng, TryRng};

use crate::{pac, peripherals};

const REFRESH_COUNT: u32 = 31;
const MAX_CHI_SQUARED: u32 = 4;
const REVISION_0A: u8 = 0;
const CLOCK_SEL_0A: u8 = 0;
const CLOCK_SEL_1B: u8 = 4;
const REV_0A_WORDS: usize = 32;

/// The RNG Driver struct
pub struct Rng<'d, T: Instance> {
    _peripheral: Peri<'d, T>,
    revision: u8,
}

impl<'d, T: Instance> Rng<'d, T> {
    /// Initializes the RNG peripheral according to the LPC55S6x reference manual (Chapter 48.15)
    pub fn new(peripheral: Peri<'d, T>) -> Self {
        let syscon = pac::SYSCON;
        let pmc = pac::PMC;
        let rng = pac::RNG;
        let revision = syscon.dieid().read().rev_id();

        // Clear power down bit and enable RNG input clock
        pmc.pdruncfg0()
            .modify(|w| w.set_pden_rng(pac::pmc::vals::PdenRng::Poweredon));
        // AHBCLKCTRLSET2 is a write-one-to-set register; zero bits are ignored
        syscon.ahbclkctrlset(2).write(|w| w.set_data(1 << 13));

        // Assert and release TRNG reset
        syscon
            .presetctrl2()
            .modify(|w| w.set_rng_rst(pac::syscon::vals::RngRst::Asserted));
        syscon
            .presetctrl2()
            .modify(|w| w.set_rng_rst(pac::syscon::vals::RngRst::Released));

        let mut shift4x = 0;

        // The reference manual requires repeating this sequence after a failed
        // CHI test, increasing SHIFT4X each time to improve the clock ratio
        loop {
            rng.counter_cfg().modify(|w| {
                // Free-running mode is required for continuous entropy
                // accumulation and for REFRESH_CNT to keep increasing.
                w.set_mode(2);
                w.set_clock_sel(if revision == REVISION_0A {
                    CLOCK_SEL_0A
                } else {
                    CLOCK_SEL_1B
                });
            });
            rng.online_test_cfg().modify(|w| w.set_activate(true));

            while {
                let value = rng.online_test_val().read();
                value.min_chi_squared() >= value.max_chi_squared()
            } {}

            if u32::from(rng.online_test_val().read().max_chi_squared()) >= MAX_CHI_SQUARED {
                rng.online_test_cfg().modify(|w| w.set_activate(false));

                if shift4x < 7 {
                    shift4x += 1;
                    rng.counter_cfg().modify(|w| w.set_shift4x(shift4x));
                }
                continue;
            }

            break;
        }

        Self {
            _peripheral: peripheral,
            revision,
        }
    }

    /// Gets a 32-bit random number, cooperatively waiting for high-quality entropy
    pub async fn read_u32(&mut self) -> u32 {
        let rng = pac::RNG;

        if self.revision == REVISION_0A {
            return self.read_u32_0a().await;
        }

        while u32::from(rng.counter_val().read().refresh_cnt()) < REFRESH_COUNT {
            yield_now().await;
        }

        let value = rng.random_number().read().random_number();

        while u32::from(rng.online_test_val().read().max_chi_squared()) > MAX_CHI_SQUARED {
            yield_now().await;
        }

        value
    }

    async fn read_u32_0a(&mut self) -> u32 {
        let rng = pac::RNG;
        let mut value = rng.random_number().read().random_number();

        // Rev 0A has no linear entropy accumulation. XOR 32 refreshed
        // samples as per the reference manual
        for _ in 0..REV_0A_WORDS {
            while rng.counter_val().read().refresh_cnt() == 0 {
                yield_now().await;
            }
            value ^= rng.random_number().read().random_number();
        }

        while u32::from(rng.online_test_val().read().max_chi_squared()) > MAX_CHI_SQUARED {
            yield_now().await;
        }

        value
    }

    /// Asynchronously fills a byte slice with high-quality entropy
    pub async fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let random = self.read_u32().await.to_ne_bytes();
            chunk.copy_from_slice(&random[..chunk.len()]);
        }
    }

    /// Gets a 32-bit random number, blocking until it is available
    pub fn blocking_read_u32(&mut self) -> u32 {
        let rng = pac::RNG;

        if self.revision == REVISION_0A {
            return self.blocking_read_u32_0a();
        }

        while u32::from(rng.counter_val().read().refresh_cnt()) < REFRESH_COUNT {}
        let value = rng.random_number().read().random_number();
        while u32::from(rng.online_test_val().read().max_chi_squared()) > MAX_CHI_SQUARED {}

        value
    }

    fn blocking_read_u32_0a(&mut self) -> u32 {
        let rng = pac::RNG;
        let mut value = rng.random_number().read().random_number();

        for _ in 0..REV_0A_WORDS {
            while rng.counter_val().read().refresh_cnt() == 0 {}
            value ^= rng.random_number().read().random_number();
        }

        while u32::from(rng.online_test_val().read().max_chi_squared()) > MAX_CHI_SQUARED {}

        value
    }

    /// Fills a byte slice with high-quality entropy, blocking until complete
    pub fn blocking_fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let random = self.blocking_read_u32().to_ne_bytes();
            chunk.copy_from_slice(&random[..chunk.len()]);
        }
    }
}

impl<'d, T: Instance> TryRng for Rng<'d, T> {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.blocking_read_u32())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(((self.blocking_read_u32() as u64) << 32) | (self.blocking_read_u32() as u64))
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d, T: Instance> TryCryptoRng for Rng<'d, T> {}

#[allow(private_bounds)]
pub trait Instance: PeripheralType {}

impl Instance for peripherals::RNG {}
