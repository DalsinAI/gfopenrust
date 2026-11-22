#!/bin/sh
# Copyright (c) 2026 AmiCradle Team. MIT.
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/gfopenrust-amiga-sys.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

cd "$repo"
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

cc=${M68K_CC:-m68k-linux-gnu-gcc}
nm=${M68K_NM:-m68k-linux-gnu-nm}
objdump=${M68K_OBJDUMP:-m68k-linux-gnu-objdump}

"$cc" -m68040 -msoft-float -c crates/amiga-sys/src/amiga_calls.S \
    -o "$tmp/amiga_calls.o"
"$nm" -g --defined-only "$tmp/amiga_calls.o" > "$tmp/symbols.txt"

for symbol in \
    gf_lib_call0 \
    gf_lib_call_d0_d1_a0_a1 \
    gf_lib_call_d0_d1_d2 \
    gf_lib_call_a0 \
    gf_lib_call_a0_d0 \
    gf_lib_call_a0_a1_d0 \
    gf_lib_call_a0_a1_a2_a3_d0 \
    gf_lib_call_a1 \
    gf_dos_call_d1_d2_d3 \
    gf_dos_call_d1_d2_d3_d4
do
    grep -Eq "[[:space:]]${symbol}$" "$tmp/symbols.txt"
done

"$objdump" -dr "$tmp/amiga_calls.o" > "$tmp/disassembly.txt"
# Every library-vector trampoline must make an indirect call through an
# address register.  The implementation deliberately copies A6 and adds the
# vector offset in a scratch address register before the call, so pin the ABI
# property without requiring one particular scratch register.
grep -Eq 'jsr[[:space:]]+%a[0-4]@' "$tmp/disassembly.txt"
grep -Eq 'moveml[[:space:]]+%d2-%d4/%fp' "$tmp/disassembly.txt"

if [ "${RUN_M68K_RUST:-0}" = 1 ]; then
    CARGO_TARGET_DIR="$tmp/rust-target" RUSTC_BOOTSTRAP=1 \
    RUSTFLAGS='-C target-cpu=M68040 -C target-feature=-isa-68881,-isa-68882 -C code-model=large' \
        cargo build --release --target m68k-unknown-none-elf \
        -Zbuild-std=core,compiler_builtins \
        -Zbuild-std-features=compiler-builtins-mem \
        -p amiga-sys
fi

echo "amiga-sys verification passed"
