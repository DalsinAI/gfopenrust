# Native AmigaOS target and runtime

This is plan step 4's first executable slice.  It reuses the proven entry and
pool design in GFOpenELF rather than inventing a second ABI:

- `targets/m68k-amigaos.json` fixes the target at big-endian 32-bit m68k,
  68040, soft float, the large code model, static relocation and panic abort;
- `crates/amiga-runtime` supplies the AmigaDOS D0/A0 entry, opens
  dos.library and owns one Exec allocation pool;
- its global allocator preserves alignments above the pool's eight-byte base
  alignment by storing the original block immediately before the aligned
  answer;
- `examples/amiga-hello` links `core`, `alloc`, the runtime and `amiga-sys`
  into one native ELF executable.

## Verification

```sh
tests/check-amiga-runtime.sh
```

The gate runs all host tests and lints, builds the custom target, and verifies
that the result is a static 32-bit big-endian m68k executable with `_start`,
`gf_amiga_main`, every ABI trampoline, no unresolved symbols and no dynamic
relocations.

The current compile receipt uses the installed Rust 1.98 compiler as a target
and linkage smoke test.  A release candidate must be rebuilt with the pinned,
patched Rust 1.97 `rust-m68k` stove, because the project has already proven
that stock Rust 1.97 miscompiles m68k programs.  The linked proof must then run
through elf.library on AmigaOS before step 4 can be marked done.

## Runtime contract

An application exports this C-ABI entry:

```rust
#[no_mangle]
pub extern "C" fn gf_amiga_main(args: *const u8, length: i32) -> i32
```

It calls `amiga_runtime::init()` before allocation or DOS access, drops all
allocated values, then calls `amiga_runtime::shutdown()`.  This first runtime
is deliberately single-task.  The `std` task implementation will add a
protected allocation policy before exposing Rust threads.
