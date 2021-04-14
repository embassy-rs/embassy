use darling::{FromMeta, Result};
use proc_macro2::Span;
use syn::{LitStr, Path};

#[derive(Debug)]
pub struct ModulePrefix {
    literal: LitStr,
}

impl ModulePrefix {
    pub fn new(path: &str) -> Self {
        let literal = LitStr::new(path, Span::call_site());
        Self { literal }
    }

    pub fn append(&self, component: &str) -> ModulePrefix {
        let mut lit = self.literal().value();
        lit.push_str(component);
        Self::new(lit.as_str())
    }

    pub fn path(&self) -> Path {
        self.literal.parse().unwrap()
    }

    pub fn literal(&self) -> &LitStr {
        &self.literal
    }
}

impl FromMeta for ModulePrefix {
    fn from_string(value: &str) -> Result<Self> {
        Ok(ModulePrefix::new(value))
    }
}

impl Default for ModulePrefix {
    fn default() -> ModulePrefix {
        ModulePrefix::new("::")
    }
}
