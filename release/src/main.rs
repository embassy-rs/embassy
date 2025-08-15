use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Bfs;
use petgraph::{Directed, Direction};
use toml_edit::{DocumentMut, Item, Value};
use types::*;

mod types;

/// Tool to traverse and operate on intra-repo Rust crate dependencies
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to embassy repository
    #[arg(short, long)]
    repo: PathBuf,

    /// Command to perform on each crate
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// All crates and their direct dependencies
    List,
    /// List all dependencies for a crate
    Dependencies {
        /// Crate name to print dependencies for.
        #[arg(value_name = "CRATE")]
        crate_name: String,
    },
    /// List all dependencies for a crate
    Dependents {
        /// Crate name to print dependencies for.
        #[arg(value_name = "CRATE")]
        crate_name: String,
    },

    /// SemverCheck
    SemverCheck {
        /// Crate to check. Will traverse that crate an it's dependents. If not specified checks all crates.
        #[arg(value_name = "CRATE")]
        crate_name: String,
    },
    /// Prepare to release a crate and all dependents that needs updating
    /// - Semver checks
    /// - Bump versions and commit
    /// - Create tag.
    PrepareRelease {
        /// Crate to release. Will traverse that crate an it's dependents. If not specified checks all crates.
        #[arg(value_name = "CRATE")]
        crate_name: String,
    },
}

fn load_release_config(repo: &Path) -> ReleaseConfig {
    let config_path = repo.join("release/config.toml");
    if !config_path.exists() {
        return HashMap::new();
    }
    let content = fs::read_to_string(&config_path).expect("Failed to read release/config.toml");
    toml::from_str(&content).expect("Invalid TOML format in release/config.toml")
}

fn update_version(c: &mut Crate, new_version: &str) -> Result<()> {
    let path = &c.path;
    c.version = new_version.to_string();
    let content = fs::read_to_string(&path)?;
    let mut doc: DocumentMut = content.parse()?;
    for section in ["package"] {
        if let Some(Item::Table(dep_table)) = doc.get_mut(section) {
            dep_table.insert("version", Item::Value(Value::from(new_version)));
        }
    }
    fs::write(&path, doc.to_string())?;
    Ok(())
}

fn update_versions(to_update: &Crate, dep: &CrateId, new_version: &str) -> Result<()> {
    let path = &to_update.path;
    let content = fs::read_to_string(&path)?;
    let mut doc: DocumentMut = content.parse()?;
    let mut changed = false;
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(Item::Table(dep_table)) = doc.get_mut(section) {
            if let Some(item) = dep_table.get_mut(&dep) {
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
        fs::write(&path, doc.to_string())?;
        println!("🔧 Updated {} to {} in {}", dep, new_version, path.display());
    }
    Ok(())
}

fn list_crates(path: &PathBuf) -> Result<BTreeMap<CrateId, Crate>> {
    let d = std::fs::read_dir(path)?;
    let release_config = load_release_config(path);
    let mut crates = BTreeMap::new();
    for c in d {
        let entry = c?;
        let name = entry.file_name().to_str().unwrap().to_string();
        if entry.file_type()?.is_dir() && name.starts_with("embassy-") {
            let entry = entry.path().join("Cargo.toml");
            if entry.exists() {
                let content = fs::read_to_string(&entry)?;
                let parsed: ParsedCrate = toml::from_str(&content)?;
                let id = parsed.package.name;

                let mut dependencies = Vec::new();
                for (k, _) in parsed.dependencies {
                    if k.starts_with("embassy-") {
                        dependencies.push(k);
                    }
                }

                let path = path.join(entry);
                if let Some(config) = release_config.get(&id) {
                    crates.insert(
                        id.clone(),
                        Crate {
                            name: id,
                            version: parsed.package.version,
                            path,
                            dependencies,
                            config: config.clone(),
                        },
                    );
                }
            }
        }
    }
    Ok(crates)
}

fn build_graph(crates: &BTreeMap<CrateId, Crate>) -> (Graph<CrateId, ()>, HashMap<CrateId, NodeIndex>) {
    let mut graph = Graph::<CrateId, (), Directed>::new();
    let mut node_indices: HashMap<CrateId, NodeIndex> = HashMap::new();

    // Helper to insert or get existing node
    let get_or_insert_node = |id: CrateId, graph: &mut Graph<CrateId, ()>, map: &mut HashMap<CrateId, NodeIndex>| {
        if let Some(&idx) = map.get(&id) {
            idx
        } else {
            let idx = graph.add_node(id.clone());
            map.insert(id, idx);
            idx
        }
    };

    for krate in crates.values() {
        get_or_insert_node(krate.name.clone(), &mut graph, &mut node_indices);
    }

    for krate in crates.values() {
        // Insert crate node if not exists
        let crate_idx = get_or_insert_node(krate.name.clone(), &mut graph, &mut node_indices);

        // Insert dependencies and connect edges
        for dep in krate.dependencies.iter() {
            let dep_idx = get_or_insert_node(dep.clone(), &mut graph, &mut node_indices);
            graph.add_edge(crate_idx, dep_idx, ());
        }
    }

    (graph, node_indices)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let root = args.repo.canonicalize()?;
    let mut crates = list_crates(&root)?;
    let (mut graph, indices) = build_graph(&crates);

    match args.command {
        Command::List => {
            let ordered = petgraph::algo::toposort(&graph, None).unwrap();
            for node in ordered.iter() {
                if graph.neighbors_directed(*node, Direction::Incoming).count() == 0 {
                    let start = graph.node_weight(*node).unwrap();
                    let mut bfs = Bfs::new(&graph, *node);
                    while let Some(node) = bfs.next(&graph) {
                        let weight = graph.node_weight(node).unwrap();
                        let c = crates.get(weight).unwrap();
                        if weight == start {
                            println!("+ {}-{}", weight, c.version);
                        } else {
                            println!("|- {}-{}", weight, c.version);
                        }
                    }
                    println!("");
                }
            }
        }
        Command::Dependencies { crate_name } => {
            let idx = indices.get(&crate_name).expect("unable to find crate in tree");
            let mut bfs = Bfs::new(&graph, *idx);
            while let Some(node) = bfs.next(&graph) {
                let weight = graph.node_weight(node).unwrap();
                let crt = crates.get(weight).unwrap();
                if *weight == crate_name {
                    println!("+ {}-{}", weight, crt.version);
                } else {
                    println!("|- {}-{}", weight, crt.version);
                }
            }
        }
        Command::Dependents { crate_name } => {
            let idx = indices.get(&crate_name).expect("unable to find crate in tree");
            let weight = graph.node_weight(*idx).unwrap();
            let crt = crates.get(weight).unwrap();
            println!("+ {}-{}", weight, crt.version);
            for parent in graph.neighbors_directed(*idx, Direction::Incoming) {
                let weight = graph.node_weight(parent).unwrap();
                let crt = crates.get(weight).unwrap();
                println!("|- {}-{}", weight, crt.version);
            }
        }
        Command::SemverCheck { crate_name } => {
            let c = crates.get(&crate_name).unwrap();
            check_semver(&c)?;
        }
        Command::PrepareRelease { crate_name } => {
            let start = indices.get(&crate_name).expect("unable to find crate in tree");
            graph.reverse();

            let mut bfs = Bfs::new(&graph, *start);

            while let Some(node) = bfs.next(&graph) {
                let weight = graph.node_weight(node).unwrap();
                println!("Preparing {}", weight);
                let mut c = crates.get_mut(weight).unwrap();
                let ver = semver::Version::parse(&c.version)?;
                let newver = if let Err(_) = check_semver(&c) {
                    println!("Semver check failed, bumping minor!");
                    semver::Version::new(ver.major, ver.minor + 1, 0)
                } else {
                    semver::Version::new(ver.major, ver.minor, ver.patch + 1)
                };

                println!("Updating {} from {} -> {}", weight, c.version, newver.to_string());
                let newver = newver.to_string();

                update_version(&mut c, &newver)?;
                let c = crates.get(weight).unwrap();

                // Update all nodes further down the tree
                let mut bfs = Bfs::new(&graph, node);
                while let Some(dep_node) = bfs.next(&graph) {
                    let dep_weight = graph.node_weight(dep_node).unwrap();
                    let dep = crates.get(dep_weight).unwrap();
                    update_versions(dep, &c.name, &newver)?;
                }

                // Update changelog
                update_changelog(&root, &c)?;
            }

            let weight = graph.node_weight(*start).unwrap();
            let c = crates.get(weight).unwrap();
            publish_release(&root, &c, false)?;

            println!("# Please inspect changes and run the following commands when happy:");

            println!("git commit -a -m 'chore: prepare crate releases'");
            let mut bfs = Bfs::new(&graph, *start);
            while let Some(node) = bfs.next(&graph) {
                let weight = graph.node_weight(node).unwrap();
                let c = crates.get(weight).unwrap();
                println!("git tag {}-v{}", weight, c.version);
            }

            println!("");
            println!("# Run these commands to publish the crate and dependents:");

            let mut bfs = Bfs::new(&graph, *start);
            while let Some(node) = bfs.next(&graph) {
                let weight = graph.node_weight(node).unwrap();
                let c = crates.get(weight).unwrap();

                let mut args: Vec<String> = vec![
                    "publish".to_string(),
                    "--manifest-path".to_string(),
                    c.path.display().to_string(),
                ];

                if let Some(features) = &c.config.features {
                    args.push("--features".into());
                    args.push(features.join(","));
                }

                if let Some(target) = &c.config.target {
                    args.push("--target".into());
                    args.push(target.clone());
                }

                /*
                let mut dry_run = args.clone();
                dry_run.push("--dry-run".to_string());

                println!("cargo {}", dry_run.join(" "));
                */
                println!("cargo {}", args.join(" "));
            }

            println!("");
            println!("# Run this command to push changes and tags:");
            println!("git push --tags");
        }
    }
    Ok(())
}

fn check_semver(c: &Crate) -> Result<()> {
    let mut args: Vec<String> = vec![
        "semver-checks".to_string(),
        "--manifest-path".to_string(),
        c.path.display().to_string(),
        "--default-features".to_string(),
    ];
    if let Some(features) = &c.config.features {
        args.push("--features".into());
        args.push(features.join(","));
    }

    let status = ProcessCommand::new("cargo").args(&args).output()?;

    println!("{}", core::str::from_utf8(&status.stdout).unwrap());
    eprintln!("{}", core::str::from_utf8(&status.stderr).unwrap());
    if !status.status.success() {
        return Err(anyhow!("semver check failed"));
    } else {
        Ok(())
    }
}

fn update_changelog(repo: &Path, c: &Crate) -> Result<()> {
    let args: Vec<String> = vec![
        "release".to_string(),
        "replace".to_string(),
        "--config".to_string(),
        repo.join("release").join("release.toml").display().to_string(),
        "--manifest-path".to_string(),
        c.path.display().to_string(),
        "--execute".to_string(),
        "--no-confirm".to_string(),
    ];

    let status = ProcessCommand::new("cargo").args(&args).output()?;

    println!("{}", core::str::from_utf8(&status.stdout).unwrap());
    eprintln!("{}", core::str::from_utf8(&status.stderr).unwrap());
    if !status.status.success() {
        return Err(anyhow!("release replace failed"));
    } else {
        Ok(())
    }
}

fn publish_release(_repo: &Path, c: &Crate, push: bool) -> Result<()> {
    let mut args: Vec<String> = vec![
        "publish".to_string(),
        "--manifest-path".to_string(),
        c.path.display().to_string(),
    ];

    if let Some(features) = &c.config.features {
        args.push("--features".into());
        args.push(features.join(","));
    }

    if let Some(target) = &c.config.target {
        args.push("--target".into());
        args.push(target.clone());
    }

    if !push {
        args.push("--dry-run".to_string());
        args.push("--allow-dirty".to_string());
        args.push("--keep-going".to_string());
    }

    let status = ProcessCommand::new("cargo").args(&args).output()?;

    println!("{}", core::str::from_utf8(&status.stdout).unwrap());
    eprintln!("{}", core::str::from_utf8(&status.stderr).unwrap());
    if !status.status.success() {
        return Err(anyhow!("publish failed"));
    } else {
        Ok(())
    }
}
