//! Random Number Generator (RNG)
#![macro_use]

use core::convert::Infallible;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{Peri, interrupt, pac, peripherals, rcc};

static RNG_WAKER: AtomicWaker = AtomicWaker::new();

#[cfg(not(rng_v1))]
/// Health-test programming profile used during RNG conditioning reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HealthTestConfig {
    /// Program the recommended threshold values from the reference manual.
    Recommended,
    /// Keep current HTCR values untouched.
    KeepCurrent,
}

#[cfg(not(rng_v1))]
/// RNG configuration knobs for reset/initialization policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RngConfig {
    /// NIST/custom conditioning mode selection.
    pub nistc: pac::rng::vals::Nistc,
    /// RNG clock divider before sampling.
    pub clkdiv: pac::rng::vals::Clkdiv,
    /// RNG configuration 1 profile.
    pub rng_config1: pac::rng::vals::RngConfig1,
    /// RNG configuration 2 profile.
    pub rng_config2: pac::rng::vals::RngConfig2,
    /// RNG configuration 3 profile.
    pub rng_config3: pac::rng::vals::RngConfig3,
    /// Enable clock error detector (CED bit, active-low semantics in HW).
    pub clock_error_detector: bool,
    /// Disable automatic reset on seed error when supported (ARDIS bit).
    #[cfg(any(rng_v3, rng_wba6))]
    pub auto_reset_disable: bool,
    /// Lock RNG configuration after setup.
    pub config_lock: bool,
    /// Health-test threshold programming behavior.
    pub health_test_config: HealthTestConfig,
}

#[cfg(not(rng_v1))]
impl Default for RngConfig {
    fn default() -> Self {
        Self {
            nistc: pac::rng::vals::Nistc::Custom,
            clkdiv: pac::rng::vals::Clkdiv::NoDiv,
            rng_config1: pac::rng::vals::RngConfig1::ConfigA,
            rng_config2: pac::rng::vals::RngConfig2::ConfigAB,
            rng_config3: pac::rng::vals::RngConfig3::ConfigA,
            clock_error_detector: true,
            #[cfg(any(rng_v3, rng_wba6))]
            auto_reset_disable: false,
            config_lock: false,
            health_test_config: HealthTestConfig::Recommended,
        }
    }
}

/// WBA-specific health test configuration values for RNG
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Htcfg {
    /// WBA-specific health test configuration (0x0000AAC7)
    /// Corresponds to configuration A, B, and C thresholds as recommended in the reference manual
    WbaRecommended = 0x0000_AAC7,
}

impl Htcfg {
    /// Convert to the raw u32 value for register access
    #[allow(dead_code)]
    fn value(self) -> u32 {
        self as u32
    }
}

/// RNG error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Seed error.
    SeedError,
    /// Clock error. Double-check the RCC configuration,
    /// see the Reference Manual for details on restrictions
    /// on RNG clocks.
    ClockError,
}

/// RNG interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let bits = T::regs().sr().read();
        if bits.drdy() || bits.seis() || bits.ceis() {
            T::regs().cr().modify(|reg| reg.set_ie(false));
            RNG_WAKER.wake();
        }
    }
}

/// RNG driver.
pub struct Rng<'d, T: Instance> {
    _inner: Peri<'d, T>,
}

impl<'d, T: Instance> Rng<'d, T> {
    /// Create a new RNG driver.
    pub fn new(
        inner: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        #[cfg(rng_v1)]
        {
            Self::new_inner(inner)
        }
        #[cfg(not(rng_v1))]
        {
            Self::new_inner(inner, RngConfig::default())
        }
    }

    #[cfg(not(rng_v1))]
    /// Create a new RNG driver with explicit configuration policy.
    pub fn new_with_config(
        inner: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: RngConfig,
    ) -> Self {
        Self::new_inner(inner, config)
    }

    #[cfg(rng_v1)]
    fn new_inner(inner: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();

        // Verify clock is available
        T::frequency();

        let mut random = Self { _inner: inner };
        random.reset();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        random
    }

    #[cfg(not(rng_v1))]
    fn new_inner(inner: Peri<'d, T>, config: RngConfig) -> Self {
        rcc::enable_and_reset::<T>();

        // Verify clock is available
        T::frequency();

        let mut random = Self { _inner: inner };
        random.reset_with_config(config);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        random
    }

    /// Reset the RNG.
    #[cfg(rng_v1)]
    pub fn reset(&mut self) {
        T::regs().cr().write(|reg| {
            reg.set_rngen(false);
        });
        T::regs().sr().modify(|reg| {
            reg.set_seis(false);
            reg.set_ceis(false);
        });
        T::regs().cr().modify(|reg| {
            reg.set_rngen(true);
        });
        // Reference manual says to discard the first.
        let _ = self.next_u32();
    }

    /// Reset the RNG.
    #[cfg(not(rng_v1))]
    pub fn reset(&mut self) {
        self.reset_with_config(RngConfig::default());
    }

    #[cfg(not(rng_v1))]
    /// Reset the RNG with a caller-provided configuration policy.
    pub fn reset_with_config(&mut self, config: RngConfig) {
        T::regs().cr().write(|reg| {
            reg.set_condrst(true);
            reg.set_nistc(config.nistc);
            // set RNG config "A" according to reference manual
            // this has to be written within the same write access as setting the CONDRST bit
            reg.set_rng_config1(config.rng_config1);
            reg.set_clkdiv(config.clkdiv);
            reg.set_rng_config2(config.rng_config2);
            reg.set_rng_config3(config.rng_config3);
            reg.set_ced(!config.clock_error_detector);
            #[cfg(any(rng_v3, rng_wba6))]
            reg.set_ardis(config.auto_reset_disable);
            reg.set_ie(false);
            reg.set_rngen(true);
        });
        // wait for CONDRST to be set
        while !T::regs().cr().read().condrst() {}

        // Set health test configuration values
        match config.health_test_config {
            HealthTestConfig::Recommended => {
                #[cfg(not(rng_wba6))]
                {
                    // magic number must be written immediately before every read or write access to HTCR
                    T::regs().htcr().write(|w| w.set_htcfg(pac::rng::vals::Htcfg::Magic));
                    // write recommended value according to reference manual
                    // note: HTCR can only be written during conditioning
                    T::regs()
                        .htcr()
                        .write(|w| w.set_htcfg(pac::rng::vals::Htcfg::Recommended));
                }
                #[cfg(rng_wba6)]
                {
                    // For WBA6, set RNG_HTCR0 to the recommended value for configurations A, B, and C
                    // This value corresponds to the health test thresholds specified in the reference manual
                    T::regs().htcr(0).write(|w| w.0 = Htcfg::WbaRecommended.value());
                }
            }
            HealthTestConfig::KeepCurrent => {}
        }

        // finish conditioning
        T::regs().cr().modify(|reg| {
            reg.set_rngen(true);
            reg.set_condrst(false);
            reg.set_configlock(config.config_lock);
        });

        // According to reference manual for RNGv3: SEIS must be cleared manually.
        // RNGv2 does not say anything about SEIS clearing, but ST Cube HAL clears it.
        T::regs().sr().modify(|reg| {
            reg.set_seis(false);
        });

        // According to reference manual: after software reset, wait for random number to be ready
        // The next_u32() call will wait for DRDY, completing the initialization
        let _ = self.next_u32();
    }

    /// Try to recover from a seed error.
    pub fn recover_seed_error(&mut self) {
        self.reset();
        // reset should also clear the SEIS flag
        if T::regs().sr().read().seis() {
            warn!("recovering from seed error failed");
            return;
        }
        // wait for SECS to be cleared by RNG
        while T::regs().sr().read().secs() {}
    }

    /// Fill the given slice with random values.
    pub async fn async_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        for chunk in dest.chunks_mut(4) {
            let mut bits = T::regs().sr().read();
            if !bits.seis() && !bits.ceis() && !bits.drdy() {
                // wait for interrupt
                poll_fn(|cx| {
                    // quick check to avoid registration if already done.
                    let bits = T::regs().sr().read();
                    if bits.drdy() || bits.seis() || bits.ceis() {
                        return Poll::Ready(());
                    }
                    RNG_WAKER.register(cx.waker());
                    T::regs().cr().modify(|reg| reg.set_ie(true));
                    // Need to check condition **after** `register` to avoid a race
                    // condition that would result in lost notifications.
                    let bits = T::regs().sr().read();
                    if bits.drdy() || bits.seis() || bits.ceis() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;

                // Re-read the status register after wait.
                bits = T::regs().sr().read()
            }
            if bits.seis() {
                // in case of noise-source or seed error we try to recover here
                // but we must not use the data in DR and we return an error
                // to leave retry-logic to the application
                self.recover_seed_error();
                return Err(Error::SeedError);
            } else if bits.ceis() {
                // clock error detected, DR could still be used but keep it safe,
                // clear the error and abort
                T::regs().sr().modify(|sr| sr.set_ceis(false));
                return Err(Error::ClockError);
            } else if bits.drdy() {
                // DR can be read up to four times until the output buffer is empty
                // DRDY is cleared automatically when that happens
                let random_word = T::regs().dr().read();
                // reference manual: always check if DR is zero
                if random_word == 0 {
                    return Err(Error::SeedError);
                }
                // write bytes to chunk
                for (dest, src) in chunk.iter_mut().zip(random_word.to_ne_bytes().iter()) {
                    *dest = *src
                }
            }
        }

        Ok(())
    }

    /// Get a random u32
    pub fn next_u32(&mut self) -> u32 {
        loop {
            let sr = T::regs().sr().read();
            if sr.seis() | sr.ceis() {
                self.reset();
            } else if sr.drdy() {
                return T::regs().dr().read();
            }
        }
    }

    /// Get a random u64
    pub fn next_u64(&mut self) -> u64 {
        let mut rand = self.next_u32() as u64;
        rand |= (self.next_u32() as u64) << 32;
        rand
    }

    /// Fill a slice with random bytes
    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let rand = self.next_u32();
            for (slot, num) in chunk.iter_mut().zip(rand.to_ne_bytes().iter()) {
                *slot = *num
            }
        }
    }
}

impl<'d, T: Instance> Drop for Rng<'d, T> {
    fn drop(&mut self) {
        T::regs().cr().modify(|reg| {
            reg.set_rngen(false);
        });
        rcc::disable::<T>();
    }
}

impl<'d, T: Instance> crate::low_power::SealedSuspendablePeripheral for Rng<'d, T> {
    #[cfg(all(feature = "low-power", rng_v1))]
    type InternalState = Peri<'d, T>;

    #[cfg(all(feature = "low-power", not(rng_v1)))]
    type InternalState = (Peri<'d, T>, RngConfig);

    #[cfg(feature = "low-power")]
    fn suspend(self) -> Self::InternalState {
        #[cfg(not(rng_v1))]
        {
            let cr = T::regs().cr().read();

            let config = RngConfig {
                nistc: cr.nistc(),
                clkdiv: cr.clkdiv(),
                rng_config1: cr.rng_config1(),
                rng_config2: cr.rng_config2(),
                rng_config3: cr.rng_config3(),
                clock_error_detector: !cr.ced(),
                #[cfg(any(rng_v3, rng_wba6))]
                auto_reset_disable: cr.ardis(),
                ..Default::default()
            };

            unsafe { (self._inner.clone_unchecked(), config) }
        }

        #[cfg(rng_v1)]
        {
            unsafe { self._inner.clone_unchecked() }
        }
    }

    #[cfg(feature = "low-power")]
    fn resume(state: Self::InternalState) -> Self {
        #[cfg(not(rng_v1))]
        {
            Self::new_inner(state.0, state.1)
        }

        #[cfg(rng_v1)]
        {
            Self::new_inner(state)
        }
    }
}

impl<'d, T: Instance> rand_core_06::RngCore for Rng<'d, T> {
    fn next_u32(&mut self) -> u32 {
        self.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl<'d, T: Instance> rand_core_06::CryptoRng for Rng<'d, T> {}

impl<'d, T: Instance> rand_core_09::RngCore for Rng<'d, T> {
    fn next_u32(&mut self) -> u32 {
        self.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.fill_bytes(dest);
    }
}

impl<'d, T: Instance> rand_core_09::CryptoRng for Rng<'d, T> {}

impl<'d, T: Instance> rand_core_10::TryRng for Rng<'d, T> {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.next_u32())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(self.next_u64())
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        self.fill_bytes(dest);

        Ok(())
    }
}

impl<'d, T: Instance> rand_core_10::TryCryptoRng for Rng<'d, T> {}

trait SealedInstance {
    fn regs() -> pac::rng::Rng;
}

/// RNG instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this RNG instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, rng, RNG, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::rng::Rng {
                crate::pac::$inst
            }
        }
    };
);
