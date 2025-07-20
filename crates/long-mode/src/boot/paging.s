.code32


.section .text
.global setup_identity_paging
// Setup the identity paging for the first 8 large pages of memory (ie 16MiB). This should be largely enough for the kernel bootstrapper
setup_identity_paging:
    leal p4_table, %eax
    leal p3_table, %ecx

    // Set the present and writable flags
    // 0b11 = Present | Writable
    orl $0b11, %ecx

    // set p4 table to point to p3 table
    movl %ecx, (%eax)
    
    // set p3 table to point to p2 table
    leal p3_table, %eax
    leal p2_table, %ecx
    // Set the present and writable flags
    // 0x3 = 0b11 = Present | Writable
    orl $0b11, %ecx

    // set p3 table to point to p2 table
    movl %ecx, (%eax)

    // map first 10 large pages of memory to identity map
    movl $0, %edx
    // Reload the address of the p2 table
    leal p2_table, %eax

    .loop:
        // Address = %edx * LARGE_PAGE_SIZE
        // Page Table Entry = Address | Present | Write | Large Page
        movl %edx, %ecx
        imull $0x200000, %ecx
        // Set the present, writable, and large page flags
        // 0b10000011 = Present | Writable | Large Page
        orl $0b10000011, %ecx

        // set the page table entry at the edx index
        movl %ecx, (%eax, %edx, 8)

        addl $1, %edx
        cmpl $8, %edx
        jl .loop

    leal paging_ok_string, %eax
    call ok

    ret

.section .bss
.align 4096

.global p4_table
.equ PAGE_SIZE, 4096
p4_table:
    .space 4096
p3_table:
    .space 4096
p2_table:
    .space 4096

.section .rodata
paging_ok_string:
    .asciz "Paging setup successful!"
p4_table_string:
    .asciz "P4 Table: "
p3_table_string:
    .asciz "P3 Table: "
p2_table_string:
    .asciz "P2 Table: "