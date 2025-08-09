OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

KERNEL_LMA = 2M;
KERNEL_VMA = 0xffffffff80000000;      /* higher‑half base          */
PAGE_SIZE         = 0x1000;                  /* 4 KiB                     */

SECTIONS
{
    /* keep the file above 2 MiB so real-mode BIOS data is untouched */
    . = KERNEL_LMA;

    /* Multiboot header **must** be < 64 KiB from file start */
    .multiboot2_header : { KEEP(*(.multiboot2_header)) }

    /* ----------------------------------------------------------------- */
    /*  1. Early bootstrap (assembly only, identity mapped)              */
    /* ----------------------------------------------------------------- */
    .text.boot : {
        /* Include all sections from the boot library */
        /boot/*.o *(.text*)
    }

    .rodata.boot : {
       /* Include all sections from the boot library */
       /boot/*.o *(.rodata*) 
    }

    .data.boot : {
        /* Include all sections from the boot library */
       /boot/*.o *(.data*) 
    }

    .bss.boot : {
        /* Include all sections from the boot library */
       /boot/*.o *(.bss*) 
    }

    /* ----------------------------------------------------------------- */
    /*  2. Higher‑half kernel text                                        */
    /* ----------------------------------------------------------------- */
    /* Point the location counter at the virtual base */
    . = KERNEL_VMA;

    .text ALIGN(4096) : AT(ADDR(.text) - KERNEL_VMA) 
    {
        /* Pull in every .text.* section EXCLUDE the ones from boot library */
        EXCLUDE_FILE(/boot/*.o) *(.text*)
    } 

    /* ----------------------------------------------------------------- */
    /*  3. Read‑only data                                                 */
    /* ----------------------------------------------------------------- */
    .rodata ALIGN(4096) : AT(ADDR(.rodata) - KERNEL_VMA)
    {
        EXCLUDE_FILE(/boot/*.o) *(.rodata*)
    }

    /* ----------------------------------------------------------------- */
    /*  4. Writable data and BSS                                          */
    /* ----------------------------------------------------------------- */
    .data ALIGN(4096) : AT(ADDR(.data) - KERNEL_VMA)
    {
        EXCLUDE_FILE(/boot/*.o) *(.data*)
    }

    .bss : AT(ADDR(.bss) - KERNEL_VMA)
    {
        EXCLUDE_FILE(/boot/*.o) *(COMMON)
        EXCLUDE_FILE(/boot/*.o) *(.bss*)
    }
}
