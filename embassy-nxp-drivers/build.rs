//! Emits the cfgs the driver cores gate on.
//!
//! nxp-pac (`links = "nxp-pac"`) exports the selected chip's IP presence cfgs (`gpio`, ...) and
//! IP-option cfgs (`<ip>_has_<x>`, named after `FSL_FEATURE_<IP>_HAS_<X>`) as `DEP_NXP_PAC_*`.
//! A module compiles when its cargo feature is on and the chip has the IP. Without an export for
//! the chip (`DEP_NXP_PAC_CFGS` unset), the module feature alone decides and no IP-option cfg is
//! set. `DEP_NXP_PAC_ALL_CFGS` only declares the names for `check-cfg`. The driver source gates
//! only on the cfg names.

use std::env;

/// Driver modules, each behind the cargo feature of the same name.
const MODULES: &[&str] = &["gpio"];

fn list(key: &str) -> Vec<String> {
    env::var(key)
        .unwrap_or_default()
        .split(',')
        .filter(|c| !c.is_empty())
        .map(str::to_owned)
        .collect()
}

fn main() {
    let exported = env::var_os("DEP_NXP_PAC_CFGS").is_some();
    let chip = list("DEP_NXP_PAC_CFGS");

    for cfg in list("DEP_NXP_PAC_ALL_CFGS")
        .iter()
        .map(String::as_str)
        .chain(MODULES.iter().copied())
    {
        println!("cargo::rustc-check-cfg=cfg({cfg})");
    }

    for module in MODULES {
        let enabled = env::var_os(format!("CARGO_FEATURE_{}", module.to_uppercase())).is_some();
        let present = !exported || chip.iter().any(|c| c == module);
        if enabled && present {
            println!("cargo::rustc-cfg={module}");
        } else if enabled {
            println!("cargo::warning=feature `{module}` is enabled, but the selected chip has no {module}");
        }
    }

    for cfg in chip.iter().filter(|c| c.contains("_has_")) {
        println!("cargo::rustc-cfg={cfg}");
    }
}
