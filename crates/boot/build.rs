use std::env;
use std::path::PathBuf;

fn main() {
    // Get the current directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Set linker script
    let linker_script = PathBuf::from(&manifest_dir).join("link.ld");

    // Verify the linker script exists
    if !linker_script.exists() {
        panic!("Linker script not found: {}", linker_script.display());
    }

    println!("cargo:rustc-link-arg=--script={}", linker_script.display());

    // Additional linker flags for protected mode
    println!("cargo:rustc-link-arg=--gc-sections");
    println!("cargo:rustc-link-arg=-nostdlib");

    // Tell Cargo to rerun if the linker script changes
    println!("cargo:rerun-if-changed={}", linker_script.display());
}
