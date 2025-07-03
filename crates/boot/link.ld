OUTPUT_FORMAT(elf32-i386)
ENTRY(_start)

SECTIONS
{
    /* keep the file above 1 MiB so real-mode BIOS data is untouched */
    . = 1M;

    /* Multiboot header **must** be < 64 KiB from file start */
    .multiboot2_header : { KEEP(*(.multiboot2_header)) }

    .text  : { *(.text*) }
    
    .data  : { *(.data*) }
    .rodata  : { *(.rodata*) }
    
    .bss   : { *(.bss*)  *(COMMON) }
}
