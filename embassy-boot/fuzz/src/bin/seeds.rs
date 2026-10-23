//! Writes the seed corpus: the fixed scenarios of the sweep in `tests/power_fail.rs` on every model, plain and
//! with a power cut at a few operations, in `corpus/power_fail/` (or the folder given as the argument).

use embassy_boot_fuzz::scenario::{Layout, Step, model, run};
use embassy_boot_fuzz::zoo;

/// The scenarios of `sweep` in `tests/power_fail.rs`.
fn scenarios() -> Vec<(&'static str, Vec<Step>)> {
    use Step::*;
    let dl = |chunk| Download { chunk };
    vec![
        ("update-confirm", vec![dl(0), MarkUpdated, Boot, MarkBooted, Boot]),
        ("update-revert", vec![dl(0), MarkUpdated, Boot, Boot, Boot]),
        ("update-cancelled", vec![dl(1), MarkUpdated, MarkBooted, Boot]),
        ("update-mark-dfu", vec![dl(0), MarkUpdated, Boot, MarkDfu, Boot]),
        (
            "two-updates",
            vec![dl(0), MarkUpdated, Boot, MarkBooted, dl(2), MarkUpdated, Boot, Boot],
        ),
    ]
}

fn input(model: usize, steps: &[Step]) -> Vec<u8> {
    let mut out = vec![model as u8];
    out.extend(Layout::encode(1, 0, 0, 1));
    for step in steps {
        step.encode(&mut out);
    }
    out
}

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "corpus/power_fail".into());
    std::fs::create_dir_all(&dir).expect("create the corpus folder");
    let mut count = 0;
    for m in 0..zoo::COUNT {
        let name = model(m).0;
        for (scenario, steps) in scenarios() {
            let plain = input(m, &steps);
            let ops = run(&plain).ops.min(u16::MAX as u64) as u16;
            // Plain, then a cut at about eight points of the run, each with three tear patterns.
            let mut inputs = vec![(format!("{name}-{scenario}"), plain)];
            for after in (0..ops).step_by((ops as usize / 8).max(1)) {
                for tear in 0..3 {
                    let mut cut = vec![Step::Cut { after, tear }];
                    cut.extend(&steps);
                    inputs.push((format!("{name}-{scenario}-cut{after}-tear{tear}"), input(m, &cut)));
                }
            }
            for (file, data) in inputs {
                std::fs::write(format!("{dir}/{file}"), data).expect("write a seed");
                count += 1;
            }
        }
    }
    println!("{count} seeds in {dir}");
}
