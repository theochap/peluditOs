//! src/boot/multiboot_header.rs
//! A 24-byte Multiboot 2 header (magic + arch + len + checksum + END tag)

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
