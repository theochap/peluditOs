.code32

.section .text
.global enable_long_mode
enable_long_mode:
    // load P4 to cr3 register (cpu uses this to access the P4 table)
    leal p4_table, %eax
    movl %eax, %cr3

    // enable PAE-flag in cr4 (Physical Address Extension)
    movl %cr4, %eax
    orl $(1 << 5), %eax
    movl %eax, %cr4

    // set the long mode bit in the EFER MSR (model specific register)
    movl $0xC0000080, %ecx
    rdmsr
    orl $(1 << 8), %eax
    wrmsr

    // enable paging in the cr0 register
    movl %cr0, %eax
    orl $(1 << 31), %eax
    movl %eax, %cr0

    leal long_mode_ok_string, %eax
    call ok

    ret

.section .rodata
enabling_long_mode_string:
    .asciz "Enabling long mode..."
long_mode_ok_string:
    .asciz "Entered compatibility mode!"