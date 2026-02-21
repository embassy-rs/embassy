use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let chip_name = match env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_MCXA"))
        .get_one()
    {
        Ok(x) => x,
        Err(GetOneError::None) => panic!("No mcxaxxx Cargo feature enabled"),
        Err(GetOneError::Multiple) => panic!("Multiple mcxaxxx Cargo features enabled"),
    }
    .strip_prefix("CARGO_FEATURE_")
    .unwrap()
    .to_ascii_lowercase();

    eprintln!("chip: {chip_name}");

    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(feature = "memory-x") {
        let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
        gen_memory_x(out_dir);
        println!("cargo:rustc-link-search={}", out_dir.display());
    }
}

fn gen_memory_x(out_dir: &Path) {
    let mut memory_x = String::new();

    let flash = if cfg!(feature = "_flash-512k") {
        512
    } else /* if cfg!(feature = "_flash-1024k") */ {
        1024
    };

    let ram = if cfg!(feature = "_sram-128k") {
        128
    } else /* if cfg!(feature = "_sram-256k") */ {
        256
    };

    write!(memory_x, "MEMORY\n{{\n").unwrap();
    writeln!(memory_x, "    FLASH : ORIGIN = 0x00000000, LENGTH = {:>4}K", flash,).unwrap();
    writeln!(memory_x, "    RAM   : ORIGIN = 0x20000000, LENGTH = {:>4}K", ram).unwrap();
    write!(memory_x, "}}").unwrap();

    std::fs::write(out_dir.join("memory.x"), memory_x.as_bytes()).unwrap();
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
