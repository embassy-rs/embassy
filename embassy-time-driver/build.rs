use std::path::PathBuf;

fn main() {
    let tick_rate_str = std::env::var("CARGO_CFG_EMBASSY_TIME_DRIVER_TICK_HZ")
        .unwrap_or("1_000_000".to_string())
        .replace("_", "");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("tick_rate.rs");
    std::fs::write(out_path, tick_rate_str).unwrap();
}
