// Copyright (c) 2026 AmiCradle Team. MIT.
#![cfg_attr(target_arch = "m68k", no_main)]
#![cfg_attr(target_arch = "m68k", no_std)]

#[cfg(target_arch = "m68k")]
extern crate alloc;

#[cfg(target_arch = "m68k")]
use alloc::vec::Vec;

#[cfg(target_arch = "m68k")]
#[no_mangle]
pub extern "C" fn gf_amiga_main(_args: *const u8, _length: i32) -> i32 {
    if !unsafe { amiga_runtime::init() } {
        return 20;
    }

    let mut proof = Vec::with_capacity(4);
    proof.extend_from_slice(&[68u8, 69, 70, 71]);
    let valid = proof == [68u8, 69, 70, 71];
    unsafe {
        amiga_runtime::put_str(b"GFOpenRust native AmigaOS runtime\n\0".as_ptr());
    }
    drop(proof);
    unsafe { amiga_runtime::shutdown() };

    if valid {
        0
    } else {
        20
    }
}

#[cfg(target_arch = "m68k")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(not(target_arch = "m68k"))]
fn main() {}
