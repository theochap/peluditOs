#![no_std]
#![no_main]

use crate::boot::{
    MB2_ARCH_I386, MB2_END_TAG_FLAGS, MB2_END_TAG_SIZE, MB2_END_TAG_TYPE, MB2_HEADER_LEN,
    MB2_MAGIC, Multiboot2Header,
};

mod boot;

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

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, byte) in "HELLO".bytes().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 3) = byte;
            *vga_buffer.offset(i as isize * 3 + 1) = 0xb;
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
