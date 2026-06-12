// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    // docs.rs has no lv_conf.h and oxivgl-sys skips C compilation under DOCS_RS,
    // so skip image asset generation here too — only Rust docs need to render.
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    emit_font_cfgs();

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

/// Mirror oxivgl-sys: expose `lv_font_*` cfgs for optional built-in/custom fonts.
fn emit_font_cfgs() {
    for cfg in [
        "lv_font_montserrat_14",
        "lv_font_montserrat_16",
        "lv_font_montserrat_14_latin",
        "lv_font_montserrat_16_latin",
    ] {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }

    let conf_dir = match std::env::var("DEP_LV_CONFIG_PATH") {
        Ok(p) => std::path::PathBuf::from(p),
        Err(_) => return,
    };
    let lv_conf_h = conf_dir.join("lv_conf.h");
    if !lv_conf_h.exists() {
        return;
    }
    println!("cargo:rerun-if-changed={}", lv_conf_h.display());

    let content = std::fs::read_to_string(&lv_conf_h).expect("failed to read lv_conf.h");
    for (macro_name, cfg_name) in [
        ("LV_FONT_MONTSERRAT_14", "lv_font_montserrat_14"),
        ("LV_FONT_MONTSERRAT_16", "lv_font_montserrat_16"),
        ("LV_FONT_MONTSERRAT_14_LATIN", "lv_font_montserrat_14_latin"),
        ("LV_FONT_MONTSERRAT_16_LATIN", "lv_font_montserrat_16_latin"),
    ] {
        if lv_conf_def_enabled(&content, macro_name) {
            println!("cargo:rustc-cfg={cfg_name}");
        }
    }
}

fn lv_conf_def_enabled(content: &str, name: &str) -> bool {
    for line in content.lines() {
        let line = line.split("//").next().unwrap_or(line).trim();
        let Some(rest) = line.strip_prefix("#define ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        if parts.next() != Some(name) {
            continue;
        }
        return parts.next().is_some_and(|v| v == "1");
    }
    false
}
