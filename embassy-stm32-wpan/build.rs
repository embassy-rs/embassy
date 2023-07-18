use std::path::PathBuf;
use std::{env, fs};

fn main() {
    match env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_STM32"))
        .get_one()
    {
        Ok(_) => {}
        Err(GetOneError::None) => panic!("No stm32xx Cargo feature enabled"),
        Err(GetOneError::Multiple) => panic!("Multiple stm32xx Cargo features enabled"),
    }

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // ========
    // stm32wb tl_mbox link sections

    let out_file = out_dir.join("tl_mbox.x").to_string_lossy().to_string();
    fs::write(out_file, fs::read_to_string("tl_mbox.x.in").unwrap()).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=tl_mbox.x.in");
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
