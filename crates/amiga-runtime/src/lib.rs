// Copyright (c) 2026 AmiCradle Team. MIT.
//! Minimal runtime services for native AmigaOS 3.x Rust programs.
//!
//! The startup archive translates AmigaDOS's D0/A0 entry state into the
//! stack ABI expected by `gf_amiga_main`.  [`init`] opens dos.library and
//! creates the Exec pool used by the global allocator.  All owned values must
//! be dropped before [`shutdown`] deletes that pool.

#![no_std]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

#[cfg(target_arch = "m68k")]
core::arch::global_asm!(include_str!("amiga_start.S"));

#[cfg(target_arch = "m68k")]
use amiga_sys::{exec, APtr, LibraryBase, MEMF_ANY};
#[cfg(target_arch = "m68k")]
use core::alloc::GlobalAlloc;
#[cfg(any(target_arch = "m68k", test))]
use core::alloc::Layout;
#[cfg(target_arch = "m68k")]
use core::ffi::c_void;
#[cfg(target_arch = "m68k")]
use core::ffi::CStr;
#[cfg(target_arch = "m68k")]
use core::marker::PhantomData;

#[cfg(any(target_arch = "m68k", test))]
const BASE_ALIGNMENT: usize = 8;
#[cfg(any(target_arch = "m68k", test))]
const HEADER_SIZE: usize = core::mem::size_of::<usize>();

#[cfg(any(target_arch = "m68k", test))]
fn allocation_size(layout: Layout) -> Option<(usize, bool)> {
    let size = layout.size().max(1);
    if layout.align() <= BASE_ALIGNMENT {
        Some((size, false))
    } else {
        size.checked_add(layout.align())
            .and_then(|value| value.checked_add(HEADER_SIZE))
            .map(|value| (value, true))
    }
}

#[cfg(target_arch = "m68k")]
static mut SYS_BASE: APtr = core::ptr::null_mut();
#[cfg(target_arch = "m68k")]
static mut DOS_BASE: APtr = core::ptr::null_mut();
#[cfg(target_arch = "m68k")]
static mut POOL: APtr = core::ptr::null_mut();

#[cfg(any(target_arch = "m68k", test))]
unsafe fn copy_bytes(destination: *mut u8, source: *const u8, length: usize) {
    for offset in 0..length {
        unsafe {
            destination
                .add(offset)
                .write_volatile(source.add(offset).read_volatile());
        }
    }
}

#[cfg(any(target_arch = "m68k", test))]
unsafe fn fill_bytes(destination: *mut u8, value: u8, length: usize) {
    for offset in 0..length {
        unsafe { destination.add(offset).write_volatile(value) };
    }
}

/// Compiler memory-copy ABI used by freestanding native applications.
///
/// Keeping this in the shared runtime prevents every application from
/// exporting its own copy and avoids compiler-builtins relocations which the
/// Amiga Hunk linker cannot represent.
///
/// # Safety
/// The two ranges must be valid for `length` bytes and must not overlap.
#[cfg(target_arch = "m68k")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    destination: *mut c_void,
    source: *const c_void,
    length: usize,
) -> *mut c_void {
    unsafe { copy_bytes(destination.cast(), source.cast(), length) };
    destination
}

/// Compiler byte-fill ABI used by freestanding native applications.
///
/// # Safety
/// `destination` must be valid for `length` writable bytes.
#[cfg(target_arch = "m68k")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(
    destination: *mut c_void,
    value: i32,
    length: usize,
) -> *mut c_void {
    unsafe { fill_bytes(destination.cast(), value as u8, length) };
    destination
}

/// Initialise the program's DOS base and allocation pool.
///
/// # Safety
/// Call once, before any allocation or DOS call, from the program's initial
/// task.  No other task may use this runtime while it is initialising.
#[cfg(target_arch = "m68k")]
unsafe fn init() -> bool {
    if !POOL.is_null() {
        return true;
    }

    let sys = exec::sys_base();
    SYS_BASE = sys.as_ptr();
    DOS_BASE = exec::open_library(sys, b"dos.library\0".as_ptr().cast(), 36)
        .map_or(core::ptr::null_mut(), |dos| dos.as_ptr());

    // Use the universally available AllocMem path. CreatePool varies across
    // compatible Exec implementations; it can be re-enabled as an optional
    // optimisation once native probes cover every supported kernel.
    POOL = core::ptr::null_mut();
    true
}

/// Delete the allocation pool and close dos.library.
///
/// # Safety
/// Every Rust value that could own an allocation must be dropped first.  No
/// other task may be using this runtime.
#[cfg(target_arch = "m68k")]
unsafe fn shutdown() {
    let Some(sys) = LibraryBase::from_raw(SYS_BASE) else {
        return;
    };
    let pool = POOL;
    let dos = DOS_BASE;
    POOL = core::ptr::null_mut();
    DOS_BASE = core::ptr::null_mut();
    SYS_BASE = core::ptr::null_mut();
    if !pool.is_null() {
        exec::delete_pool(sys, pool);
    }
    if let Some(dos) = LibraryBase::from_raw(dos) {
        exec::close_library(sys, dos);
    }
}

/// Write a NUL-terminated string through dos.library.
///
/// # Safety
/// [`init`] must have succeeded and `string` must point to a readable,
/// NUL-terminated byte string.
#[cfg(target_arch = "m68k")]
unsafe fn put_str(string: *const u8) -> i32 {
    let Some(dos) = LibraryBase::from_raw(DOS_BASE) else {
        return -1;
    };
    amiga_sys::dos::put_str(dos, string.cast())
}

/// The live, single-task AmigaOS runtime.
///
/// Applications receive this only inside [`run`], so their allocations are
/// dropped before the underlying Exec pool is deleted.
#[cfg(target_arch = "m68k")]
pub struct Runtime {
    _private: (),
}

#[cfg(target_arch = "m68k")]
impl Runtime {
    fn start() -> Option<Self> {
        if unsafe { init() } {
            Some(Self { _private: () })
        } else {
            None
        }
    }

    /// Write one NUL-terminated string through dos.library.
    pub fn put_str(&self, string: &CStr) -> i32 {
        unsafe { put_str(string.as_ptr().cast()) }
    }

    /// Yield this task for a number of AmigaDOS ticks (normally 1/50 second).
    pub fn delay(&self, ticks: u32) {
        let Some(dos) = (unsafe { LibraryBase::from_raw(DOS_BASE) }) else {
            return;
        };
        unsafe { amiga_sys::dos::delay(dos, ticks) };
    }

    /// Open an Amiga shared library for no longer than this runtime callback.
    pub fn open_library<'runtime>(
        &'runtime self,
        name: &CStr,
        minimum_version: u32,
    ) -> Option<OpenLibrary<'runtime>> {
        let sys = unsafe { LibraryBase::from_raw(SYS_BASE) }?;
        let base = unsafe { exec::open_library(sys, name.as_ptr().cast(), minimum_version) }?;
        Some(OpenLibrary {
            sys,
            base,
            _runtime: PhantomData,
        })
    }

    /// Sleep until at least one signal in `signals` is received.
    pub fn wait(&self, signals: u32) -> u32 {
        let Some(sys) = (unsafe { LibraryBase::from_raw(SYS_BASE) }) else {
            return 0;
        };
        unsafe { exec::wait(sys, signals) }
    }

    /// Remove the next message from an Exec message port.
    ///
    /// # Safety
    /// `port` must point to a live `MsgPort` owned by this task.
    pub unsafe fn get_message(&self, port: *mut c_void) -> *mut c_void {
        let Some(sys) = LibraryBase::from_raw(SYS_BASE) else {
            return core::ptr::null_mut();
        };
        exec::get_msg(sys, port)
    }

    /// Reply an Exec message after the application has finished reading it.
    ///
    /// # Safety
    /// `message` must be a message removed from one of this task's ports and
    /// must not be accessed again after this call.
    pub unsafe fn reply_message(&self, message: *mut c_void) {
        let Some(sys) = LibraryBase::from_raw(SYS_BASE) else {
            return;
        };
        exec::reply_msg(sys, message);
    }

    /// Block until `port` contains at least one message.
    ///
    /// # Safety
    /// `port` must point to a live `MsgPort` owned by this task.
    pub unsafe fn wait_port(&self, port: *mut c_void) -> *mut c_void {
        let Some(sys) = LibraryBase::from_raw(SYS_BASE) else {
            return core::ptr::null_mut();
        };
        if port.is_null() {
            return core::ptr::null_mut();
        }

        exec::wait_port(sys, port)
    }
}

/// An open Amiga library tied to the runtime that owns Exec.
///
/// Typed binding crates use the unsafe register calls below, keeping raw
/// vectors and pointers out of application code.  Dropping the handle closes
/// the library before [`run`] tears the runtime down.
#[cfg(target_arch = "m68k")]
pub struct OpenLibrary<'runtime> {
    sys: LibraryBase,
    base: LibraryBase,
    _runtime: PhantomData<&'runtime Runtime>,
}

#[cfg(target_arch = "m68k")]
impl OpenLibrary<'_> {
    /// Raw library base for typed binding shims whose register contract cannot
    /// be represented by the generic vector helpers.
    pub fn raw_base(&self) -> *mut c_void {
        self.base.as_ptr()
    }

    /// # Safety
    /// `lvo` must have no arguments.
    pub unsafe fn call0(&self, lvo: i32) -> u32 {
        amiga_sys::library::call0(self.base, lvo)
    }

    /// # Safety
    /// `lvo` must have the `a0,a1` signature and every argument must meet
    /// the vector's contract.
    pub unsafe fn call_a0_a1(&self, lvo: i32, a0: *mut c_void, a1: *mut c_void) -> u32 {
        amiga_sys::library::call_a0_a1(self.base, lvo, a0, a1)
    }
    /// # Safety
    /// `lvo` must have the `a0` signature and `a0` must meet the vector's
    /// pointer contract.
    pub unsafe fn call_a0(&self, lvo: i32, a0: *mut c_void) -> u32 {
        amiga_sys::library::call_a0(self.base, lvo, a0)
    }

    /// # Safety
    /// `lvo` must have the `a0,d0` signature and every argument must meet the
    /// vector's contract.
    pub unsafe fn call_a0_d0(&self, lvo: i32, a0: *mut c_void, d0: u32) -> u32 {
        amiga_sys::library::call_a0_d0(self.base, lvo, a0, d0)
    }

    /// # Safety
    /// `lvo` must have the `a0,a1,d0` signature and every argument must meet
    /// the vector's contract.
    pub unsafe fn call_a0_a1_d0(&self, lvo: i32, a0: *mut c_void, a1: *mut c_void, d0: u32) -> u32 {
        amiga_sys::library::call_a0_a1_d0(self.base, lvo, a0, a1, d0)
    }

    /// # Safety
    /// `lvo` must have the `a0,a1,a2,a3,d0` signature and every argument
    /// must meet the vector's contract.
    pub unsafe fn call_a0_a1_a2_a3_d0(
        &self,
        lvo: i32,
        a0: *mut c_void,
        a1: *mut c_void,
        a2: *mut c_void,
        a3: *mut c_void,
        d0: u32,
    ) -> u32 {
        amiga_sys::library::call_a0_a1_a2_a3_d0(self.base, lvo, a0, a1, a2, a3, d0)
    }
}

#[cfg(target_arch = "m68k")]
impl Drop for OpenLibrary<'_> {
    fn drop(&mut self) {
        unsafe { exec::close_library(self.sys, self.base) };
    }
}

#[cfg(target_arch = "m68k")]
impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe { shutdown() };
    }
}

/// Initialise AmigaOS services, run `application`, then release them.
///
/// A failed DOS or pool initialisation returns AmigaDOS failure code 20.
/// The callback returns only an integer, so values allocated inside it are
/// dropped before the runtime and its pool.
#[cfg(target_arch = "m68k")]
pub fn run(application: impl FnOnce(&Runtime) -> i32) -> i32 {
    let Some(runtime) = Runtime::start() else {
        return 20;
    };
    application(&runtime)
}

pub struct ExecPoolAllocator;

#[cfg(target_arch = "m68k")]
unsafe impl GlobalAlloc for ExecPoolAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Some((size, has_header)) = allocation_size(layout) else {
            return core::ptr::null_mut();
        };
        let Ok(size32) = u32::try_from(size) else {
            return core::ptr::null_mut();
        };
        let Some(sys) = LibraryBase::from_raw(SYS_BASE) else {
            return core::ptr::null_mut();
        };
        let raw = if POOL.is_null() {
            exec::alloc_mem(sys, size32, MEMF_ANY)
        } else {
            exec::alloc_pooled(sys, POOL, size32)
        }
        .cast::<u8>();
        if raw.is_null() || !has_header {
            return raw;
        }
        let Some(address) = (raw as usize)
            .checked_add(HEADER_SIZE)
            .and_then(|value| value.checked_add(layout.align() - 1))
            .map(|value| value & !(layout.align() - 1))
        else {
            if POOL.is_null() {
                exec::free_mem(sys, raw.cast(), size32);
            } else {
                exec::free_pooled(sys, POOL, raw.cast(), size32);
            }
            return core::ptr::null_mut();
        };
        ((address - HEADER_SIZE) as *mut usize).write(raw as usize);
        address as *mut u8
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        let Some((size, has_header)) = allocation_size(layout) else {
            return;
        };
        let Ok(size32) = u32::try_from(size) else {
            return;
        };
        let Some(sys) = LibraryBase::from_raw(SYS_BASE) else {
            return;
        };
        if pointer.is_null() {
            return;
        }
        let raw = if has_header {
            ((pointer as usize - HEADER_SIZE) as *const usize).read() as *mut u8
        } else {
            pointer
        };
        if POOL.is_null() {
            exec::free_mem(sys, raw.cast(), size32);
        } else {
            exec::free_pooled(sys, POOL, raw.cast(), size32);
        }
    }
}

#[cfg(target_arch = "m68k")]
#[global_allocator]
static ALLOCATOR: ExecPoolAllocator = ExecPoolAllocator;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_allocations_use_the_pool_block_directly() {
        let layout = Layout::from_size_align(17, 8).unwrap();
        assert_eq!(allocation_size(layout), Some((17, false)));
    }

    #[test]
    fn aligned_allocations_reserve_a_back_pointer() {
        let layout = Layout::from_size_align(17, 64).unwrap();
        assert_eq!(allocation_size(layout), Some((17 + 64 + HEADER_SIZE, true)));
    }

    #[test]
    fn compiler_memory_copy_handles_non_word_lengths() {
        let source = [0x10, 0x20, 0x30, 0x40, 0x50];
        let mut destination = [0u8; 5];
        unsafe { copy_bytes(destination.as_mut_ptr(), source.as_ptr(), source.len()) };
        assert_eq!(destination, source);
    }

    #[test]
    fn compiler_memory_fill_keeps_the_low_byte() {
        let mut destination = [0u8; 7];
        unsafe { fill_bytes(destination.as_mut_ptr(), 0xa5, destination.len()) };
        assert_eq!(destination, [0xa5; 7]);
    }
}
