use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ParsedCrate {
    pub package: ParsedPackage,
    pub dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ParsedPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize, Default)]
pub struct Metadata {
    #[serde(default)]
    pub embassy: MetadataEmbassy,
}

#[derive(Debug, Deserialize, Default)]
pub struct MetadataEmbassy {
    #[serde(default)]
    pub skip: bool,
    #[serde(default)]
    pub build: Vec<BuildConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub features: Vec<String>,
    pub target: Option<String>,
}

pub type CrateId = String;

#[derive(Debug, Clone)]
pub struct Crate {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: Vec<CrateId>,
    pub config: BuildConfig, // TODO make this a vec.
}
