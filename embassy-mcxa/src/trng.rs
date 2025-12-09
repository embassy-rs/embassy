//! True Random Number Generator

use embassy_hal_internal::Peri;
use embassy_hal_internal::interrupt::InterruptExt;
use maitake_sync::WaitCell;
use mcxa_pac::trng0::osc2_ctl::TrngEntCtl;

use crate::clocks::enable_and_reset;
use crate::clocks::periph_helpers::NoConfig;
use crate::interrupt::typelevel;
use crate::interrupt::typelevel::Handler;
use crate::peripherals::TRNG0;

static WAIT_CELL: WaitCell = WaitCell::new();

/// TRNG Driver
pub struct Trng<'d> {
    _peri: Peri<'d, TRNG0>,
}

impl<'d> Trng<'d> {
    /// Instantiates a new TRNG peripheral driver.
    pub fn new(_peri: Peri<'d, TRNG0>, config: Config) -> Self {
        _ = unsafe { enable_and_reset::<TRNG0>(&NoConfig) };

        Self::configure(config);
        Self { _peri }
    }

    fn configure(config: Config) {
        regs()
            .mctl()
            .modify(|_, w| w.rst_def().set_bit().prgm().enable().err().clear_bit_by_one());

        regs().scml().write(|w| unsafe {
            w.mono_max()
                .bits(config.monobit_limit_max)
                .mono_rng()
                .bits(config.monobit_limit_max - config.monobit_limit_min)
        });

        regs().scr1l().write(|w| unsafe {
            w.run1_max()
                .bits(config.run_length1_limit_max)
                .run1_rng()
                .bits(config.run_length1_limit_max - config.run_length1_limit_min)
        });

        regs().scr2l().write(|w| unsafe {
            w.run2_max()
                .bits(config.run_length2_limit_max)
                .run2_rng()
                .bits(config.run_length2_limit_max - config.run_length2_limit_min)
        });

        regs().scr3l().write(|w| unsafe {
            w.run3_max()
                .bits(config.run_length3_limit_max)
                .run3_rng()
                .bits(config.run_length3_limit_max - config.run_length3_limit_min)
        });

        regs().scr4l().write(|w| unsafe {
            w.run4_max()
                .bits(config.run_length4_limit_max)
                .run4_rng()
                .bits(config.run_length4_limit_max - config.run_length4_limit_min)
        });

        regs().scr5l().write(|w| unsafe {
            w.run5_max()
                .bits(config.run_length5_limit_max)
                .run5_rng()
                .bits(config.run_length5_limit_max - config.run_length5_limit_min)
        });

        regs().scr6pl().write(|w| unsafe {
            w.run6p_max()
                .bits(config.run_length6_limit_max)
                .run6p_rng()
                .bits(config.run_length6_limit_max - config.run_length6_limit_min)
        });

        regs()
            .pkrmax()
            .write(|w| unsafe { w.pkr_max().bits(config.poker_limit_max) });

        regs()
            .frqmax()
            .write(|w| unsafe { w.frq_max().bits(config.freq_counter_max) });

        regs()
            .frqmin()
            .write(|w| unsafe { w.frq_min().bits(config.freq_counter_min) });

        regs()
            .sblim()
            .write(|w| unsafe { w.sb_lim().bits(config.sparse_bit_limit) });

        regs().scmisc().write(|w| unsafe {
            w.lrun_max()
                .bits(config.long_run_limit_max)
                .rty_ct()
                .bits(config.retry_count)
        });

        regs()
            .mctl()
            .modify(|_, w| w.dis_slf_tst().variant(config.self_test.into()));

        regs().sdctl().write(|w| unsafe {
            w.samp_size()
                .bits(config.sample_size)
                .ent_dly()
                .bits(config.entropy_delay)
        });

        regs()
            .osc2_ctl()
            .modify(|_, w| w.trng_ent_ctl().variant(config.osc_mode.into()));

        regs().mctl().modify(|_, w| w.prgm().disable());

        let _ = regs().ent(7).read().bits();
    }

    fn start() {
        regs().mctl().modify(|_, w| w.trng_acc().set_bit());
    }

    fn stop() {
        regs().mctl().modify(|_, w| w.trng_acc().clear_bit());
    }

    fn blocking_wait_for_generation() {
        while regs().mctl().read().ent_val().bit_is_clear() {}
    }

    fn fill_chunk(chunk: &mut [u8]) {
        let mut entropy = [0u32; 8];

        for (i, item) in entropy.iter_mut().enumerate() {
            *item = regs().ent(i).read().bits();
        }

        let entropy: [u8; 32] = unsafe { core::mem::transmute(entropy) };

        chunk.copy_from_slice(&entropy[..chunk.len()]);
    }

    // Blocking API

    /// Fill the buffer with random bytes, blocking version.
    pub fn blocking_fill_bytes(&mut self, buf: &mut [u8]) {
        if buf.is_empty() {
            return; // nothing to fill
        }

        Self::start();
        for chunk in buf.chunks_mut(32) {
            Self::blocking_wait_for_generation();
            Self::fill_chunk(chunk);
        }
        Self::stop();
    }

    /// Return a random u32, blocking version.
    pub fn blocking_next_u32(&mut self) -> u32 {
        Self::start();
        Self::blocking_wait_for_generation();
        let result = regs().ent(0).read().bits();

        // New random bytes are generated only after reading ENT7
        let _ = regs().ent(7).read().bits();
        Self::stop();

        result
    }

    /// Return a random u64, blocking version.
    pub fn blocking_next_u64(&mut self) -> u64 {
        Self::start();
        Self::blocking_wait_for_generation();

        let mut result = u64::from(regs().ent(0).read().bits()) << 32;
        result |= u64::from(regs().ent(1).read().bits());

        // New random bytes are generated only after reading ENT7
        let _ = regs().ent(7).read().bits();
        Self::stop();

        result
    }
}

impl Drop for Trng<'_> {
    fn drop(&mut self) {
        // reset the TRNG
        regs().mctl().write(|w| w.rst_def().set_bit());
    }
}

/// TRNG Async Driver
pub struct AsyncTrng<'d> {
    _peri: Peri<'d, TRNG0>,
}

impl<'d> AsyncTrng<'d> {
    /// Instantiates a new TRNG peripheral driver.
    pub fn new(
        _peri: Peri<'d, TRNG0>,
        _irq: impl crate::interrupt::typelevel::Binding<typelevel::TRNG0, InterruptHandler> + 'd,
        config: Config,
    ) -> Self {
        _ = unsafe { enable_and_reset::<TRNG0>(&NoConfig) };

        Trng::configure(config);

        crate::pac::Interrupt::TRNG0.unpend();
        unsafe {
            crate::pac::Interrupt::TRNG0.enable();
        }

        Self { _peri }
    }

    fn enable_ints() {
        regs().int_mask().write(|w| {
            w.hw_err()
                .set_bit()
                .ent_val()
                .set_bit()
                .frq_ct_fail()
                .set_bit()
                .intg_flt()
                .set_bit()
        });
    }

    async fn wait_for_generation() -> Result<(), Error> {
        WAIT_CELL
            .wait_for(|| {
                Self::enable_ints();
                regs().mctl().read().ent_val().bit_is_set()
            })
            .await
            .map_err(|_| Error::ErrorStatus)
    }

    // Async API

    /// Fill the buffer with random bytes, async version.
    pub async fn async_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        if buf.is_empty() {
            return Ok(()); // nothing to fill
        }

        Trng::start();
        for chunk in buf.chunks_mut(32) {
            Self::wait_for_generation().await?;
            Trng::fill_chunk(chunk);
        }
        Trng::stop();

        Ok(())
    }

    /// Return a random u32, async version.
    pub async fn async_next_u32(&mut self) -> Result<u32, Error> {
        Trng::start();
        Self::wait_for_generation().await?;
        let result = regs().ent(0).read().bits();

        // New random bytes are generated only after reading ENT7
        let _ = regs().ent(7).read().bits();
        Trng::stop();

        Ok(result)
    }

    /// Return a random u64, async version.
    pub async fn async_next_u64(&mut self) -> Result<u64, Error> {
        Trng::start();
        Self::wait_for_generation().await?;

        let mut result = u64::from(regs().ent(0).read().bits()) << 32;
        result |= u64::from(regs().ent(1).read().bits());

        // New random bytes are generated only after reading ENT7
        let _ = regs().ent(7).read().bits();
        Trng::stop();

        Ok(result)
    }

    // Blocking API

    /// Fill the buffer with random bytes, blocking version.
    pub fn blocking_fill_bytes(&mut self, buf: &mut [u8]) {
        if buf.is_empty() {
            return; // nothing to fill
        }

        Trng::start();
        for chunk in buf.chunks_mut(32) {
            Trng::blocking_wait_for_generation();
            Trng::fill_chunk(chunk);
        }
        Trng::stop();
    }

    /// Return a random u32, blocking version.
    pub fn blocking_next_u32(&mut self) -> u32 {
        Trng::start();
        Trng::blocking_wait_for_generation();
        let result = regs().ent(0).read().bits();

        // New random bytes are generated only after reading ENT7
        let _ = regs().ent(7).read().bits();
        Trng::stop();

        result
    }

    /// Return a random u64, blocking version.
    pub fn blocking_next_u64(&mut self) -> u64 {
        Trng::start();
        Trng::blocking_wait_for_generation();

        let mut result = u64::from(regs().ent(0).read().bits()) << 32;
        result |= u64::from(regs().ent(1).read().bits());

        // New random bytes are generated only after reading ENT7
        let _ = regs().ent(7).read().bits();
        Trng::stop();

        result
    }
}

impl Drop for AsyncTrng<'_> {
    fn drop(&mut self) {
        // reset the TRNG
        regs().mctl().write(|w| w.rst_def().set_bit());
    }
}

fn regs() -> &'static crate::pac::trng0::RegisterBlock {
    unsafe { &*crate::pac::Trng0::ptr() }
}

/// Trng errors
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Integrity error.
    IntegrityError,

    /// Frequency counter fail
    FrequencyCountFail,

    /// Error status
    ErrorStatus,

    /// Buffer argument is invalid
    InvalidBuffer,
}

/// I2C interrupt handler.
pub struct InterruptHandler;

impl Handler<typelevel::TRNG0> for InterruptHandler {
    unsafe fn on_interrupt() {
        if regs().int_status().read().bits() != 0 {
            regs().int_ctrl().write(|w| {
                w.hw_err()
                    .clear_bit()
                    .ent_val()
                    .clear_bit()
                    .frq_ct_fail()
                    .clear_bit()
                    .intg_flt()
                    .clear_bit()
            });

            WAIT_CELL.wake();
        }
    }
}

/// True random number generator configuration parameters.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Total number of Entropy samples that will be taken during
    /// Entropy generation.
    pub sample_size: u16,

    /// Length (in system clocks) of each Entropy sample taken.
    pub entropy_delay: u16,

    /// Enable or disable internal self-tests.
    pub self_test: SelfTest,

    /// Frequency Counter Maximum Limit
    pub freq_counter_max: u32,

    /// Frequency Counter Minimum Limit
    pub freq_counter_min: u32,

    /// Statistical check monobit max limit
    pub monobit_limit_max: u16,

    /// Statistical check monobit min limit
    pub monobit_limit_min: u16,

    /// Statistical check run length 1 limit max
    pub run_length1_limit_max: u16,

    /// Statistical check run length 1 limit min
    pub run_length1_limit_min: u16,

    /// Statistical check run length 2 limit max
    pub run_length2_limit_max: u16,

    /// Statistical check run length 2 limit min
    pub run_length2_limit_min: u16,

    /// Statistical check run length 3 limit max
    pub run_length3_limit_max: u16,

    /// Statistical check run length 3 limit min
    pub run_length3_limit_min: u16,

    /// Statistical check run length 4 limit max
    pub run_length4_limit_max: u16,

    /// Statistical check run length 4 limit min
    pub run_length4_limit_min: u16,

    /// Statistical check run length 5 limit max
    pub run_length5_limit_max: u16,

    /// Statistical check run length 5 limit min
    pub run_length5_limit_min: u16,

    /// Statistical check run length 6 limit max
    pub run_length6_limit_max: u16,

    /// Statistical check run length 6 limit min
    pub run_length6_limit_min: u16,

    /// Retry count
    pub retry_count: u8,

    /// Long run limit max
    pub long_run_limit_max: u8,

    /// Sparse bit limit
    pub sparse_bit_limit: u16,

    /// Poker limit max
    pub poker_limit_max: u32,

    /// Oscillator mode
    pub osc_mode: OscMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sample_size: 1024,
            entropy_delay: 32_000,
            self_test: SelfTest::Enabled,
            freq_counter_max: 75_000,
            freq_counter_min: 30_000,
            monobit_limit_max: 596,
            monobit_limit_min: 427,
            run_length1_limit_max: 187,
            run_length1_limit_min: 57,
            run_length2_limit_max: 105,
            run_length2_limit_min: 28,
            run_length3_limit_max: 97,
            run_length3_limit_min: 33,
            run_length4_limit_max: 0,
            run_length4_limit_min: 0,
            run_length5_limit_max: 0,
            run_length5_limit_min: 0,
            run_length6_limit_max: 0,
            run_length6_limit_min: 0,
            retry_count: 2,
            long_run_limit_max: 32,
            sparse_bit_limit: 0,
            poker_limit_max: 0,
            osc_mode: OscMode::DualOscs,
        }
    }
}

/// Enable or disable internal self-tests.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SelfTest {
    /// Disabled.
    Disabled,

    /// Enabled.
    Enabled,
}

impl From<SelfTest> for bool {
    fn from(value: SelfTest) -> Self {
        match value {
            SelfTest::Disabled => true,
            SelfTest::Enabled => false,
        }
    }
}

/// Oscillator mode.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum OscMode {
    /// Single oscillator using OSC1.
    SingleOsc1,

    /// Dual oscillator.
    DualOscs,

    /// Single oscillator using OSC2.
    SingleOsc2,
}

impl From<OscMode> for TrngEntCtl {
    fn from(value: OscMode) -> Self {
        match value {
            OscMode::SingleOsc1 => Self::TrngEntCtlSingleOsc1,
            OscMode::DualOscs => Self::TrngEntCtlDualOscs,
            OscMode::SingleOsc2 => Self::TrngEntCtlSingleOsc2,
        }
    }
}

impl<'d> rand_core_06::RngCore for Trng<'d> {
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d> rand_core_06::CryptoRng for Trng<'d> {}

impl<'d> rand_core_09::RngCore for Trng<'d> {
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
}

impl<'d> rand_core_09::CryptoRng for Trng<'d> {}

impl<'d> rand_core_06::RngCore for AsyncTrng<'d> {
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d> rand_core_06::CryptoRng for AsyncTrng<'d> {}

impl<'d> rand_core_09::RngCore for AsyncTrng<'d> {
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
}

impl<'d> rand_core_09::CryptoRng for AsyncTrng<'d> {}
