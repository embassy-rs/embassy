// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    // docs.rs has no lv_conf.h and oxivgl-sys skips C compilation under DOCS_RS,
    // so skip image asset generation here too — only Rust docs need to render.
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let target = std::env::var("TARGET").unwrap_or_default();
    if target.starts_with("xtensa-") {
        println!("cargo:rustc-link-arg=-Tlinkall.x");
    }
    // All targets: oxivgl-sys's build.rs (cc crate) compiles LVGL
    // (including lv_calendar_chinese.c) via recursive add_c_files().

    // Example PNG assets are only needed for upstream OxivGL host demos.
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("-none-") {
        return;
    }

    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
    );
    if !manifest_dir.join("examples/assets/img_cogwheel_argb.png").exists() {
        return;
    }

    let cfg = oxivgl_build::ImageConfig::from_env();
    cfg.image_asset("img_cogwheel_argb", "examples/assets/img_cogwheel_argb.png");
    cfg.image_asset("img_skew_strip", "examples/assets/img_skew_strip.png");
    cfg.image_asset("img_star", "examples/assets/img_star.png");
}
