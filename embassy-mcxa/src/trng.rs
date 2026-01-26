//! True Random Number Generator

use core::marker::PhantomData;

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
        regs()
            .mctl()
            .modify(|_, w| w.rst_def().set_bit().prgm().enable().err().clear_bit_by_one());

        regs().scml().write(|w| unsafe {
            w.mono_max()
                .bits(config.monobit_limit_max)
                .mono_rng()
                .bits(config.monobit_limit_range)
        });

        regs().scr1l().write(|w| unsafe {
            w.run1_max()
                .bits(config.run_length1_limit_max)
                .run1_rng()
                .bits(config.run_length1_limit_range)
        });

        regs().scr2l().write(|w| unsafe {
            w.run2_max()
                .bits(config.run_length2_limit_max)
                .run2_rng()
                .bits(config.run_length2_limit_range)
        });

        regs().scr3l().write(|w| unsafe {
            w.run3_max()
                .bits(config.run_length3_limit_max)
                .run3_rng()
                .bits(config.run_length3_limit_range)
        });

        regs().scr4l().write(|w| unsafe {
            w.run4_max()
                .bits(config.run_length4_limit_max)
                .run4_rng()
                .bits(config.run_length4_limit_range)
        });

        regs().scr5l().write(|w| unsafe {
            w.run5_max()
                .bits(config.run_length5_limit_max)
                .run5_rng()
                .bits(config.run_length5_limit_range)
        });

        regs().scr6pl().write(|w| unsafe {
            w.run6p_max()
                .bits(config.run_length6_limit_max)
                .run6p_rng()
                .bits(config.run_length6_limit_range)
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

        Self::start();
    }

    fn start() {
        regs().mctl().modify(|_, w| w.trng_acc().set_bit());
    }

    fn stop() {
        regs().mctl().modify(|_, w| w.trng_acc().clear_bit());
    }

    fn blocking_wait_for_generation() {
        while regs().mctl().read().ent_val().bit_is_clear() {
            if regs().mctl().read().err().bit_is_set() {
                regs().mctl().modify(|_, w| w.err().clear_bit_by_one());
            }
        }
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

        for chunk in buf.chunks_mut(32) {
            Self::blocking_wait_for_generation();
            Self::fill_chunk(chunk);
        }
    }

    /// Return a random u32, blocking version.
    pub fn blocking_next_u32(&mut self) -> u32 {
        Self::blocking_wait_for_generation();
        // New random bytes are generated only after reading ENT7
        regs().ent(7).read().bits()
    }

    /// Return a random u64, blocking version.
    pub fn blocking_next_u64(&mut self) -> u64 {
        Self::blocking_wait_for_generation();

        let mut result = u64::from(regs().ent(6).read().bits()) << 32;
        // New random bytes are generated only after reading ENT7
        result |= u64::from(regs().ent(7).read().bits());
        result
    }

    /// Return the full block of `[u32; 8]` generated by the hardware,
    /// blocking version.
    pub fn blocking_next_block(&mut self, block: &mut [u32; BLOCK_SIZE]) {
        Self::blocking_wait_for_generation();
        for (reg, result) in regs().ent_iter().zip(block.iter_mut()) {
            *result = reg.read().bits();
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
        Ok(regs().ent(7).read().bits())
    }

    /// Return a random u64, async version.
    pub async fn async_next_u64(&mut self) -> Result<u64, Error> {
        Self::wait_for_generation().await?;

        let mut result = u64::from(regs().ent(6).read().bits()) << 32;
        // New random bytes are generated only after reading ENT7
        result |= u64::from(regs().ent(7).read().bits());

        Ok(result)
    }

    /// Return the full block of `[u32; 8]` generated by the hardware,
    /// async version.
    pub async fn async_next_block(&mut self, block: &mut [u32; BLOCK_SIZE]) -> Result<(), Error> {
        Self::wait_for_generation().await?;

        for (reg, result) in regs().ent_iter().zip(block.iter_mut()) {
            *result = reg.read().bits();
        }

        Ok(())
    }
}

impl<M: Mode> Drop for Trng<'_, M> {
    fn drop(&mut self) {
        // wait until allowed to stop
        while regs().mctl().read().tstop_ok().bit_is_clear() {}
        // stop
        Self::stop();
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

/// TRNG interrupt handler.
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
            OscMode::SingleOsc1 => Self::TrngEntCtlSingleOsc1,
            OscMode::DualOscs => Self::TrngEntCtlDualOscs,
            OscMode::SingleOsc2 => Self::TrngEntCtlSingleOsc2,
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
