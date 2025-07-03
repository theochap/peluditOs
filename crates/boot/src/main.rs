#![no_std]
#![no_main]

use core::{
    arch::{asm, naked_asm},
    fmt::Write,
};

use pelu_graphics::kprintln;

use crate::boot::{
    MB2_ARCH_I386, MB2_END_TAG_FLAGS, MB2_END_TAG_SIZE, MB2_END_TAG_TYPE, MB2_HEADER_LEN,
    MB2_MAGIC, Multiboot2Header,
};

mod boot;

const MULTIBOOT2_MAGIC_EAX: u32 = 0x36d76289;

const STACK_SIZE: usize = 4096;

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

/// Set up the stack pointer (ESP) to use our allocated stack
/// This should be called early in the boot process
unsafe fn setup_stack() {
    let stack_top = get_stack_top(); // Add the size directly

    unsafe {
        asm!(
        "mov esp, {stack_top}",
            stack_top = in(reg) stack_top,
        )
    }
}

#[unsafe(naked)]
/// Check that the CPUID instruction is supported on this CPU.
/// If it is, the function will return 1, otherwise it will return 0.
unsafe extern "C" fn check_cpuid() -> u32 {
    naked_asm!(
        "// Check if CPUID is supported by attempting to flip the ID bit (bit 21)
        // in the FLAGS register. If we can flip it, CPUID is available.

        // Copy FLAGS in to EAX via stack
        pushfd
        pop eax

        // Copy to ECX as well for comparing later on
        mov ecx, eax

        // Flip the ID bit
        xor eax, 1 << 21

        // Copy EAX to FLAGS via the stack
        push eax
        popfd

        // Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
        pushfd
        pop eax

        // Restore FLAGS from the old version stored in ECX (i.e. flipping the
        // ID bit back if it was ever flipped).
        push ecx
        popfd

        // Compare EAX and ECX. If they are equal then that means the bit
        // wasn't flipped, and CPUID isn't supported.
        cmp eax, ecx
        je 2f
        
        // CPUID is supported - return 1 in EAX
        mov eax, 1
        ret
        
        2:
        // CPUID is not supported - return 0 in EAX
        mov eax, 0
        ret",
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn entrypoint() {
    kprintln!("=== Multiboot2 Information ===");
    // We need to ensure that the EAX register contains the multiboot2 magic number.
    let mut eax_value: u32;
    let mut ebx_value: u32;

    unsafe {
        asm!("
        mov {0:e}, eax
        mov {1:e}, ebx
        ", out(reg) eax_value, out(reg) ebx_value);
    }

    kprintln!("EAX value: {eax_value:X}");
    kprintln!("EBX value: {ebx_value:X}");

    if eax_value != MULTIBOOT2_MAGIC_EAX {
        panic!("EAX value is not the multiboot2 magic number! Impossible to boot...");
    }

    kprintln!();
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
    let stack_start = unsafe { core::ptr::addr_of_mut!(__BOOT_STACK) as usize };
    let stack_end = stack_start + STACK_SIZE;
    let esp_addr = current_esp as usize;

    if esp_addr >= stack_start && esp_addr <= stack_end {
        kprintln!("Stack setup successful - ESP is within our allocated stack!");
    } else {
        kprintln!("Stack setup failed - ESP is outside our allocated stack!");
    }
    kprintln!("Stack range: {:#x} - {:#x}", stack_start, stack_end);

    kprintln!();

    kprintln!("=== CPUID Verification ===");
    if unsafe { check_cpuid() } == 0 {
        panic!("CPUID is not supported! Impossible to boot...");
    }

    kprintln!("CPUID is supported!");

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

/// This is the official entrypoint of the kernel. It simply sets up the stack and calls the entrypoint method.
/// We need to use naked assembly here because we need to set up the stack pointer (ESP) to our custom stack.
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() {
    naked_asm!(
        "
        // Set up our custom stack first
        lea esp, [__BOOT_STACK + {stack_size}]

        // Now jump to the main entry point
        jmp entrypoint",
        stack_size = const STACK_SIZE,
    )
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    kprintln!("Panic: {_info}");
    loop {}
}
