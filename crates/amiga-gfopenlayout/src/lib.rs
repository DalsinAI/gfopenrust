// Copyright (c) 2026 AmiCradle Team. MIT.
//! Safe, bounded GFOpenLayout window binding for native AmigaOS 3.x apps.

#![no_std]
#![cfg_attr(not(target_arch = "m68k"), allow(dead_code))]

#[cfg(target_arch = "m68k")]
use core::ffi::CStr;

#[cfg(target_arch = "m68k")]
use amiga_runtime::{OpenLibrary, Runtime};
#[cfg(target_arch = "m68k")]
use core::ffi::c_void;
#[cfg(target_arch = "m68k")]
use core::ptr::NonNull;

const LVO_OPEN_WINDOW_TAG_LIST: i32 = -606;
const LVO_CLOSE_WINDOW: i32 = -72;

const LVO_OLUI_NEW: i32 = -426;
const LVO_OLUI_FREE: i32 = -432;
const LVO_OLUI_LOAD: i32 = -450;
const LVO_OLUI_AMIGA_IDCMP: i32 = -780;
const LVO_OLUI_AMIGA_NEW: i32 = -786;
const LVO_OLUI_AMIGA_FREE: i32 = -792;
const LVO_OLUI_AMIGA_THEME: i32 = -798;
const LVO_OLUI_AMIGA_LAYOUT: i32 = -822;
const LVO_OLUI_AMIGA_DRAW: i32 = -828;
const LVO_OLUI_AMIGA_EVENT: i32 = -834;

const TAG_DONE: u32 = 0;
const WA_DUMMY: u32 = 0x8000_0063;
const WA_LEFT: u32 = WA_DUMMY + 1;
const WA_TOP: u32 = WA_DUMMY + 2;
const WA_WIDTH: u32 = WA_DUMMY + 3;
const WA_HEIGHT: u32 = WA_DUMMY + 4;
const WA_IDCMP: u32 = WA_DUMMY + 7;
const WA_FLAGS: u32 = WA_DUMMY + 8;
const WA_TITLE: u32 = WA_DUMMY + 11;
const WA_MIN_WIDTH: u32 = WA_DUMMY + 15;
const WA_MIN_HEIGHT: u32 = WA_DUMMY + 16;
const WA_MAX_WIDTH: u32 = WA_DUMMY + 17;
const WA_MAX_HEIGHT: u32 = WA_DUMMY + 18;

const WFLG_SIZEGADGET: u32 = 0x0000_0001;
const WFLG_DRAGBAR: u32 = 0x0000_0002;
const WFLG_DEPTHGADGET: u32 = 0x0000_0004;
const WFLG_CLOSEGADGET: u32 = 0x0000_0008;
const WFLG_SIMPLE_REFRESH: u32 = 0x0000_0040;
const WFLG_ACTIVATE: u32 = 0x0000_1000;
const WINDOW_FLAGS: u32 = WFLG_SIZEGADGET
    | WFLG_DRAGBAR
    | WFLG_DEPTHGADGET
    | WFLG_CLOSEGADGET
    | WFLG_SIMPLE_REFRESH
    | WFLG_ACTIVATE;

const WINDOW_USER_PORT_OFFSET: usize = 86;
const MSG_PORT_SIGNAL_BIT_OFFSET: usize = 15;
const OLUI_AMIGA_CLOSE: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg(target_arch = "m68k")]
struct TagItem {
    tag: u32,
    data: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenError {
    LayoutLibraryUnavailable,
    IntuitionLibraryUnavailable,
    WindowUnavailable,
    UiUnavailable,
    InvalidMarkup,
    AmigaGlueUnavailable,
}

/// A GFOpenLayout UI hosted in a resizable Intuition window.
#[cfg(target_arch = "m68k")]
pub struct Window<'runtime> {
    layout: OpenLibrary<'runtime>,
    intuition: OpenLibrary<'runtime>,
    window: NonNull<c_void>,
    ui: NonNull<c_void>,
    glue: NonNull<c_void>,
}

#[cfg(target_arch = "m68k")]
impl<'runtime> Window<'runtime> {
    pub fn open(
        runtime: &'runtime Runtime,
        title: &CStr,
        markup: &CStr,
        css: &CStr,
        width: u16,
        height: u16,
    ) -> Result<Self, OpenError> {
        let layout = runtime
            .open_library(c"gfopenlayout.library", 18)
            .ok_or(OpenError::LayoutLibraryUnavailable)?;
        let intuition = runtime
            .open_library(c"intuition.library", 39)
            .ok_or(OpenError::IntuitionLibraryUnavailable)?;
        let idcmp = unsafe { layout.call0(LVO_OLUI_AMIGA_IDCMP) };
        let tags = [
            TagItem {
                tag: WA_LEFT,
                data: 24,
            },
            TagItem {
                tag: WA_TOP,
                data: 18,
            },
            TagItem {
                tag: WA_WIDTH,
                data: u32::from(width),
            },
            TagItem {
                tag: WA_HEIGHT,
                data: u32::from(height),
            },
            TagItem {
                tag: WA_IDCMP,
                data: idcmp,
            },
            TagItem {
                tag: WA_FLAGS,
                data: WINDOW_FLAGS,
            },
            TagItem {
                tag: WA_TITLE,
                data: title.as_ptr() as usize as u32,
            },
            TagItem {
                tag: WA_MIN_WIDTH,
                data: 480,
            },
            TagItem {
                tag: WA_MIN_HEIGHT,
                data: 320,
            },
            TagItem {
                tag: WA_MAX_WIDTH,
                data: u32::MAX,
            },
            TagItem {
                tag: WA_MAX_HEIGHT,
                data: u32::MAX,
            },
            TagItem {
                tag: TAG_DONE,
                data: 0,
            },
        ];
        let window = NonNull::new(unsafe {
            intuition.call_a0_a1(
                LVO_OPEN_WINDOW_TAG_LIST,
                core::ptr::null_mut(),
                tags.as_ptr().cast_mut().cast(),
            ) as usize as *mut c_void
        })
        .ok_or(OpenError::WindowUnavailable)?;
        let ui = match NonNull::new(unsafe { layout.call0(LVO_OLUI_NEW) } as usize as *mut c_void) {
            Some(ui) => ui,
            None => {
                unsafe { intuition.call_a0(LVO_CLOSE_WINDOW, window.as_ptr()) };
                return Err(OpenError::UiUnavailable);
            }
        };
        let mut error = [0u8; 192];
        let loaded = unsafe {
            layout.call_a0_a1_a2_a3_d0(
                LVO_OLUI_LOAD,
                ui.as_ptr(),
                markup.as_ptr().cast_mut().cast(),
                css.as_ptr().cast_mut().cast(),
                error.as_mut_ptr().cast(),
                error.len() as u32,
            )
        };
        if loaded == 0 {
            unsafe {
                layout.call_a0(LVO_OLUI_FREE, ui.as_ptr());
                intuition.call_a0(LVO_CLOSE_WINDOW, window.as_ptr());
            }
            return Err(OpenError::InvalidMarkup);
        }
        let glue = match NonNull::new(unsafe {
            layout.call_a0_a1(LVO_OLUI_AMIGA_NEW, ui.as_ptr(), window.as_ptr()) as usize
                as *mut c_void
        }) {
            Some(glue) => glue,
            None => {
                unsafe {
                    layout.call_a0(LVO_OLUI_FREE, ui.as_ptr());
                    intuition.call_a0(LVO_CLOSE_WINDOW, window.as_ptr());
                }
                return Err(OpenError::AmigaGlueUnavailable);
            }
        };
        unsafe {
            layout.call_a0_a1(LVO_OLUI_AMIGA_THEME, glue.as_ptr(), core::ptr::null_mut());
            layout.call_a0(LVO_OLUI_AMIGA_LAYOUT, glue.as_ptr());
            layout.call_a0(LVO_OLUI_AMIGA_DRAW, glue.as_ptr());
        }
        Ok(Self {
            layout,
            intuition,
            window,
            ui,
            glue,
        })
    }

    /// Run the Intuition event loop until the close gadget is selected.
    pub fn run(&mut self, runtime: &Runtime) {
        let port = unsafe {
            (self
                .window
                .as_ptr()
                .cast::<u8>()
                .add(WINDOW_USER_PORT_OFFSET) as *const *mut c_void)
                .read_unaligned()
        };
        if port.is_null() {
            return;
        }
        let signal_bit = unsafe {
            self.window
                .as_ptr()
                .cast::<u8>()
                .add(WINDOW_USER_PORT_OFFSET)
                .cast::<*mut u8>()
                .read_unaligned()
                .add(MSG_PORT_SIGNAL_BIT_OFFSET)
                .read()
        };
        let signal = 1u32.checked_shl(u32::from(signal_bit)).unwrap_or(0);
        if signal == 0 {
            return;
        }
        let mut closed = false;
        while !closed {
            runtime.wait(signal);
            loop {
                let message = unsafe { runtime.get_message(port) };
                if message.is_null() {
                    break;
                }
                let result = unsafe {
                    self.layout
                        .call_a0_a1(LVO_OLUI_AMIGA_EVENT, self.glue.as_ptr(), message)
                };
                unsafe { runtime.reply_message(message) };
                if result == OLUI_AMIGA_CLOSE {
                    closed = true;
                }
            }
            unsafe { self.layout.call_a0(LVO_OLUI_AMIGA_DRAW, self.glue.as_ptr()) };
        }
    }
}

#[cfg(target_arch = "m68k")]
impl Drop for Window<'_> {
    fn drop(&mut self) {
        unsafe {
            self.layout.call_a0(LVO_OLUI_AMIGA_FREE, self.glue.as_ptr());
            self.layout.call_a0(LVO_OLUI_FREE, self.ui.as_ptr());
            self.intuition
                .call_a0(LVO_CLOSE_WINDOW, self.window.as_ptr());
        }
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_tags_match_amigaos_ndk() {
        assert_eq!(WA_LEFT, 0x8000_0064);
        assert_eq!(WA_IDCMP, 0x8000_006a);
        assert_eq!(WA_TITLE, 0x8000_006e);
    }

    #[test]
    fn gfopenlayout_vectors_match_version_18_sfd() {
        assert_eq!(LVO_OLUI_NEW, -426);
        assert_eq!(LVO_OLUI_LOAD, -450);
        assert_eq!(LVO_OLUI_AMIGA_EVENT, -834);
    }
}
