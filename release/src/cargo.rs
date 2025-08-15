//! Tools for working with Cargo.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context as _, Result};
use clap::ValueEnum as _;
use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Formatted, Item, Value};

use crate::{windows_safe_path, Crate};

#[derive(Clone, Debug, PartialEq)]
pub enum CargoAction {
    Build(PathBuf),
    Run,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub executable: PathBuf,
}

/// Execute cargo with the given arguments and from the specified directory.
pub fn run(args: &[String], cwd: &Path) -> Result<()> {
    run_with_env::<[(&str, &str); 0], _, _>(args, cwd, [], false)?;
    Ok(())
}

/// Execute cargo with the given arguments and from the specified directory.
pub fn run_with_env<I, K, V>(args: &[String], cwd: &Path, envs: I, capture: bool) -> Result<String>
where
    I: IntoIterator<Item = (K, V)> + core::fmt::Debug,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    if !cwd.is_dir() {
        bail!("The `cwd` argument MUST be a directory");
    }

    // Make sure to not use a UNC as CWD!
    // That would make `OUT_DIR` a UNC which will trigger things like the one fixed in https://github.com/dtolnay/rustversion/pull/51
    // While it's fixed in `rustversion` it's not fixed for other crates we are
    // using now or in future!
    let cwd = windows_safe_path(cwd);

    println!(
        "Running `cargo {}` in {:?} - Environment {:?}",
        args.join(" "),
        cwd,
        envs
    );

    let mut command = Command::new(get_cargo());

    command
        .args(args)
        .current_dir(cwd)
        .envs(envs)
        .stdout(if capture { Stdio::piped() } else { Stdio::inherit() })
        .stderr(if capture { Stdio::piped() } else { Stdio::inherit() });

    if args.iter().any(|a| a.starts_with('+')) {
        // Make sure the right cargo runs
        command.env_remove("CARGO");
    }

    let output = command.stdin(Stdio::inherit()).output()?;

    // Make sure that we return an appropriate exit code here, as Github Actions
    // requires this in order to function correctly:
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        bail!("Failed to execute cargo subcommand `cargo {}`", args.join(" "),)
    }
}

fn get_cargo() -> String {
    // On Windows when executed via `cargo run` (e.g. via the xtask alias) the
    // `cargo` on the search path is NOT the cargo-wrapper but the `cargo` from the
    // toolchain - that one doesn't understand `+toolchain`
    #[cfg(target_os = "windows")]
    let cargo = if let Ok(cargo) = std::env::var("CARGO_HOME") {
        format!("{cargo}/bin/cargo")
    } else {
        String::from("cargo")
    };

    #[cfg(not(target_os = "windows"))]
    let cargo = String::from("cargo");

    cargo
}

#[derive(Debug, Default)]
pub struct CargoArgsBuilder {
    toolchain: Option<String>,
    subcommand: String,
    target: Option<String>,
    features: Vec<String>,
    args: Vec<String>,
}

impl CargoArgsBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            toolchain: None,
            subcommand: String::new(),
            target: None,
            features: vec![],
            args: vec![],
        }
    }

    #[must_use]
    pub fn toolchain<S>(mut self, toolchain: S) -> Self
    where
        S: Into<String>,
    {
        self.toolchain = Some(toolchain.into());
        self
    }

    #[must_use]
    pub fn subcommand<S>(mut self, subcommand: S) -> Self
    where
        S: Into<String>,
    {
        self.subcommand = subcommand.into();
        self
    }

    #[must_use]
    pub fn target<S>(mut self, target: S) -> Self
    where
        S: Into<String>,
    {
        self.target = Some(target.into());
        self
    }

    #[must_use]
    pub fn features(mut self, features: &[String]) -> Self {
        self.features = features.to_vec();
        self
    }

    #[must_use]
    pub fn arg<S>(mut self, arg: S) -> Self
    where
        S: Into<String>,
    {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub fn args<S>(mut self, args: &[S]) -> Self
    where
        S: Clone + Into<String>,
    {
        for arg in args {
            self.args.push(arg.clone().into());
        }
        self
    }

    pub fn add_arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub fn build(&self) -> Vec<String> {
        let mut args = vec![];

        if let Some(ref toolchain) = self.toolchain {
            args.push(format!("+{toolchain}"));
        }

        args.push(self.subcommand.clone());

        if let Some(ref target) = self.target {
            args.push(format!("--target={target}"));
        }

        if !self.features.is_empty() {
            args.push(format!("--features={}", self.features.join(",")));
        }

        for arg in self.args.iter() {
            args.push(arg.clone());
        }

        args
    }
}

#[derive(Debug, Default)]
pub struct CargoBatchBuilder {
    commands: Vec<Vec<String>>,
}

impl CargoBatchBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    #[must_use]
    pub fn command(mut self, args: Vec<String>) -> Self {
        self.commands.push(args);
        self
    }

    pub fn add_command(&mut self, args: Vec<String>) -> &mut Self {
        self.commands.push(args);
        self
    }

    #[must_use]
    pub fn build(&self) -> Vec<String> {
        let mut args = vec!["batch".to_string()];

        for command in &self.commands {
            args.push("---".to_string());
            args.extend(command.clone());
        }

        args
    }
}
