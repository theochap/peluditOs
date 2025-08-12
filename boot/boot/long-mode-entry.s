# 64-bit code section
.code64
.section .text
long_mode_entry:
    # Clear segment registers for 64-bit mode
    mov $0, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss

    # Jump to kmain function
    # The linker will resolve this to the correct higher-half address
    call kmain

    # If kmain returns (it shouldn't), halt
    hlt