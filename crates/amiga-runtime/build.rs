// Copyright (c) 2026 AmiCradle Team. MIT.
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn run(command: &mut Command, what: &str) {
    let status = command.status().unwrap_or_else(|error| {
        panic!("could not start {what}: {error}");
    });
    if !status.success() {
        panic!("{what} failed with {status}");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=src/amiga_start.S");
    println!("cargo:rerun-if-env-changed=M68K_CC");
    println!("cargo:rerun-if-env-changed=M68K_AR");

    let target = env::var("TARGET").expect("Cargo did not set TARGET");
    if !target.starts_with("m68k-") {
        return;
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo did not set OUT_DIR"));
    let object = out.join("amiga_start.o");
    let archive = out.join("libamiga_start.a");
    let cc = env::var("M68K_CC").unwrap_or_else(|_| "m68k-linux-gnu-gcc".into());
    let ar = env::var("M68K_AR").unwrap_or_else(|_| "m68k-linux-gnu-ar".into());

    run(
        Command::new(cc)
            .arg("-m68040")
            .arg("-msoft-float")
            .arg("-c")
            .arg("src/amiga_start.S")
            .arg("-o")
            .arg(&object),
        "AmigaOS startup assembly",
    );
    run(
        Command::new(ar).arg("crs").arg(&archive).arg(&object),
        "AmigaOS startup archive",
    );

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=amiga_start");
}
