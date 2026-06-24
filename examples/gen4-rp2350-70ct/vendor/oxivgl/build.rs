// SPDX-License-Identifier: GPL-3.0-only

/// Built-in LVGL font faces that may be disabled in the application's
/// `lv_conf.h`. Must match `GATED_FONTS` in `oxivgl-sys/build.rs` and the
/// `#[cfg(font_*)]`-gated consts in `src/fonts.rs`.
const GATED_FONTS: &[&str] = &[
    "montserrat_8", "montserrat_10", "montserrat_12", "montserrat_14",
    "montserrat_16", "montserrat_18", "montserrat_20", "montserrat_22",
    "montserrat_24", "montserrat_26", "montserrat_28", "montserrat_30",
    "montserrat_32", "montserrat_34", "montserrat_36", "montserrat_38",
    "montserrat_40", "montserrat_42", "montserrat_44", "montserrat_46",
    "montserrat_48", "dejavu_16_persian_hebrew", "source_han_sans_sc_14_cjk",
    "source_han_sans_sc_16_cjk",
];

/// Turn the `DEP_LV_FONT_<NAME>` metadata emitted by `oxivgl-sys` (which knows
/// exactly which font symbols the LVGL build exposed) into `font_<name>` cfgs,
/// so `src/fonts.rs` compiles a `Font` const only for faces that actually
/// exist. Declares every possible flag with `rustc-check-cfg` so the disabled
/// ones don't trip the `unexpected_cfgs` lint.
fn emit_font_cfgs() {
    for name in GATED_FONTS {
        println!("cargo::rustc-check-cfg=cfg(font_{name})");
        let dep = format!("DEP_LV_FONT_{}", name.to_uppercase());
        if std::env::var_os(&dep).is_some() {
            println!("cargo::rustc-cfg=font_{name}");
        }
    }
}

fn main() {
    // Font gating must run on every build (including docs.rs) so the `Font`
    // consts match the symbols `oxivgl-sys` exposed.
    emit_font_cfgs();

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

    // (gen4 patch) The upstream build converts three example PNG assets via
    // LVGL's `LVGLImage.py`, which fails against newer `pypng` (`asRGBA8` was
    // renamed to `asRGBA`). The gen4 OxivGL demo uses none of these example
    // images, so the conversion is dropped entirely to keep the build
    // self-contained and free of a host Python/pypng dependency.
}
