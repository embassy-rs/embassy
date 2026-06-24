// SPDX-License-Identifier: MIT OR Apache-2.0
use cc::Build;
#[cfg(feature = "drivers")]
use std::collections::HashSet;
use std::{
    env,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

const LVGL_VERSION: &str = "9.5.0";
const LVGL_SHA256: &str = "34a955cdf3a2d005507b704e87357af669a114523b6d3f77b5344fdc68717bc6";

/// Built-in LVGL font faces that the application may disable in its `lv_conf.h`.
/// Each entry is the `lv_font_<NAME>` suffix; a face is "available" only when
/// its `LV_FONT_*` option is enabled, in which case bindgen emits the matching
/// `extern static` and we surface a `font_<NAME>` flag for `oxivgl` to gate on.
/// Keep in sync with the list in `oxivgl/build.rs` and the consts in
/// `oxivgl/src/fonts.rs`.
const GATED_FONTS: &[&str] = &[
    "montserrat_8", "montserrat_10", "montserrat_12", "montserrat_14",
    "montserrat_16", "montserrat_18", "montserrat_20", "montserrat_22",
    "montserrat_24", "montserrat_26", "montserrat_28", "montserrat_30",
    "montserrat_32", "montserrat_34", "montserrat_36", "montserrat_38",
    "montserrat_40", "montserrat_42", "montserrat_44", "montserrat_46",
    "montserrat_48", "dejavu_16_persian_hebrew", "source_han_sans_sc_14_cjk",
    "source_han_sans_sc_16_cjk",
];

/// Inspect the generated `bindings.rs` and emit a `cargo:font_<NAME>=1`
/// metadata value for every built-in font whose `extern static` is present.
/// Via `links = "lv"` this reaches `oxivgl`'s build script as
/// `DEP_LV_FONT_<NAME>`, which it turns into a `font_<NAME>` cfg. Using the
/// generated bindings as the source of truth means the flag matches symbol
/// availability exactly — including faces left at their `lv_conf_internal.h`
/// default rather than spelled out in the app's `lv_conf.h`.
fn emit_font_flags(bindings_path: &Path) {
    let src = std::fs::read_to_string(bindings_path).unwrap_or_default();
    for name in GATED_FONTS {
        if contains_ident(&src, &format!("lv_font_{name}")) {
            println!("cargo:font_{name}=1");
        }
    }
}

/// True if `ident` occurs in `src` as a whole identifier — i.e. not
/// immediately followed by another identifier character. Robust to bindgen's
/// spacing (`lv_font_x :` vs `lv_font_x:`) and collision-free across numeric
/// suffixes (`montserrat_4` does not match `montserrat_40`).
fn contains_ident(src: &str, ident: &str) -> bool {
    let bytes = src.as_bytes();
    let mut from = 0;
    while let Some(pos) = src[from..].find(ident) {
        let end = from + pos + ident.len();
        let next = bytes.get(end).copied().unwrap_or(b' ');
        if !next.is_ascii_alphanumeric() && next != b'_' {
            return true;
        }
        from = end;
    }
    false
}

/// Download and extract LVGL source tree into `out_dir/lvgl-{version}/`.
/// Returns the path to the extracted LVGL root.
/// Respects `LVGL_SRC_DIR` env var override for local development.
fn ensure_lvgl_source(out_dir: &Path) -> PathBuf {
    // User override: use local LVGL source
    if let Ok(dir) = env::var("LVGL_SRC_DIR") {
        let p = PathBuf::from(dir);
        if p.join("lv_version.h").exists() {
            return p;
        }
        panic!("LVGL_SRC_DIR={} does not contain lv_version.h", p.display());
    }

    let lvgl_dir = out_dir.join(format!("lvgl-{LVGL_VERSION}"));
    if lvgl_dir.join("lv_version.h").exists() {
        return lvgl_dir;
    }

    let url = format!("https://github.com/lvgl/lvgl/archive/refs/tags/v{LVGL_VERSION}.tar.gz");
    eprintln!("Downloading LVGL v{LVGL_VERSION} from {url}");

    let mut resp = ureq::get(&url).call().expect("Failed to download LVGL");
    let tarball = resp
        .body_mut()
        .with_config()
        .limit(100 * 1024 * 1024)
        .read_to_vec()
        .expect("Failed to read LVGL tarball");

    // Verify SHA256
    let hash = format!("{:x}", Sha256::digest(&tarball));
    assert_eq!(hash, LVGL_SHA256, "LVGL tarball SHA256 mismatch!");

    // Extract
    let decoder = flate2::read::GzDecoder::new(&tarball[..]);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(out_dir)
        .expect("Failed to extract LVGL tarball");

    assert!(
        lvgl_dir.join("lv_version.h").exists(),
        "LVGL extraction failed"
    );
    lvgl_dir
}

static CONFIG_NAME: &str = "DEP_LV_CONFIG_PATH";

// See https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-450750547
#[cfg(feature = "drivers")]
#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);
#[cfg(feature = "drivers")]
impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn main() {
    // docs.rs has no network access, so we cannot download LVGL or run
    // bindgen. Use a pre-generated host (x86_64-linux) bindings file and
    // skip both the C compilation and bindgen pipelines entirely.
    if env::var("DOCS_RS").is_ok() {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let bindings_path = out_path.join("bindings.rs");
        std::fs::copy(manifest_dir.join("bindings_docsrs.rs"), &bindings_path)
            .expect("failed to install bundled bindings_docsrs.rs");
        emit_font_flags(&bindings_path);
        return;
    }

    let project_dir = canonicalize(PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()));
    let shims_dir = project_dir.join("shims");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lvgl_dir = ensure_lvgl_source(&out_path);
    let lvgl_src = lvgl_dir.join("src");

    #[cfg(feature = "rust_timer")]
    let timer_shim = shims_dir.join("timer");

    let font_extra_src: Option<PathBuf>;
    if let Ok(v) = env::var("PWD") {
        let current_dir = canonicalize(PathBuf::from(v));
        font_extra_src = {
            if let Ok(p) = env::var("LVGL_FONTS_DIR") {
                Some(canonicalize(PathBuf::from(p)))
            } else if current_dir.join("fonts").exists() {
                Some(current_dir.join("fonts"))
            } else {
                None
            }
        };
    } else {
        font_extra_src = None
    }

    // Some basic defaults; SDL2 is the only driver enabled in the provided
    // driver config by default
    #[cfg(feature = "drivers")]
    let incl_extra =
        env::var("LVGL_INCLUDE").unwrap_or("/usr/include,/usr/local/include".to_string());

    let cflags_extra_string = env::var("LVGL_CFLAGS").unwrap_or_default();

    let cflags_extra = if cflags_extra_string.is_empty() {
        None
    } else {
        Some(cflags_extra_string.split(','))
    };

    #[cfg(feature = "drivers")]
    let link_extra = env::var("LVGL_LINK").unwrap_or("SDL2".to_string());

    #[cfg(feature = "drivers")]
    let drivers = project_dir.join("lv_drivers");

    let lv_config_dir = {
        let conf_path = env::var(CONFIG_NAME).map(PathBuf::from).unwrap_or_else(|_| {
            // On docs.rs the workspace .cargo/config.toml is unavailable, so
            // fall back to the bundled default config to allow doc rendering.
            if env::var("DOCS_RS").is_ok() {
                return project_dir.join("default-conf");
            }
            panic!(
                "The environment variable {} is required to be defined",
                CONFIG_NAME
            );
        });

        if !conf_path.exists() {
            panic!(
                "Directory {} referenced by {} needs to exist",
                conf_path.to_string_lossy(),
                CONFIG_NAME
            );
        }
        if !conf_path.is_dir() {
            panic!("{} needs to be a directory", CONFIG_NAME);
        }
        if !conf_path.join("lv_conf.h").exists() {
            panic!(
                "Directory {} referenced by {} needs to contain a file called lv_conf.h",
                conf_path.to_string_lossy(),
                CONFIG_NAME
            );
        }
        #[cfg(feature = "drivers")]
        if !conf_path.join("lv_drv_conf.h").exists() {
            panic!(
                "Directory {} referenced by {} needs to contain a file called lv_drv_conf.h",
                conf_path.to_string_lossy(),
                CONFIG_NAME
            );
        }

        if let Some(p) = &font_extra_src {
            println!("cargo:rerun-if-changed={}", p.to_str().unwrap())
        }

        println!(
            "cargo:rerun-if-changed={}",
            conf_path.join("lv_conf.h").to_str().unwrap()
        );
        #[cfg(feature = "drivers")]
        println!(
            "cargo:rerun-if-changed={}",
            conf_path.join("lv_drv_conf.h").to_str().unwrap()
        );
        conf_path
    };

    #[cfg(feature = "drivers")]
    {
        println!("cargo:rerun-if-env-changed=LVGL_INCLUDE");
        println!("cargo:rerun-if-env-changed=LVGL_LINK");
    }

    let mut cfg = Build::new();
    let target_str = env::var("TARGET").unwrap_or_default();
    if target_str.starts_with("xtensa-") {
        cfg.flag("-mlongcalls");
    }
    if let Some(p) = &font_extra_src {
        add_c_files(&mut cfg, p)
    }
    patch_btnmatrix_text_length(&lvgl_src);
    println!("cargo:SRC_DIR={}", lvgl_dir.display());
    add_c_files(&mut cfg, &lvgl_src);
    add_c_files(&mut cfg, &lv_config_dir);
    add_c_files(&mut cfg, &shims_dir);
    #[cfg(feature = "drivers")]
    add_c_files(&mut cfg, &drivers);

    // SDL2 is only used for native host development builds (not embedded ARM).
    // (gen4 patch) The upstream 0.2.2 linked SDL2 for *any* non-xtensa target,
    // which breaks bare-metal thumb firmware links. Restrict it to native host.
    let host = env::var("HOST").expect("Cargo build scripts always have HOST");
    let is_native_host = target_str == host;
    if is_native_host && !target_str.starts_with("xtensa-") {
        if let Ok(lib) = pkg_config::probe_library("sdl2") {
            for p in &lib.include_paths {
                cfg.include(p);
            }
        }
        println!("cargo:rustc-link-lib=SDL2");
    }

    cfg.define("LV_CONF_INCLUDE_SIMPLE", Some("1"))
        .include(&lvgl_dir)
        .include(&lvgl_src)
        .warnings(false)
        .include(&lv_config_dir);
    if let Some(p) = &font_extra_src {
        cfg.include(p);
    }
    #[cfg(feature = "rust_timer")]
    cfg.include(&timer_shim);
    #[cfg(feature = "drivers")]
    cfg.include(&drivers);
    #[cfg(feature = "drivers")]
    cfg.includes(incl_extra.split(','));

    if let Some(ref cflags_extra) = cflags_extra {
        cflags_extra.clone().for_each(|e| {
            let mut it = e.split('=');
            cfg.define(it.next().unwrap(), it.next().unwrap_or_default());
        });
    }

    let mut cc_args = vec![
        "-DLV_CONF_INCLUDE_SIMPLE=1",
        "-I",
        lv_config_dir.to_str().unwrap(),
        "-I",
        lvgl_dir.to_str().unwrap(),
        "-fvisibility=default",
    ];

    // For Xtensa targets, auto-detect the ESP-capable clang if LIBCLANG_PATH
    // doesn't already point to one. The system clang doesn't understand Xtensa.
    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    if target.starts_with("xtensa-") {
        let current = env::var("LIBCLANG_PATH").unwrap_or_default();
        if !current.contains("esp") {
            // Search common ESP clang locations (devcontainer, CI image).
            let suffix = "toolchains/esp/xtensa-esp32-elf-clang/esp-20.1.1_20250829/esp-clang/lib";
            let candidates = [
                format!("{}/.rustup/{suffix}", env::var("HOME").unwrap_or_default()),
                format!("{}/{suffix}", env::var("RUSTUP_HOME").unwrap_or_default()),
            ];
            for path in &candidates {
                if std::path::Path::new(path).exists() {
                    env::set_var("LIBCLANG_PATH", path);
                    break;
                }
            }
        }
    }

    // Set correct target triple for bindgen when cross-compiling
    if target != host {
        cc_args.push("-target");
        cc_args.push(target.as_str());
    }

    let mut additional_args = Vec::new();
    // Add SDL2 include paths for bindgen on native host builds only.
    if is_native_host && !target.starts_with("xtensa-") {
        // libclang may predefine __ARM_ARCH even on x86_64 hosts, which forces
        // LV_USE_SDL=0 in lv_conf.h and drops SDL driver symbols from bindgen.
        additional_args.push("-U__ARM_ARCH".to_string());
        additional_args.push("-D__linux__".to_string());
        if let Ok(lib) = pkg_config::probe_library("sdl2") {
            for p in &lib.include_paths {
                additional_args.push("-I".to_string());
                additional_args.push(p.to_str().unwrap().to_string());
            }
        }
    }
    // (gen4 patch) Embedded ARM: point bindgen at the newlib headers shipped
    // with arm-none-eabi-gcc so `inttypes.h` / `stdint.h` resolve when
    // cross-compiling for bare-metal thumb targets (the system clang headers
    // alone fail with "'inttypes.h' file not found").
    if target.contains("-none-") && target.contains("thumb") {
        for inc in ["/usr/lib/arm-none-eabi/include", "/usr/arm-none-eabi/include"] {
            if std::path::Path::new(inc).join("inttypes.h").exists() {
                additional_args.push("-isystem".to_string());
                additional_args.push(inc.to_string());
                break;
            }
        }
        if let Ok(output) = std::process::Command::new("arm-none-eabi-gcc")
            .arg("-print-sysroot")
            .output()
        {
            let sysroot = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !sysroot.is_empty() {
                additional_args.push("-isystem".to_string());
                additional_args.push(format!("{sysroot}/include"));
            }
        }
        if let Ok(entries) = std::fs::read_dir("/usr/lib/gcc/arm-none-eabi") {
            for entry in entries.flatten() {
                let inc = entry.path().join("include");
                if inc.is_dir() {
                    additional_args.push("-isystem".to_string());
                    additional_args.push(inc.to_string_lossy().to_string());
                    break;
                }
            }
        }
    }
    if target.ends_with("emscripten") {
        match env::var("EMSDK") {
            Ok(em_path) =>
        {
            additional_args.push("-I".to_string());
            additional_args.push(format!(
                "{}/upstream/emscripten/system/include/libc",
                em_path
            ));
            additional_args.push("-I".to_string());
            additional_args.push(format!(
                "{}/upstream/emscripten/system/lib/libc/musl/arch/emscripten",
                em_path
            ));
            additional_args.push("-I".to_string());
            additional_args.push(format!(
                "{}/upstream/emscripten/system/include/SDL",
                em_path
            ));
        }
        Err(_) => panic!("The EMSDK environment variable is not set. Has emscripten been properly initialized?")
        }
    }

    #[cfg(feature = "drivers")]
    let ignored_macros = IgnoreMacros(
        vec![
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
            "IPPORT_RESERVED".into(),
        ]
        .into_iter()
        .collect(),
    );

    let bindings =
        bindgen::Builder::default().header(shims_dir.join("lvgl_sys.h").to_str().unwrap());
    let bindings = add_font_headers(bindings, &font_extra_src);
    #[cfg(feature = "drivers")]
    let bindings = bindings
        .header(shims_dir.join("lvgl_drv.h").to_str().unwrap())
        .parse_callbacks(Box::new(ignored_macros));
    #[cfg(feature = "rust_timer")]
    let bindings = bindings.header(shims_dir.join("rs_timer.h").to_str().unwrap());

    let extra_clang_args: Vec<String> = env::var("BINDGEN_EXTRA_CLANG_ARGS")
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_owned)
        .collect();

    let bindings = bindings
        .generate_comments(false)
        .derive_default(true)
        .layout_tests(false)
        .use_core()
        .ctypes_prefix("core::ffi")
        .clang_args(&cc_args)
        .clang_args(&additional_args)
        .clang_args(
            cflags_extra
                .map(|s| s.collect::<Vec<_>>())
                .unwrap_or(Vec::new()),
        )
        .clang_args(&extra_clang_args)
        .wrap_unsafe_ops(true)
        .wrap_static_fns(true)
        .wrap_static_fns_path(out_path.join("static_fns.c"))
        .generate()
        .expect("Unable to generate bindings");

    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Can't write bindings!");

    // bindgen 0.72 emits `transmute` for signed↔unsigned bitfield casts;
    // newer rustc warns (unnecessary_transmutes). Patch to use direct casts.
    fix_bindgen_transmutes(&bindings_path);

    // Surface which built-in fonts actually made it into the bindings so
    // `oxivgl` can gate its font consts and avoid forcing every face on.
    emit_font_flags(&bindings_path);

    cfg.file(out_path.join("static_fns.c"));
    cfg.compile("lvgl");

    #[cfg(feature = "drivers")]
    link_extra.split(',').for_each(|a| {
        println!("cargo:rustc-link-lib={a}");
        //println!("cargo:rustc-link-search=")
    })
}

fn add_font_headers(
    bindings: bindgen::Builder,
    dir: &Option<impl AsRef<Path>>,
) -> bindgen::Builder {
    if let Some(p) = dir {
        let mut temp = bindings;
        for e in p.as_ref().read_dir().unwrap() {
            let e = e.unwrap();
            let path = e.path();
            if !e.file_type().unwrap().is_dir()
                && path.extension().and_then(|s| s.to_str()) == Some("h")
            {
                temp = temp.header(path.to_str().unwrap());
            }
        }
        temp
    } else {
        bindings
    }
}

/// LVGL 9.5 does not preserve text_length through the draw task pipeline
/// on 32-bit targets, truncating button text to 1 character.
fn patch_btnmatrix_text_length(lvgl_src: &Path) {
    let file = lvgl_src.join("widgets/buttonmatrix/lv_buttonmatrix.c");
    if !file.exists() {
        return;
    }
    let code = std::fs::read_to_string(&file).unwrap();
    let needle = "draw_label_dsc_act.text_local = true;\n        draw_label_dsc_act.base.id1";
    if code.contains(needle) && !code.contains("draw_label_dsc_act.text_length") {
        let patched = code.replace(
            needle,
            "draw_label_dsc_act.text_local = true;\n        draw_label_dsc_act.text_length = (uint32_t)lv_strlen(txt);\n        draw_label_dsc_act.base.id1",
        );
        std::fs::write(&file, patched).unwrap();
    }
}

fn add_c_files(build: &mut cc::Build, path: impl AsRef<Path>) {
    for e in path.as_ref().read_dir().unwrap() {
        let e = e.unwrap();
        let path = e.path();
        if e.file_type().unwrap().is_dir() {
            add_c_files(build, e.path());
        } else if path.extension().and_then(|s| s.to_str()) == Some("c") {
            build.file(&path);
        }
    }
}

/// Replace unnecessary `transmute` calls in bindgen bitfield accessors and
/// strip `unsafe` blocks that become safe after removal.
/// bindgen 0.72 uses transmute for integer casts that rustc now warns about.
fn fix_bindgen_transmutes(path: &Path) {
    let mut code = std::fs::read_to_string(path).unwrap();

    // Phase 1: Replace `::core::mem::transmute(INNER)` → `(INNER) as _`.
    // Uses paren-matching to handle multi-line expressions.
    // Support both spaced (`:: core :: mem :: transmute (`) and compact
    // (`::core::mem::transmute(`) formats emitted by different bindgen versions.
    let needles = [
        ":: core :: mem :: transmute (",
        "::core::mem::transmute(",
    ];
    while let Some((start, needle_len)) = needles
        .iter()
        .filter_map(|n| code.find(n).map(|pos| (pos, n.len())))
        .min_by_key(|(pos, _)| *pos)
    {
        let inner_start = start + needle_len;
        let mut depth: u32 = 1;
        let mut end = inner_start;
        for ch in code[inner_start..].chars() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            end += ch.len_utf8();
        }
        let inner = code[inner_start..end].to_string();
        let replacement = format!("({}) as _", inner);
        code = format!("{}{}{}", &code[..start], replacement, &code[end + 1..]);
    }

    // Phase 2: Strip `unsafe { ... }` blocks that no longer contain unsafe ops.
    // Keep blocks containing `raw_get`, `raw_set`, or `addr_of` (raw-pointer ops).
    let unsafe_kw = "unsafe {";
    let mut result = String::with_capacity(code.len());
    let mut pos = 0;
    let bytes = code.as_bytes();
    while pos < code.len() {
        if let Some(rel) = code[pos..].find(unsafe_kw) {
            let block_start = pos + rel;
            let brace_start = block_start + unsafe_kw.len() - 1; // position of '{'
                                                                 // Find matching '}'
            let mut depth: u32 = 1;
            let mut end = brace_start + 1;
            while end < code.len() && depth > 0 {
                match bytes[end] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                end += 1;
            }
            let body = &code[brace_start + 1..end - 1]; // between { and }
                                                        // Only strip unsafe from blocks whose body is purely safe after
                                                        // transmute removal: bitfield get/set and simple casts.
            let is_safe_body = !body.contains("unsafe")
                && !body.contains("raw_get")
                && !body.contains("raw_set")
                && !body.contains("addr_of")
                && !body.contains("write_bytes")
                && !body.contains("assume_init")
                && !body.contains("from_raw")
                && !body.contains("as_ptr")
                && !body.contains("read_unaligned")
                && !body.contains("write_unaligned")
                && !body.contains("copy_nonoverlapping")
                && (body.contains("_bitfield_1") || body.contains("as _"));
            let needs_unsafe = !is_safe_body;

            // Copy text before `unsafe`
            result.push_str(&code[pos..block_start]);

            if needs_unsafe {
                // Keep the entire `unsafe { ... }` block
                result.push_str(&code[block_start..end]);
            } else {
                // Strip `unsafe { }`, keep the body with adjusted whitespace.
                // Single-line: `unsafe { EXPR }` → `EXPR`
                // Multi-line: preserve inner indentation as-is.
                let trimmed = body.trim();
                if !body.contains('\n') {
                    result.push_str(trimmed);
                } else {
                    result.push_str(body);
                }
            }
            pos = end;
        } else {
            result.push_str(&code[pos..]);
            break;
        }
    }

    std::fs::write(path, result).unwrap();
}

fn canonicalize(path: impl AsRef<Path>) -> PathBuf {
    let canonicalized = path.as_ref().canonicalize().unwrap();
    let canonicalized = &*canonicalized.to_string_lossy();

    PathBuf::from(canonicalized.strip_prefix(r"\\?\").unwrap_or(canonicalized))
}
