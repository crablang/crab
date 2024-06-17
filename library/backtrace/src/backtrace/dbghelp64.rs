//! Backtrace strategy for Windows `x86_64` and `aarch64` platforms.
//!
//! This module contains the ability to capture a backtrace on Windows using
//!  `RtlVirtualUnwind` to walk the stack one frame at a time. This function is much faster than using
//! `dbghelp!StackWalk*` because it does not load debug info to report inlined frames.
//! We still report inlined frames during symbolization by consulting the appropriate
//! `dbghelp` functions.

#![allow(bad_style)]

use super::super::windows::*;
use core::ffi::c_void;

#[derive(Clone, Copy)]
pub struct Frame {
    base_address: *mut c_void,
    ip: *mut c_void,
    sp: *mut c_void,
    #[cfg(not(target_env = "gnu"))]
    inline_context: Option<DWORD>,
}

// we're just sending around raw pointers and reading them, never interpreting
// them so this should be safe to both send and share across threads.
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    pub fn ip(&self) -> *mut c_void {
        self.ip
    }

    pub fn sp(&self) -> *mut c_void {
        self.sp
    }

    pub fn symbol_address(&self) -> *mut c_void {
        self.ip
    }

    pub fn module_base_address(&self) -> Option<*mut c_void> {
        Some(self.base_address)
    }

    #[cfg(not(target_env = "gnu"))]
    pub fn inline_context(&self) -> Option<DWORD> {
        self.inline_context
    }
}

#[repr(C, align(16))] // required by `CONTEXT`, is a FIXME in winapi right now
struct MyContext(CONTEXT);

#[cfg(any(target_arch = "x86_64", target_arch = "arm64ec"))]
impl MyContext {
    #[inline(always)]
    fn ip(&self) -> DWORD64 {
        self.0.Rip
    }

    #[inline(always)]
    fn sp(&self) -> DWORD64 {
        self.0.Rsp
    }
}

#[cfg(target_arch = "aarch64")]
impl MyContext {
    #[inline(always)]
    fn ip(&self) -> DWORD64 {
        self.0.Pc
    }

    #[inline(always)]
    fn sp(&self) -> DWORD64 {
        self.0.Sp
    }
}

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "arm64ec"
))]
#[inline(always)]
pub unsafe fn trace(cb: &mut dyn FnMut(&super::Frame) -> bool) {
    use core::ptr;

    // Capture the initial context to start walking from.
    let mut context = core::mem::zeroed::<MyContext>();
    RtlCaptureContext(&mut context.0);

    loop {
        let ip = context.ip();

        // The base address of the module containing the function will be stored here
        // when RtlLookupFunctionEntry returns successfully.
        let mut base = 0;
        let fn_entry = RtlLookupFunctionEntry(ip, &mut base, ptr::null_mut());
        if fn_entry.is_null() {
            // No function entry could be found - this may indicate a corrupt
            // stack or that a binary was unloaded (amongst other issues). Stop
            // walking and don't call the callback as we can't be confident in
            // this frame or the rest of the stack.
            break;
        }

        let frame = super::Frame {
            inner: Frame {
                base_address: base as *mut c_void,
                ip: ip as *mut c_void,
                sp: context.sp() as *mut c_void,
                #[cfg(not(target_env = "gnu"))]
                inline_context: None,
            },
        };

        // We've loaded all the info about the current frame, so now call the
        // callback.
        if !cb(&frame) {
            // Callback told us to stop, so we're done.
            break;
        }

        // Unwind to the next frame.
        let previous_ip = ip;
        let previous_sp = context.sp();
        let mut handler_data = 0usize;
        let mut establisher_frame = 0;
        RtlVirtualUnwind(
            0,
            base,
            ip,
            fn_entry,
            &mut context.0,
            ptr::addr_of_mut!(handler_data).cast::<PVOID>(),
            &mut establisher_frame,
            ptr::null_mut(),
        );

        // RtlVirtualUnwind indicates the end of the stack in two different ways:
        // * On x64, it sets the instruction pointer to 0.
        // * On ARM64, it leaves the context unchanged (easiest way to check is
        //   to see if the instruction and stack pointers are the same).
        // If we detect either of these, then unwinding is completed.
        let ip = context.ip();
        if ip == 0 || (ip == previous_ip && context.sp() == previous_sp) {
            break;
        }
    }
}
