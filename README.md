# GFOpenAmigaRust

Rust for the Amiga: programs written in Rust that run on AmigaOS 3.x on a
68040.

## Where it stands (23 October 2026)

- **The compiler works.** Stock Rust 1.97 builds 68k code that prints wrong
  numbers, hangs, and crashes the compiler. LLVM's M68k backend had six
  bugs, which are now fixed. The `rust-m68k` stove in
  [amicradle-stoves](https://github.com/DalsinAI/amicradle-stoves/tree/main/rust-m68k)
  builds Rust 1.97.0 on the fixed LLVM. Its output matches a PC's at every
  optimisation level.
- **Programs load through elf.library.** Rust builds a 68k ELF, and
  [elf.library](https://github.com/DalsinAI/amicradle-guest/tree/main/libraries/openelf)
  loads it from the Shell. elf.library's `rust/hello` (`core` and `alloc` on an
  exec memory pool) has not been run on an Amiga yet; its fixture runs under
  qemu in elf.library's tests.
- **In progress:** `crates/amiga-sys` now defines the first native Exec and
  DOS interfaces, including assembly trampolines for Amiga's register-based
  library calls. `targets/m68k-amigaos.json` and `crates/amiga-runtime` now
  produce a linked AmigaDOS-entry ELF with an Exec-pool global allocator.
  `crates/amiga-gfopensynth` adds the first safe typed shared-library binding:
  short MIDI messages, SysEx and reset through `gfopensynth.library` version
  1, with the library and synth lifetimes tied to the runtime callback.
  `crates/amiga-gfopenlayout` adds bounded native windows, layout loading,
  controls and event dispatch behind a safe application callback.  Those
  runtime and service paths have driven native Craft editors in an isolated
  AmigaOS 3.x 68040 lab; broader platform validation and `std` remain. See
  [docs/PLAN.md](docs/PLAN.md),
  [docs/AMIGA-SYS.md](docs/AMIGA-SYS.md) and
  [docs/AMIGA-RUNTIME.md](docs/AMIGA-RUNTIME.md).

## Building for the 68040

Install the stove (see its README), then build with its Cargo settings: soft
float and the large code model.

    RUSTC_BOOTSTRAP=1 cargo +rust-m68k build --release --target m68k-unknown-none-elf \
        --config /path/to/amicradle-stoves/rust-m68k/cargo-config.toml \
        -Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem

The native target and runtime proof build with:

    tests/check-amiga-runtime.sh

## Licence

MIT (see `LICENSE`). Copyright (c) 2026 AmiCradle Team.

Designed, tested and delivered by The AmiCradle Team.
