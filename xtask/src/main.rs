#![allow(dead_code)]
#![deny(unused_must_use)]

use std::format;
use std::{env, fs, path::PathBuf};

use std::path::Path;
use walkdir::WalkDir;
use xshell::{cmd, Cmd};
use yaml_rust::YamlLoader;

extern crate yaml_rust;

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    match &args[..] {
        ["ci"] => task_ci()?,
        ["core"] => task_check(Realm::Core)?,
        ["metapac"] => task_metapac_gen()?,
        ["examples"] => task_check(Realm::Examples)?,
        ["fmt-check"] => task_cargo_fmt_check()?,
        ["fmt"] => task_cargo_fmt()?,
        _ => {
            println!("");
            println!("USAGE: cargo xtask [command]");
            println!("");
            println!("Commands:");
            println!("  ci        :: Runs entire CI");
            println!("  core      :: Builds the core");
            println!("  metapac   :: Builds the metapac");
            println!("  examples  :: Builds the examples");
            println!("  fmt-check :: Checks rustfmt");
            println!("  fmt       :: Performs rustfmt");
            println!("");
        }
    }
    Ok(())
}

fn task_ci() -> Result<(), anyhow::Error> {
    task_check(Realm::Core)?;
    task_check(Realm::Examples)?;
    task_metapac_gen()?;
    task_cargo_fmt_check()?;
    Ok(())
}

#[derive(Copy, Clone)]
enum Realm {
    All,
    Core,
    Examples,
}

impl Realm {
    fn accepts(&self, package: &str) -> bool {
        match self {
            Realm::All => true,
            Realm::Core => !package.contains("examples"),
            Realm::Examples => package.contains("examples"),
        }
    }
}

fn task_check(realm: Realm) -> Result<(), anyhow::Error> {
    let _e = xshell::pushenv("CI", "true");

    let matrix_yaml = root_dir()
        .join(".github")
        .join("workflows")
        .join("rust.yml");

    let matrix = YamlLoader::load_from_str(&*fs::read_to_string(matrix_yaml).unwrap()).unwrap();

    let matrix = &matrix.get(0).unwrap()["jobs"]["ci"]["strategy"]["matrix"]["include"];

    let entries = matrix.as_vec().unwrap();

    for entry in entries {
        let package = entry["package"].as_str().unwrap();
        if !realm.accepts(package) {
            continue;
        }
        let target = entry["target"].as_str().unwrap();
        let features = entry["features"].as_str();
        let package_dir = root_dir().join(entry["package"].as_str().unwrap());
        let _p = xshell::pushd(package_dir)?;
        banner(&*format!(
            "Building {} [target={}] [features={}]",
            package,
            target,
            features.unwrap_or("default-features")
        ));

        let root_cargo_dir = root_dir().join(".cargo");
        fs::create_dir_all(root_cargo_dir.clone()).unwrap();
        fs::write(
            root_cargo_dir.join("config"),
            "[target.\"cfg(all())\"]\nrustflags = [\"-D\", \"warnings\"]",
        )
        .unwrap();

        let mut args = Vec::new();
        args.push("check");
        args.push("--target");
        args.push(target);

        if let Some(features) = features {
            args.push("--features");
            args.push(features);
        }

        let command = Cmd::new(PathBuf::from("cargo"));
        let command = command.args(args);
        let result = command.run();

        fs::remove_file(root_cargo_dir.join("config")).unwrap();

        result?;
    }

    Ok(())
}

fn task_metapac_gen() -> Result<(), anyhow::Error> {
    banner("Building metapac");
    let _p = xshell::pushd(root_dir().join("stm32-metapac-gen"));
    cmd!("cargo run").run()?;
    Ok(())
}

fn task_cargo_fmt() -> Result<(), anyhow::Error> {
    for entry in WalkDir::new(root_dir())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(".rs") {
            if !is_primary_source(entry.path()) {
                continue;
            }
            let mut args = Vec::new();
            args.push("--skip-children");
            args.push("--unstable-features");
            args.push("--edition=2018");
            args.push(&*entry.path().to_str().unwrap());
            let command = Cmd::new("rustfmt");
            command.args(args).run()?;
        }
    }

    Ok(())
}

fn task_cargo_fmt_check() -> Result<(), anyhow::Error> {
    let mut actual_result = Ok(());
    for entry in WalkDir::new(root_dir())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(".rs") {
            if !is_primary_source(entry.path()) {
                continue;
            }
            let mut args = Vec::new();
            args.push("--check");
            args.push("--skip-children");
            args.push("--unstable-features");
            args.push("--edition=2018");
            args.push(&*entry.path().to_str().unwrap());
            let command = Cmd::new("rustfmt");
            if let Err(result) = command.args(args).run() {
                actual_result = Err(result.into());
            }
        }
    }

    actual_result
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}

fn examples_dir() -> PathBuf {
    root_dir().join("examples")
}

fn is_primary_source(path: &Path) -> bool {
    let mut current = path;

    loop {
        let current_file_name = current.file_name().unwrap().to_str().unwrap();
        if current_file_name == "target"
            || current_file_name == "stm32-metapac-gen"
            || current_file_name == "stm32-data"
        {
            return false;
        }

        if let Some(path) = current.parent() {
            current = path.into();
            if current == root_dir() {
                return true;
            }
        } else {
            return false;
        }
    }
}

fn banner(text: &str) {
    println!("------------------------------------------------------------------------------------------------------------------------------------------------");
    println!("== {}", text);
    println!("------------------------------------------------------------------------------------------------------------------------------------------------");
}
