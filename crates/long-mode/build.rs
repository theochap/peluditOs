use std::{env, path::PathBuf};

use cc::Build;

fn main() {
    println!("cargo:rerun-if-changed=src/boot/asm/*");
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
        .include("src/boot/asm")
        .files(vec![
            "src/boot/asm/main.s",
            "src/boot/asm/multiboot.s",
            "src/boot/asm/print.s",
            "src/boot/asm/long-mode.s",
            "src/boot/asm/paging.s",
            "src/boot/asm/cpuid.s",
        ])
        .flag("-x")
        .flag("assembler")
        .compile("boot");
}
