//! Power-fail test of embassy-boot on simulated flash of several MCU families.
//!
//! The real `BootLoader` and `BlockingFirmwareUpdater` run on a NOR flash model with the write rules of one
//! family (program unit, what may be programmed over what, erase value, ECC). Power is cut at any program or
//! erase, which leaves that operation torn; after every boot the test checks that ACTIVE is one whole image,
//! that neither image is lost, that the state the application reads matches ACTIVE, that an unconfirmed image
//! gets exactly one trial boot, that boots settle, and that no family rule was broken.
//!
//! Every run is a byte string: byte 0 picks the model, the next four the layout, the rest are steps
//! (download, mark_updated, boot, mark_booted, mark_dfu, cut). So a fuzzer can drive it, and any failure is a
//! short hex string that replays exactly.
//!
//! # Running it
//!
//! ```text
//! cd embassy-boot
//! cargo test --test power_fail -- --nocapture             # sweep + random run, every model
//! POWER_FAIL_MODEL=g0 cargo test --test power_fail -- --nocapture         # one model, by name
//! POWER_FAIL_CASES=200000 cargo test --release --test power_fail random   # a longer random run
//! cargo test --features flash-erase-zero --test power_fail -- --nocapture # the build for 0x00-erased flash
//! POWER_FAIL_REPLAY=<hex> cargo test --test power_fail replay -- --nocapture
//! ```
//!
//! - `sweep` runs a few fixed scenarios (update and confirm, update and revert, update cancelled, two updates,
//!   mark_dfu) and cuts power at every single flash operation of each, with three tear patterns.
//! - `random` runs `POWER_FAIL_CASES` (default 1000) random scenarios with several cuts each.
//! - A failure prints the model, layout, steps, a trace of every flash operation from the start (the download's
//!   own writes left out) with the page map of ACTIVE, DFU and STATE after each boot, the broken invariant, and
//!   the input as hex. It is first shrunk to a smaller input that fails the same way. Paste the hex into
//!   `POWER_FAIL_REPLAY` to see it again, or into a `#[test]` calling `replay_hex`.
//! - Models whose flash erases to 0x00 (L0, L1) are expected to fail, in both builds (see `zoo::expect`).
//!   The default build is the configuration of the L0/L1 boot examples. The test checks that the failure
//!   still reproduces and prints it; it is green as long as the zoo's expectations hold.
//! - Known classes are counted and printed with an example, not failed:
//!   - `ecc-state-unreadable`: on ECC flash a torn operation leaves a STATE word that faults on every read;
//!     embassy-boot has no answer for it yet.
//!   - `confirm-cut-reads-revert`: a cut inside `mark_booted`/`mark_dfu` after its progress-invalid write;
//!     the next boot takes the revert path with every copy skipped, the new image stays in ACTIVE and the
//!     state reads `Revert`.
//!   - `torn-magic-reads-swap`: on byte-wide flash a torn write of another magic can read as SWAP (0xF0 has
//!     every 1 bit of 0xE0, 0xD0 and 0xC0), and the bootloader swaps in whatever DFU holds.
//!   - `zero-over-zero-refused`: U5, WBA, L5, U3 (and G4 if zero over zero is refused) refuse the revert
//!     tail's progress-invalid write when a cut made it run twice; `prepare_boot` then fails at every boot.
//!   - `ecc-overwrite`: H7 family: clearing the magic programs over a used flash word, which the manual
//!     calls "not recommended"; `ecc-overwrite-fault`: a cut before the erase that follows leaves it faulting.
//!   - `nwrite-exceeded`, `block-writes-exceeded`: nRF writes per word (or per 512 B block on the nRF52832)
//!     between erases beyond the limit.
//!
//! Models (`POWER_FAIL_MODEL` takes the name): F1, F4, F7, L0, L0c3, L1, G0, G4, G4zz, C0, U5, L5, H7, H7ab,
//! Nrf52, Nrf52832, Nrf53net, Rram, Qspi; each cites its reference manual in `power_fail/zoo.rs`, and the
//! `rules` test checks what each one does with a few writes.
//!
//! The model of what a cut leaves is pessimistic (any bit pattern between old and new, ECC faults on half
//! written units): vendors only say the content is "not guaranteed".

#[path = "power_fail/flash.rs"]
mod flash;
#[path = "power_fail/scenario.rs"]
mod scenario;
#[path = "power_fail/zoo.rs"]
mod zoo;

use std::collections::BTreeMap;

use flash::Rng;
use scenario::{Layout, Outcome, Step, model, run};
use zoo::Expect;

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}

/// Zoo entries to run: all, or the one `POWER_FAIL_MODEL` names.
fn models() -> Vec<usize> {
    let only = std::env::var("POWER_FAIL_MODEL").ok().map(|s| s.to_lowercase());
    (0..zoo::COUNT)
        .filter(|&i| only.as_ref().is_none_or(|o| model(i).0.to_lowercase() == *o))
        .collect()
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
        // Drop windows of 4, 2 and 1 bytes: whole steps first.
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

fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex input"))
        .collect()
}

/// Runs a reproduction printed by a failure and panics with its report if it still fails.
#[allow(dead_code)]
fn replay_hex(input: &str) {
    let outcome = run(&unhex(input));
    if outcome.failure.is_some() {
        panic!("{}", outcome.report);
    }
}

fn input(model: usize, layout: [u8; 4], steps: &[Step]) -> Vec<u8> {
    let mut out = vec![model as u8];
    out.extend(layout);
    for step in steps {
        step.encode(&mut out);
    }
    out
}

/// Prints a shrunk reproduction of `data` and returns the report.
fn reproduction(data: &[u8]) -> String {
    let small = shrink(data);
    let outcome = run(&small);
    format!(
        "{}(shrunk to {} bytes from {}: {})\n",
        outcome.report,
        small.len(),
        data.len(),
        hex(data)
    )
}

#[derive(Default)]
struct Tally {
    runs: usize,
    known: BTreeMap<&'static str, (usize, Vec<u8>)>,
    unexpected: Vec<Vec<u8>>,
    failed: Vec<Vec<u8>>,
}

impl Tally {
    fn add(&mut self, data: Vec<u8>, outcome: &Outcome) {
        self.runs += 1;
        let Some(f) = &outcome.failure else { return };
        match (outcome.expect, f.known) {
            (Expect::Finding(_), _) => self.failed.push(data),
            (_, Some(class)) => {
                let entry = self.known.entry(class).or_insert((0, data));
                entry.0 += 1;
            }
            (_, None) => self.unexpected.push(data),
        }
    }

    /// Prints what was found and panics on what was not expected.
    fn finish(self, name: &str, expect: Expect) {
        println!("{name}: {} runs", self.runs);
        for (class, (count, data)) in &self.known {
            println!(
                "{name}: known class {class}: {count} runs, for example:\n{}",
                reproduction(data)
            );
        }
        match expect {
            Expect::Finding(what) => {
                let Some(first) = self.failed.iter().min_by_key(|d| d.len()) else {
                    panic!("{name}: expected finding not reproduced ({what}): update the zoo");
                };
                println!(
                    "{name}: FINDING ({what}): {} of {} runs fail, for example:\n{}",
                    self.failed.len(),
                    self.runs,
                    reproduction(first)
                );
            }
            _ => {
                if let Some(first) = self.unexpected.iter().min_by_key(|d| d.len()) {
                    panic!(
                        "{name}: {} of {} runs failed:\n{}",
                        self.unexpected.len(),
                        self.runs,
                        reproduction(first)
                    );
                }
            }
        }
    }
}

/// Fixed scenarios; the sweep cuts power at every operation of each.
fn scenarios() -> Vec<(&'static str, Vec<Step>)> {
    use Step::*;
    let dl = |chunk| Download { chunk };
    vec![
        ("update, confirm", vec![dl(0), MarkUpdated, Boot, MarkBooted, Boot]),
        ("update, no confirm: revert", vec![dl(0), MarkUpdated, Boot, Boot, Boot]),
        ("update cancelled", vec![dl(1), MarkUpdated, MarkBooted, Boot]),
        ("update, mark_dfu", vec![dl(0), MarkUpdated, Boot, MarkDfu, Boot]),
        (
            "two updates",
            vec![dl(0), MarkUpdated, Boot, MarkBooted, dl(2), MarkUpdated, Boot, Boot],
        ),
    ]
}

#[test]
fn sweep() {
    const TEARS: u8 = 3;
    for m in models() {
        let (name, erased) = model(m);
        let expect = zoo::expect(name, erased);
        if expect == Expect::Skip {
            continue;
        }
        let mut tally = Tally::default();
        for (_, steps) in scenarios() {
            let layout = Layout::encode(1, 0, 0, 1);
            let plain = input(m, layout, &steps);
            let outcome = run(&plain);
            tally.add(plain, &outcome);
            for after in 0..outcome.ops.min(u16::MAX as u64) as u16 {
                for tear in 0..TEARS {
                    let mut cut = vec![Step::Cut { after, tear }];
                    cut.extend(&steps);
                    let data = input(m, layout, &cut);
                    let outcome = run(&data);
                    tally.add(data, &outcome);
                }
            }
        }
        tally.finish(&format!("sweep {name}"), expect);
    }
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

#[test]
fn random() {
    let cases = env_usize("POWER_FAIL_CASES", 1000);
    let mut tallies: BTreeMap<usize, Tally> = BTreeMap::new();
    let wanted = models();
    for case in 0..cases {
        let mut rng = Rng(case as u64 ^ 0x5EED);
        let m = case % zoo::COUNT;
        if !wanted.contains(&m) {
            continue;
        }
        let (name, erased) = model(m);
        if zoo::expect(name, erased) == Expect::Skip {
            continue;
        }
        let layout = [rng.next() as u8, rng.next() as u8, rng.next() as u8, rng.next() as u8];
        let data = input(m, layout, &random_steps(&mut rng));
        let outcome = run(&data);
        tallies.entry(m).or_default().add(data, &outcome);
    }
    for m in wanted {
        let (name, erased) = model(m);
        let expect = zoo::expect(name, erased);
        if expect == Expect::Skip {
            continue;
        }
        let tally = tallies.remove(&m).unwrap_or_default();
        assert!(tally.runs > 0 || cases < zoo::COUNT, "random: model {name} never ran");
        tally.finish(&format!("random {name}"), expect);
    }
}

/// `POWER_FAIL_REPLAY=<hex>`: runs that input and prints its whole trace.
#[test]
fn replay() {
    let Ok(input) = std::env::var("POWER_FAIL_REPLAY") else {
        return;
    };
    let outcome = run(&unhex(&input));
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

/// Two cuts in a row, each right after the revert tail's progress-invalid write: the third write of that word
/// is beyond nRF52's n_WRITE = 2. The random run seldom puts two cuts in the same window, so it is kept here.
#[test]
fn nrf52_two_cuts_exceed_n_write() {
    if zoo::expect("Nrf52", 0xFF) == Expect::Skip {
        return;
    }
    let outcome = run(&unhex("0f010000010528000000000102020501000002"));
    let known = outcome.failure.as_ref().and_then(|f| f.known);
    assert_eq!(known, Some("nwrite-exceeded"), "{}", outcome.report);
}

/// More downloads than one letter of ASCII can name: the image names ran out of one-byte letters and ACTIVE
/// read as not one whole image (found by the fuzzer, `fuzz/README.md`).
#[test]
fn many_downloads_keep_image_names() {
    let mut steps = vec![Step::Download { chunk: 0 }; 150];
    steps.extend([Step::MarkUpdated, Step::Boot, Step::MarkBooted, Step::Boot]);
    for m in 0..zoo::COUNT {
        let (name, erased) = model(m);
        if zoo::expect(name, erased) != Expect::Pass {
            continue;
        }
        let outcome = run(&input(m, Layout::encode(1, 0, 0, 1), &steps));
        assert!(
            outcome.failure.as_ref().is_none_or(|f| f.known.is_some()),
            "{}",
            outcome.report
        );
    }
}
