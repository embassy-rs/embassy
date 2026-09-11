//! CRACEN power management and internal access to its true random number generator.
//!
//! The AES and public key engines draw from the generator for their side-channel
//! countermeasures, whoever owns the `CRYPTO_RNG` peripheral. So the generator runs from its
//! first use until CRACEN powers down.

use core::sync::atomic::{AtomicU16, Ordering};

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

static ACTIVE_USERS: AtomicU16 = AtomicU16::new(0);

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

/// Programs the health-test cut-offs. Needed after every power-up.
#[cfg(feature = "_nrf54lm20")]
fn configure_health_tests() {
    let r = core().rngcontrol();
    r.repeatthreshold()
        .write(|w| w.set_repeatthreshold(RNG_REPEATTHRESHOLD_VAL));
    r.proptestcutoff()
        .write(|w| w.set_proptestcutoff(RNG_PROPTESTCUTOFF_VAL));
}

/// Starts the TRNG if it is not running, and waits for its start-up phase to end.
///
/// CRACEN must be powered. Callable from any context: the check-and-start is done in a short
/// critical section, since the RNG driver and the engine countermeasures share the generator.
pub(crate) fn ensure_rng_running() {
    let r = core();
    critical_section::with(|_| {
        if !r.rngcontrol().control().read().enable() {
            // Before the RNG is started, and after the power-up that cleared them.
            #[cfg(feature = "_nrf54lm20")]
            configure_health_tests();

            #[cfg(feature = "_nrf54lm20")]
            r.rngcontrol().cooldownperiod().write(|w| {
                w.set_cooldownperiod(0);
            });

            // Modify, not write: NB128BITBLOCKS defaults to 4 and zero is not a legal
            // value, so the other fields have to survive the start.
            r.rngcontrol().control().modify(|w| {
                w.set_enable(true);
            });
        }
    });

    while r.rngcontrol().status().read().state() == pac::cracencore::vals::State::Startup {}
}

/// Reads one word from the running TRNG.
pub(crate) fn read_rng_word() -> u32 {
    let r = core();
    // A failed health test parks the FSM in `Error`, where the FIFO never fills
    // again — so without this check the poll never returns, and being a blocking
    // loop in a sync fn it takes the whole executor with it. There is nothing to
    // do but fail loudly: measured on an nRF54LM20A, neither a SoftRst nor
    // reprogramming the cut-offs revives a TRNG that has already reached `Error`,
    // only a reset of the part does, and an infallible API cannot report. The
    // cut-offs programmed in `ensure_rng_running` are what keeps this unreached.
    loop {
        // Another context may pop the FIFO between the level check and the read, so the
        // two happen together. A few instructions.
        let word = critical_section::with(|_| {
            if r.rngcontrol().fifolevel().read() != 0 {
                Some(r.rngcontrol().fifo(0).read())
            } else {
                None
            }
        });
        if let Some(word) = word {
            break word;
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
