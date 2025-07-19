#![no_std]
#![no_main]

use core::{
    arch::{asm, naked_asm},
    fmt::Write,
};

use pelu_graphics::kprintln;

use crate::{
    boot::{
        MB2_ARCH_I386, MB2_END_TAG_FLAGS, MB2_END_TAG_SIZE, MB2_END_TAG_TYPE, MB2_HEADER_LEN,
        MB2_MAGIC, Multiboot2Header,
    },
    long_mode_setup::enable_long_mode,
};

mod boot;
mod long_mode_setup;
mod paging;

const MULTIBOOT2_MAGIC_EAX: u32 = 0x36d76289;

const STACK_SIZE: usize = 4096;

/// CPUID leaf for extended cpuid arguments
const CPUID_EXTENDED_ARGS: u32 = 0x80000000;

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

/// Stack area allocated in .bss section for ESP
/// 16KB stack should be sufficient for boot operations
#[unsafe(link_section = ".bss")]
#[unsafe(no_mangle)]
static mut __BOOT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

// Get the top of the stack (end of the array since stacks grow downward)
// This function returns the address that ESP should be set to
fn get_stack_top() -> *mut u8 {
    unsafe {
        let stack_ptr = &raw mut __BOOT_STACK as *mut u8;
        stack_ptr.add(STACK_SIZE - 1)
    }
}

extern "C" fn check_stack() {
    kprintln!("=== Stack Verification ===");
    // Display stack information
    // Verify our stack setup from naked assembly
    let expected_stack_top = get_stack_top();
    let mut current_esp: *mut u8;
    unsafe {
        asm!("mov {0}, esp", out(reg) current_esp);
    }
    kprintln!("Stack size: {} bytes", STACK_SIZE);
    kprintln!("Expected stack top: {:#x}", expected_stack_top as usize);
    kprintln!("Current ESP: {:#x}", current_esp as usize);

    // Check if ESP is within our stack range
    let stack_start = &raw mut __BOOT_STACK as usize;
    let stack_end = stack_start + STACK_SIZE;
    let esp_addr = current_esp as usize;

    if esp_addr >= stack_start && esp_addr <= stack_end {
        kprintln!("Stack setup successful - ESP is within our allocated stack!");
    } else {
        kprintln!("Stack setup failed - ESP is outside our allocated stack!");
    }

    kprintln!("Stack range: {:#x} - {:#x}", stack_start, stack_end);
    kprintln!();
}

#[unsafe(no_mangle)]
pub extern "C" fn entrypoint(eax: u32, ebx: u32) {
    // We need to ensure that the EAX register contains the multiboot2 magic number.
    // Note: this needs to be done before we execute any other code.
    kprintln!("=== Multiboot2 Information ===");
    kprintln!("EAX value: {eax:X}");
    kprintln!("EBX value: {ebx:X}");

    if eax != MULTIBOOT2_MAGIC_EAX {
        panic!("EAX value is not the multiboot2 magic number! Impossible to boot...");
    }
    kprintln!();

    check_stack();

    enable_long_mode();

    loop {}
}

/// This is the official entrypoint of the kernel. It simply sets up the stack and calls the entrypoint method.
/// We need to use naked assembly here because we need to set up the stack pointer (ESP) to our custom stack.
///
/// # Safety
/// This function uses naked assembly to set up the stack pointer (ESP) to our custom stack.
/// It should be only when exciting the multiboot2 loader.
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() {
    naked_asm!(
        "
        // Set up our custom stack first
        lea esp, [__BOOT_STACK + {stack_size}]

        // Align stack to 16 bytes (good practice)
        and esp, 0xFFFFFFF0

        // Push arguments for entrypoint (right to left)
        push ebx
        push eax

        // Now call the main entry point
        call entrypoint

        // If entrypoint returns, halt
        hlt
        ",
        stack_size = const STACK_SIZE,
    )
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    kprintln!("Panic: {_info}");
    loop {}
}
