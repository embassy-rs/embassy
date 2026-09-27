//! Build script for the STM32N6 examples.
//!
//! Besides wiring up the linker scripts it optionally *imports* a real
//! ST Edge AI generated network so the `npu_mobilenet` example can run genuine
//! Neural-ART (ATON) microcode. The model is not vendored in this repository
//! (it is published by ST under SLA0044); instead this script derives the two
//! binary assets the example needs from a checkout of ST's public
//! `STM32N6-GettingStarted-ImageClassification` repository.
//!
//! Point `STM32N6_GETTINGSTARTED_MODEL_DIR` at the model directory, e.g.
//!
//! ```text
//! STM32N6_GETTINGSTARTED_MODEL_DIR=/path/to/STM32N6-GettingStarted-ImageClassification/Model/NUCLEO-N657X0-Q \
//!     cargo build --bin npu_mobilenet
//! ```
//!
//! From `<dir>` two artifacts are produced in `$OUT_DIR/npu/`:
//!
//! * `ec_blob.bin`   – `_ec_blob_network_1[]` of `network_ecblobs.h`, decoded
//!                     from the generated C array into raw little-endian bytes;
//! * `weights.bin`   – a verbatim copy of `network_data.xSPI2.bin` (the encoded
//!                     network parameters that must live at `0x7038_0000`).
//!
//! When the variable is unset (or the files are missing) empty placeholders are
//! written and the `npu_model_assets` cfg is *not* set, so the rest of the crate
//! keeps building. The example itself is gated behind the `npu-model` feature.

use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(npu_model_assets)");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    println!("cargo:rerun-if-env-changed=STM32N6_GETTINGSTARTED_MODEL_DIR");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("npu");
    fs::create_dir_all(&out).unwrap();

    let blob_out = out.join("ec_blob.bin");
    let weights_out = out.join("weights.bin");

    let model_dir = env::var_os("STM32N6_GETTINGSTARTED_MODEL_DIR").map(PathBuf::from);
    let mut ok = false;

    if let Some(dir) = model_dir {
        println!("cargo:rerun-if-changed={}", dir.display());
        let header = dir.join("network_ecblobs.h");
        let weights = dir.join("network_data.xSPI2.bin");

        if header.is_file() && weights.is_file() {
            match import_blob(&header, &blob_out) {
                Ok(words) => {
                    fs::copy(&weights, &weights_out).unwrap();
                    println!(
                        "cargo:warning=imported NPU model: {words} u64 blob words from {}",
                        header.display()
                    );
                    ok = true;
                }
                Err(e) => println!("cargo:warning=failed to import NPU blob: {e}"),
            }
        } else {
            println!(
                "cargo:warning=STM32N6_GETTINGSTARTED_MODEL_DIR={} does not contain network_ecblobs.h + network_data.xSPI2.bin",
                dir.display()
            );
        }
    }

    if ok {
        println!("cargo:rustc-cfg=npu_model_assets");
    } else {
        // Keep `include_bytes!` in the example compiling; it checks the magic at
        // runtime and the example is only built with `--features npu-model`.
        fs::write(&blob_out, []).unwrap();
        fs::write(&weights_out, []).unwrap();
    }
}

/// Decode the `static const uint64_t _ec_blob_network_1[N] = { 0x.., ... };`
/// array from a generated `network_ecblobs.h` into little-endian bytes.
fn import_blob(header: &Path, out: &Path) -> Result<usize, String> {
    let text = fs::read_to_string(header).map_err(|e| e.to_string())?;

    let start = text.find("_ec_blob_network_1[").ok_or("_ec_blob_network_1 not found")?;
    let body_start = text[start..].find('{').ok_or("array body not found")? + start;
    let body_end = text[body_start..].find("};").ok_or("array end not found")? + body_start;
    let body = &text[body_start + 1..body_end];

    let mut bytes = Vec::new();
    let mut words = 0usize;
    let mut it = body.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        if c == '0' && matches!(it.peek(), Some((_, 'x' | 'X'))) {
            it.next(); // consume 'x'
            let hex_start = i + 2;
            let mut end = hex_start;
            while let Some((j, d)) = it.peek() {
                if d.is_ascii_hexdigit() {
                    end = *j + 1;
                    it.next();
                } else {
                    break;
                }
            }
            let value = u64::from_str_radix(&body[hex_start..end], 16).map_err(|e| e.to_string())?;
            bytes.extend_from_slice(&value.to_le_bytes());
            words += 1;
        }
    }

    if words == 0 {
        return Err("no literals decoded".into());
    }
    fs::write(out, &bytes).map_err(|e| e.to_string())?;
    Ok(words)
}
