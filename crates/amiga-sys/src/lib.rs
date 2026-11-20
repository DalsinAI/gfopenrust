// Copyright (c) 2026 AmiCradle Team. MIT.
//! Raw, `no_std` AmigaOS 3.x interfaces for native m68k Rust programs.
//!
//! Amiga libraries use A6 as the library base and registers for arguments.
//! The assembly linked by this crate translates Rust's m68k SysV calls to
//! that ABI.  The wrappers are deliberately small and unsafe: ownership,
//! task lifetime and pointer validity remain the caller's responsibility.

#![no_std]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

use core::ffi::{c_char, c_void};
use core::ptr::NonNull;

#[cfg(target_arch = "m68k")]
core::arch::global_asm!(include_str!("amiga_calls.S"));

pub type Byte = i8;
pub type UByte = u8;
pub type Word = i16;
pub type UWord = u16;
pub type Long = i32;
pub type ULong = u32;
pub type BPtr = i32;
pub type APtr = *mut c_void;
pub type StrPtr = *mut c_char;
pub type ConstStrPtr = *const c_char;

pub const MEMF_ANY: ULong = 0;
pub const MEMF_PUBLIC: ULong = 1;
pub const MEMF_CLEAR: ULong = 0x0001_0000;

pub const MODE_OLDFILE: Long = 1005;
pub const MODE_NEWFILE: Long = 1006;
pub const MODE_READWRITE: Long = 1004;
pub const OFFSET_BEGINNING: Long = -1;
pub const OFFSET_CURRENT: Long = 0;
pub const OFFSET_END: Long = 1;

pub const ERROR_NO_FREE_STORE: Long = 103;
pub const ERROR_FILE_NOT_OBJECT: Long = 121;
pub const ERROR_OBJECT_NOT_FOUND: Long = 205;
pub const ERROR_OBJECT_WRONG_TYPE: Long = 212;
pub const ERROR_BAD_HUNK: Long = 235;

#[repr(C, packed(2))]
pub struct Node {
    pub succ: *mut Node,
    pub pred: *mut Node,
    pub node_type: UByte,
    pub priority: Byte,
    pub name: StrPtr,
}

#[repr(C, packed(2))]
pub struct Library {
    pub node: Node,
    pub flags: UByte,
    pub pad: UByte,
    pub neg_size: UWord,
    pub pos_size: UWord,
    pub version: UWord,
    pub revision: UWord,
    pub id_string: APtr,
    pub sum: ULong,
    pub open_count: UWord,
}

#[repr(C)]
pub struct SignalSemaphore {
    body: [UWord; 23],
}

#[cfg(target_arch = "m68k")]
const _: [(); 14] = [(); core::mem::size_of::<Node>()];
#[cfg(target_arch = "m68k")]
const _: [(); 34] = [(); core::mem::size_of::<Library>()];
#[cfg(target_arch = "m68k")]
const _: [(); 46] = [(); core::mem::size_of::<SignalSemaphore>()];

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct LibraryBase(NonNull<c_void>);

impl LibraryBase {
    /// # Safety
    /// `raw` must be an open Amiga library base that remains open for every
    /// call made through the returned value.
    pub unsafe fn from_raw(raw: APtr) -> Option<Self> {
        NonNull::new(raw).map(Self)
    }

    pub const fn as_ptr(self) -> APtr {
        self.0.as_ptr()
    }
}

#[cfg(target_arch = "m68k")]
extern "C" {
    fn gf_lib_call0(base: APtr, lvo: Long) -> ULong;
    fn gf_lib_call_d0_d1_a0_a1(
        base: APtr,
        lvo: Long,
        d0: ULong,
        d1: ULong,
        a0: APtr,
        a1: APtr,
    ) -> ULong;
    fn gf_lib_call_d0_d1_d2(base: APtr, lvo: Long, d0: ULong, d1: ULong, d2: ULong) -> ULong;
    fn gf_lib_call_a0(base: APtr, lvo: Long, a0: APtr) -> ULong;
    fn gf_lib_call_a0_d0(base: APtr, lvo: Long, a0: APtr, d0: ULong) -> ULong;
    fn gf_lib_call_a0_a1_d0(base: APtr, lvo: Long, a0: APtr, a1: APtr, d0: ULong) -> ULong;
    fn gf_lib_call_a0_a1_a2_a3_d0(
        base: APtr,
        lvo: Long,
        a0: APtr,
        a1: APtr,
        a2: APtr,
        a3: APtr,
        d0: ULong,
    ) -> ULong;
    fn gf_lib_call_a1(base: APtr, lvo: Long, a1: APtr) -> ULong;
    fn gf_dos_call_d1_d2_d3(base: APtr, lvo: Long, d1: ULong, d2: ULong, d3: ULong) -> ULong;
    fn gf_dos_call_d1_d2_d3_d4(
        base: APtr,
        lvo: Long,
        d1: ULong,
        d2: ULong,
        d3: ULong,
        d4: ULong,
    ) -> ULong;
}

#[cfg(target_arch = "m68k")]
#[inline]
unsafe fn lib_call0(base: LibraryBase, lvo: Long) -> ULong {
    gf_lib_call0(base.as_ptr(), lvo)
}

#[cfg(target_arch = "m68k")]
#[inline]
unsafe fn lib_call4(
    base: LibraryBase,
    lvo: Long,
    d0: ULong,
    d1: ULong,
    a0: APtr,
    a1: APtr,
) -> ULong {
    gf_lib_call_d0_d1_a0_a1(base.as_ptr(), lvo, d0, d1, a0, a1)
}

#[cfg(target_arch = "m68k")]
#[inline]
unsafe fn lib_call3d(base: LibraryBase, lvo: Long, d0: ULong, d1: ULong, d2: ULong) -> ULong {
    gf_lib_call_d0_d1_d2(base.as_ptr(), lvo, d0, d1, d2)
}

/// Raw register-shaped calls for typed Amiga library binding crates.
///
/// The LVO is negative (for example `-30` for a library's first public
/// vector).  The typed binding owns all pointer and register validation.
pub mod library {
    #[cfg(target_arch = "m68k")]
    use super::{
        gf_lib_call_a0, gf_lib_call_a0_a1_a2_a3_d0, gf_lib_call_a0_a1_d0, gf_lib_call_a0_d0,
        lib_call0, lib_call4, APtr, LibraryBase, Long, ULong,
    };

    /// # Safety
    /// `base` must remain open and `lvo` must name a vector with no arguments.
    #[cfg(target_arch = "m68k")]
    pub unsafe fn call0(base: LibraryBase, lvo: Long) -> ULong {
        lib_call0(base, lvo)
    }

    /// # Safety
    /// `base` must remain open, `lvo` must name a vector with exactly the
    /// `a0,a1` signature, and both pointers must meet that vector's contract.
    #[cfg(target_arch = "m68k")]
    pub unsafe fn call_a0_a1(base: LibraryBase, lvo: Long, a0: APtr, a1: APtr) -> ULong {
        lib_call4(base, lvo, 0, 0, a0, a1)
    }

    /// # Safety
    /// `base` must remain open, `lvo` must name a vector with exactly the
    /// `a0` signature, and `a0` must meet that vector's pointer contract.
    #[cfg(target_arch = "m68k")]
    pub unsafe fn call_a0(base: LibraryBase, lvo: Long, a0: APtr) -> ULong {
        gf_lib_call_a0(base.as_ptr(), lvo, a0)
    }

    /// # Safety
    /// `base` must remain open, `lvo` must name a vector with exactly the
    /// `a0,d0` signature, and every argument must meet its contract.
    #[cfg(target_arch = "m68k")]
    pub unsafe fn call_a0_d0(base: LibraryBase, lvo: Long, a0: APtr, d0: ULong) -> ULong {
        gf_lib_call_a0_d0(base.as_ptr(), lvo, a0, d0)
    }

    /// # Safety
    /// `base` must remain open, `lvo` must name a vector with exactly the
    /// `a0,a1,d0` signature, and every argument must meet its contract.
    #[cfg(target_arch = "m68k")]
    pub unsafe fn call_a0_a1_d0(
        base: LibraryBase,
        lvo: Long,
        a0: APtr,
        a1: APtr,
        d0: ULong,
    ) -> ULong {
        gf_lib_call_a0_a1_d0(base.as_ptr(), lvo, a0, a1, d0)
    }

    /// # Safety
    /// `base` must remain open, `lvo` must name a vector with exactly the
    /// `a0,a1,a2,a3,d0` signature, and every pointer must meet its contract.
    #[cfg(target_arch = "m68k")]
    pub unsafe fn call_a0_a1_a2_a3_d0(
        base: LibraryBase,
        lvo: Long,
        a0: APtr,
        a1: APtr,
        a2: APtr,
        a3: APtr,
        d0: ULong,
    ) -> ULong {
        gf_lib_call_a0_a1_a2_a3_d0(base.as_ptr(), lvo, a0, a1, a2, a3, d0)
    }
}

pub mod exec {
    use super::*;

    pub const LVO_FORBID: Long = -132;
    pub const LVO_PERMIT: Long = -138;
    pub const LVO_ALLOC_MEM: Long = -198;
    pub const LVO_FREE_MEM: Long = -210;
    pub const LVO_FIND_TASK: Long = -294;
    pub const LVO_WAIT: Long = -318;
    pub const LVO_GET_MSG: Long = -372;
    pub const LVO_REPLY_MSG: Long = -378;
    pub const LVO_WAIT_PORT: Long = -384;
    pub const LVO_CLOSE_LIBRARY: Long = -414;
    pub const LVO_OPEN_LIBRARY: Long = -552;
    pub const LVO_INIT_SEMAPHORE: Long = -558;
    pub const LVO_OBTAIN_SEMAPHORE: Long = -564;
    pub const LVO_RELEASE_SEMAPHORE: Long = -570;
    pub const LVO_CACHE_CLEAR_U: Long = -636;
    pub const LVO_CREATE_POOL: Long = -696;
    pub const LVO_DELETE_POOL: Long = -702;
    pub const LVO_ALLOC_POOLED: Long = -708;
    pub const LVO_FREE_POOLED: Long = -714;

    /// Read ExecBase from AmigaOS's documented absolute address 4.
    ///
    /// # Safety
    /// This is only valid in a running AmigaOS m68k process.
    #[cfg(target_arch = "m68k")]
    pub unsafe fn sys_base() -> LibraryBase {
        let raw = core::ptr::read_volatile(4usize as *const APtr);
        LibraryBase::from_raw(raw).expect("AmigaOS SysBase at address 4 was null")
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn open_library(
        sys_base: LibraryBase,
        name: ConstStrPtr,
        minimum_version: ULong,
    ) -> Option<LibraryBase> {
        let raw = lib_call4(
            sys_base,
            LVO_OPEN_LIBRARY,
            minimum_version,
            0,
            core::ptr::null_mut(),
            name.cast_mut().cast(),
        ) as usize as APtr;
        LibraryBase::from_raw(raw)
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn close_library(sys_base: LibraryBase, library: LibraryBase) {
        lib_call4(
            sys_base,
            LVO_CLOSE_LIBRARY,
            0,
            0,
            core::ptr::null_mut(),
            library.as_ptr(),
        );
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn alloc_mem(sys_base: LibraryBase, size: ULong, flags: ULong) -> APtr {
        lib_call4(
            sys_base,
            LVO_ALLOC_MEM,
            size,
            flags,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as usize as APtr
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn free_mem(sys_base: LibraryBase, memory: APtr, size: ULong) {
        lib_call4(
            sys_base,
            LVO_FREE_MEM,
            size,
            0,
            core::ptr::null_mut(),
            memory,
        );
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn create_pool(
        sys_base: LibraryBase,
        flags: ULong,
        puddle_size: ULong,
        threshold_size: ULong,
    ) -> APtr {
        lib_call3d(
            sys_base,
            LVO_CREATE_POOL,
            flags,
            puddle_size,
            threshold_size,
        ) as usize as APtr
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn delete_pool(sys_base: LibraryBase, pool: APtr) {
        lib_call4(sys_base, LVO_DELETE_POOL, 0, 0, pool, core::ptr::null_mut());
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn alloc_pooled(sys_base: LibraryBase, pool: APtr, size: ULong) -> APtr {
        lib_call4(
            sys_base,
            LVO_ALLOC_POOLED,
            size,
            0,
            pool,
            core::ptr::null_mut(),
        ) as usize as APtr
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn free_pooled(sys_base: LibraryBase, pool: APtr, memory: APtr, size: ULong) {
        lib_call4(sys_base, LVO_FREE_POOLED, size, 0, pool, memory);
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn forbid(sys_base: LibraryBase) {
        lib_call0(sys_base, LVO_FORBID);
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn permit(sys_base: LibraryBase) {
        lib_call0(sys_base, LVO_PERMIT);
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn wait(sys_base: LibraryBase, signals: ULong) -> ULong {
        lib_call3d(sys_base, LVO_WAIT, signals, 0, 0)
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn get_msg(sys_base: LibraryBase, port: APtr) -> APtr {
        gf_lib_call_a0(sys_base.as_ptr(), LVO_GET_MSG, port) as usize as APtr
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn reply_msg(sys_base: LibraryBase, message: APtr) {
        gf_lib_call_a1(sys_base.as_ptr(), LVO_REPLY_MSG, message);
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn wait_port(sys_base: LibraryBase, port: APtr) -> APtr {
        gf_lib_call_a0(sys_base.as_ptr(), LVO_WAIT_PORT, port) as usize as APtr
    }
}

pub mod dos {
    use super::*;

    pub const LVO_OPEN: Long = -30;
    pub const LVO_CLOSE: Long = -36;
    pub const LVO_READ: Long = -42;
    pub const LVO_WRITE: Long = -48;
    pub const LVO_SEEK: Long = -66;
    pub const LVO_IO_ERR: Long = -132;
    pub const LVO_SET_IO_ERR: Long = -462;
    pub const LVO_RUN_COMMAND: Long = -504;
    pub const LVO_PUT_STR: Long = -948;
    pub const LVO_VPRINTF: Long = -954;

    #[cfg(target_arch = "m68k")]
    #[inline]
    unsafe fn call3(dos_base: LibraryBase, lvo: Long, d1: ULong, d2: ULong, d3: ULong) -> ULong {
        gf_dos_call_d1_d2_d3(dos_base.as_ptr(), lvo, d1, d2, d3)
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn open(dos_base: LibraryBase, name: ConstStrPtr, mode: Long) -> BPtr {
        call3(dos_base, LVO_OPEN, name as usize as ULong, mode as ULong, 0) as BPtr
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn close(dos_base: LibraryBase, file: BPtr) -> Long {
        call3(dos_base, LVO_CLOSE, file as ULong, 0, 0) as Long
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn read(dos_base: LibraryBase, file: BPtr, buffer: APtr, length: ULong) -> Long {
        call3(
            dos_base,
            LVO_READ,
            file as ULong,
            buffer as usize as ULong,
            length,
        ) as Long
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn write(
        dos_base: LibraryBase,
        file: BPtr,
        buffer: *const c_void,
        length: ULong,
    ) -> Long {
        call3(
            dos_base,
            LVO_WRITE,
            file as ULong,
            buffer as usize as ULong,
            length,
        ) as Long
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn seek(dos_base: LibraryBase, file: BPtr, position: Long, mode: Long) -> Long {
        call3(
            dos_base,
            LVO_SEEK,
            file as ULong,
            position as ULong,
            mode as ULong,
        ) as Long
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn io_err(dos_base: LibraryBase) -> Long {
        call3(dos_base, LVO_IO_ERR, 0, 0, 0) as Long
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn set_io_err(dos_base: LibraryBase, code: Long) -> Long {
        call3(dos_base, LVO_SET_IO_ERR, code as ULong, 0, 0) as Long
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn put_str(dos_base: LibraryBase, string: ConstStrPtr) -> Long {
        call3(dos_base, LVO_PUT_STR, string as usize as ULong, 0, 0) as Long
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn vprintf(dos_base: LibraryBase, format: ConstStrPtr, args: *const ULong) -> Long {
        call3(
            dos_base,
            LVO_VPRINTF,
            format as usize as ULong,
            args as usize as ULong,
            0,
        ) as Long
    }

    #[cfg(target_arch = "m68k")]
    pub unsafe fn run_command(
        dos_base: LibraryBase,
        segment: BPtr,
        stack_size: ULong,
        args: ConstStrPtr,
        length: ULong,
    ) -> Long {
        gf_dos_call_d1_d2_d3_d4(
            dos_base.as_ptr(),
            LVO_RUN_COMMAND,
            segment as ULong,
            stack_size,
            args as usize as ULong,
            length,
        ) as Long
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_constants_match_the_amiga_abi() {
        assert_eq!(MEMF_CLEAR, 1 << 16);
        assert_eq!(MODE_OLDFILE, 1005);
        assert_eq!(OFFSET_BEGINNING, -1);
        assert_eq!(exec::LVO_OPEN_LIBRARY, -552);
        assert_eq!(exec::LVO_ALLOC_POOLED, -708);
        assert_eq!(dos::LVO_RUN_COMMAND, -504);
        assert_eq!(dos::LVO_PUT_STR, -948);
    }

    #[test]
    fn semaphore_storage_matches_exec() {
        assert_eq!(core::mem::size_of::<SignalSemaphore>(), 46);
        assert_eq!(core::mem::align_of::<SignalSemaphore>(), 2);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn library_layout_matches_the_32_bit_abi() {
        assert_eq!(core::mem::size_of::<Node>(), 14);
        assert_eq!(core::mem::size_of::<Library>(), 34);
        assert_eq!(core::mem::align_of::<Library>(), 2);
    }
}
