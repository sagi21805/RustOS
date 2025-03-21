.section .boot, "awx"
.global start
.code16

# This stage initializes the stack, enables the A20 line

start:
    # zero segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    # clear the direction flag (e.g. go forward in memory when using
    # instructions like lodsb)
    cld

    # initialize stack
    mov sp, 0x7c00

enable_a20:
    # enable A20-Line via IO-Port 92, might not work on all motherboards
    in al, 0x92
    test al, 2
    jnz enable_a20_after
    or al, 2
    and al, 0xFE
    out 0x92, al
enable_a20_after:

check_int13h_extensions:
    mov ah, 0x41
    mov bx, 0x55aa
    # dl contains drive number
    int 0x13
    jnc .int13_pass
.int13_pass:
    pop ax      # pop error code again

rust:
    # push arguments
    push dx    # disk number
    call _start
    # Fail code if first stage returns
spin:
    hlt
    jmp spin