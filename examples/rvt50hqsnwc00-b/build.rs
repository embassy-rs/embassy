use std::env;
use std::path::Path;

fn main() {
    // Tell cargo to look for memory.x in the parent directory
    let out_dir = env::var_os("OUT_DIR").unwrap();
    
    // Link memory.x from embassy-stm32
    println!("cargo:rustc-link-search={}", 
        Path::new("../../embassy-stm32").display()
    );
    
    // Re-run if memory.x changes
    println!("cargo:rerun-if-changed=../../embassy-stm32/memory.x");
    
    // Set the link script
    println!("cargo:rustc-link-arg=-T../../embassy-stm32/memory.x");
    
    // Also include the default link.x if it exists
    if Path::new("memory.x").exists() {
        println!("cargo:rustc-link-arg=-Tmemory.x");
        println!("cargo:rerun-if-changed=memory.x");
    }
}
