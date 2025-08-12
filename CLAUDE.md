# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PeluditOS is an x86-64 operating system kernel written in Rust. The kernel uses a custom bootloader sequence that transitions from protected mode to long mode (64-bit) using multiboot2 protocol and Limine bootloader.

## Build Commands

### Kernel Building
```bash
# Build the kernel binary for x86_64 target
cargo build --bin peluditOS_x86_64 --target x86_64-unknown-none --release

# Full rebuild and run in QEMU (using Justfile)
just restart-os

# Individual just commands:
just build-long      # Build kernel binary
just rebuild-iso     # Create bootable ISO image
just qemu-run        # Run in QEMU emulator
```

### Testing
```bash
# Run all tests (uses host target with std library available)
cargo test

# Run tests for a specific crate
cargo test -p pelu-alloc

# Run a specific test
cargo test test_allocate_memzone

# Run tests in a specific module
cargo test physical_alloc::
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Run clippy linter
cargo clippy --target x86_64-unknown-none

# For test code (uses host target)
cargo clippy --tests
```

## Architecture Overview

### Boot Process
1. **Multiboot2 Entry** (`crates/long-mode/boot/main.s`): Initial entry point from bootloader
2. **Protected Mode Setup** (`boot/multiboot.s`): Validates multiboot magic, sets up stack
3. **Long Mode Transition** (`boot/compatibility-mode.s`, `boot/paging.s`): 
   - Checks CPUID for long mode support
   - Sets up 4-level page tables (PML4, PDPT, PD, PT)
   - Loads GDT with 64-bit segments
   - Enables PAE and long mode via control registers
4. **Kernel Entry** (`crates/long-mode/src/main.rs:kmain`): Rust kernel main function

### Memory Management Architecture

The allocator system (`crates/allocators`) implements a hierarchical memory management design:

#### Core Allocator Types
- **KMalloc** (`kmalloc.rs`): Bootstrap bump allocator for initial kernel heap setup
- **KBox** (`kbox.rs`): Owned smart pointer for heap-allocated values (kernel equivalent of Box)
- **KArc** (`karc.rs`): Reference-counted smart pointer for shared ownership
- **KStack** (`kstack.rs`): Stack-based collection for kernel use

#### Physical Memory Management
- **Buddy Allocator** (`physical_alloc/buddy.rs`): Manages physical page allocation using buddy system algorithm
- **MemZone** (`physical_alloc/memzone.rs`): Hierarchical memory zones from 4KB to 2MB
  - Uses recursive type system: MemZone4K -> MemZone8K -> ... -> MemZone2M
  - Each zone can be split into two smaller zones or consolidated
- **MemoryMap** (`physical_alloc/memmap.rs`): Tracks available/reserved physical memory regions

#### Testing Strategy
The allocator crate uses conditional compilation for testing:
- Production code: `#![cfg_attr(not(test), no_std)]` - compiles as no_std for kernel
- Test code: When `cargo test` runs, std library is available for using Vec, HashMap, etc. to mock physical memory

## Key Design Patterns

1. **No-std Kernel Code**: All kernel code must work without standard library
2. **Static Lifetime References**: Allocators return `&'static mut T` for kernel-lifetime allocations
3. **Error Handling**: Uses Result types with custom error enums (no panics in production)
4. **Assembly Integration**: Boot code written in AT&T syntax assembly, linked with Rust via build.rs

## Development Notes

- Target triple: `x86_64-unknown-none` (bare metal, no OS)
- Bootloader: Limine with multiboot2 protocol
- Image format: ISO with both BIOS and UEFI support
- Virtual memory: 4-level paging with 2MB pages initially mapped