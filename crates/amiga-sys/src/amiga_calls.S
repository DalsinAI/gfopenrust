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
        move.l  %a6,%a0
        adda.l  12(%sp),%a0
        jsr     (%a0)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_lib_call_d0_d1_a0_a1
gf_lib_call_d0_d1_a0_a1:
        movem.l %a2/%a6,-(%sp)
        move.l  12(%sp),%a6
        move.l  %a6,%a2
        adda.l  16(%sp),%a2
        move.l  20(%sp),%d0
        move.l  24(%sp),%d1
        move.l  28(%sp),%a0
        move.l  32(%sp),%a1
        jsr     (%a2)
        movem.l (%sp)+,%a2/%a6
        rts

/* Exec memory calls use dedicated stubs.  Besides matching the official C
 * inline sequence exactly, this avoids depending on a computed jump through
 * a negative library-vector address in the allocator's hottest path. */
        .globl  gf_exec_alloc_mem
gf_exec_alloc_mem:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%d0
        move.l  12(%sp),%d1
        move.l  4,%a6
        .short  0x4eae,0xff3a
        move.l  (%sp)+,%a6
        rts

        .globl  gf_exec_free_mem
gf_exec_free_mem:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a1
        move.l  12(%sp),%d0
        move.l  4,%a6
        .short  0x4eae,0xff2e
        move.l  (%sp)+,%a6
        rts

        .globl  gf_lib_call_d0_d1_d2
gf_lib_call_d0_d1_d2:
        movem.l %d2/%a6,-(%sp)
        move.l  12(%sp),%a6
        move.l  %a6,%a0
        adda.l  16(%sp),%a0
        move.l  20(%sp),%d0
        move.l  24(%sp),%d1
        move.l  28(%sp),%d2
        jsr     (%a0)
        movem.l (%sp)+,%d2/%a6
        rts

        .globl  gf_lib_call_a0
gf_lib_call_a0:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a6
        move.l  %a6,%a1
        adda.l  12(%sp),%a1
        move.l  16(%sp),%a0
        jsr     (%a1)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_lib_call_a0_d0
gf_lib_call_a0_d0:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a6
        move.l  %a6,%a1
        adda.l  12(%sp),%a1
        move.l  16(%sp),%a0
        move.l  20(%sp),%d0
        jsr     (%a1)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_lib_call_a0_a1_d0
gf_lib_call_a0_a1_d0:
        movem.l %a2/%a6,-(%sp)
        move.l  12(%sp),%a6
        move.l  %a6,%a2
        adda.l  16(%sp),%a2
        move.l  20(%sp),%a0
        move.l  24(%sp),%a1
        move.l  28(%sp),%d0
        jsr     (%a2)
        movem.l (%sp)+,%a2/%a6
        rts

        .globl  gf_lib_call_a0_a1_a2_a3_d0
gf_lib_call_a0_a1_a2_a3_d0:
        movem.l %a2-%a4/%a6,-(%sp)
        move.l  20(%sp),%a6
        move.l  %a6,%a4
        adda.l  24(%sp),%a4
        move.l  28(%sp),%a0
        move.l  32(%sp),%a1
        move.l  36(%sp),%a2
        move.l  40(%sp),%a3
        move.l  44(%sp),%d0
        jsr     (%a4)
        movem.l (%sp)+,%a2-%a4/%a6
        rts

        .globl  gf_lib_call_a1
gf_lib_call_a1:
        move.l  %a6,-(%sp)
        move.l  8(%sp),%a6
        move.l  %a6,%a0
        adda.l  12(%sp),%a0
        move.l  16(%sp),%a1
        jsr     (%a0)
        move.l  (%sp)+,%a6
        rts

        .globl  gf_dos_call_d1_d2_d3
gf_dos_call_d1_d2_d3:
        movem.l %d2-%d3/%a6,-(%sp)
        move.l  16(%sp),%a6
        move.l  %a6,%a0
        adda.l  20(%sp),%a0
        move.l  24(%sp),%d1
        move.l  28(%sp),%d2
        move.l  32(%sp),%d3
        jsr     (%a0)
        movem.l (%sp)+,%d2-%d3/%a6
        rts

        .globl  gf_dos_call_d1_d2_d3_d4
gf_dos_call_d1_d2_d3_d4:
        movem.l %d2-%d4/%a6,-(%sp)
        move.l  20(%sp),%a6
        move.l  %a6,%a0
        adda.l  24(%sp),%a0
        move.l  28(%sp),%d1
        move.l  32(%sp),%d2
        move.l  36(%sp),%d3
        move.l  40(%sp),%d4
        jsr     (%a0)
        movem.l (%sp)+,%d2-%d4/%a6
        rts

        .section .note.GNU-stack,"",@progbits
