use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rustc-link-arg-bins=-Tmemory.x");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");

    Ok(())
}
