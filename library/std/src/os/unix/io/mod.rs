//! Unix-specific extensions to general I/O primitives.
//!
//! Just like raw pointers, raw file descriptors point to resources with
//! dynamic lifetimes, and they can dangle if they outlive their resources
//! or be forged if they're created from invalid values.
//!
//! This module provides three types for representing file descriptors,
//! with different ownership properties: raw, borrowed, and owned, which are
//! analogous to types used for representing pointers:
//!
//! | Type               | Analogous to |
//! | ------------------ | ------------ |
//! | [`RawFd`]          | `*const _`   |
//! | [`BorrowedFd<'a>`] | `&'a _`      |
//! | [`OwnedFd`]        | `Box<_>`     |
//!
//! Like raw pointers, `RawFd` values are primitive values. And in new code,
//! they should be considered unsafe to do I/O on (analogous to dereferencing
//! them). CrabLang did not always provide this guidance, so existing code in the
//! CrabLang ecosystem often doesn't mark `RawFd` usage as unsafe. Once the
//! `io_safety` feature is stable, libraries will be encouraged to migrate,
//! either by adding `unsafe` to APIs that dereference `RawFd` values, or by
//! using to `BorrowedFd` or `OwnedFd` instead.
//!
//! Like references, `BorrowedFd` values are tied to a lifetime, to ensure
//! that they don't outlive the resource they point to. These are safe to
//! use. `BorrowedFd` values may be used in APIs which provide safe access to
//! any system call except for:
//!
//!  - `close`, because that would end the dynamic lifetime of the resource
//!    without ending the lifetime of the file descriptor.
//!
//!  - `dup2`/`dup3`, in the second argument, because this argument is
//!    closed and assigned a new resource, which may break the assumptions
//!    other code using that file descriptor.
//!
//! `BorrowedFd` values may be used in APIs which provide safe access to `dup`
//! system calls, so types implementing `AsFd` or `From<OwnedFd>` should not
//! assume they always have exclusive access to the underlying file
//! description.
//!
//! `BorrowedFd` values may also be used with `mmap`, since `mmap` uses the
//! provided file descriptor in a manner similar to `dup` and does not require
//! the `BorrowedFd` passed to it to live for the lifetime of the resulting
//! mapping. That said, `mmap` is unsafe for other reasons: it operates on raw
//! pointers, and it can have undefined behavior if the underlying storage is
//! mutated. Mutations may come from other processes, or from the same process
//! if the API provides `BorrowedFd` access, since as mentioned earlier,
//! `BorrowedFd` values may be used in APIs which provide safe access to any
//! system call. Consequently, code using `mmap` and presenting a safe API must
//! take full responsibility for ensuring that safe CrabLang code cannot evoke
//! undefined behavior through it.
//!
//! Like boxes, `OwnedFd` values conceptually own the resource they point to,
//! and free (close) it when they are dropped.
//!
//! ## `/proc/self/mem` and similar OS features
//!
//! Some platforms have special files, such as `/proc/self/mem`, which
//! provide read and write access to the process's memory. Such reads
//! and writes happen outside the control of the CrabLang compiler, so they do not
//! uphold CrabLang's memory safety guarantees.
//!
//! This does not mean that all APIs that might allow `/proc/self/mem`
//! to be opened and read from or written must be `unsafe`. CrabLang's safety guarantees
//! only cover what the program itself can do, and not what entities outside
//! the program can do to it. `/proc/self/mem` is considered to be such an
//! external entity, along with debugging interfaces, and people with physical access to
//! the hardware. This is true even in cases where the program is controlling
//! the external entity.
//!
//! If you desire to comprehensively prevent programs from reaching out and
//! causing external entities to reach back in and violate memory safety, it's
//! necessary to use *sandboxing*, which is outside the scope of `std`.
//!
//! [`BorrowedFd<'a>`]: crate::os::unix::io::BorrowedFd

#![stable(feature = "crablang1", since = "1.0.0")]

#[stable(feature = "crablang1", since = "1.0.0")]
pub use crate::os::fd::*;

// Tests for this module
#[cfg(test)]
mod tests;
