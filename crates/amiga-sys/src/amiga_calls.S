/* AmigaOS library-call trampolines for the m68k SysV C ABI.
 *
 * Rust passes these arguments on the stack.  An Amiga library call instead
 * puts its base in A6, its arguments in the documented data/address
 * registers, and jumps through the negative library vector.  These routines
 * bridge the two ABIs while preserving every C callee-saved register they
 * borrow.  D0 is the call's answer.
 *
 * Copyright (c) 2026 AmiCradle Team. MIT. */
        .text
        .align  2

        .globl  gf_lib_call0
gf_lib_call0:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a6
        adda.l  12(%sp),%a6
        jsr     (%a6)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_lib_call_d0_d1_a0_a1
gf_lib_call_d0_d1_a0_a1:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a6
        adda.l  12(%sp),%a6
        move.l  16(%sp),%d0
        move.l  20(%sp),%d1
        move.l  24(%sp),%a0
        move.l  28(%sp),%a1
        jsr     (%a6)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_lib_call_d0_d1_d2
gf_lib_call_d0_d1_d2:
        movem.l %d2/%a6,-(%sp)
        move.l  12(%sp),%a6
        adda.l  16(%sp),%a6
        move.l  20(%sp),%d0
        move.l  24(%sp),%d1
        move.l  28(%sp),%d2
        jsr     (%a6)
        movem.l (%sp)+,%d2/%a6
        rts

        .globl  gf_lib_call_a0
gf_lib_call_a0:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a6
        adda.l  12(%sp),%a6
        move.l  16(%sp),%a0
        jsr     (%a6)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_lib_call_a0_d0
gf_lib_call_a0_d0:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a6
        adda.l  12(%sp),%a6
        move.l  16(%sp),%a0
        move.l  20(%sp),%d0
        jsr     (%a6)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_lib_call_a0_a1_d0
gf_lib_call_a0_a1_d0:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a6
        adda.l  12(%sp),%a6
        move.l  16(%sp),%a0
        move.l  20(%sp),%a1
        move.l  24(%sp),%d0
        jsr     (%a6)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_dos_call_d1_d2_d3
gf_dos_call_d1_d2_d3:
        movem.l %d2-%d3/%a6,-(%sp)
        move.l  16(%sp),%a6
        adda.l  20(%sp),%a6
        move.l  24(%sp),%d1
        move.l  28(%sp),%d2
        move.l  32(%sp),%d3
        jsr     (%a6)
        movem.l (%sp)+,%d2-%d3/%a6
        rts

        .globl  gf_dos_call_d1_d2_d3_d4
gf_dos_call_d1_d2_d3_d4:
        movem.l %d2-%d4/%a6,-(%sp)
        move.l  20(%sp),%a6
        adda.l  24(%sp),%a6
        move.l  28(%sp),%d1
        move.l  32(%sp),%d2
        move.l  36(%sp),%d3
        move.l  40(%sp),%d4
        jsr     (%a6)
        movem.l (%sp)+,%d2-%d4/%a6
        rts

        .section .note.GNU-stack,"",@progbits
