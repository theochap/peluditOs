.code32

# Include external files
.include "multiboot.s"
.include "print.s"
.include "cpuid.s"
.include "paging.s"
.include "compatibility-mode.s"
.include "gdt.s"
.include "long-mode-entry.s"


.section .text
.global _start

_start:
    # Set up our custom stack first
    lea __BOOT_STACK, %esp 
    addl $0x1000, %esp

    call check_multiboot

    call check_cpuid

    call setup_identity_paging

    call setup_paging_for_kernel

    call enable_compatibility_mode

    call setup_gdt

    call jump_to_long_mode

    hlt

// Far jump to long mode. Done through a far pointer in memory.
jump_to_long_mode:
    ljmp *long_mode_entry_ptr
    ret

# Far pointer for the jump to 64-bit mode
.section .data
.align 8
long_mode_entry_ptr:
    .long long_mode_entry  # offset
    .word (gdt_code - gdt_begin)            # segment selector

.section .bss
.align 16
__BOOT_STACK:
    .space 0x1000

