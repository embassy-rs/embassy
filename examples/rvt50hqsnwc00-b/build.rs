use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rerun-if-changed=lvgl/lv_conf.h");
    println!("cargo:rerun-if-changed=lvgl-port/port.c");
    println!("cargo:rerun-if-changed=lvgl-port/port.h");

    if env::var("CARGO_FEATURE_LVGL").is_ok() {
        compile_lvgl_port();
    }
}

fn compile_lvgl_port() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let port_dir = manifest_dir.join("lvgl-port");
    let lv_config_dir = manifest_dir.join("lvgl");
    let vendor = lvgl_sys_vendor_dir().join("vendor");

    cc::Build::new()
        .file(port_dir.join("port.c"))
        .include(&port_dir)
        .include(&lv_config_dir)
        .include(vendor.join("lvgl").join("src"))
        .include(&vendor)
        .define("LV_CONF_INCLUDE_SIMPLE", "1")
        .warnings(false)
        .compile("rvt50_lvgl_port");
}

fn lvgl_sys_vendor_dir() -> PathBuf {
    if let Ok(path) = env::var("LVGL_SYS_VENDOR") {
        return PathBuf::from(path);
    }

    let cargo_home = env::var("CARGO_HOME")
        .or_else(|_| env::var("HOME").map(|home| format!("{home}/.cargo")))
        .expect("CARGO_HOME or HOME must be set");

    for base in [
        PathBuf::from(&cargo_home).join("registry/src"),
        PathBuf::from(&cargo_home).join("git/checkouts"),
    ] {
        if let Some(path) = find_lvgl_sys_vendor(&base) {
            return path;
        }
    }

    panic!(
        "Could not locate lvgl-sys vendor directory. Set LVGL_SYS_VENDOR to the lvgl-sys crate root."
    );
}

fn find_lvgl_sys_vendor(base: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(base).ok()?;
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name()?.to_string_lossy();
        if name.starts_with("lvgl-sys-") {
            matches.push(path);
        } else if name.starts_with("lv_binding_rust-") {
            if let Some(nested) = find_nested_lvgl_sys(&path) {
                matches.push(nested);
            }
        }
    }

    matches.sort_by_key(|path| path.file_name().map(|name| name.to_string_lossy().to_string()));
    for candidate in matches {
        if candidate.join("vendor/lvgl/src").is_dir() {
            return Some(candidate);
        }
    }
    None
}

fn find_nested_lvgl_sys(checkout: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(checkout).ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join("lvgl-sys");
        if candidate.join("vendor/lvgl/src").is_dir() {
            return Some(candidate);
        }
    }
    None
}
