#![no_std]
#![no_main]

use core::{arch::naked_asm, fmt::Write};

use pelu_graphics::kprintln;

use crate::{
    boot::check_multiboot2_header,
    compat_mode::enable_long_mode,
    stack::{STACK_SIZE, check_stack},
};

mod boot;
mod compat_mode;
mod gdt;
mod paging;
mod stack;

#[unsafe(no_mangle)]
pub extern "C" fn entrypoint(eax: u32, ebx: u32) {
    check_multiboot2_header(eax, ebx);

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
