use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use cfg_aliases::cfg_aliases;
#[cfg(feature = "_rt1xxx")]
use nxp_pac::metadata;
#[allow(unused)]
use proc_macro2::TokenStream;
#[allow(unused)]
use quote::quote;

#[path = "./build_common.rs"]
mod common;

fn main() {
    let mut cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut cfgs);

    let chip_name = match env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_MIMXRT") || x.starts_with("CARGO_FEATURE_LPC"))
        .get_one()
    {
        Ok(x) => x,
        Err(GetOneError::None) => panic!("No mimxrt/lpc Cargo feature enabled"),
        Err(GetOneError::Multiple) => panic!("Multiple mimxrt/lpc Cargo features enabled"),
    }
    .strip_prefix("CARGO_FEATURE_")
    .unwrap()
    .to_ascii_lowercase();

    cfg_aliases! {
        rt1xxx: { any(feature = "mimxrt1011", feature = "mimxrt1062") },
        gpio1: { any(feature = "mimxrt1011", feature = "mimxrt1062") },
        gpio2: { any(feature = "mimxrt1011", feature = "mimxrt1062") },
        gpio3: { feature = "mimxrt1062" },
        gpio4: { feature = "mimxrt1062" },
        gpio5: { any(feature = "mimxrt1011", feature = "mimxrt1062") },
    }

    eprintln!("chip: {chip_name}");

    generate_code();
}

#[cfg(feature = "_rt1xxx")]
fn generate_iomuxc() -> TokenStream {
    use proc_macro2::{Ident, Span};

    let pads = metadata::iomuxc::IOMUXC_REGISTERS.iter().map(|registers| {
        let name = Ident::new(&registers.name, Span::call_site());
        let address = registers.pad_ctl;

        quote! {
            pub const #name: u32 = #address;
        }
    });

    let muxes = metadata::iomuxc::IOMUXC_REGISTERS.iter().map(|registers| {
        let name = Ident::new(&registers.name, Span::call_site());
        let address = registers.mux_ctl;

        quote! {
            pub const #name: u32 = #address;
        }
    });

    quote! {
        pub mod iomuxc {
            pub mod pads {
                #(#pads)*
            }

            pub mod muxes {
                #(#muxes)*
            }
        }
    }
}

fn generate_code() {
    #[allow(unused)]
    use std::fmt::Write;

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    #[allow(unused_mut)]
    let mut output = String::new();

    #[cfg(feature = "_rt1xxx")]
    writeln!(&mut output, "{}", generate_iomuxc()).unwrap();

    let out_file = out_dir.join("_generated.rs").to_string_lossy().to_string();
    fs::write(&out_file, output).unwrap();
    rustfmt(&out_file);
}

/// rustfmt a given path.
/// Failures are logged to stderr and ignored.
fn rustfmt(path: impl AsRef<Path>) {
    let path = path.as_ref();
    match Command::new("rustfmt").args([path]).output() {
        Err(e) => {
            eprintln!("failed to exec rustfmt {:?}: {:?}", path, e);
        }
        Ok(out) => {
            if !out.status.success() {
                eprintln!("rustfmt {:?} failed:", path);
                eprintln!("=== STDOUT:");
                std::io::stderr().write_all(&out.stdout).unwrap();
                eprintln!("=== STDERR:");
                std::io::stderr().write_all(&out.stderr).unwrap();
            }
        }
    }
}

enum GetOneError {
    None,
    Multiple,
}

trait IteratorExt: Iterator {
    fn get_one(self) -> Result<Self::Item, GetOneError>;
}

impl<T: Iterator> IteratorExt for T {
    fn get_one(mut self) -> Result<Self::Item, GetOneError> {
        match self.next() {
            None => Err(GetOneError::None),
            Some(res) => match self.next() {
                Some(_) => Err(GetOneError::Multiple),
                None => Ok(res),
            },
        }
    }
}
