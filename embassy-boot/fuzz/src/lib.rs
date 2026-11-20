//! Coverage-guided fuzzing of the embassy-boot power-fail test in `tests/power_fail.rs`. The input format is
//! the same as `POWER_FAIL_REPLAY` takes.
#![allow(dead_code)]

#[path = "../../tests/power_fail/flash.rs"]
pub mod flash;
#[path = "../../tests/power_fail/scenario.rs"]
pub mod scenario;
#[path = "../../tests/power_fail/zoo.rs"]
pub mod zoo;

use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

use zoo::Expect;

/// `POWER_FAIL_KNOWN_CRASH`: comma-separated `all`, `class` or `class:Model`.
static KNOWN_CRASH: LazyLock<Vec<String>> = LazyLock::new(|| {
    let value = std::env::var("POWER_FAIL_KNOWN_CRASH").unwrap_or_default();
    value
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
});

/// Runs per known class.
static COUNTS: Mutex<BTreeMap<&'static str, u64>> = Mutex::new(BTreeMap::new());

/// Runs one input. Panics with the report on a failure that is not a known class, or on a known class that
/// `POWER_FAIL_KNOWN_CRASH` selects.
pub fn check(data: &[u8]) {
    let outcome = scenario::run(data);
    let Some(failure) = &outcome.failure else {
        return;
    };
    let known = match (outcome.expect, failure.known) {
        // Not built in this configuration, or expected to fail in it.
        (Expect::Skip | Expect::Finding(_), _) => return,
        (_, Some(class)) => class,
        (_, None) => panic!("{}", outcome.report),
    };
    let model = scenario::model(data.first().copied().unwrap_or(0) as usize).name;
    let selector = format!("{known}:{model}").to_lowercase();
    if KNOWN_CRASH.iter().any(|s| s == "all" || s == known || *s == selector) {
        panic!(
            "known class {known} on {model} (POWER_FAIL_KNOWN_CRASH):\n{}",
            outcome.report
        );
    }
    let mut counts = COUNTS.lock().unwrap_or_else(|e| e.into_inner());
    let n = counts.entry(known).or_default();
    *n += 1;
    // Print the first example of each class, then at every power of two.
    if n.is_power_of_two() {
        eprintln!(
            "known class {known} on {model}: {n} runs; example {}",
            scenario::hex(data)
        );
    }
}
