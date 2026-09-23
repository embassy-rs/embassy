//! CRACEN power management and internal access to its true random number generator.
//!
//! The AES and public key engines draw from the generator for their side-channel
//! countermeasures, whoever owns the `CRYPTO_RNG` peripheral. So the generator runs from its
//! first use until CRACEN powers down.

use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use crate::pac;

pub(crate) mod cmdma;

/// TRNG health-test cut-offs for CRACEN Lite, from NCS `RNG_REPEATTHRESHOLD_VAL` and
/// `RNG_PROPTESTCUTOFF_VAL`.
///
/// On CRACEN Lite these registers reset to 4 and 13. A working noise source fails those
/// tests. NCS reprograms them on every power-up, as a workaround for the bad hardware
/// defaults. The registers revert whenever CRACEN powers down.
///
/// nRF54L15/L10/L05 reset them to sane values (41 and 793), so only CRACEN Lite gets this.
#[cfg(feature = "_nrf54lm20")]
const RNG_REPEATTHRESHOLD_VAL: u8 = 21;
#[cfg(feature = "_nrf54lm20")]
const RNG_PROPTESTCUTOFF_VAL: u16 = 311;

/// Number of 128-bit blocks the AES conditioning function draws per output block. Zero is
/// not a legal value, and the register is cleared by the soft reset, so it is programmed on
/// every start. This is the reset value, and what NCS programs.
const RNG_NB128BITBLOCKS: u8 = 4;

/// TRNG timing, from NCS `nrfx_cracen.c`. The reset values are `0xffff` for both timers,
/// which keeps the ring oscillators spinning long after the FIFO is full.
#[cfg(not(feature = "_nrf54lm20"))]
const RNG_OFF_TIMER_VAL: u16 = 0;
#[cfg(not(feature = "_nrf54lm20"))]
const RNG_CLK_DIV: u8 = 0;
#[cfg(not(feature = "_nrf54lm20"))]
const RNG_INIT_WAIT_VAL: u16 = 512;

/// Size of the AES conditioning key, in words.
#[cfg(not(feature = "_nrf54lm20"))]
const RNG_KEY_WORDS: usize = 4;

/// How many times a start is retried before giving up on the generator.
///
/// A health test trips every few thousand starts on an nRF54L15, so one retry is normally
/// enough. Failing this many times in a row means the noise source is broken.
const RNG_MAX_STARTS: u32 = 16;

static ACTIVE_USERS: AtomicU16 = AtomicU16::new(0);

/// Whether the generator is started, configured, and past its start-up tests.
///
/// Cleared when CRACEN powers down, which stops the generator and resets its configuration,
/// and when a health test trips, so that the next use restarts it.
static RNG_READY: AtomicBool = AtomicBool::new(false);

/// Powers CRACEN up if it is not already, and keeps it powered until the returned handle is
/// dropped.
pub(crate) fn activate() -> ActivationHandle {
    // The count and the enable register have to change together, so that a release racing
    // with this activation cannot switch the block off underneath it. A few instructions.
    critical_section::with(|_| {
        if ACTIVE_USERS.fetch_add(1, Ordering::Relaxed) == 0 {
            pac::CRACEN.enable().write(|w| {
                w.set_cryptomaster(true);
                w.set_rng(true);
                w.set_pkeikg(true);
            });
        }
    });
    ActivationHandle { _private: () }
}

fn release() {
    critical_section::with(|_| {
        if ACTIVE_USERS.fetch_sub(1, Ordering::Relaxed) == 1 {
            // Powering down also stops the generator and resets its configuration.
            RNG_READY.store(false, Ordering::Relaxed);
            pac::CRACEN.enable().write(|w| {
                w.set_cryptomaster(false);
                w.set_rng(false);
                w.set_pkeikg(false);
            });
        }
    });
}

/// Keeps CRACEN powered while it exists.
pub(crate) struct ActivationHandle {
    _private: (),
}

impl Drop for ActivationHandle {
    fn drop(&mut self) {
        release()
    }
}

fn core() -> pac::cracencore::Cracencore {
    pac::CRACENCORE
}

/// Programs the health-test cut-offs. Needed after every start.
///
/// On CRACEN Lite these registers reset to values a working noise source fails.
#[cfg(feature = "_nrf54lm20")]
fn configure_health_tests() {
    let r = core().rngcontrol();
    r.repeatthreshold()
        .write(|w| w.set_repeatthreshold(RNG_REPEATTHRESHOLD_VAL));
    r.proptestcutoff()
        .write(|w| w.set_proptestcutoff(RNG_PROPTESTCUTOFF_VAL));
}

/// Configures the generator and starts it, following NCS `nrfx_cracen.c`.
///
/// The soft reset clears the continuous tests, the conditioning function and the FIFO. It is
/// what brings back a generator that a health-test failure has parked in `Error`, so this
/// runs on a restart as well as on the first start after a power-up.
fn start_rng() {
    let r = core().rngcontrol();

    #[cfg(not(feature = "_nrf54lm20"))]
    r.control()
        .write(|w| w.set_softrst(pac::cracencore::vals::ControlSoftrst::Ctest));
    #[cfg(feature = "_nrf54lm20")]
    r.control().write(|w| w.set_softrst(true));

    #[cfg(feature = "_nrf54lm20")]
    {
        configure_health_tests();
        r.cooldownperiod().write(|w| w.set_cooldownperiod(0));
    }
    #[cfg(not(feature = "_nrf54lm20"))]
    {
        r.swofftmrval().write(|w| w.set_swofftmrval(RNG_OFF_TIMER_VAL));
        r.clkdiv().write(|w| w.set_clkdiv(RNG_CLK_DIV));
        r.initwaitval().write(|w| w.set_initwaitval(RNG_INIT_WAIT_VAL));
    }

    // Clears the soft reset and starts the generator.
    r.control().write(|w| {
        w.set_enable(true);
        w.set_nb128bitblocks(RNG_NB128BITBLOCKS);
    });
}

/// Starts the generator and waits for it to produce entropy.
///
/// Returns `false` if a health test tripped on the way, which leaves the generator in
/// `Error` for the caller to restart.
fn start_rng_and_wait() -> bool {
    let r = core().rngcontrol();
    start_rng();

    loop {
        match r.status().read().state() {
            // The start-up tests failed, or an AIS31 noise alarm fired.
            pac::cracencore::vals::State::Error => return false,
            // Still starting up.
            pac::cracencore::vals::State::Reset | pac::cracencore::vals::State::Startup => continue,
            _ => {}
        }

        // The conditioning function runs with a key drawn from the generator's own first
        // output. NCS does the same; its entropy is only NIST 800-90B and AIS31 compliant
        // this way. CRACEN Lite has no key register.
        #[cfg(not(feature = "_nrf54lm20"))]
        {
            if (r.fifolevel().read() as usize) < RNG_KEY_WORDS {
                continue;
            }
            for i in 0..RNG_KEY_WORDS {
                let word = r.fifo(0).read();
                r.key(i).write_value(word);
            }
        }

        return true;
    }
}

/// Starts the TRNG if it is not running, and waits for its start-up phase to end.
///
/// CRACEN must be powered. Callable from any context: the whole start happens in one
/// critical section, since the RNG driver and the engine countermeasures share the
/// generator.
pub(crate) fn ensure_rng_running() {
    if RNG_READY.load(Ordering::Relaxed) {
        return;
    }

    critical_section::with(|_| {
        // Another context may have started it while this one waited for the lock.
        if RNG_READY.load(Ordering::Relaxed) {
            return;
        }

        for _ in 0..RNG_MAX_STARTS {
            if start_rng_and_wait() {
                RNG_READY.store(true, Ordering::Relaxed);
                return;
            }
        }

        panic!("CRACEN RNG failed its health tests {} times in a row", RNG_MAX_STARTS);
    });
}

/// Reads one word from the running TRNG.
pub(crate) fn read_rng_word() -> u32 {
    let r = core().rngcontrol();
    loop {
        // Another context may pop the FIFO between the level check and the read, so the
        // two happen together. A few instructions.
        let word = critical_section::with(|_| {
            // A failed health test parks the FSM in `Error`, where the FIFO never fills
            // again — so without this check the poll below never returns, and being a
            // blocking loop in a sync fn it would take the whole executor with it. The
            // entropy still in the FIFO is discarded along with the restart.
            if r.status().read().state() == pac::cracencore::vals::State::Error {
                RNG_READY.store(false, Ordering::Relaxed);
                return None;
            }
            if r.fifolevel().read() != 0 {
                Some(r.fifo(0).read())
            } else {
                None
            }
        });
        if let Some(word) = word {
            break word;
        }
        ensure_rng_running();
    }
}

/// Returns one word of entropy from the TRNG. CRACEN must be powered.
pub(crate) fn random_word() -> u32 {
    let mut word = [0];
    random_words(&mut word);
    word[0]
}

/// Fills `out` with random words. CRACEN must be powered.
pub(crate) fn random_words(out: &mut [u32]) {
    ensure_rng_running();
    for word in out.iter_mut() {
        *word = read_rng_word();
    }
}
