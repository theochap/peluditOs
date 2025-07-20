.equ EXECUTABLE, (1 << 43)
.equ DESCRIPTOR_TABLE, (1 << 44)
.equ PRESENT, (1 << 47)
.equ LONG_MODE, (1 << 53)

.section .text
.global setup_gdt
setup_gdt:
    lea gdt_ptr, %eax
    lgdt (%eax)

    leal gdt_loaded_string, %eax
    call ok

    ret

.section .data
.global gdt_begin
gdt_begin:
    .quad 0
    .quad EXECUTABLE | DESCRIPTOR_TABLE | PRESENT | LONG_MODE
gdt_ptr:
    .word . - gdt_begin - 1
    .quad gdt_begin

.section .rodata
gdt_loaded_string:
    .asciz "GDT loaded!"