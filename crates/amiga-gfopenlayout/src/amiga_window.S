/* Native accessors for Intuition structures whose classic ABI contains
 * two-byte-aligned pointer fields. Keeping this load in m68k assembly avoids
 * depending on the compiler's lowering of Rust read_unaligned for a pointer.
 *
 * Copyright (c) 2026 AmiCradle Team. MIT. */
        .text
        .align  2

        .globl  gf_window_user_port
        .type   gf_window_user_port,@function
gf_window_user_port:
        move.l  4(%sp),%a0
        move.l  86(%a0),%d0
        rts
        .size   gf_window_user_port,.-gf_window_user_port

        .section .note.GNU-stack,"",@progbits
