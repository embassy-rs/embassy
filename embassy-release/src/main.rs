use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use serde::Deserialize;
use toml_edit::{DocumentMut, Item, Value};
use walkdir::WalkDir;

/// Tool to traverse and operate on intra-repo Rust crate dependencies
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the root crate
    #[arg(value_name = "CRATE_PATH")]
    crate_path: PathBuf,

    /// Command to perform on each crate
    #[command(subcommand)]
    command: Command,

    /// Traversal order
    #[arg(short, long, default_value = "post")]
    order: TraversalOrder,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum TraversalOrder {
    Pre,
    Post,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print all dependencies
    Dependencies,

    /// Release crate
    Release {
        #[command(subcommand)]
        kind: ReleaseKind,
    },
}

#[derive(Debug, Subcommand, Clone, Copy, PartialEq)]
enum ReleaseKind {
    Patch,
    Minor,
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<Package>,
    dependencies: Option<Deps>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Dep {
    Version(String),
    DetailedTable(HashMap<String, toml::Value>),
}

type Deps = std::collections::HashMap<String, Dep>;

#[derive(Debug, Deserialize)]
struct CrateConfig {
    features: Option<Vec<String>>,
    target: Option<String>,
}

type ReleaseConfig = HashMap<String, CrateConfig>;

fn find_path_deps(cargo_path: &Path) -> Vec<PathBuf> {
    let content = fs::read_to_string(cargo_path).unwrap_or_else(|_| {
        panic!("Failed to read {:?}", cargo_path);
    });
    let parsed: CargoToml = toml::from_str(&content).unwrap_or_else(|e| {
        panic!("Failed to parse {:?}: {}", cargo_path, e);
    });

    let mut paths = vec![];
    if let Some(deps) = parsed.dependencies {
        for (_name, dep) in deps {
            match dep {
                Dep::Version(_) => {
                    // External dependency — skip
                }
                Dep::DetailedTable(table) => {
                    if let Some(toml::Value::String(path)) = table.get("path") {
                        let dep_path = cargo_path.parent().unwrap().join(path).canonicalize().unwrap();
                        paths.push(dep_path);
                    }
                }
            }
        }
    }

    paths
}

fn visit_recursive(
    root_crate: &Path,
    visited: &mut HashSet<PathBuf>,
    output: &mut Vec<PathBuf>,
    order: &TraversalOrder,
) {
    if !visited.insert(root_crate.to_path_buf()) {
        return;
    }

    let cargo_toml = root_crate.join("Cargo.toml");
    let deps = find_path_deps(&cargo_toml);

    if *order == TraversalOrder::Pre {
        output.push(root_crate.to_path_buf());
    }

    let mut deps_sorted = deps;
    deps_sorted.sort();
    for dep in deps_sorted {
        visit_recursive(&dep, visited, output, order);
    }

    if *order == TraversalOrder::Post {
        output.push(root_crate.to_path_buf());
    }
}

fn get_crate_metadata(crate_path: &Path) -> Option<(String, String)> {
    let cargo_toml = crate_path.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml).ok()?;
    let parsed: CargoToml = toml::from_str(&content).ok()?;
    let pkg = parsed.package?;
    let name = pkg.name;
    let version = pkg.version?;
    Some((name, version))
}

fn load_release_config() -> ReleaseConfig {
    let config_path = PathBuf::from("release/config.toml");
    if !config_path.exists() {
        return HashMap::new();
    }
    let content = fs::read_to_string(&config_path).expect("Failed to read release/config.toml");
    toml::from_str(&content).expect("Invalid TOML format in release/config.toml")
}

fn bump_dependency_versions(crate_name: &str, new_version: &str) -> Result<(), String> {
    let mut cargo_files: Vec<PathBuf> = WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == "Cargo.toml")
        .map(|e| e.into_path())
        .collect();

    cargo_files.sort();

    for path in cargo_files {
        let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let mut doc: DocumentMut = content
            .parse()
            .map_err(|e| format!("Failed to parse TOML in {}: {}", path.display(), e))?;

        let mut changed = false;

        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(Item::Table(dep_table)) = doc.get_mut(section) {
                if let Some(item) = dep_table.get_mut(crate_name) {
                    match item {
                        // e.g., foo = "0.1.0"
                        Item::Value(Value::String(_)) => {
                            *item = Item::Value(Value::from(new_version));
                            changed = true;
                        }
                        // e.g., foo = { version = "...", ... }
                        Item::Value(Value::InlineTable(inline)) => {
                            if inline.contains_key("version") {
                                inline["version"] = Value::from(new_version);
                                changed = true;
                            }
                        }
                        _ => {} // Leave unusual formats untouched
                    }
                }
            }
        }

        if changed {
            fs::write(&path, doc.to_string()).map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
            println!("🔧 Updated {} to {} in {}", crate_name, new_version, path.display());
        }
    }

    Ok(())
}

fn run_release_command(
    crate_path: &Path,
    crate_name: &str,
    version: &str,
    kind: &ReleaseKind,
    config: Option<&CrateConfig>,
) -> Result<(), String> {
    let kind_str = match kind {
        ReleaseKind::Patch => "patch",
        ReleaseKind::Minor => "minor",
    };

    if *kind == ReleaseKind::Minor {
        bump_dependency_versions(crate_name, version)?;
    }

    let mut args: Vec<String> = vec!["release".into(), kind_str.into()];

    if let Some(cfg) = config {
        if let Some(features) = &cfg.features {
            args.push("--features".into());
            args.push(features.join(","));
        }
        if let Some(target) = &cfg.target {
            args.push("--target".into());
            args.push(target.clone());
        }
    }

    let status = ProcessCommand::new("cargo")
        .args(&args)
        .current_dir(crate_path)
        .status()
        .map_err(|e| format!("Failed to run cargo release: {}", e))?;

    if !status.success() {
        return Err(format!("`cargo release {}` failed in crate {}", kind_str, crate_name));
    }

    //args.push("--execute".into());
    //let status = ProcessCommand::new("cargo")
    //    .args(&args)
    //    .current_dir(crate_path)
    //    .status()
    //    .map_err(|e| format!("Failed to run cargo release --execute: {}", e))?;

    //if !status.success() {
    //    return Err(format!(
    //        "`cargo release {kind_str} --execute` failed in crate {crate_name}"
    //    ));
    //}

    Ok(())
}

fn main() {
    let args = Args::parse();
    let root = args.crate_path.canonicalize().expect("Invalid root crate path");

    match args.command {
        Command::Dependencies => {
            let mut visited = HashSet::new();
            let mut ordered = vec![];
            visit_recursive(&root, &mut visited, &mut ordered, &args.order);
            for path in ordered {
                if let Some((name, _)) = get_crate_metadata(&path) {
                    println!("{name}");
                } else {
                    eprintln!("Warning: could not read crate name from {:?}", path);
                }
            }
        }
        Command::Release { kind } => {
            let config = load_release_config();
            let path = root;
            match get_crate_metadata(&path) {
                Some((name, version)) => {
                    println!("🚀 Releasing {name}...");
                    let crate_cfg = config.get(&name);
                    match run_release_command(&path, &name, &version, &kind, crate_cfg) {
                        Ok(_) => {
                            println!("✅ Released {name}");
                        }
                        Err(e) => {
                            eprintln!("❌ Error releasing {name}:\n{e}");
                            eprintln!("\nYou may retry with: `cargo run -- {path:?} release {kind:?}`");
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    eprintln!("Warning: Could not parse crate metadata in {:?}", path);
                }
            }
        }
    }
}
