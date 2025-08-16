use std::collections::BTreeMap;
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
    #[serde(default = "default_publish")]
    pub publish: bool,
    #[serde(default)]
    pub metadata: Metadata,
}

fn default_publish() -> bool {
    true
}

#[derive(Debug, Deserialize, Default)]
pub struct Metadata {
    #[serde(default)]
    pub embassy: MetadataEmbassy,
}

#[allow(dead_code)]
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
    #[serde(rename = "artifact-dir")]
    pub artifact_dir: Option<String>,
}

pub type CrateId = String;

#[derive(Debug, Clone)]
pub struct Crate {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: Vec<CrateId>,
    pub configs: Vec<BuildConfig>,
    pub publish: bool,
}
