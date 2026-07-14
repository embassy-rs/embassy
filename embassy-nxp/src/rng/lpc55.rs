#![macro_use]

use crate::pac::*;
use crate::peripherals;
use core::convert::Infallible;
use core::future::poll_fn;
use core::task::Poll;
use embassy_hal_internal::{Peri, PeripheralType};
use rand_core::{TryCryptoRng, TryRng};

/// The RNG Driver struct.
pub struct Rng<'d, T: Instance> {
    _peripheral: Peri<'d, T>,
}

impl<'d, T: Instance> Rng<'d, T> {
    /// Initializes the RNG peripheral according to the LPC55S6x reference manual (Chapter 48.15)
    pub fn new(peripheral: Peri<'d, T>) -> Self {
        // Grab pointers to the hardware registers
        let syscon = unsafe { &*SYSCON::ptr() };
        let pmc = unsafe { &*PMC::ptr() };
        let rng = unsafe { &*RNG::ptr() };

        unsafe {
            // Clear power down bit and enable RNG input clock
            pmc.pdruncfg0.modify(|_, w| w.pden_rng().clear_bit());
            syscon.ahbclkctrlset[2].write(|w| w.bits(0x0000_2000));

            // Assert and release TRNG reset
            syscon.presetctrl2.modify(|_, w| w.rng_rst().set_bit());
            syscon.presetctrl2.modify(|_, w| w.rng_rst().clear_bit());
        }

        let mut shiftx_val = 0;

        // Loop for CHI computing stabilization (Rev 1B)
        loop {
            // Step 2: Set clock and activate
            rng.counter_cfg.modify(|_, w| unsafe { w.clock_sel().bits(4) });
            rng.online_test_cfg.modify(|_, w| w.activate().set_bit());

            // Step 3: Wait for MIN_CHI_SQUARED to be smaller than MAX_CHI_SQUARED
            while rng.online_test_val.read().min_chi_squared().bits()
                >= rng.online_test_val.read().max_chi_squared().bits()
            {}

            // Step 4: Check if MAX_CHI_SQUARED > 4
            if rng.online_test_val.read().max_chi_squared().bits() > 4 {
                // Reset activation
                rng.online_test_cfg.modify(|_, w| w.activate().clear_bit());

                if shiftx_val < 7 {
                    shiftx_val += 1;
                    rng.counter_cfg.modify(|_, w| unsafe { w.shift4x().bits(shiftx_val) });
                }
                // Go back to the top of the loop (Step 2)
                continue;
            } else {
                // Initialization complete
                break;
            }
        }

        Self {
            _peripheral: peripheral,
        }
    }

    /// Asynchronously gets a 32-bit random number ensuring high-quality entropy.
    pub async fn read_u32(&mut self) -> u32 {
        let rng = unsafe { &*RNG::ptr() };

        // 1. Wait for COUNTER_VAL.REFRESH_CNT to become 31 (Non-blocking)
        poll_fn(|cx| {
            if rng.counter_val.read().refresh_cnt().bits() >= 31 {
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await;

        // 2. Read new Random number
        let value = rng.random_number.read().random_number().bits();

        // 3. Perform online CHI computing check (Non-blocking)
        poll_fn(|cx| {
            if rng.online_test_val.read().max_chi_squared().bits() <= 4 {
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await;

        value
    }

    pub fn blocking_read_u32(&mut self) -> u32 {
        let rng = unsafe { &*RNG::ptr() };

        while rng.counter_val.read().refresh_cnt().bits() < 31 {}
        let value = rng.random_number.read().random_number().bits();
        while rng.online_test_val.read().max_chi_squared().bits() > 4 {}

        value
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
        for chunk in dest.chunks_mut(4) {
            let rand = self.blocking_read_u32().to_ne_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&rand[..len]);
        }
        Ok(())
    }
}

impl<'d, T: Instance> TryCryptoRng for Rng<'d, T> {}

#[allow(private_bounds)]
pub trait Instance: PeripheralType {}

impl Instance for peripherals::RNG {}

pub(crate) fn init() {
    // Clocks and power are enabled per-instance in Rng::new().
}
