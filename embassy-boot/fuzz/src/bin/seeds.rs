//! Writes the seed corpus: the sweep scenarios on every model, with and without power cuts, to
//! `corpus/power_fail/` (or the folder given as argument).
//!
//! Also writes `<folder>.list` with the absolute file paths for `-seed_inputs=@<folder>.list`, so that the
//! load order does not depend on the file system.

use embassy_boot_fuzz::scenario::{DEFAULT_LAYOUT, Step, input, model, run, scenarios};
use embassy_boot_fuzz::zoo;

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| "corpus/power_fail".into());
    let dir = arg.trim_end_matches('/');
    std::fs::create_dir_all(dir).expect("create the corpus folder");
    let mut files = Vec::new();
    for m in 0..zoo::COUNT {
        let name = model(m).name;
        for (scenario, steps) in scenarios() {
            let plain = input(m, DEFAULT_LAYOUT, &steps);
            let ops = run(&plain).ops.min(u16::MAX as u64) as u16;
            let mut inputs = vec![(format!("{name}-{scenario}"), plain)];
            // A cut at about eight points of the run, three tear patterns each.
            for after in (0..ops).step_by((ops as usize / 8).max(1)) {
                for tear in 0..3 {
                    let mut cut = vec![Step::Cut { after, tear }];
                    cut.extend(&steps);
                    let file = format!("{name}-{scenario}-cut{after}-tear{tear}");
                    inputs.push((file, input(m, DEFAULT_LAYOUT, &cut)));
                }
            }
            for (file, data) in inputs {
                let path = format!("{dir}/{file}");
                std::fs::write(&path, data).expect("write a seed");
                // libFuzzer silently skips a listed file it cannot find.
                let path = std::fs::canonicalize(&path).expect("absolute seed path");
                files.push(path.to_str().expect("UTF-8 seed path").to_string());
            }
        }
    }
    std::fs::write(format!("{dir}.list"), files.join(",")).expect("write the seed list");
    println!("{} seeds in {dir}, listed in {dir}.list", files.len());
}
