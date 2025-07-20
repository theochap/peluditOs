.code32

# Include external files
.include "multiboot.s"
.include "print.s"
.include "cpuid.s"
.include "paging.s"
.include "long-mode.s"

.section .text
.global _start

_start:
    # Set up our custom stack first
    lea __BOOT_STACK, %esp 
    addl $0x1000, %esp

    call check_multiboot

    call check_cpuid

    call setup_identity_paging

    call enable_long_mode

    hlt

.section .bss
.align 16
__BOOT_STACK:
    .space 0x1000
