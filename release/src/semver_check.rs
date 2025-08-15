use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use cargo_semver_checks::{Check, GlobalConfig, ReleaseType, Rustdoc};

use crate::cargo::CargoArgsBuilder;
use crate::types::Crate;
use crate::windows_safe_path;

/// Return the minimum required bump for the next release.
/// Even if nothing changed this will be [ReleaseType::Patch]
pub fn minimum_update(krate: &Crate) -> Result<ReleaseType, anyhow::Error> {
    println!("Crate = {:?}", krate);

    let package_name = krate.name.clone();
    let package_path = krate.path.clone();
    let current_path = build_doc_json(krate)?;

    let baseline = Rustdoc::from_registry_latest_crate_version();
    let doc = Rustdoc::from_path(&current_path);
    let mut semver_check = Check::new(doc);
    semver_check.with_default_features();
    semver_check.set_baseline(baseline);
    semver_check.set_packages(vec![package_name]);
    let extra_current_features = krate.config.features.clone();
    let extra_baseline_features = krate.config.features.clone();
    semver_check.set_extra_features(extra_current_features, extra_baseline_features);
    if let Some(target) = &krate.config.target {
        semver_check.set_build_target(target.clone());
    }
    let mut cfg = GlobalConfig::new();
    cfg.set_log_level(Some(log::Level::Trace));
    let result = semver_check.check_release(&mut cfg)?;
    log::info!("Result {:?}", result);

    let mut min_required_update = ReleaseType::Patch;
    for (_, report) in result.crate_reports() {
        if let Some(required_bump) = report.required_bump() {
            let required_is_stricter =
                (min_required_update == ReleaseType::Patch) || (required_bump == ReleaseType::Major);
            if required_is_stricter {
                min_required_update = required_bump;
            }
        }
    }

    Ok(min_required_update)
}

pub(crate) fn build_doc_json(krate: &Crate) -> Result<PathBuf, anyhow::Error> {
    let target_dir = std::env::var("CARGO_TARGET_DIR");

    let target_path = if let Ok(target) = target_dir {
        PathBuf::from(target)
    } else {
        PathBuf::from(&krate.path).join("target")
    };

    let current_path = target_path;
    let current_path = if let Some(target) = &krate.config.target {
        current_path.join(target.clone())
    } else {
        current_path
    };
    let current_path = current_path
        .join("doc")
        .join(format!("{}.json", krate.name.to_string().replace("-", "_")));

    std::fs::remove_file(&current_path).ok();
    let features = krate.config.features.clone();

    log::info!("Building doc json for {} with features: {:?}", krate.name, features);

    let envs = vec![(
        "RUSTDOCFLAGS",
        "--cfg docsrs --cfg not_really_docsrs --cfg semver_checks",
    )];

    // always use `specific nightly` toolchain so we don't have to deal with potentially
    // different versions of the doc-json
    let cargo_builder = CargoArgsBuilder::default()
        .toolchain("nightly-2025-06-29")
        .subcommand("rustdoc")
        .features(&features);
    let cargo_builder = if let Some(target) = &krate.config.target {
        cargo_builder.target(target.clone())
    } else {
        cargo_builder
    };

    let cargo_builder = cargo_builder
        .arg("-Zunstable-options")
        .arg("-Zhost-config")
        .arg("-Ztarget-applies-to-host")
        .arg("--lib")
        .arg("--output-format=json")
        .arg("-Zbuild-std=alloc,core")
        .arg("--config=host.rustflags=[\"--cfg=instability_disable_unstable_docs\"]");
    let cargo_args = cargo_builder.build();
    log::debug!("{cargo_args:#?}");
    crate::cargo::run_with_env(&cargo_args, &krate.path, envs, false)?;
    Ok(current_path)
}
