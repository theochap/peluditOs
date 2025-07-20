.code32

.equ MULTIBOOT2_MAGIC, 0xE85250D6
.equ MULTIBOOT2_ARCH_I386, 0
.equ MULTIBOOT2_HEADER_LEN, .multiboot2_header_end - multiboot2_header
.equ MULTIBOOT2_CHECKSUM, ((0 - (MULTIBOOT2_MAGIC + MULTIBOOT2_ARCH_I386 + MULTIBOOT2_HEADER_LEN)) & 0xFFFFFFFF)
.equ MULTIBOOT2_END_TAG_TYPE, 0
.equ MULTIBOOT2_END_TAG_FLAGS, 0
.equ MULTIBOOT2_END_TAG_SIZE, 8

.section .multiboot2_header

.align 8
.global multiboot2_header
multiboot2_header:
    .long MULTIBOOT2_MAGIC
    .long MULTIBOOT2_ARCH_I386
    .long MULTIBOOT2_HEADER_LEN
    .long MULTIBOOT2_CHECKSUM
    .end_tag_begin:
    .short MULTIBOOT2_END_TAG_TYPE
    .short MULTIBOOT2_END_TAG_FLAGS
    .long . - .end_tag_begin
.multiboot2_header_end:

.section .text
.fail_multiboot:
    leal multiboot_failed_string, %eax
    call error
    ret

.global check_multiboot
check_multiboot:
    pushl %eax
    leal eax_value, %eax
    call print_string
    movl (%esp), %eax
    call print_hex
    call print_newline

    popl %eax
    cmpl $0x36d76289, %eax
    jne .fail_multiboot
    
    leal multiboot_ok_string, %eax
    call ok

    ret

.section .rodata
eax_value:
    .asciz "EAX: "
multiboot_failed_string:
    .asciz "Multiboot failed. Invalid EAX value"
multiboot_ok_string:
    .asciz "Multiboot EAX value is correct"