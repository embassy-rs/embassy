//! True Random Number Generator (TRNG) driver.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::Not;
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::peripherals::TRNG;
use crate::{interrupt, pac};

trait SealedInstance {
    fn regs() -> pac::trng::Trng;
    fn waker() -> &'static AtomicWaker;
}

/// TRNG peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    /// Interrupt for this peripheral.
    type Interrupt: Interrupt;
}

impl SealedInstance for TRNG {
    fn regs() -> rp_pac::trng::Trng {
        pac::TRNG
    }

    fn waker() -> &'static AtomicWaker {
        static WAKER: AtomicWaker = AtomicWaker::new();
        &WAKER
    }
}

impl Instance for TRNG {
    type Interrupt = interrupt::typelevel::TRNG_IRQ;
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
/// TRNG ROSC Inverter chain length options.
pub enum InverterChainLength {
    None = 0,
    One,
    Two,
    Three,
    Four,
}

impl From<InverterChainLength> for u8 {
    fn from(value: InverterChainLength) -> Self {
        value as u8
    }
}

/// Configuration for the TRNG.
///
/// - Three built in entropy checks
/// - ROSC frequency controlled by selecting one of ROSC chain lengths
/// - Sample period in terms of system clock ticks
///
/// The RP2350 datasheet (12.12.2) suggests sample count settings of 20-25 for
/// an average generation time of about 2 milliseconds. On real hardware that
/// setting only works while the core is busy: with the core sleeping in WFE
/// (which is what the async executor does while waiting for the TRNG
/// interrupt) the ring oscillator output becomes regular enough that every
/// generation fails the autocorrelation or CRNGT health check.
///
/// Measured on a Pico 2 with the core sleeping, filling 1024 bytes:
///
/// | `sample_count` | result                     |
/// |----------------|----------------------------|
/// | 25             | never completes            |
/// | 50             | 761 ms, many failed checks |
/// | 100            | 19 ms                      |
/// | 200            | 38 ms                      |
/// | 1000           | 232 ms                     |
///
/// The default is 200. Autocorrelation failures are retried automatically
/// (up to 1000 times per block before panicking), so a marginal value degrades
/// into slower generation rather than a hang. A value of 25 with a sleeping
/// core fails every attempt and panics.
///
/// Note, Pico SDK and Bootrom don't use any of the entropy checks and sample the ROSC directly
/// by setting the sample period to 0. Random data collected this way is then passed through
/// either hardware accelerated SHA256 (Bootrom) or xoroshiro128** (version 1.0!).
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Bypass TRNG autocorrelation test
    pub disable_autocorrelation_test: bool,
    /// Bypass CRNGT test
    pub disable_crngt_test: bool,
    /// When set, the Von-Neuman balancer is bypassed (including the
    /// 32 consecutive bits test)
    pub disable_von_neumann_balancer: bool,
    /// Sets the number of rng_clk cycles between two consecutive
    /// ring oscillator samples.
    /// Note: If the von Neumann decorrelator is bypassed, the minimum value for
    /// sample counter must not be less than seventeen
    pub sample_count: u32,
    /// Selects the number of inverters (out of four possible
    /// selections) in the ring oscillator (the entropy source). Higher values select
    /// longer inverter chain lengths.
    pub inverter_chain_length: InverterChainLength,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // WARNING: Disabling these tests increases likelihood of poor rng results.
            disable_autocorrelation_test: false,
            disable_crngt_test: false,
            disable_von_neumann_balancer: false,
            sample_count: 200,
            inverter_chain_length: InverterChainLength::One,
        }
    }
}

/// True Random Number Generator Driver for RP2350
///
/// This driver provides async and blocking options.
///
/// See [Config] for configuration details.
///
/// Usage example:
/// ```no_run
/// use embassy_executor::Spawner;
/// use embassy_rp::trng::Trng;
/// use embassy_rp::peripherals::TRNG;
/// use embassy_rp::bind_interrupts;
///
/// bind_interrupts!(struct Irqs {
///     TRNG_IRQ => embassy_rp::trng::InterruptHandler<TRNG>;
/// });
///
/// #[embassy_executor::main]
/// async fn main(spawner: Spawner) {
///     let peripherals = embassy_rp::init(Default::default());
///     let mut trng = Trng::new(peripherals.TRNG, Irqs, embassy_rp::trng::Config::default());
///
///     let mut randomness = [0u8; 58];
///     loop {
///         trng.fill_bytes(&mut randomness).await;
///         assert_ne!(randomness, [0u8; 58]);
///     }
///}
/// ```
pub struct Trng<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    config: Config,
}

/// 12.12.1. Overview
/// On request, the TRNG block generates a block of 192 entropy bits generated by automatically processing a series of
/// periodic samples from the TRNG block’s internal Ring Oscillator (ROSC).
const TRNG_BLOCK_SIZE_BITS: usize = 192;
const TRNG_BLOCK_SIZE_BYTES: usize = TRNG_BLOCK_SIZE_BITS / 8;

/// Consecutive failed health checks tolerated before giving up on a block.
///
/// Health check failures are expected and are retried transparently. Hitting this
/// many in a row means the ROSC is not producing usable entropy at all with the
/// current [`Config`].
const MAX_HEALTH_CHECK_RETRIES: u32 = 1000;

impl<'d, T: Instance> Trng<'d, T> {
    /// Create a new TRNG driver.
    pub fn new(_trng: Peri<'d, T>, _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd, config: Config) -> Self {
        let trng = Trng {
            phantom: PhantomData,
            config,
        };
        trng.reset_rng();
        trng
    }

    /// Clear all status bits and unmask all interrupt sources, then start generation.
    ///
    /// Clearing the status first means a stale bit from an earlier run cannot be
    /// mistaken for a failure of the run being started.
    fn start_rng(&self) {
        let regs = T::regs();
        regs.rng_icr().write(|w| {
            w.set_ehr_valid(true);
            w.set_autocorr_err(true);
            w.set_crngt_err(true);
            w.set_vn_err(true);
        });
        self.unmask_irq();
        regs.rnd_source_enable().write(|w| w.set_rnd_src_en(true));
    }

    fn stop_rng(&self) {
        let regs = T::regs();
        regs.rnd_source_enable().write(|w| w.set_rnd_src_en(false));
        regs.rst_bits_counter().write(|w| w.set_rst_bits_counter(true));
    }

    /// Soft-reset the TRNG and apply the configuration.
    ///
    /// The reset takes a number of cycles to complete and register writes made
    /// during that window are lost. Like the Arm CryptoCell reference driver,
    /// keep writing the sample count until it reads back before configuring the
    /// rest.
    fn reset_rng(&self) {
        let regs = T::regs();
        regs.trng_sw_reset().write(|w| w.set_trng_sw_reset(true));
        loop {
            regs.sample_cnt1().write(|w| *w = self.config.sample_count);
            if regs.sample_cnt1().read() == self.config.sample_count {
                break;
            }
        }
        self.initialize_rng();
    }

    fn initialize_rng(&self) {
        let regs = T::regs();

        regs.trng_config().write(|w| {
            w.set_rnd_src_sel(self.config.inverter_chain_length.into());
        });

        regs.sample_cnt1().write(|w| {
            *w = self.config.sample_count;
        });

        regs.trng_debug_control().write(|w| {
            w.set_auto_correlate_bypass(self.config.disable_autocorrelation_test);
            w.set_trng_crngt_bypass(self.config.disable_crngt_test);
            w.set_vnc_bypass(self.config.disable_von_neumann_balancer);
        });
    }

    fn enable_irq(&self) {
        // A pending interrupt left over from a blocking call would otherwise
        // fire immediately and mask the interrupt before the real one.
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() }
    }

    /// Handle health check status bits. Returns true if generation had to be restarted.
    ///
    /// A CRNGT or von Neumann error means a stretch of bits was rejected. The
    /// block keeps running, so those bits only need to be cleared. The
    /// autocorrelation error stops the block, and only a reset clears its status
    /// bit, so generation is reset and restarted.
    fn handle_health_check_status(&self) -> bool {
        let regs = T::regs();
        let isr = regs.rng_isr().read();
        if isr.autocorr_err() {
            self.reset_rng();
            self.start_rng();
            return true;
        }
        if isr.crngt_err() || isr.vn_err() {
            regs.rng_icr().write(|w| {
                w.set_crngt_err(true);
                w.set_vn_err(true);
            });
        }
        false
    }

    /// Unmask all interrupt sources. The interrupt handler masks them when it fires.
    fn unmask_irq(&self) {
        T::regs().rng_imr().write(|w| {
            w.set_ehr_valid_int_mask(false);
            w.set_autocorr_err_int_mask(false);
            w.set_crngt_err_int_mask(false);
            w.set_vn_err_int_mask(false);
        });
    }

    fn blocking_wait_for_successful_generation(&self) {
        let regs = T::regs();
        let mut failures = 0;
        while regs.trng_valid().read().ehr_valid().not() {
            if self.handle_health_check_status() {
                failures += 1;
                if failures >= MAX_HEALTH_CHECK_RETRIES {
                    panic!(
                        "TRNG: {} consecutive health check failures. Increase Config::sample_count.",
                        MAX_HEALTH_CHECK_RETRIES
                    );
                }
            }
        }
    }

    /// Read out a completed block. Reading `EHR_DATA5` clears the result registers
    /// and starts the next generation.
    fn read_ehr_registers_into_array(&mut self, buffer: &mut [u8; TRNG_BLOCK_SIZE_BYTES]) {
        let regs = T::regs();
        let ehr_data_regs = [
            regs.ehr_data0(),
            regs.ehr_data1(),
            regs.ehr_data2(),
            regs.ehr_data3(),
            regs.ehr_data4(),
            regs.ehr_data5(),
        ];

        for (i, reg) in ehr_data_regs.iter().enumerate() {
            buffer[i * 4..i * 4 + 4].copy_from_slice(&reg.read().to_ne_bytes());
        }
    }

    /// Fill the buffer with random bytes, async version.
    pub async fn fill_bytes(&mut self, destination: &mut [u8]) {
        if destination.is_empty() {
            return; // Nothing to fill
        }

        self.start_rng();
        self.enable_irq();

        // Stop the block and the interrupt on completion and on cancellation.
        let _guard = OnDrop::new(|| {
            let regs = T::regs();
            regs.rnd_source_enable().write(|w| w.set_rnd_src_en(false));
            regs.rst_bits_counter().write(|w| w.set_rst_bits_counter(true));
            T::Interrupt::disable();
        });

        let mut bytes_transferred = 0usize;
        let mut failures = 0;
        let mut buffer = [0u8; TRNG_BLOCK_SIZE_BYTES];

        let regs = T::regs();
        let waker = T::waker();
        let destination_length = destination.len();

        poll_fn(|context| {
            waker.register(context.waker());
            loop {
                if bytes_transferred == destination_length {
                    return Poll::Ready(());
                }
                if regs.trng_valid().read().ehr_valid() {
                    self.read_ehr_registers_into_array(&mut buffer);
                    let n = (destination_length - bytes_transferred).min(TRNG_BLOCK_SIZE_BYTES);
                    destination[bytes_transferred..bytes_transferred + n].copy_from_slice(&buffer[..n]);
                    bytes_transferred += n;
                    failures = 0;
                    regs.rng_icr().write(|w| w.set_ehr_valid(true));
                    continue;
                }
                if self.handle_health_check_status() {
                    failures += 1;
                    if failures >= MAX_HEALTH_CHECK_RETRIES {
                        panic!(
                            "TRNG: {} consecutive health check failures. Increase Config::sample_count.",
                            MAX_HEALTH_CHECK_RETRIES
                        );
                    }
                }
                // The interrupt handler masks the interrupt when it fires;
                // rearm it now that the status has been handled.
                self.unmask_irq();
                return Poll::Pending;
            }
        })
        .await
    }

    /// Fill the buffer with random bytes, blocking version.
    pub fn blocking_fill_bytes(&mut self, destination: &mut [u8]) {
        if destination.is_empty() {
            return; // Nothing to fill
        }
        self.start_rng();

        let mut buffer = [0u8; TRNG_BLOCK_SIZE_BYTES];

        for chunk in destination.chunks_mut(TRNG_BLOCK_SIZE_BYTES) {
            self.blocking_wait_for_successful_generation();
            self.read_ehr_registers_into_array(&mut buffer);
            chunk.copy_from_slice(&buffer[..chunk.len()])
        }
        self.stop_rng()
    }

    /// Return a random u32, blocking.
    pub fn blocking_next_u32(&mut self) -> u32 {
        let regs = T::regs();
        self.start_rng();
        self.blocking_wait_for_successful_generation();
        // 12.12.3 After successful generation, read the last result register, EHR_DATA[5] to
        // clear all of the result registers.
        let result = regs.ehr_data5().read();
        self.stop_rng();
        result
    }

    /// Return a random u64, blocking.
    pub fn blocking_next_u64(&mut self) -> u64 {
        let regs = T::regs();
        self.start_rng();
        self.blocking_wait_for_successful_generation();

        let low = regs.ehr_data4().read() as u64;
        // 12.12.3 After successful generation, read the last result register, EHR_DATA[5] to
        // clear all of the result registers.
        let result = (regs.ehr_data5().read() as u64) << 32 | low;
        self.stop_rng();
        result
    }
}

impl<'d, T: Instance> rand_core_06::RngCore for Trng<'d, T> {
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

impl<'d, T: Instance> rand_core_06::CryptoRng for Trng<'d, T> {}

impl<'d, T: Instance> rand_core_09::RngCore for Trng<'d, T> {
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

impl<'d, T: Instance> rand_core_09::CryptoRng for Trng<'d, T> {}

impl<'d, T: Instance> rand_core_10::TryRng for Trng<'d, T> {
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

impl<'d, T: Instance> rand_core_10::TryCryptoRng for Trng<'d, T> {}

/// TRNG interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _trng: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let isr = regs.rng_isr().read();
        if isr.ehr_valid() || isr.autocorr_err() || isr.crngt_err() || isr.vn_err() {
            // The interrupt is level triggered from the status bits, and the
            // autocorrelation error can only be cleared by a reset, so mask
            // everything here and let the task handle the event and rearm.
            regs.rng_imr().write(|w| {
                w.set_ehr_valid_int_mask(true);
                w.set_autocorr_err_int_mask(true);
                w.set_crngt_err_int_mask(true);
                w.set_vn_err_int_mask(true);
            });
            T::waker().wake();
        }
    }
}
