#!/bin/sh
# Copyright (c) 2026 AmiCradle Team. MIT.
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo"

tests/check-amiga-sys.sh

RUSTC_BOOTSTRAP=1 cargo build --release \
    --target targets/m68k-amigaos.json \
    -Zjson-target-spec \
    -Zbuild-std=core,alloc,compiler_builtins \
    -Zbuild-std-features=compiler-builtins-mem \
    -p amiga-hello

program=target/m68k-amigaos/release/amiga-hello
nm=${M68K_NM:-m68k-linux-gnu-nm}
readelf=${M68K_READELF:-m68k-linux-gnu-readelf}
objdump=${M68K_OBJDUMP:-m68k-linux-gnu-objdump}

file "$program" | grep -Eq 'ELF 32-bit MSB executable, Motorola m68k.*statically linked'
test -z "$("$nm" -u "$program")"

for symbol in \
    _start \
    gf_amiga_main \
    gf_lib_call0 \
    gf_lib_call_d0_d1_a0_a1 \
    gf_lib_call_d0_d1_d2 \
    gf_dos_call_d1_d2_d3 \
    gf_dos_call_d1_d2_d3_d4
do
    "$nm" -g --defined-only "$program" | grep -Eq "[[:space:]]${symbol}$"
done

"$readelf" -rW "$program" | grep -Fq 'There are no relocations in this file.'
"$objdump" -dr "$program" | grep -Eq '<_start>:'
"$objdump" -dr "$program" | grep -Eq 'jsr[[:space:]].*<gf_amiga_main>'

echo "AmigaOS runtime verification passed: $program"
