use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for (name, bytes) in [
        ("memory.x", include_bytes!("memory.x").as_slice()),
        ("no-unwind.x", include_bytes!("no-unwind.x").as_slice()),
    ] {
        File::create(out.join(name)).unwrap().write_all(bytes).unwrap();
        println!("cargo:rerun-if-changed={name}");
    }
    println!("cargo:rustc-link-search={}", out.display());

    // Order matters: `no-unwind.x` runs `/DISCARD/` on `.eh_frame` before
    // `riscv-rt`'s `link.x` can place those sections at the default VMA, which
    // would cause `R_RISCV_32_PCREL` overflow against `.text` at 0x80000000.
    println!("cargo:rustc-link-arg-bins=-Tmemory.x");
    println!("cargo:rustc-link-arg-bins=-Tno-unwind.x");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
}
