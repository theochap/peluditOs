use std::{env, path::PathBuf};

use cc::Build;

fn main() {
    println!("cargo:rerun-if-changed=src/boot/*");
    println!("cargo:rerun-if-changed=link.ld");

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

    Build::new()
        .include("src/boot")
        .files(vec![
            "src/boot/main.s",
            "src/boot/multiboot.s",
            "src/boot/print.s",
            "src/boot/long-mode.s",
            "src/boot/paging.s",
            "src/boot/cpuid.s",
            "src/boot/gdt.s",
        ])
        .flag("-x")
        .flag("assembler")
        .compile("boot");
}
