// Copyright (c) 2026 AmiCradle Team. MIT.
#![cfg_attr(target_arch = "m68k", no_main)]
#![cfg_attr(target_arch = "m68k", no_std)]

#[cfg(target_arch = "m68k")]
#[no_mangle]
pub extern "C" fn gf_amiga_main(_args: *const u8, _length: i32) -> i32 {
    amiga_runtime::run(|runtime| {
        let Ok(mut synth) = amiga_gfopensynth::Synth::open(runtime) else {
            runtime.put_str(c"gfopensynth.library unavailable\n");
            return 20;
        };

        synth.reset();
        let Ok(silence) = amiga_gfopensynth::ShortMessage::all_notes_off(0) else {
            return 20;
        };
        synth.send_short(silence);
        runtime.put_str(c"GFOpenSynth binding ready\n");
        0
    })
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
