use std::path::PathBuf;

use cargo_semver_checks::{Check, GlobalConfig, LintConfig, LintLevel, ReleaseType, RequiredSemverUpdate, Rustdoc};

use crate::cargo::CargoArgsBuilder;
use crate::types::{BuildConfig, Crate};

/// Return the minimum required bump for the next release.
/// Even if nothing changed this will be [ReleaseType::Patch]
pub fn minimum_update(krate: &Crate) -> Result<ReleaseType, anyhow::Error> {
    let config = krate.configs.first().unwrap(); // TODO

    let package_name = krate.name.clone();
    let current_path = build_doc_json(krate, config)?;

    // TODO: Prevent compiler panic on current compiler version
    std::env::set_var("RUSTFLAGS", "--cap-lints=warn");

    let baseline = Rustdoc::from_registry_latest_crate_version();
    let doc = Rustdoc::from_path(&current_path);
    let mut semver_check = Check::new(doc);
    semver_check.with_default_features();
    semver_check.set_baseline(baseline);
    semver_check.set_packages(vec![package_name]);
    let extra_current_features = config.features.clone();
    let extra_baseline_features = config.features.clone();
    semver_check.set_extra_features(extra_current_features, extra_baseline_features);
    if let Some(target) = &config.target {
        semver_check.set_build_target(target.clone());
    }
    let mut cfg = GlobalConfig::new();
    cfg.set_log_level(Some(log::Level::Trace));

    let mut lint_cfg = LintConfig::new();
    // Disable this lint because we provide the rustdoc json only, so it can't do feature comparison.
    lint_cfg.set("feature_missing", LintLevel::Allow, RequiredSemverUpdate::Minor, 0);
    cfg.set_lint_config(lint_cfg);
    let result = semver_check.check_release(&mut cfg)?;

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

pub(crate) fn build_doc_json(krate: &Crate, config: &BuildConfig) -> Result<PathBuf, anyhow::Error> {
    let target_dir = std::env::var("CARGO_TARGET_DIR");

    let target_path = if let Ok(target) = target_dir {
        PathBuf::from(target)
    } else {
        PathBuf::from(&krate.path).join("target")
    };

    let current_path = target_path;
    let current_path = if let Some(target) = &config.target {
        current_path.join(target.clone())
    } else {
        current_path
    };
    let current_path = current_path
        .join("doc")
        .join(format!("{}.json", krate.name.to_string().replace("-", "_")));

    std::fs::remove_file(&current_path).ok();
    let features = config.features.clone();

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
    let cargo_builder = if let Some(target) = &config.target {
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
