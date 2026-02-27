//! True Random Number Generator

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embassy_hal_internal::interrupt::InterruptExt;
use maitake_sync::WaitCell;

use crate::clocks::enable_and_reset;
use crate::clocks::periph_helpers::NoConfig;
use crate::interrupt::typelevel;
use crate::interrupt::typelevel::Handler;
use crate::pac::trng::vals::TrngEntCtl;
use crate::peripherals::TRNG0;

static WAIT_CELL: WaitCell = WaitCell::new();
const BLOCK_SIZE: usize = 8;

#[allow(private_bounds)]
pub trait Mode: sealed::SealedMode {}

mod sealed {
    pub trait SealedMode {}
}

macro_rules! define_mode {
    ($mode:ident) => {
        pub struct $mode;
        impl sealed::SealedMode for $mode {}
        impl Mode for $mode {}
    };
}

define_mode!(Blocking);
define_mode!(Async);

/// TRNG Driver
pub struct Trng<'d, M: Mode> {
    _peri: Peri<'d, TRNG0>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> Trng<'d, M> {
    fn new_inner(_peri: Peri<'d, TRNG0>, config: Config) -> Self {
        // No clock: No WakeGuard!
        _ = unsafe { enable_and_reset::<TRNG0>(&NoConfig) };

        Self::configure(config);
        Self {
            _peri,
            _phantom: PhantomData,
        }
    }

    fn configure(config: Config) {
        regs().mctl().modify(|w| {
            w.set_rst_def(true);
            w.set_prgm(true);
            w.set_err(true)
        });

        regs().scml().write(|w| {
            w.set_mono_max(config.monobit_limit_max);
            w.set_mono_rng(config.monobit_limit_range);
        });

        regs().scr1l().write(|w| {
            w.set_run1_max(config.run_length1_limit_max);
            w.set_run1_rng(config.run_length1_limit_range);
        });

        regs().scr2l().write(|w| {
            w.set_run2_max(config.run_length2_limit_max);
            w.set_run2_rng(config.run_length2_limit_range);
        });

        regs().scr3l().write(|w| {
            w.set_run3_max(config.run_length3_limit_max);
            w.set_run3_rng(config.run_length3_limit_range);
        });

        regs().scr4l().write(|w| {
            w.set_run4_max(config.run_length4_limit_max);
            w.set_run4_rng(config.run_length4_limit_range);
        });

        regs().scr5l().write(|w| {
            w.set_run5_max(config.run_length5_limit_max);
            w.set_run5_rng(config.run_length5_limit_range);
        });

        regs().scr6pl().write(|w| {
            w.set_run6p_max(config.run_length6_limit_max);
            w.set_run6p_rng(config.run_length6_limit_range);
        });

        regs().pkrmax().write(|w| w.set_pkr_max(config.poker_limit_max));

        regs().frqmax().write(|w| w.set_frq_max(config.freq_counter_max));

        regs().frqmin().write(|w| w.set_frq_min(config.freq_counter_min));

        regs().sblim().write(|w| w.set_sb_lim(config.sparse_bit_limit));

        regs().scmisc().write(|w| {
            w.set_lrun_max(config.long_run_limit_max);
            w.set_rty_ct(config.retry_count);
        });

        regs().mctl().modify(|w| w.set_dis_slf_tst(config.self_test.into()));

        regs().sdctl().write(|w| {
            w.set_samp_size(config.sample_size);
            w.set_ent_dly(config.entropy_delay);
        });

        regs().osc2_ctl().modify(|w| w.set_trng_ent_ctl(config.osc_mode.into()));

        regs().mctl().modify(|w| w.set_prgm(false));

        let _ = regs().ent(7).read();

        Self::start();
    }

    fn start() {
        regs().mctl().modify(|w| w.set_trng_acc(true));
    }

    fn stop() {
        regs().mctl().modify(|w| w.set_trng_acc(false));
    }

    fn blocking_wait_for_generation() {
        while !regs().mctl().read().ent_val() {
            if regs().mctl().read().err() {
                regs().mctl().modify(|w| w.set_err(true));
            }
        }
    }

    fn fill_chunk(chunk: &mut [u8]) {
        let mut entropy = [0u32; 8];

        for (i, item) in entropy.iter_mut().enumerate() {
            *item = regs().ent(i).read().ent();
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

        for chunk in buf.chunks_mut(32) {
            Self::blocking_wait_for_generation();
            Self::fill_chunk(chunk);
        }
    }

    /// Return a random u32, blocking version.
    pub fn blocking_next_u32(&mut self) -> u32 {
        Self::blocking_wait_for_generation();
        // New random bytes are generated only after reading ENT7
        regs().ent(7).read().ent()
    }

    /// Return a random u64, blocking version.
    pub fn blocking_next_u64(&mut self) -> u64 {
        Self::blocking_wait_for_generation();

        let mut result = u64::from(regs().ent(6).read().ent()) << 32;
        // New random bytes are generated only after reading ENT7
        result |= u64::from(regs().ent(7).read().ent());
        result
    }

    /// Return the full block of `[u32; 8]` generated by the hardware,
    /// blocking version.
    pub fn blocking_next_block(&mut self, block: &mut [u32; BLOCK_SIZE]) {
        Self::blocking_wait_for_generation();
        for (reg, result) in (0..8).map(|i| regs().ent(i)).zip(block.iter_mut()) {
            *result = reg.read().ent();
        }
    }
}

impl<'d> Trng<'d, Blocking> {
    /// Instantiates a new TRNG peripheral driver with 128 samples of entropy.
    pub fn new_blocking_128(_peri: Peri<'d, TRNG0>) -> Self {
        Self::new_inner(
            _peri,
            Config {
                sample_size: 128,
                retry_count: 1,
                long_run_limit_max: 29,
                monobit_limit_max: 94,
                monobit_limit_range: 61,
                run_length1_limit_max: 39,
                run_length1_limit_range: 39,
                run_length2_limit_max: 24,
                run_length2_limit_range: 25,
                run_length3_limit_max: 17,
                run_length3_limit_range: 18,
                ..Default::default()
            },
        )
    }

    /// Instantiates a new TRNG peripheral driver with  256 samples of entropy.
    pub fn new_blocking_256(_peri: Peri<'d, TRNG0>) -> Self {
        Self::new_inner(
            _peri,
            Config {
                sample_size: 256,
                retry_count: 1,
                long_run_limit_max: 31,
                monobit_limit_max: 171,
                monobit_limit_range: 86,
                run_length1_limit_max: 63,
                run_length1_limit_range: 56,
                run_length2_limit_max: 38,
                run_length2_limit_range: 38,
                run_length3_limit_max: 25,
                run_length3_limit_range: 26,
                ..Default::default()
            },
        )
    }

    /// Instantiates a new TRNG peripheral driver with 512 samples of entropy.
    pub fn new_blocking_512(_peri: Peri<'d, TRNG0>) -> Self {
        Self::new_inner(_peri, Default::default())
    }

    /// Instantiates a new TRNG peripheral driver.
    ///
    /// NOTE: this constructor makes no attempt at validating the
    /// parameters. If you get this wrong, the security guarantees of
    /// the TRNG with regards to entropy may be violated
    pub fn new_blocking_with_custom_config(_peri: Peri<'d, TRNG0>, config: Config) -> Self {
        Self::new_inner(_peri, config)
    }
}

impl<'d> Trng<'d, Async> {
    /// Instantiates a new TRNG peripheral driver with 128 samples of entropy.
    pub fn new_128(
        _peri: Peri<'d, TRNG0>,
        _irq: impl crate::interrupt::typelevel::Binding<typelevel::TRNG0, InterruptHandler> + 'd,
    ) -> Self {
        let inst = Self::new_inner(
            _peri,
            Config {
                sample_size: 128,
                retry_count: 1,
                long_run_limit_max: 29,
                monobit_limit_max: 94,
                monobit_limit_range: 61,
                run_length1_limit_max: 39,
                run_length1_limit_range: 39,
                run_length2_limit_max: 24,
                run_length2_limit_range: 25,
                run_length3_limit_max: 17,
                run_length3_limit_range: 18,
                ..Default::default()
            },
        );
        crate::pac::Interrupt::TRNG0.unpend();
        unsafe {
            crate::pac::Interrupt::TRNG0.enable();
        }
        inst
    }

    /// Instantiates a new TRNG peripheral driver with 256 samples of entropy.
    pub fn new_256(
        _peri: Peri<'d, TRNG0>,
        _irq: impl crate::interrupt::typelevel::Binding<typelevel::TRNG0, InterruptHandler> + 'd,
    ) -> Self {
        let inst = Self::new_inner(
            _peri,
            Config {
                sample_size: 256,
                retry_count: 1,
                long_run_limit_max: 31,
                monobit_limit_max: 171,
                monobit_limit_range: 86,
                run_length1_limit_max: 63,
                run_length1_limit_range: 56,
                run_length2_limit_max: 38,
                run_length2_limit_range: 38,
                run_length3_limit_max: 25,
                run_length3_limit_range: 26,
                ..Default::default()
            },
        );
        crate::pac::Interrupt::TRNG0.unpend();
        unsafe {
            crate::pac::Interrupt::TRNG0.enable();
        }
        inst
    }

    /// Instantiates a new TRNG peripheral driver with 512 samples of entropy.
    pub fn new_512(
        _peri: Peri<'d, TRNG0>,
        _irq: impl crate::interrupt::typelevel::Binding<typelevel::TRNG0, InterruptHandler> + 'd,
    ) -> Self {
        let inst = Self::new_inner(_peri, Default::default());
        crate::pac::Interrupt::TRNG0.unpend();
        unsafe {
            crate::pac::Interrupt::TRNG0.enable();
        }
        inst
    }

    /// Instantiates a new TRNG peripheral driver.
    ///
    /// NOTE: this constructor makes no attempt at validating the
    /// parameters. If you get this wrong, the security guarantees of
    /// the TRNG with regards to entropy may be violated
    pub fn new_with_custom_config(
        _peri: Peri<'d, TRNG0>,
        _irq: impl crate::interrupt::typelevel::Binding<typelevel::TRNG0, InterruptHandler> + 'd,
        config: Config,
    ) -> Self {
        let inst = Self::new_inner(_peri, config);
        crate::pac::Interrupt::TRNG0.unpend();
        unsafe {
            crate::pac::Interrupt::TRNG0.enable();
        }
        inst
    }

    fn enable_ints() {
        regs().int_mask().write(|w| {
            w.set_hw_err(true);
            w.set_ent_val(true);
            w.set_frq_ct_fail(true);
            w.set_intg_flt(true);
        });
    }

    async fn wait_for_generation() -> Result<(), Error> {
        WAIT_CELL
            .wait_for(|| {
                Self::enable_ints();
                regs().mctl().read().ent_val()
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

        for chunk in buf.chunks_mut(32) {
            Self::wait_for_generation().await?;
            Self::fill_chunk(chunk);
        }

        Ok(())
    }

    /// Return a random u32, async version.
    pub async fn async_next_u32(&mut self) -> Result<u32, Error> {
        Self::wait_for_generation().await?;
        // New random bytes are generated only after reading ENT7
        Ok(regs().ent(7).read().ent())
    }

    /// Return a random u64, async version.
    pub async fn async_next_u64(&mut self) -> Result<u64, Error> {
        Self::wait_for_generation().await?;

        let mut result = u64::from(regs().ent(6).read().ent()) << 32;
        // New random bytes are generated only after reading ENT7
        result |= u64::from(regs().ent(7).read().ent());

        Ok(result)
    }

    /// Return the full block of `[u32; 8]` generated by the hardware,
    /// async version.
    pub async fn async_next_block(&mut self, block: &mut [u32; BLOCK_SIZE]) -> Result<(), Error> {
        Self::wait_for_generation().await?;

        for (reg, result) in (0..8).map(|i| regs().ent(i)).zip(block.iter_mut()) {
            *result = reg.read().ent();
        }

        Ok(())
    }
}

impl<M: Mode> Drop for Trng<'_, M> {
    fn drop(&mut self) {
        // wait until allowed to stop
        while !regs().mctl().read().tstop_ok() {}
        // stop
        Self::stop();
        // reset the TRNG
        regs().mctl().write(|w| w.set_rst_def(true));
    }
}

fn regs() -> crate::pac::trng::Trng {
    crate::pac::TRNG0
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

/// TRNG interrupt handler.
pub struct InterruptHandler;

impl Handler<typelevel::TRNG0> for InterruptHandler {
    unsafe fn on_interrupt() {
        crate::perf_counters::incr_interrupt_trng();
        if regs().int_status().read().0 != 0 {
            regs().int_ctrl().write(|w| {
                w.set_hw_err(false);
                w.set_ent_val(false);
                w.set_frq_ct_fail(false);
                w.set_intg_flt(false);
            });
            crate::perf_counters::incr_interrupt_trng_wake();
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

    /// Statistical check monobit range
    pub monobit_limit_range: u16,

    /// Statistical check run length 1 limit max
    pub run_length1_limit_max: u16,

    /// Statistical check run length 1 limit range
    pub run_length1_limit_range: u16,

    /// Statistical check run length 2 limit max
    pub run_length2_limit_max: u16,

    /// Statistical check run length 2 limit range
    pub run_length2_limit_range: u16,

    /// Statistical check run length 3 limit max
    pub run_length3_limit_max: u16,

    /// Statistical check run length 3 limit range
    pub run_length3_limit_range: u16,

    /// Statistical check run length 4 limit max
    pub run_length4_limit_max: u16,

    /// Statistical check run length 4 limit range
    pub run_length4_limit_range: u16,

    /// Statistical check run length 5 limit max
    pub run_length5_limit_max: u16,

    /// Statistical check run length 5 limit range
    pub run_length5_limit_range: u16,

    /// Statistical check run length 6 limit max
    pub run_length6_limit_max: u16,

    /// Statistical check run length 6 limit range
    pub run_length6_limit_range: u16,

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
            sample_size: 512,
            entropy_delay: 32_000,
            self_test: SelfTest::Enabled,
            freq_counter_max: 75_000,
            freq_counter_min: 30_000,
            monobit_limit_max: 317,
            monobit_limit_range: 122,
            run_length1_limit_max: 107,
            run_length1_limit_range: 80,
            run_length2_limit_max: 62,
            run_length2_limit_range: 55,
            run_length3_limit_max: 39,
            run_length3_limit_range: 39,
            run_length4_limit_max: 0,
            run_length4_limit_range: 0,
            run_length5_limit_max: 0,
            run_length5_limit_range: 0,
            run_length6_limit_max: 0,
            run_length6_limit_range: 0,
            retry_count: 1,
            long_run_limit_max: 32,
            sparse_bit_limit: 0,
            poker_limit_max: 0,
            osc_mode: OscMode::DualOscs,
        }
    }
}

/// Sample size.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SampleSize {
    /// 128 bits
    _128,

    /// 256 bits
    _256,

    /// 512 bits
    _512,
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
            OscMode::SingleOsc1 => Self::TRNG_ENT_CTL_SINGLE_OSC1,
            OscMode::DualOscs => Self::TRNG_ENT_CTL_DUAL_OSCS,
            OscMode::SingleOsc2 => Self::TRNG_ENT_CTL_SINGLE_OSC2,
        }
    }
}

impl<'d, M: Mode> rand_core_06::RngCore for Trng<'d, M> {
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

impl<'d, M: Mode> rand_core_06::CryptoRng for Trng<'d, M> {}

impl<'d, M: Mode> rand_core_09::RngCore for Trng<'d, M> {
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

impl<'d, M: Mode> rand_core_09::CryptoRng for Trng<'d, M> {}

impl<'d, M: Mode> rand_core_06::block::BlockRngCore for Trng<'d, M> {
    type Item = u32;
    type Results = [Self::Item; BLOCK_SIZE];

    fn generate(&mut self, results: &mut Self::Results) {
        self.blocking_next_block(results);
    }
}

impl<'d, M: Mode> rand_core_09::block::BlockRngCore for Trng<'d, M> {
    type Item = u32;
    type Results = [Self::Item; BLOCK_SIZE];

    fn generate(&mut self, results: &mut Self::Results) {
        self.blocking_next_block(results);
    }
}

impl<'d, M: Mode> rand_core_09::block::CryptoBlockRng for Trng<'d, M> {}
