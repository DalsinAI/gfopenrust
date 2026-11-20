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
use core::ffi::CStr;

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
    let Some(dos) = exec::open_library(sys, b"dos.library\0".as_ptr().cast(), 36) else {
        SYS_BASE = core::ptr::null_mut();
        return false;
    };
    DOS_BASE = dos.as_ptr();

    POOL = exec::create_pool(sys, MEMF_ANY, 8192, 2048);
    if POOL.is_null() {
        exec::close_library(sys, dos);
        DOS_BASE = core::ptr::null_mut();
        SYS_BASE = core::ptr::null_mut();
        return false;
    }
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
        if POOL.is_null() {
            return core::ptr::null_mut();
        }
        let raw = exec::alloc_pooled(sys, POOL, size32).cast::<u8>();
        if raw.is_null() || !has_header {
            return raw;
        }
        let Some(address) = (raw as usize)
            .checked_add(HEADER_SIZE)
            .and_then(|value| value.checked_add(layout.align() - 1))
            .map(|value| value & !(layout.align() - 1))
        else {
            exec::free_pooled(sys, POOL, raw.cast(), size32);
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
        if pointer.is_null() || POOL.is_null() {
            return;
        }
        let raw = if has_header {
            ((pointer as usize - HEADER_SIZE) as *const usize).read() as *mut u8
        } else {
            pointer
        };
        exec::free_pooled(sys, POOL, raw.cast(), size32);
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
}
