use core::{arch::asm, fmt::Write};

use pelu_graphics::kprintln;

use crate::{
    CPUID_EXTENDED_ARGS,
    paging::{IDENT_PAGING, setup_identity_paging},
};

#[repr(u8)]
enum LongModeSupportErr {
    NoCpuid = 0,
    ExtendedMode = 1,
    LongMode = 2,
}

struct Regs {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

fn cpuid(leaf: u32, sub_leaf: u32) -> Regs {
    let mut regs = Regs {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
    };

    unsafe {
        asm!(
            "mov {0:e}, ebx",
            "cpuid",
            "xchg {0:e}, ebx",
            out(reg) regs.ebx,
            inout("eax") leaf => regs.eax,
            inout("ecx") sub_leaf => regs.ecx,
            out("edx") regs.edx,
            options(nostack, preserves_flags),
        );
    }

    regs
}

/// Check that the CPUID instruction is supported on this CPU.
/// If it is, the function will return 1, otherwise it will return 0.
///
/// Courtesy of `<https://wiki.osdev.org/CPUID>`
fn support_cpuid() -> Result<(), LongModeSupportErr> {
    let mut out: u32;
    // Check if CPUID is supported by attempting to flip the ID bit (bit 21)
    unsafe {
        asm!("
        pushfd                               // Save EFLAGS
        pushfd                               // Store EFLAGS
        xor dword ptr [esp],0x00200000           // Invert the ID bit in stored EFLAGS
        popfd                                // Load stored EFLAGS (with ID bit inverted)
        pushfd                               // Store EFLAGS again (ID bit may or may not be inverted)
        pop {0:e}                            // reg = modified EFLAGS (ID bit may or may not be inverted)
        xor {0:e},[esp]                      // reg = whichever bits were changed
        popfd                                // Restore original EFLAGS
        and {0:e},0x00200000                 // reg = zero if ID bit can't be changed, else non-zero
",
            out(reg) out,
        )
    }

    if out == 0 {
        return Err(LongModeSupportErr::NoCpuid);
    }

    let regs = cpuid(CPUID_EXTENDED_ARGS, 0);
    if regs.eax < CPUID_EXTENDED_ARGS + 1 {
        return Err(LongModeSupportErr::ExtendedMode);
    }

    let regs = cpuid(CPUID_EXTENDED_ARGS + 1, 0);

    if regs.edx & (1 << 29) == 0 {
        return Err(LongModeSupportErr::LongMode);
    }

    Ok(())
}

pub fn enter_long_mode() {
    kprintln!("=== Long Mode Setup ===");
    kprintln!("Entering long mode...");
    kprintln!();

    // We need to update the CPU registers to enable long mode.
    unsafe {
        asm!(
            "
            // save eax and ecx registers
            push eax
            push ecx

            // load P4 to cr3 register (cpu uses this to access the P4 table)
            mov cr3, {p4_table}


            // enable PAE-flag in cr4 (Physical Address Extension)
            mov eax, cr4
            or eax, 1 << 5
            mov cr4, eax

            // set the long mode bit in the EFER MSR (model specific register)
            mov ecx, 0xC0000080
            rdmsr
            or eax, 1 << 8
            wrmsr

            // enable paging in the cr0 register
            mov eax, cr0
            or eax, 1 << 31
            mov cr0, eax

            // restore eax and ecx registers
            pop ecx
            pop eax",

            p4_table = in(reg) &raw const IDENT_PAGING.level_4 as *mut u8,
        )
    }

    kprintln!("Long mode setup successful!");
    kprintln!();
}

/// Sets all the necessary CPU registers to enable long mode.
pub fn enable_long_mode() {
    kprintln!("=== CPUID Verification ===");
    match support_cpuid() {
        Ok(_) => kprintln!("CPUID long mode is supported!"),
        Err(LongModeSupportErr::NoCpuid) => {
            panic!("CPUID is not supported! Impossible to boot...");
        }
        Err(LongModeSupportErr::ExtendedMode) => {
            panic!("CPUID extended args are not supported! Impossible to boot...");
        }
        Err(LongModeSupportErr::LongMode) => {
            panic!("Long mode is not supported! Impossible to boot...");
        }
    }
    kprintln!();

    // Now that we know we can use long mode, setup identity paging.
    setup_identity_paging();

    // Now that we have identity paging, we can enter long mode.
    enter_long_mode();
}
