use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use log::info;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Bfs;
use petgraph::{Directed, Direction};
use simple_logger::SimpleLogger;
use toml_edit::{DocumentMut, Item, Value};
use types::*;

mod build;
mod cargo;
mod semver_check;
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

    /// Build
    Build {
        /// Crate to check. If not specified checks all crates.
        #[arg(value_name = "CRATE")]
        crate_name: Option<String>,
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

fn update_version(c: &mut Crate, new_version: &str) -> Result<()> {
    let path = c.path.join("Cargo.toml");
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
    let path = to_update.path.join("Cargo.toml");
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

fn list_crates(root: &PathBuf) -> Result<BTreeMap<CrateId, Crate>> {
    let d = std::fs::read_dir(root)?;
    let mut crates = BTreeMap::new();
    for c in d {
        let entry = c?;
        if entry.file_type()?.is_dir() {
            let path = root.join(entry.path());
            let entry = path.join("Cargo.toml");
            if entry.exists() {
                let content = fs::read_to_string(&entry)?;
                let parsed: ParsedCrate = toml::from_str(&content)?;
                let id = parsed.package.name;

                let metadata = &parsed.package.metadata.embassy;

                let mut dependencies = Vec::new();
                for (k, _) in parsed.dependencies {
                    if k.starts_with("embassy-") {
                        dependencies.push(k);
                    }
                }

                let mut configs = metadata.build.clone();
                if configs.is_empty() {
                    configs.push(BuildConfig {
                        features: vec![],
                        target: None,
                    })
                }

                crates.insert(
                    id.clone(),
                    Crate {
                        name: id,
                        version: parsed.package.version,
                        path,
                        dependencies,
                        configs,
                    },
                );
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

struct Context {
    root: PathBuf,
    crates: BTreeMap<String, Crate>,
    graph: Graph<String, ()>,
    indices: HashMap<String, NodeIndex>,
}

fn load_context(args: &Args) -> Result<Context> {
    let root = args.repo.canonicalize()?;
    let crates = list_crates(&root)?;
    let (graph, indices) = build_graph(&crates);

    Ok(Context {
        root,
        crates,
        graph,
        indices,
    })
}

fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();
    let args = Args::parse();
    let mut ctx = load_context(&args)?;

    match args.command {
        Command::List => {
            let ordered = petgraph::algo::toposort(&ctx.graph, None).unwrap();
            for node in ordered.iter() {
                let start = ctx.graph.node_weight(*node).unwrap();
                let mut bfs = Bfs::new(&ctx.graph, *node);
                while let Some(node) = bfs.next(&ctx.graph) {
                    let weight = ctx.graph.node_weight(node).unwrap();
                    let c = ctx.crates.get(weight).unwrap();
                    if weight == start {
                        println!("+ {}-{}", weight, c.version);
                    } else {
                        println!("|- {}-{}", weight, c.version);
                    }
                }
                println!("");
            }
        }
        Command::Dependencies { crate_name } => {
            let idx = ctx.indices.get(&crate_name).expect("unable to find crate in tree");
            let mut bfs = Bfs::new(&ctx.graph, *idx);
            while let Some(node) = bfs.next(&ctx.graph) {
                let weight = ctx.graph.node_weight(node).unwrap();
                let crt = ctx.crates.get(weight).unwrap();
                if *weight == crate_name {
                    println!("+ {}-{}", weight, crt.version);
                } else {
                    println!("|- {}-{}", weight, crt.version);
                }
            }
        }
        Command::Dependents { crate_name } => {
            let idx = ctx.indices.get(&crate_name).expect("unable to find crate in tree");
            let weight = ctx.graph.node_weight(*idx).unwrap();
            let crt = ctx.crates.get(weight).unwrap();
            println!("+ {}-{}", weight, crt.version);
            for parent in ctx.graph.neighbors_directed(*idx, Direction::Incoming) {
                let weight = ctx.graph.node_weight(parent).unwrap();
                let crt = ctx.crates.get(weight).unwrap();
                println!("|- {}-{}", weight, crt.version);
            }
        }
        Command::Build { crate_name } => {
            build::build(&ctx)?;
        }
        Command::SemverCheck { crate_name } => {
            let c = ctx.crates.get(&crate_name).unwrap();
            check_semver(&c)?;
        }
        Command::PrepareRelease { crate_name } => {
            let start = ctx.indices.get(&crate_name).expect("unable to find crate in tree");
            let mut rgraph = ctx.graph.clone();
            rgraph.reverse();

            let mut bfs = Bfs::new(&rgraph, *start);

            while let Some(node) = bfs.next(&rgraph) {
                let weight = rgraph.node_weight(node).unwrap();
                println!("Preparing {}", weight);
                let mut c = ctx.crates.get_mut(weight).unwrap();
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
                let c = ctx.crates.get(weight).unwrap();

                // Update all nodes further down the tree
                let mut bfs = Bfs::new(&rgraph, node);
                while let Some(dep_node) = bfs.next(&rgraph) {
                    let dep_weight = rgraph.node_weight(dep_node).unwrap();
                    let dep = ctx.crates.get(dep_weight).unwrap();
                    update_versions(dep, &c.name, &newver)?;
                }

                // Update changelog
                update_changelog(&ctx.root, &c)?;
            }

            let weight = rgraph.node_weight(*start).unwrap();
            let c = ctx.crates.get(weight).unwrap();
            publish_release(&ctx.root, &c, false)?;

            println!("# Please inspect changes and run the following commands when happy:");

            println!("git commit -a -m 'chore: prepare crate releases'");
            let mut bfs = Bfs::new(&rgraph, *start);
            while let Some(node) = bfs.next(&rgraph) {
                let weight = rgraph.node_weight(node).unwrap();
                let c = ctx.crates.get(weight).unwrap();
                println!("git tag {}-v{}", weight, c.version);
            }

            println!("");
            println!("# Run these commands to publish the crate and dependents:");

            let mut bfs = Bfs::new(&rgraph, *start);
            while let Some(node) = bfs.next(&rgraph) {
                let weight = rgraph.node_weight(node).unwrap();
                let c = ctx.crates.get(weight).unwrap();

                let mut args: Vec<String> = vec![
                    "publish".to_string(),
                    "--manifest-path".to_string(),
                    c.path.join("Cargo.toml").display().to_string(),
                ];

                let config = c.configs.first().unwrap(); // TODO
                args.push("--features".into());
                args.push(config.features.join(","));

                if let Some(target) = &config.target {
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
    let min_version = semver_check::minimum_update(c)?;
    println!("Version should be bumped to {:?}", min_version);
    Ok(())
}

fn update_changelog(repo: &Path, c: &Crate) -> Result<()> {
    let args: Vec<String> = vec![
        "release".to_string(),
        "replace".to_string(),
        "--config".to_string(),
        repo.join("release").join("release.toml").display().to_string(),
        "--manifest-path".to_string(),
        c.path.join("Cargo.toml").display().to_string(),
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
    let config = c.configs.first().unwrap(); // TODO

    let mut args: Vec<String> = vec![
        "publish".to_string(),
        "--manifest-path".to_string(),
        c.path.join("Cargo.toml").display().to_string(),
    ];

    args.push("--features".into());
    args.push(config.features.join(","));

    if let Some(target) = &config.target {
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

/// Make the path "Windows"-safe
pub fn windows_safe_path(path: &Path) -> PathBuf {
    PathBuf::from(path.to_str().unwrap().to_string().replace("\\\\?\\", ""))
}
