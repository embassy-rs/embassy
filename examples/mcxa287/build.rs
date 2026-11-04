use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());
    // CI overrides target rustflags, so required linker scripts belong here.
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rerun-if-changed=memory.x");
    std::fs::write(out.join("ecc-scratch.x"), include_bytes!("ecc-scratch.x")).unwrap();
    println!("cargo:rerun-if-changed=ecc-scratch.x");
    std::fs::write(out.join("ram.x"), include_bytes!("ram.x")).unwrap();
    println!("cargo:rerun-if-changed=ram.x");
    println!("cargo:rerun-if-changed=src/bin");
    // Keep every other binary's RAM layout unchanged. flip-link needs literal
    // RAM bounds, so select complete scripts rather than conditional expressions.
    for entry in std::fs::read_dir("src/bin").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let bin = path.file_stem().unwrap().to_str().unwrap();
            let script = if bin == "erm-eim" { "ecc-scratch.x" } else { "ram.x" };
            println!("cargo:rustc-link-arg-bin={bin}=-T{script}");
            println!("cargo:rustc-link-arg-bin={bin}=-Tlink.x");
        }
    }
}
