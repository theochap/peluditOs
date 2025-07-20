.code32
.section .text

// Check if the CPUID instruction is supported
// Returns 1 if CPUID is supported, 0 otherwise
.supports_cpuid:
    // Store EFLAGS to restore once the function returns
    pushfd                 

    // Store EFLAGS to test if CPUID is supported
    pushfd
    // Invert the ID bit in stored EFLAGS              
    xorl $0x00200000, (%esp)
    // Load stored EFLAGS (with ID bit inverted)
    popfd                               
    // Store EFLAGS again (ID bit may or may not be inverted)
    pushfd                              
    // reg = modified EFLAGS (ID bit may or may not be inverted)
    popl %eax                           
    // reg = whichever bits were changed
    xorl  (%esp), %eax                     
    // Restore original EFLAGS
    popfd                               
    // reg = zero if ID bit can't be changed, else non-zero
    andl $0x00200000, %eax                

    ret

.supports_extended_cpuid:
    // Check if the extended CPUID instruction is supported
    movl $0x80000000, %eax
    movl $0, %ecx
    cpuid
    cmp $0x80000000, %eax
    jl .not_supported_extended_cpuid

    // Check if the long mode is supported
    movl $0x80000001, %eax
    movl $0, %ecx
    cpuid
    shrl $29, %edx
    andl $1, %edx

    movl %edx, %eax
    ret

    .not_supported_extended_cpuid:
    movl $0, %eax
    ret

.global check_cpuid
check_cpuid:
    call .supports_cpuid

    // If the ID bit can't be changed, then CPUID is not supported
    testl %eax, %eax
    jz .cpuid_not_supported_error

    leal cpuid_supported_string, %eax
    call ok

    call .supports_extended_cpuid

    // If the extended CPUID instruction is not supported, then exit
    testl %eax, %eax
    jz .cpuid_extended_not_supported_error

    leal cpuid_extended_supported_string, %eax
    call ok

    ret

    .cpuid_not_supported_error:
    leal cpuid_failed_string, %eax
    call error
    ret

    .cpuid_extended_not_supported_error:
    leal cpuid_extended_failed_string, %eax
    call error
    ret


.section .rodata
cpuid_failed_string:
    .asciz "CPUID instruction is not supported"
cpuid_extended_failed_string:
    .asciz "CPUID extended instruction set with long mode is not supported"
cpuid_supported_string:
    .asciz "CPUID instruction is supported"
cpuid_extended_supported_string:
    .asciz "CPUID extended instruction set with long mode is supported"