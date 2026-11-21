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

        .globl  gf_window_signal_mask
        .type   gf_window_signal_mask,@function
gf_window_signal_mask:
        move.l  4(%sp),%a0
        move.l  86(%a0),%a0
        beq     .Lno_signal
        moveq   #0,%d0
        move.b  15(%a0),%d0
        moveq   #1,%d1
        lsl.l   %d0,%d1
        move.l  %d1,%d0
        rts
.Lno_signal:
        moveq   #0,%d0
        rts
        .size   gf_window_signal_mask,.-gf_window_signal_mask

/* OLUI_On's exact v18+ register ABI. Keep a6 pointing at the library base,
 * exactly as the official C inline stub does. */
        .globl  gf_openlayout_register_event_handler
        .type   gf_openlayout_register_event_handler,@function
gf_openlayout_register_event_handler:
        movem.l %a2-%a3/%a6,-(%sp)
        move.l  16(%sp),%a6
        move.l  20(%sp),%a0
        move.l  24(%sp),%a1
        moveq   #1,%d0
        /* The callback lives in this executable's text hunk. Use a PC-relative
         * address so the Amiga Hunk loader does not have to repair an absolute
         * callback pointer embedded in the instruction stream. */
        lea     gfopenlayout_event_trampoline(%pc),%a2
        move.l  28(%sp),%a3
        /* jsr -456(a6); LLVM's m68k integrated assembler does not accept
         * the indexed spelling, so encode the standard two-word form. */
        .short  0x4eae,0xfe38
        movem.l (%sp)+,%a2-%a3/%a6
        rts
        .size   gf_openlayout_register_event_handler,.-gf_openlayout_register_event_handler

        .globl  gf_copy_cstr_bounded
        .type   gf_copy_cstr_bounded,@function
gf_copy_cstr_bounded:
        move.l  %d2,-(%sp)
        move.l  8(%sp),%a0
        move.l  12(%sp),%a1
        move.l  16(%sp),%d1
        beq     .Lcopy_invalid
        sub.l   #1,%d1
        moveq   #0,%d0
.Lcopy_loop:
        cmp.l   %d1,%d0
        beq     .Lcopy_full
        move.b  (%a0)+,%d2
        move.b  %d2,(%a1)+
        beq     .Lcopy_done
        add.l   #1,%d0
        bra     .Lcopy_loop
.Lcopy_full:
        move.b  #0,(%a1)
        move.l  %d1,%d0
        bra     .Lcopy_done
.Lcopy_invalid:
        moveq   #-1,%d0
.Lcopy_done:
        move.l  (%sp)+,%d2
        rts
        .size   gf_copy_cstr_bounded,.-gf_copy_cstr_bounded

        .section .note.GNU-stack,"",@progbits
