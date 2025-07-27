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

    let out_dir = env::var("OUT_DIR").unwrap();

    Build::new()
        .include("boot")
        .files(vec![
            "boot/main.s",
            "boot/multiboot.s",
            "boot/print.s",
            "boot/long-mode-entry.s",
            "boot/compatibility-mode.s",
            "boot/paging.s",
            "boot/cpuid.s",
            "boot/gdt.s",
        ])
        .flag("-x")
        .flag("assembler")
        .out_dir(format!("{out_dir}/boot"))
        .compile("boot");
}
