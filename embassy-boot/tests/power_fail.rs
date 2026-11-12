//! Power-fail test of embassy-boot on simulated flash of several MCU families.
//!
//! The real `BootLoader` and `BlockingFirmwareUpdater` run on a NOR flash model of one family (see
//! `power_fail/zoo.rs`). Power is cut during a program or erase, and invariants are checked after every boot.
//! A run is a byte string: byte 0 selects the model, bytes 1-4 the layout, the rest are steps. The `fuzz/`
//! crate uses the same input format.
//!
//! Environment variables:
//! - `POWER_FAIL_MODEL=<name>`: run only this model. `list` prints the models.
//! - `POWER_FAIL_CASES=<n>`: number of cases for `random` (default 1000).
//! - `POWER_FAIL_REPLAY=<hex or file>`: run one input in `replay` and print its trace.
//!
//! Known failure classes (`Failure::known`) are counted and printed, but do not fail the test. Models that
//! erase to 0x00 are expected to fail (`zoo::expect`).

#[path = "power_fail/flash.rs"]
mod flash;
#[path = "power_fail/scenario.rs"]
mod scenario;
#[path = "power_fail/zoo.rs"]
mod zoo;

use std::collections::BTreeMap;

use flash::Rng;
use scenario::{DEFAULT_LAYOUT, Step, hex, input, model, run, scenarios};
use zoo::Expect;

/// The models to run: all, or the one `POWER_FAIL_MODEL` names.
fn models() -> Vec<usize> {
    let Ok(only) = std::env::var("POWER_FAIL_MODEL") else {
        return (0..zoo::COUNT).collect();
    };
    let only = only.trim().to_lowercase();
    if only == "list" {
        return Vec::new();
    }
    let found: Vec<usize> = (0..zoo::COUNT)
        .filter(|&i| model(i).name.to_lowercase() == only)
        .collect();
    if found.is_empty() {
        let names: Vec<&str> = (0..zoo::COUNT).map(|i| model(i).name).collect();
        panic!(
            "POWER_FAIL_MODEL={only:?} is not a model; valid names: {}",
            names.join(", ")
        );
    }
    found
}

#[test]
fn models_list() {
    if std::env::var("POWER_FAIL_MODEL").is_ok_and(|s| s.trim().eq_ignore_ascii_case("list")) {
        for i in 0..zoo::COUNT {
            let m = model(i);
            println!("{:<9} {}", m.name, m.source);
        }
    }
}

fn fails_like(data: &[u8], invariant: &str) -> bool {
    run(data).failure.is_some_and(|f| f.invariant == invariant)
}

/// A smaller input that fails with the same invariant: drop bytes, then make them smaller.
fn shrink(data: &[u8]) -> Vec<u8> {
    let Some(invariant) = run(data).failure.map(|f| f.invariant) else {
        return data.to_vec();
    };
    let mut best = data.to_vec();
    for _ in 0..4 {
        let before = best.clone();
        for width in [4, 2, 1] {
            let mut i = best.len();
            while i >= 5 + width {
                i -= 1;
                let mut cand = best.clone();
                cand.drain(i + 1 - width..=i);
                if fails_like(&cand, invariant) {
                    best = cand;
                    i = i.min(best.len());
                }
            }
        }
        for i in 1..best.len() {
            for v in [0, best[i] / 2, best[i].saturating_sub(1)] {
                if v < best[i] {
                    let mut cand = best.clone();
                    cand[i] = v;
                    if fails_like(&cand, invariant) {
                        best = cand;
                        break;
                    }
                }
            }
        }
        if best == before {
            break;
        }
    }
    best
}

fn unhex(s: &str) -> Vec<u8> {
    let s = s.trim();
    assert!(s.len() % 2 == 0, "{s:?} is not hex");
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex input"))
        .collect()
}

/// Runs an input given as hex and panics with its report if it fails.
fn replay_hex(input: &str) {
    let outcome = run(&unhex(input));
    if outcome.expect != Expect::Skip {
        assert!(outcome.failure.is_none(), "{}", outcome.report);
    }
}

/// The report of a shrunk version of `data`.
fn reproduction(data: &[u8]) -> String {
    let small = shrink(data);
    format!(
        "{}(shrunk to {} bytes from {}: {})\n",
        run(&small).report,
        small.len(),
        data.len(),
        hex(data)
    )
}

#[derive(Default)]
struct Tally {
    runs: usize,
    /// Per known class: count and first example.
    known: BTreeMap<&'static str, (usize, Vec<u8>)>,
    /// Failures of a model that is expected to fail.
    expected: Vec<Vec<u8>>,
    unexpected: Vec<Vec<u8>>,
}

impl Tally {
    /// Runs `data` and records the result. Returns the number of flash operations.
    fn run(&mut self, data: Vec<u8>) -> u64 {
        let outcome = run(&data);
        self.runs += 1;
        if let Some(f) = &outcome.failure {
            match (outcome.expect, f.known) {
                (Expect::Finding(_), _) => self.expected.push(data),
                (_, Some(class)) => self.known.entry(class).or_insert((0, data)).0 += 1,
                (_, None) => self.unexpected.push(data),
            }
        }
        outcome.ops
    }

    /// Prints the result and returns one summary line, or an error message.
    fn finish(self, label: &str, expect: Expect) -> Result<String, String> {
        for (class, (count, data)) in &self.known {
            println!(
                "{label}: known class {class}: {count} runs, for example:\n{}",
                reproduction(data)
            );
        }
        let known: Vec<String> = self.known.iter().map(|(c, (n, _))| format!("{c} x{n}")).collect();
        let line = format!("{label:<16} runs {:>6}  known: [{}]", self.runs, known.join(", "));
        let shortest = |v: &[Vec<u8>]| v.iter().min_by_key(|d| d.len()).map(|d| reproduction(d));
        match expect {
            Expect::Finding(what) => {
                let Some(example) = shortest(&self.expected) else {
                    return Err(format!("{label}: expected finding not reproduced ({what})"));
                };
                println!("{label}: FINDING ({what}), for example:\n{example}");
                Ok(format!("{line}  expected failures: {}", self.expected.len()))
            }
            _ => match shortest(&self.unexpected) {
                Some(example) => Err(format!("{label}: {} runs failed:\n{example}", self.unexpected.len())),
                None => Ok(line),
            },
        }
    }
}

/// Runs `cases` for every selected model, prints a summary and fails on unexpected results.
fn check_models(title: &str, mut cases: impl FnMut(usize, &mut Tally)) {
    let mut failures = Vec::new();
    for m in models() {
        let info = model(m);
        if info.expect == Expect::Skip {
            continue;
        }
        let mut tally = Tally::default();
        cases(m, &mut tally);
        match tally.finish(&format!("{title} {}", info.name), info.expect) {
            Ok(line) => println!("{line}"),
            Err(e) => failures.push(e),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The fixed scenarios with a power cut at every flash operation, three tear patterns each.
#[test]
fn sweep() {
    check_models("sweep", |m, tally| {
        for (_, steps) in scenarios() {
            let ops = tally.run(input(m, DEFAULT_LAYOUT, &steps));
            for after in 0..ops.min(u16::MAX as u64) as u16 {
                for tear in 0..3 {
                    let mut cut = vec![Step::Cut { after, tear }];
                    cut.extend(&steps);
                    tally.run(input(m, DEFAULT_LAYOUT, &cut));
                }
            }
        }
    });
}

fn random_steps(rng: &mut Rng) -> Vec<Step> {
    let count = 2 + rng.below(16);
    (0..count)
        .map(|_| match rng.below(10) {
            0 | 1 => Step::Download {
                chunk: rng.next() as u8,
            },
            2 | 3 => Step::MarkUpdated,
            4 | 5 => Step::Boot,
            6 => Step::MarkBooted,
            7 => Step::MarkDfu,
            _ => Step::Cut {
                after: rng.below(120) as u16,
                tear: rng.next() as u8,
            },
        })
        .collect()
}

/// `POWER_FAIL_CASES` random scenarios with random layouts, spread over the models.
#[test]
fn random() {
    let cases = std::env::var("POWER_FAIL_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    check_models("random", |m, tally| {
        for case in (m..cases).step_by(zoo::COUNT) {
            let mut rng = Rng(case as u64 ^ 0x5EED);
            let layout = [rng.next() as u8, rng.next() as u8, rng.next() as u8, rng.next() as u8];
            tally.run(input(m, layout, &random_steps(&mut rng)));
        }
    });
}

/// `POWER_FAIL_REPLAY=<hex or file>`: runs that input and prints its trace. A relative file path is relative
/// to `embassy-boot`.
#[test]
fn replay() {
    let Ok(arg) = std::env::var("POWER_FAIL_REPLAY") else {
        return;
    };
    let arg = arg.trim();
    let data = if std::path::Path::new(arg).is_file() {
        std::fs::read(arg).unwrap_or_else(|e| panic!("cannot read {arg}: {e}"))
    } else {
        unhex(arg)
    };
    let outcome = run(&data);
    println!("{}", outcome.report);
    assert!(outcome.failure.is_none(), "the replayed input fails");
}

/// What the flash model does on its own, per zoo entry: each probe programs a unit, then programs it again.
struct Probe;

impl zoo::Visit for Probe {
    type Out = (&'static str, Vec<String>);

    fn visit<M: flash::Model>(self) -> Self::Out {
        use embedded_storage::nor_flash::NorFlash;
        let len = M::UNIT.max(M::WRITE_SIZE);
        let fresh = || flash::Nor::<M>::new((2 * M::ERASE_SIZE).max(1024), flash::Regions::default());
        let outcome = |nor: &mut flash::Nor<M>, data: &[u8]| -> String {
            let broken = nor.broken.len();
            match nor.write(0, data) {
                Err(e) => format!("{e:?}"),
                Ok(()) if nor.broken.len() > broken => nor.broken[broken].0.to_string(),
                Ok(()) if nor.bad[0] => "Ecc".into(),
                Ok(()) => "ok".into(),
            }
        };
        let twice = |first: u8, second: u8| {
            let mut nor = fresh();
            let _ = nor.write(0, &vec![first; len]);
            outcome(&mut nor, &vec![second; len])
        };
        let mut out = vec![
            format!("zeros over F0: {}", twice(0xF0, 0)),
            format!("zeros over zeros: {}", twice(0, 0)),
            format!("C0 over F0: {}", twice(0xF0, 0xC0)),
            format!("zeros over FF: {}", twice(0xFF, 0)),
        ];
        let mut nor = fresh();
        let _ = nor.write(0, &vec![0xF0; len]);
        let _ = nor.write(0, &vec![0; len]);
        out.push(format!("third write: {}", outcome(&mut nor, &vec![0; len])));
        let mut nor = fresh();
        let block: Vec<u8> = vec![0; 512.max(len)];
        let _ = nor.write(0, &block);
        out.push(format!("512 B written twice: {}", outcome(&mut nor, &block)));
        let mut nor = fresh();
        let before = nor.ops;
        nor.erase(0, M::ERASE_SIZE as u32).unwrap();
        out.push(format!("erases in one erase: {}", nor.ops - before));
        (M::NAME, out)
    }
}

#[test]
fn rules() {
    #[rustfmt::skip]
    let want: &[(&str, [&str; 7])] = &[
        // zeros/F0, zeros/zeros, C0/F0, zeros/FF, third write, 512 B twice, physical erases
        ("F1",       ["ok", "ok", "Refused", "ok", "ok", "ok", "1"]),
        ("G0",       ["ok", "ok", "Refused", "Refused", "ok", "ok", "1"]),
        ("Qspi",     ["ok", "ok", "ok", "ok", "ok", "ok", "1"]),
        ("L0",       ["Refused", "ok", "Refused", "Refused", "Refused", "ok", "1"]),
        ("L1",       ["ok", "ok", "or-overwrite", "ok", "ok", "ok", "1"]),
        ("F4",       ["ok", "ok", "ok", "ok", "ok", "ok", "5"]),
        ("F7",       ["ok", "ok", "ok", "ok", "ok", "ok", "5"]),
        ("L0c3",     ["ok", "ok", "or-overwrite", "ok", "ok", "ok", "1"]),
        ("G4",       ["ok", "ok", "Refused", "ok", "ok", "ok", "1"]),
        ("G4zz",     ["ok", "Refused", "Refused", "ok", "Refused", "Refused", "1"]),
        ("C0",       ["ok", "ok", "Refused", "ok", "ok", "ok", "1"]),
        ("U5",       ["ok", "Refused", "Refused", "ok", "Refused", "Refused", "1"]),
        ("L5",       ["ok", "Refused", "Refused", "ok", "Refused", "Refused", "1"]),
        ("H7",       ["ecc-overwrite", "ok", "ecc-overwrite", "ok", "Ecc", "ok", "1"]),
        ("H7ab",     ["ecc-overwrite", "ok", "ecc-overwrite", "ok", "Ecc", "ok", "1"]),
        ("Nrf52",    ["ok", "ok", "ok", "ok", "nwrite-exceeded", "ok", "1"]),
        ("Nrf52832", ["ok", "ok", "ok", "ok", "ok", "block-writes-exceeded", "1"]),
        ("Nrf53net", ["ok", "ok", "ok", "ok", "nwrite-exceeded", "ok", "1"]),
        ("Rram",     ["ok", "ok", "ok", "ok", "ok", "ok", "1"]),
    ];
    assert_eq!(want.len(), zoo::COUNT, "a zoo entry without a rule test");
    let mut wrong = Vec::new();
    for i in 0..zoo::COUNT {
        let (name, got) = zoo::with_model(i, Probe);
        let row = want
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("no rule test for {name}"));
        for (line, want) in got.iter().zip(row.1) {
            if !line.ends_with(&format!(": {want}")) {
                wrong.push(format!("{name}: {line}, want {want}"));
            }
        }
    }
    assert!(wrong.is_empty(), "rule tests:\n{}", wrong.join("\n"));
}

/// Two cuts in a row after the revert tail's progress-invalid write: the third write of that word exceeds
/// nRF52's n_WRITE = 2. The random run seldom finds this.
#[test]
fn nrf52_two_cuts_exceed_n_write() {
    let outcome = run(&unhex("0f010000010528000000000102020501000002"));
    if outcome.expect == Expect::Skip {
        return;
    }
    let known = outcome.failure.as_ref().and_then(|f| f.known);
    assert_eq!(known, Some("nwrite-exceeded"), "{}", outcome.report);
}

/// More downloads than there are one-byte image names (found by the fuzzer).
#[test]
fn many_downloads_keep_image_names() {
    let mut steps = vec![Step::Download { chunk: 0 }; 150];
    steps.extend([Step::MarkUpdated, Step::Boot, Step::MarkBooted, Step::Boot]);
    for m in (0..zoo::COUNT).filter(|&m| model(m).expect == Expect::Pass) {
        let outcome = run(&input(m, DEFAULT_LAYOUT, &steps));
        assert!(
            outcome.failure.as_ref().is_none_or(|f| f.known.is_some()),
            "{}",
            outcome.report
        );
    }
}

/// embassy#7235: a power cut during the STATE erase, fixed by 153d2e802.
#[test]
fn issue_7235_torn_state_erase() {
    replay_hex("0001000001051800020000010203");
}
