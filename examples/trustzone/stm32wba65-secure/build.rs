fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=veneer.ld");
    println!("cargo:rustc-link-search={manifest_dir}");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-T{manifest_dir}/veneer.ld");
    // Keep the CMSE veneer at the address the non-secure image calls.
    println!("cargo:rustc-link-arg-bins=--section-start=.gnu.sgstubs=0x0C0FF000");
}
