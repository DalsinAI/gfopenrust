# `amiga-sys`: the first native AmigaOS platform crate

`crates/amiga-sys` is the low-level ABI shared by native 68k Rust programs.
It is `no_std`; it does not pretend that AmigaOS is Unix, and it does not
route an application through a host process.

## What this slice contains

- the 32-bit Amiga types and the public prefixes of Exec's `Node` and
  `Library`;
- Exec memory, pool and library calls;
- DOS file, stream, error, command and formatted-output calls;
- m68k assembly trampolines from Rust's stack-based SysV ABI to Amiga
  library vectors (A6 plus documented argument registers);
- host tests for interface constants, storage and 32-bit layouts.

The call facts come from GFOpenELF's clean-room `openelf_amiga.h`, already
used by native AmigaOS programs and the ELF loader.  The Rust crate owns its
assembly and declarations; it does not copy an SDK implementation.

## Building

Host-side tests need only stable Rust:

```sh
tests/check-amiga-sys.sh
```

That gate runs formatting, unit tests, linting, a real m68k assembly build,
symbol inspection and disassembly checks.  Set `RUN_M68K_RUST=1` to include
the freestanding Rust cross-build when `rust-src` and the m68k GNU tools are
installed:

```sh
RUN_M68K_RUST=1 tests/check-amiga-sys.sh
```

A 68040 build uses GFOpenRust's patched Rust 1.97 toolchain and the
`m68k-unknown-none-elf` target.  The crate embeds the ABI trampolines with
Rust's integrated assembler, so consuming applications do not need a second
cross-compiler or a toolchain-specific archive format.  The standalone gate
still assembles and disassembles the source with GNU m68k tools as an
independent instruction-level check.

This is deliberately only plan step 3.  The next slice adds the
`m68k-amigaos` target, startup and an Exec-pool global allocator; then `std`
can implement files, environment, time and tasks on these calls.

The cross-build proves the Rust objects and the ten ABI trampolines coexist
in the target archive.  It is not a substitute for running a linked program
on AmigaOS.  Step 3 stays in progress until the native call probe passes.
