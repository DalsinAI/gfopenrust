/* Native AmigaDOS program entry.
 *
 * AmigaDOS supplies D0 = command-line length and A0 = command-line bytes.
 * The application supplies:
 *     int gf_amiga_main(const unsigned char *args, long length)
 *
 * Copyright (c) 2026 AmiCradle Team. MIT. */
        .text
        .align  2
        .globl  _start
        .type   _start,@function
_start:
        move.l  %d0,-(%sp)
        move.l  %a0,-(%sp)
        jsr     gf_amiga_main
        addq.l  #8,%sp
        rts
        .size   _start,.-_start

        .section .note.GNU-stack,"",@progbits
