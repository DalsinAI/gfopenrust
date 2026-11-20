/* Native accessors for Intuition structures whose classic ABI contains
 * two-byte-aligned pointer fields. Keeping this load in m68k assembly avoids
 * depending on the compiler's lowering of Rust read_unaligned for a pointer.
 *
 * Copyright (c) 2026 AmiCradle Team. MIT. */
        .text
        .align  2

        .globl  gf_window_get_message
        .type   gf_window_get_message,@function
gf_window_get_message:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a0
        move.l  86(%a0),%a0
        beq     .Lno_message_port
        move.l  4,%a6
        adda.l  #-372,%a6
        jsr     (%a6)
        move.l  %d0,%a0
        move.l  (%sp)+,%a6
        rts
.Lno_message_port:
        moveq   #0,%d0
        suba.l  %a0,%a0
        move.l  (%sp)+,%a6
        rts
        .size   gf_window_get_message,.-gf_window_get_message

        .globl  gf_reply_message
        .type   gf_reply_message,@function
gf_reply_message:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a1
        move.l  4,%a6
        adda.l  #-378,%a6
        jsr     (%a6)
        move.l  (%sp)+,%a6
        rts
        .size   gf_reply_message,.-gf_reply_message

        .section .note.GNU-stack,"",@progbits
