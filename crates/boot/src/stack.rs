use core::{arch::asm, fmt::Write};

use pelu_graphics::kprintln;

pub const STACK_SIZE: usize = 4096;

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

pub fn check_stack() {
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
