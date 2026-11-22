// Copyright (c) 2026 AmiCradle Team. MIT.
//! Safe, bounded GFOpenLayout window binding for native AmigaOS 3.x apps.

#![no_std]
#![cfg_attr(not(target_arch = "m68k"), allow(dead_code))]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

#[cfg(target_arch = "m68k")]
extern crate alloc;

#[cfg(target_arch = "m68k")]
use core::ffi::CStr;

#[cfg(target_arch = "m68k")]
use amiga_runtime::{OpenLibrary, Runtime};
#[cfg(target_arch = "m68k")]
use core::ffi::c_void;
#[cfg(target_arch = "m68k")]
use core::ptr::NonNull;

#[cfg(target_arch = "m68k")]
core::arch::global_asm!(include_str!("amiga_window.S"));

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn gf_openlayout_register_event_handler(
        library_base: *mut c_void,
        ui: *mut c_void,
        id: *const core::ffi::c_char,
        userdata: *mut c_void,
    ) -> u32;
    fn gf_copy_cstr_bounded(
        source: *const core::ffi::c_char,
        target: *mut u8,
        capacity: usize,
    ) -> usize;
    fn gf_window_get_message(window: *mut c_void) -> *mut c_void;
    fn gf_reply_message(message: *mut c_void);
    fn gf_window_signal_mask(window: *mut c_void) -> u32;
}

const LVO_OPEN_WINDOW_TAG_LIST: i32 = -606;
const LVO_CLOSE_WINDOW: i32 = -72;
const LVO_MODIFY_IDCMP: i32 = -150;

const LVO_OLUI_NEW: i32 = -426;
const LVO_OLUI_FREE: i32 = -432;
const LVO_OLUI_LOAD: i32 = -450;
const LVO_OLUI_SET_ZOOM: i32 = -462;
const LVO_OLUI_FIND: i32 = -504;
const LVO_OLUI_SET_TEXT: i32 = -528;
const LVO_OLUI_SET_DISABLED: i32 = -552;
const LVO_OLUI_SET_HIDDEN: i32 = -558;
const LVO_OLUI_SET_STYLE: i32 = -852;
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
const WFLG_REPORTMOUSE: u32 = 0x0000_0200;
const WFLG_ACTIVATE: u32 = 0x0000_1000;
const WINDOW_FLAGS: u32 = WFLG_SIZEGADGET
    | WFLG_DRAGBAR
    | WFLG_DEPTHGADGET
    | WFLG_CLOSEGADGET
    | WFLG_SIMPLE_REFRESH
    | WFLG_REPORTMOUSE
    | WFLG_ACTIVATE;
const WINDOW_IDCMP: u32 = 0x0060_061e;

const OLUI_AMIGA_CLOSE: u32 = 2;
const OLUI_CLICK: i32 = 1;
const OLUI_CHANGE: i32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg(target_arch = "m68k")]
struct TagItem {
    tag: u32,
    data: u32,
}

#[repr(C)]
#[cfg(target_arch = "m68k")]
struct RawEvent {
    event_type: i32,
    element: *mut c_void,
    id: *const core::ffi::c_char,
    item: *mut c_void,
    value: *const core::ffi::c_char,
    index: i32,
    checked: i32,
    x: i32,
    y: i32,
    qualifiers: u32,
    dx: i32,
    dy: i32,
}

/// A bounded GFOpenLayout event borrowed for the duration of its callback.
#[cfg(target_arch = "m68k")]
pub struct Event<'event> {
    pub kind: EventKind,
    pub id: &'event CStr,
    /// Zero-based position in [`EventSink::click_ids`] for an id-specific
    /// registration. Generic registrations report zero.
    pub registration: usize,
    pub value: Option<&'event CStr>,
    pub index: i32,
    pub checked: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    Click,
    Change,
    Other(i32),
}

/// Receives UI events without exposing the C callback ABI to applications.
#[cfg(target_arch = "m68k")]
pub trait EventSink {
    /// Handle one event and report whether application-visible state changed.
    /// Returning false keeps GFOpenLayout's active press and focus state
    /// untouched between related native input messages.
    fn on_event(&mut self, event: Event<'_>) -> bool;

    /// Button ids this sink handles. Registering each id separately also lets
    /// the safe bridge retain the id without borrowing GFOpenLayout's event
    /// record after the native callback returns.
    fn click_ids(&self) -> &'static [&'static CStr] {
        &[]
    }

    /// Return a native Intuition title after an event, when the application
    /// wants immediate feedback without rebuilding the GFOpenLayout tree.
    fn window_title(&self) -> Option<&CStr> {
        None
    }

    /// Apply any state changes after GFOpenLayout has finished dispatching.
    fn refresh(&mut self, _ui: &mut Ui<'_, '_>) {}
}

/// Safe mutations for the loaded GFOpenLayout tree.
#[cfg(target_arch = "m68k")]
pub struct Ui<'window, 'runtime> {
    layout: &'window OpenLibrary<'runtime>,
    ui: NonNull<c_void>,
    needs_layout: bool,
}

#[cfg(target_arch = "m68k")]
impl Ui<'_, '_> {
    fn find(&self, id: &CStr) -> Option<NonNull<c_void>> {
        NonNull::new(unsafe {
            self.layout.call_a0_a1(
                LVO_OLUI_FIND,
                self.ui.as_ptr(),
                id.as_ptr().cast_mut().cast(),
            ) as usize as *mut c_void
        })
    }

    pub fn set_text(&mut self, id: &CStr, text: &CStr) -> bool {
        let Some(element) = self.find(id) else {
            return false;
        };
        unsafe {
            self.layout.call_a0_a1_a2_a3_d0(
                LVO_OLUI_SET_TEXT,
                self.ui.as_ptr(),
                element.as_ptr(),
                text.as_ptr().cast_mut().cast(),
                core::ptr::null_mut(),
                0,
            ) != 0
        }
    }

    pub fn set_style(&mut self, id: &CStr, declarations: &CStr) -> bool {
        let Some(element) = self.find(id) else {
            return false;
        };
        let changed = unsafe {
            self.layout.call_a0_a1_a2_a3_d0(
                LVO_OLUI_SET_STYLE,
                self.ui.as_ptr(),
                element.as_ptr(),
                declarations.as_ptr().cast_mut().cast(),
                core::ptr::null_mut(),
                0,
            ) != 0
        };
        self.needs_layout |= changed;
        changed
    }

    pub fn set_hidden(&mut self, id: &CStr, hidden: bool) -> bool {
        let Some(element) = self.find(id) else {
            return false;
        };
        unsafe {
            self.layout.call_a0_a1_d0(
                LVO_OLUI_SET_HIDDEN,
                self.ui.as_ptr(),
                element.as_ptr(),
                u32::from(hidden),
            );
        }
        true
    }

    pub fn set_disabled(&mut self, id: &CStr, disabled: bool) -> bool {
        let Some(element) = self.find(id) else {
            return false;
        };
        unsafe {
            self.layout.call_a0_a1_d0(
                LVO_OLUI_SET_DISABLED,
                self.ui.as_ptr(),
                element.as_ptr(),
                u32::from(disabled),
            );
        }
        self.needs_layout = true;
        true
    }

    pub fn set_zoom(&mut self, percent: u16) {
        unsafe {
            self.layout.call_a0_d0(
                LVO_OLUI_SET_ZOOM,
                self.ui.as_ptr(),
                u32::from(percent.clamp(25, 400)),
            );
        }
        self.needs_layout = true;
    }
}

const EVENT_ID_CAPACITY: usize = 64;
const EVENT_VALUE_CAPACITY: usize = 192;
const EVENT_HANDLER_CAPACITY: usize = 32;

#[derive(Clone, Copy)]
#[cfg(target_arch = "m68k")]
struct PendingEvent {
    ready: bool,
    event_type: i32,
    registration: usize,
    id: [u8; EVENT_ID_CAPACITY],
    id_len: usize,
    value: [u8; EVENT_VALUE_CAPACITY],
    value_len: usize,
    has_value: bool,
    index: i32,
    checked: bool,
}

#[cfg(target_arch = "m68k")]
struct EventMailbox {
    event: PendingEvent,
}

#[derive(Clone, Copy)]
#[cfg(target_arch = "m68k")]
struct HandlerContext {
    mailbox: *mut EventMailbox,
    id: *const core::ffi::c_char,
    registration: usize,
}

#[cfg(target_arch = "m68k")]
impl HandlerContext {
    const fn new() -> Self {
        Self {
            mailbox: core::ptr::null_mut(),
            id: core::ptr::null(),
            registration: 0,
        }
    }
}

#[cfg(target_arch = "m68k")]
static mut EVENT_MAILBOX: EventMailbox = EventMailbox::new();
#[cfg(target_arch = "m68k")]
static mut HANDLER_CONTEXTS: [HandlerContext; EVENT_HANDLER_CAPACITY] =
    [HandlerContext::new(); EVENT_HANDLER_CAPACITY];

#[cfg(target_arch = "m68k")]
impl EventMailbox {
    const fn new() -> Self {
        Self {
            event: PendingEvent::new(),
        }
    }

    fn push(&mut self, event: PendingEvent) -> bool {
        let ready = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(self.event.ready)) };
        if ready {
            return false;
        }
        unsafe {
            core::ptr::write_volatile(core::ptr::addr_of_mut!(self.event), event);
        }
        true
    }

    fn dispatch_all<S: EventSink>(&mut self, sink: &mut S) -> bool {
        let ready = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(self.event.ready)) };
        if !ready {
            return false;
        }
        self.event.dispatch(sink)
    }
}

#[cfg(target_arch = "m68k")]
impl PendingEvent {
    const fn new() -> Self {
        Self {
            ready: false,
            event_type: 0,
            registration: 0,
            id: [0; EVENT_ID_CAPACITY],
            id_len: 0,
            value: [0; EVENT_VALUE_CAPACITY],
            value_len: 0,
            has_value: false,
            index: 0,
            checked: false,
        }
    }

    fn dispatch<S: EventSink>(&mut self, sink: &mut S) -> bool {
        if !self.ready {
            return false;
        }
        self.ready = false;
        if self.id_len >= self.id.len() {
            return false;
        }
        let id = unsafe {
            CStr::from_bytes_with_nul_unchecked(core::slice::from_raw_parts(
                self.id.as_ptr(),
                self.id_len + 1,
            ))
        };
        let value = if self.has_value && self.value_len < self.value.len() {
            Some(unsafe {
                CStr::from_bytes_with_nul_unchecked(core::slice::from_raw_parts(
                    self.value.as_ptr(),
                    self.value_len + 1,
                ))
            })
        } else {
            None
        };
        let kind = match self.event_type {
            OLUI_CLICK => EventKind::Click,
            OLUI_CHANGE => EventKind::Change,
            other => EventKind::Other(other),
        };
        sink.on_event(Event {
            kind,
            id,
            registration: self.registration,
            value,
            index: self.index,
            checked: self.checked,
        })
    }
}

#[cfg(target_arch = "m68k")]
fn copy_event_text(source: *const core::ffi::c_char, target: &mut [u8]) -> Option<usize> {
    if source.is_null() || target.is_empty() {
        return None;
    }
    let length = unsafe { gf_copy_cstr_bounded(source, target.as_mut_ptr(), target.len()) };
    (length != usize::MAX).then_some(length)
}

/// Fixed callback entry point used by GFOpenLayout.
///
/// Keeping this symbol non-generic is important for Amiga Hunk linkers: the
/// callback address remains a normal exported relocation instead of a hidden
/// monomorphised Rust symbol. Events are copied into bounded storage and are
/// dispatched to application code only after GFOpenLayout returns.
#[cfg(target_arch = "m68k")]
#[unsafe(no_mangle)]
unsafe extern "C" fn gfopenlayout_event_trampoline(
    _ui: *mut c_void,
    event: *const RawEvent,
    userdata: *mut c_void,
) {
    let Some(context) = (unsafe { userdata.cast::<HandlerContext>().as_ref() }) else {
        return;
    };
    let Some(mailbox) = (unsafe { context.mailbox.as_mut() }) else {
        return;
    };
    let Some(raw) = (unsafe { event.as_ref() }) else {
        return;
    };
    let mut pending = PendingEvent::new();
    let mut id_source = context.id;
    if id_source.is_null() {
        id_source = raw.id;
    }
    let Some(id_len) = copy_event_text(id_source, &mut pending.id) else {
        return;
    };
    pending.id_len = id_len;
    pending.registration = context.registration;
    pending.event_type = if context.id.is_null() {
        raw.event_type
    } else {
        OLUI_CLICK
    };
    pending.has_value = false;
    // A click does not carry a value.  Some older GFOpenLayout builds leave
    // that field unspecified for value-less events, so only dereference it
    // for the change events whose API contract actually supplies text.
    if pending.event_type == OLUI_CHANGE {
        if let Some(value_len) = copy_event_text(raw.value, &mut pending.value) {
            pending.value_len = value_len;
            pending.has_value = true;
        }
    }
    pending.index = raw.index;
    pending.checked = raw.checked != 0;
    pending.ready = true;
    let _ = mailbox.push(pending);
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
                data: WINDOW_IDCMP,
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
        // OpenWindowTagList is expected to install this IDCMP mask from
        // WA_IDCMP.  Apply it explicitly as well: this both documents the
        // runtime contract and covers Intuition variants which create the
        // UserPort only when ModifyIDCMP is called after the window exists.
        if unsafe { intuition.call_a0_d0(LVO_MODIFY_IDCMP, window.as_ptr(), WINDOW_IDCMP) } == 0 {
            unsafe { intuition.call_a0(LVO_CLOSE_WINDOW, window.as_ptr()) };
            return Err(OpenError::WindowUnavailable);
        }
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

    /// Change the native window title without rebuilding the layout tree.
    pub fn set_title(&self, title: &CStr) {
        unsafe {
            self.intuition.call_a0_a1_a2_a3_d0(
                -276,
                self.window.as_ptr(),
                title.as_ptr().cast_mut().cast(),
                usize::MAX as *mut c_void,
                core::ptr::null_mut(),
                0,
            );
        }
    }

    /// Run the Intuition event loop until the close gadget is selected.
    pub fn run(&mut self, runtime: &Runtime) {
        let mut closed = false;
        let signal = unsafe { gf_window_signal_mask(self.window.as_ptr()) };
        while !closed {
            if signal == 0 {
                runtime.delay(1);
            } else {
                runtime.wait(signal);
            }
            loop {
                let message = unsafe { gf_window_get_message(self.window.as_ptr()) };
                if message.is_null() {
                    break;
                }
                let result = unsafe {
                    self.layout
                        .call_a0_a1(LVO_OLUI_AMIGA_EVENT, self.glue.as_ptr(), message)
                };
                unsafe { gf_reply_message(message) };
                if result == OLUI_AMIGA_CLOSE {
                    closed = true;
                }
            }
            unsafe { self.layout.call_a0(LVO_OLUI_AMIGA_DRAW, self.glue.as_ptr()) };
        }
    }

    /// Run until close while safely forwarding button clicks to application state.
    ///
    /// The window is consumed so GFOpenLayout cannot retain a callback to a sink
    /// after the sink's borrow ends.
    pub fn run_with<S: EventSink>(self, runtime: &Runtime, sink: &mut S) -> bool {
        // GFOpenLayout retains its userdata until the window closes. AmigaOS
        // delivers this process's window events serially, so a fixed mailbox
        // keeps both the callback and the polling loop on one stable address.
        let pending = core::ptr::addr_of_mut!(EVENT_MAILBOX);
        unsafe {
            pending.write(EventMailbox::new());
        }
        let click_ids = sink.click_ids();
        if click_ids.len() > EVENT_HANDLER_CAPACITY {
            return false;
        }
        let registrations = click_ids.len().max(1);
        for index in 0..registrations {
            let id = click_ids
                .get(index)
                .map_or(core::ptr::null(), |id| id.as_ptr());
            let context = unsafe { core::ptr::addr_of_mut!(HANDLER_CONTEXTS[index]) };
            unsafe {
                context.write(HandlerContext {
                    mailbox: pending,
                    id,
                    registration: index,
                });
            }
            let registered = unsafe {
                gf_openlayout_register_event_handler(
                    self.layout.raw_base(),
                    self.ui.as_ptr(),
                    id,
                    context.cast(),
                )
            };
            if registered == 0 {
                return false;
            }
        }
        let mut closed = false;
        let signal = unsafe { gf_window_signal_mask(self.window.as_ptr()) };
        while !closed {
            if signal == 0 {
                runtime.delay(1);
            } else {
                runtime.wait(signal);
            }
            let mut refresh = false;
            loop {
                let message = unsafe { gf_window_get_message(self.window.as_ptr()) };
                if message.is_null() {
                    break;
                }
                let result = unsafe {
                    self.layout
                        .call_a0_a1(LVO_OLUI_AMIGA_EVENT, self.glue.as_ptr(), message)
                };
                unsafe { gf_reply_message(message) };
                if result == OLUI_AMIGA_CLOSE {
                    closed = true;
                }
                // GFOpenLayout invokes the registered callback synchronously
                // from OLUI_AmigaEvent. Drain the single bounded slot after
                // every replied message so a later message in the same batch
                // cannot overwrite or discard the preceding event. Application
                // UI mutation remains deferred until the batch is complete.
                refresh |= unsafe { (&mut *pending).dispatch_all(sink) };
            }
            if refresh {
                if let Some(title) = sink.window_title() {
                    self.set_title(title);
                }
            }
            if refresh {
                let needs_layout = {
                    let mut ui = Ui {
                        layout: &self.layout,
                        ui: self.ui,
                        needs_layout: false,
                    };
                    sink.refresh(&mut ui);
                    ui.needs_layout
                };
                if needs_layout {
                    unsafe {
                        self.layout
                            .call_a0(LVO_OLUI_AMIGA_LAYOUT, self.glue.as_ptr());
                    }
                }
            }
            // Match GFOpenLayout's native Amiga loop contract: every drained
            // Intuition batch is followed by a draw. Besides presenting damage,
            // this lets the library finish the press/focus state that a later
            // mouse-up message completes.
            unsafe {
                self.layout.call_a0(LVO_OLUI_AMIGA_DRAW, self.glue.as_ptr());
            }
        }
        true
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
        assert_ne!(WINDOW_FLAGS & WFLG_REPORTMOUSE, 0);
    }

    #[test]
    fn gfopenlayout_vectors_match_version_18_sfd() {
        assert_eq!(LVO_OLUI_NEW, -426);
        assert_eq!(LVO_OLUI_LOAD, -450);
        assert_eq!(LVO_OLUI_AMIGA_EVENT, -834);
    }
}
