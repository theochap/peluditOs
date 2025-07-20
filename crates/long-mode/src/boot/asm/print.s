.code32

.section .text
.global print_char
print_char:
    orl $0x0f00, %eax

    // load vga buffer address
    movl $0xb8000, %ebx

    // calculate offset
    movl vga_y, %ecx
    // multiply by vga width
    imul $80, %ecx
    // add vga x
    addl vga_x, %ecx

    // write character to buffer
    movl %eax, (%ebx, %ecx, 2)

    .update_cursor:
        movl vga_x, %ecx
        // if vga x is less than 80 (vga width), increment it, otherwise reset it to 0 to fit the window
        cmp $80, %ecx
        jge .update_cursor_y
        addl $1, vga_x
        jmp .update_cursor_done
    .update_cursor_y:
        movl $0, vga_x 
        movl vga_y, %ecx
        // if vga y is less than 25 (vga height), increment it, otherwise reset it to 0 to fit the window
        cmp $25, %ecx
        jge .reset_window
        addl $1, vga_y
        jmp .update_cursor_done
    .reset_window:
        movl $0, vga_x
        movl $0, vga_y
    .update_cursor_done:
        ret

.global print_newline
print_newline:
    jmp .update_cursor_y
    ret

.global print_string
print_string:
    mov %eax, %esi // load string pointer into esi
    .print_string_loop:
        lodsb
        cmp $0, %al
        je .print_string_done

        call print_char

        addl $1, %ecx
        jmp .print_string_loop
    .print_string_done:
        ret

.global print_hex
print_hex:
    pushl %eax // save argument

    movl $'0', %eax
    call print_char
    movl $'x', %eax
    call print_char

    popl %ebx

    movl $8, %ecx // 8 nibbles
    .print_hex_loop:
        xorl %eax, %eax
        mov %ebx, %eax

        // Extract the current nibble
        shrl $28, %eax

        pushl %ebx // save ebx
        pushl %ecx // save ecx
        call hex_to_ascii
        popl %ecx
        popl %ebx

        shll $4, %ebx
        decl %ecx
        jnz .print_hex_loop

    ret

// We have to special case the digits 10-15 to print 'A'-'F'
hex_to_ascii:
    cmp $10, %al
    jl .digit
    addl $0x37, %eax   # 'A' - 10 = 0x41 - 0x0A = 0x37
    jmp .print
    .digit:
    addl $0x30, %eax   # '0' = 0x30
    .print:
    call print_char
    ret

// Alternative version that stores comparison result in eax
hex_to_ascii_with_result:
    pushl %ebx
    movl %eax, %ebx    # Save original value
    
    cmp $10, %al
    setl %al           # AL = 1 if < 10, 0 if >= 10
    movzbl %al, %eax   # Zero-extend to EAX
    
    # Now EAX contains 1 if digit was < 10, 0 if >= 10
    # You can use this result for other logic
    
    movl %ebx, %eax    # Restore original value for printing
    cmp $10, %al
    jl .digit2
    addl $0x37, %eax
    jmp .print2
    .digit2:
    addl $0x30, %eax
    .print2:
    call print_char
    popl %ebx
    ret

.global error
error:
    pushl %eax

    leal error_string, %eax
    call print_string

    popl %eax
    // if the error string is empty, don't print it
    testl %eax, %eax
    jz .error_exit
    call print_string

    .error_exit:
    call print_newline
    ret

.global ok
ok:
    pushl %eax

    leal ok_string, %eax
    call print_string

    popl %eax
    // if the ok string is empty, don't print it
    testl %eax, %eax
    jz .ok_exit
    call print_string

    .ok_exit:
    call print_newline

    ret

.section .bss
vga_x:
    .long 0
vga_y:
    .long 0

.section .rodata
error_string:
    .asciz "Error: "

ok_string:
    .asciz "OK: "