mod generate {
    include!("../touch-projects/generate_touch_config.rs");
}

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    generate::generate_touch_config(&manifest_dir, &out_dir);
}
