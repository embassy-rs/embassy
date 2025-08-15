use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ParsedCrate {
    pub package: ParsedPackage,
    pub dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ParsedPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateConfig {
    pub features: Option<Vec<String>>,
    pub target: Option<String>,
}

pub type ReleaseConfig = HashMap<String, CrateConfig>;
pub type CrateId = String;

#[derive(Debug, Clone)]
pub struct Crate {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub config: CrateConfig,
    pub dependencies: Vec<CrateId>,
}
