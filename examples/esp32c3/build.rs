use std::{env, fs::File, io::Write, path::PathBuf};

// Thanks to kennytm and TheDan64 for the assert_used_features macro.
// Source:
// https://github.com/TheDan64/inkwell/blob/36c3b106e61b1b45295a35f94023d93d9328c76f/src/lib.rs#L81-L110
macro_rules! assert_unique_features {
    () => {};
    ($first:tt $(,$rest:tt)*) => {
        $(
            #[cfg(all(feature = $first, feature = $rest))]
            compile_error!(concat!("Features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
        )*
        assert_unique_features!($($rest),*);
    }
}

assert_unique_features! {"mcu-boot", "direct-boot"}

#[cfg(feature = "direct-boot")]
fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("ld/db-esp32c3-memory.x"))
        .unwrap();

    File::create(out.join("esp32c3-link.x"))
        .unwrap()
        .write_all(include_bytes!("ld/db-esp32c3-link.x"))
        .unwrap();

    File::create(out.join("riscv-link.x"))
        .unwrap()
        .write_all(include_bytes!("ld/db-riscv-link.x"))
        .unwrap();

    File::create(out.join("linkall.x"))
        .unwrap()
        .write_all(include_bytes!("ld/db-linkall.x"))
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=ld/memory.x");

    add_defaults();
}

#[cfg(not(any(feature = "mcu-boot", feature = "direct-boot")))]
fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("ld/bl-esp32c3-memory.x"))
        .unwrap();

    File::create(out.join("bl-riscv-link.x"))
        .unwrap()
        .write_all(include_bytes!("ld/bl-riscv-link.x"))
        .unwrap();

    File::create(out.join("linkall.x"))
        .unwrap()
        .write_all(include_bytes!("ld/bl-linkall.x"))
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=ld/memory.x");

    add_defaults();
}

#[cfg(feature = "mcu-boot")]
fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("ld/mb-esp32c3-memory.x"))
        .unwrap();

    File::create(out.join("esp32c3-link.x"))
        .unwrap()
        .write_all(include_bytes!("ld/mb-esp32c3-link.x"))
        .unwrap();

    File::create(out.join("riscv-link.x"))
        .unwrap()
        .write_all(include_bytes!("ld/mb-riscv-link.x"))
        .unwrap();

    File::create(out.join("linkall.x"))
        .unwrap()
        .write_all(include_bytes!("ld/mb-linkall.x"))
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=ld/memory.x");

    add_defaults();
}

fn add_defaults() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    File::create(out.join("rom-functions.x"))
        .unwrap()
        .write_all(include_bytes!("ld/rom-functions.x"))
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());
}
