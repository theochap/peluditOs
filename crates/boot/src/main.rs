#![no_std]
#![no_main]

use core::{arch::asm, fmt::Write};

use pelu_graphics::kprintln;

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

const MULTIBOOT2_MAGIC_EAX: u32 = 0x36d76289;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    // We need to ensure that the EAX register contains the multiboot2 magic number.
    // Get the value that was in EAX when we were called
    let mut eax_value: u32;

    unsafe {
        asm!("mov {0:e}, eax", out(reg) eax_value);
    }

    kprintln!("EAX value: {eax_value:X}");

    if eax_value != MULTIBOOT2_MAGIC_EAX {
        panic!("EAX value is not the multiboot2 magic number! Impossible to boot...");
    }

    kprintln!(
        "Hello, world! Pelu is booting... She is complicated sometimes. New line is working. \n"
    );

    kprintln!(
        "Writing to a new line. Non ascii characters are replaced by a white box. {} \n",
        '\x12'
    );

    kprintln!("\t Tab is working.");

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    kprintln!("Panic: {_info}");
    loop {}
}
