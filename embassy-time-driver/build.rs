use std::path::PathBuf;

fn main() {
    let tick_rates: Vec<usize> = std::env::vars()
        .filter_map(|(k, _)| {
            if let Some(speed) = k.strip_prefix("CARGO_FEATURE_TICK_HZ_") {
                Some(speed.replace("_", "").parse().unwrap())
            } else {
                None
            }
        })
        .collect();

    let tick_rate = match tick_rates.as_slice() {
        [] => 1_000_000,
        [tick_rate] => *tick_rate,
        _ => {
            println!("cargo::error=\"More than one tick rate selected for `embassy-time-driver`\"");
            return;
        }
    };

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("tick_rate.rs");
    std::fs::write(out_path, tick_rate.to_string()).unwrap();
}
