//! src/boot/multiboot_header.rs
//! A 24-byte Multiboot 2 header (magic + arch + len + checksum + END tag)

use core::fmt::Write;
use pelu_graphics::kprintln;

const MULTIBOOT2_MAGIC_EAX: u32 = 0x36d76289;
pub(crate) const MB2_MAGIC: u32 = 0xE85250D6;

/// 0 = i386 architecture.  Multiboot2 doesn’t define a separate x86-64 ID;
/// GRUB happily accepts `0` for 64-bit kernels.
pub(crate) const MB2_ARCH_I386: u32 = 0;

pub(crate) const MB2_HEADER_LEN: u32 = size_of::<Multiboot2Header>() as u32;

/// Multiboot 2 END tag (type 0, flags 0, size 8)
pub(crate) const MB2_END_TAG_TYPE: u16 = 0;
pub(crate) const MB2_END_TAG_FLAGS: u16 = 0;
pub(crate) const MB2_END_TAG_SIZE: u32 = 8;

/// The header *must* be 8-byte aligned and contained in the first 32 768
/// bytes of the file.  All modern linkers put `.multiboot2_header` very
/// early, but you can enforce its address with a linker script if needed.
#[repr(C, align(8))]
pub struct Multiboot2Header {
    /* fixed part */
    pub(crate) magic: u32,
    pub(crate) architecture: u32,
    pub(crate) header_length: u32,
    pub(crate) checksum: u32,
    /* first (and only) tag: END */
    pub(crate) end_tag_type: u16,
    pub(crate) end_tag_flags: u16,
    pub(crate) end_tag_size: u32,
}

/// A single statically-allocated instance dropped into the special section.
///
/// `#[used]` stops the optimiser from discarding it,  
/// `#[no_mangle]` keeps the symbol name stable (handy when debugging),  
/// `#[link_section]` tells the linker where to put it.
#[used]
#[unsafe(link_section = ".multiboot2_header")]
#[unsafe(no_mangle)]
pub static MULTIBOOT2_HEADER: Multiboot2Header = Multiboot2Header {
    magic: MB2_MAGIC,
    architecture: MB2_ARCH_I386,
    header_length: size_of::<Multiboot2Header>() as u32,
    checksum: ((0u64
        .wrapping_sub((MB2_MAGIC as u64) + (MB2_ARCH_I386 as u64) + (MB2_HEADER_LEN as u64)))
        & 0xFFFF_FFFF) as u32,
    end_tag_type: MB2_END_TAG_TYPE,
    end_tag_flags: MB2_END_TAG_FLAGS,
    end_tag_size: MB2_END_TAG_SIZE,
};

pub fn check_multiboot2_header(eax: u32, ebx: u32) {
    // We need to ensure that the EAX register contains the multiboot2 magic number.
    // Note: this needs to be done before we execute any other code.
    kprintln!("=== Multiboot2 Information ===");
    kprintln!("EAX value: {eax:X}");
    kprintln!("EBX value: {ebx:X}");

    if eax != MULTIBOOT2_MAGIC_EAX {
        panic!("EAX value is not the multiboot2 magic number! Impossible to boot...");
    }
    kprintln!();
}
