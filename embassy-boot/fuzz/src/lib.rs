//! Coverage-guided fuzzing of the embassy-boot power-fail test.
//!
//! The simulator, the flash models and the invariants are the files of `tests/power_fail/`, included as they
//! are, so an input here is the same byte string as `POWER_FAIL_REPLAY` takes: byte 0 picks the model, the next
//! four the layout, the rest are steps.
//!
//! [`check`] runs one input. A failure of a known class (see the top of `tests/power_fail.rs` and
//! `zoo::expect`) is counted, not a crash, unless `POWER_FAIL_KNOWN_CRASH` asks for it. Anything else panics
//! with the full report, which is what both fuzzers record as a crash.
#![allow(dead_code)]

#[path = "../../tests/power_fail/flash.rs"]
pub mod flash;
#[path = "../../tests/power_fail/scenario.rs"]
pub mod scenario;
#[path = "../../tests/power_fail/zoo.rs"]
pub mod zoo;

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use zoo::Expect;

/// Which known classes are crashes: `POWER_FAIL_KNOWN_CRASH` is `all`, or a comma-separated list of `class`
/// or `class:Model` (for example `nwrite-exceeded:Nrf52`, `torn-magic-reads-swap:Qspi`).
struct KnownCrash(Vec<(String, Option<String>)>, bool);

impl KnownCrash {
    fn from_env() -> Self {
        let value = std::env::var("POWER_FAIL_KNOWN_CRASH").unwrap_or_default();
        let all = value.trim() == "all";
        let list = value
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "all")
            .map(|s| match s.split_once(':') {
                Some((class, model)) => (class.to_string(), Some(model.to_lowercase())),
                None => (s.to_string(), None),
            })
            .collect();
        Self(list, all)
    }

    fn matches(&self, class: &str, model: &str) -> bool {
        self.1
            || self
                .0
                .iter()
                .any(|(c, m)| c == class && m.as_ref().is_none_or(|m| *m == model.to_lowercase()))
    }
}

#[derive(Default)]
struct Counts {
    runs: u64,
    known: BTreeMap<&'static str, u64>,
}

fn state() -> &'static Mutex<(KnownCrash, Counts)> {
    static STATE: OnceLock<Mutex<(KnownCrash, Counts)>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new((KnownCrash::from_env(), Counts::default())))
}

/// Runs one input; panics with the report on a failure that is not a known class.
pub fn check(data: &[u8]) {
    let outcome = scenario::run(data);
    // A model that is not a configuration anyone builds in this build (erase value vs `flash-erase-zero`).
    if outcome.expect == Expect::Skip {
        return;
    }
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    let (crash, counts) = &mut *guard;
    counts.runs += 1;
    let Some(failure) = &outcome.failure else { return };
    let model = scenario::model(data.first().copied().unwrap_or(0) as usize).0;
    let known = match (outcome.expect, failure.known) {
        // An entry built to fail in this build (L0, L1 without `flash-erase-zero`): not what is fuzzed.
        (Expect::Finding(_), _) => return,
        (_, Some(class)) => class,
        (_, None) => {
            let report = outcome.report;
            drop(guard);
            panic!("{report}")
        }
    };
    let n = counts.known.entry(known).or_default();
    *n += 1;
    let n = *n;
    if crash.matches(known, model) {
        let report = outcome.report;
        drop(guard);
        panic!("known class {known} on {model} (POWER_FAIL_KNOWN_CRASH):\n{report}");
    }
    // The first example of each class and then every power of two, so a long run shows its counts.
    if n.is_power_of_two() {
        let (runs, summary) = (counts.runs, counts.known.clone());
        let hex: String = data.iter().map(|b| format!("{b:02x}")).collect();
        eprintln!("known class {known} on {model}: {n} runs (of {runs}); example {hex}; all known: {summary:?}");
    }
}
