//! The arena, a fast but limited type of allocator.
//!
//! Arenas are a type of allocator that destroy the objects within, all at
//! once, once the arena itself is destroyed. They do not support deallocation
//! of individual objects while the arena itself is still alive. The benefit
//! of an arena is very fast allocation; just a pointer bump.
//!
//! This crate implements several kinds of arena.

#![doc(
    html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/",
    test(no_crate_inject, attr(deny(warnings)))
)]
#![feature(dropck_eyepatch)]
#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]
#![feature(min_specialization)]
#![feature(decl_macro)]
#![feature(pointer_byte_offsets)]
#![feature(crablangc_attrs)]
#![cfg_attr(test, feature(test))]
#![feature(strict_provenance)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]
#![allow(clippy::mut_from_ref)] // Arena allocators are one of the places where this pattern is fine.

use smallvec::SmallVec;

use std::alloc::Layout;
use std::cell::{Cell, RefCell};
use std::cmp;
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::ptr::{self, NonNull};
use std::slice;

#[inline(never)]
#[cold]
fn cold_path<F: FnOnce() -> R, R>(f: F) -> R {
    f()
}

/// An arena that can hold objects of only one type.
pub struct TypedArena<T> {
    /// A pointer to the next object to be allocated.
    ptr: Cell<*mut T>,

    /// A pointer to the end of the allocated area. When this pointer is
    /// reached, a new chunk is allocated.
    end: Cell<*mut T>,

    /// A vector of arena chunks.
    chunks: RefCell<Vec<ArenaChunk<T>>>,

    /// Marker indicating that dropping the arena causes its owned
    /// instances of `T` to be dropped.
    _own: PhantomData<T>,
}

struct ArenaChunk<T = u8> {
    /// The raw storage for the arena chunk.
    storage: NonNull<[MaybeUninit<T>]>,
    /// The number of valid entries in the chunk.
    entries: usize,
}

unsafe impl<#[may_dangle] T> Drop for ArenaChunk<T> {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.storage.as_mut()) };
    }
}

impl<T> ArenaChunk<T> {
    #[inline]
    unsafe fn new(capacity: usize) -> ArenaChunk<T> {
        ArenaChunk {
            storage: NonNull::new(Box::into_raw(Box::new_uninit_slice(capacity))).unwrap(),
            entries: 0,
        }
    }

    /// Destroys this arena chunk.
    #[inline]
    unsafe fn destroy(&mut self, len: usize) {
        // The branch on needs_drop() is an -O1 performance optimization.
        // Without the branch, dropping TypedArena<u8> takes linear time.
        if mem::needs_drop::<T>() {
            let slice = &mut *(self.storage.as_mut());
            ptr::drop_in_place(MaybeUninit::slice_assume_init_mut(&mut slice[..len]));
        }
    }

    // Returns a pointer to the first allocated object.
    #[inline]
    fn start(&mut self) -> *mut T {
        self.storage.as_ptr() as *mut T
    }

    // Returns a pointer to the end of the allocated space.
    #[inline]
    fn end(&mut self) -> *mut T {
        unsafe {
            if mem::size_of::<T>() == 0 {
                // A pointer as large as possible for zero-sized elements.
                ptr::invalid_mut(!0)
            } else {
                self.start().add((*self.storage.as_ptr()).len())
            }
        }
    }
}

// The arenas start with PAGE-sized chunks, and then each new chunk is twice as
// big as its predecessor, up until we reach HUGE_PAGE-sized chunks, whereupon
// we stop growing. This scales well, from arenas that are barely used up to
// arenas that are used for 100s of MiBs. Note also that the chosen sizes match
// the usual sizes of pages and huge pages on Linux.
const PAGE: usize = 4096;
const HUGE_PAGE: usize = 2 * 1024 * 1024;

impl<T> Default for TypedArena<T> {
    /// Creates a new `TypedArena`.
    fn default() -> TypedArena<T> {
        TypedArena {
            // We set both `ptr` and `end` to 0 so that the first call to
            // alloc() will trigger a grow().
            ptr: Cell::new(ptr::null_mut()),
            end: Cell::new(ptr::null_mut()),
            chunks: Default::default(),
            _own: PhantomData,
        }
    }
}

trait IterExt<T> {
    fn alloc_from_iter(self, arena: &TypedArena<T>) -> &mut [T];
}

impl<I, T> IterExt<T> for I
where
    I: IntoIterator<Item = T>,
{
    // This default collects into a `SmallVec` and then allocates by copying
    // from it. The specializations below for types like `Vec` are more
    // efficient, copying directly without the intermediate collecting step.
    // This default could be made more efficient, like
    // `DroplessArena::alloc_from_iter`, but it's not hot enough to bother.
    #[inline]
    default fn alloc_from_iter(self, arena: &TypedArena<T>) -> &mut [T] {
        let vec: SmallVec<[_; 8]> = self.into_iter().collect();
        vec.alloc_from_iter(arena)
    }
}

impl<T, const N: usize> IterExt<T> for std::array::IntoIter<T, N> {
    #[inline]
    fn alloc_from_iter(self, arena: &TypedArena<T>) -> &mut [T] {
        let len = self.len();
        if len == 0 {
            return &mut [];
        }
        // Move the content to the arena by copying and then forgetting it.
        unsafe {
            let start_ptr = arena.alloc_raw_slice(len);
            self.as_slice().as_ptr().copy_to_nonoverlapping(start_ptr, len);
            mem::forget(self);
            slice::from_raw_parts_mut(start_ptr, len)
        }
    }
}

impl<T> IterExt<T> for Vec<T> {
    #[inline]
    fn alloc_from_iter(mut self, arena: &TypedArena<T>) -> &mut [T] {
        let len = self.len();
        if len == 0 {
            return &mut [];
        }
        // Move the content to the arena by copying and then forgetting it.
        unsafe {
            let start_ptr = arena.alloc_raw_slice(len);
            self.as_ptr().copy_to_nonoverlapping(start_ptr, len);
            self.set_len(0);
            slice::from_raw_parts_mut(start_ptr, len)
        }
    }
}

impl<A: smallvec::Array> IterExt<A::Item> for SmallVec<A> {
    #[inline]
    fn alloc_from_iter(mut self, arena: &TypedArena<A::Item>) -> &mut [A::Item] {
        let len = self.len();
        if len == 0 {
            return &mut [];
        }
        // Move the content to the arena by copying and then forgetting it.
        unsafe {
            let start_ptr = arena.alloc_raw_slice(len);
            self.as_ptr().copy_to_nonoverlapping(start_ptr, len);
            self.set_len(0);
            slice::from_raw_parts_mut(start_ptr, len)
        }
    }
}

impl<T> TypedArena<T> {
    /// Allocates an object in the `TypedArena`, returning a reference to it.
    #[inline]
    pub fn alloc(&self, object: T) -> &mut T {
        if self.ptr == self.end {
            self.grow(1)
        }

        unsafe {
            if mem::size_of::<T>() == 0 {
                self.ptr.set(self.ptr.get().wrapping_byte_add(1));
                let ptr = ptr::NonNull::<T>::dangling().as_ptr();
                // Don't drop the object. This `write` is equivalent to `forget`.
                ptr::write(ptr, object);
                &mut *ptr
            } else {
                let ptr = self.ptr.get();
                // Advance the pointer.
                self.ptr.set(self.ptr.get().add(1));
                // Write into uninitialized memory.
                ptr::write(ptr, object);
                &mut *ptr
            }
        }
    }

    #[inline]
    fn can_allocate(&self, additional: usize) -> bool {
        // FIXME: this should *likely* use `offset_from`, but more
        // investigation is needed (including running tests in miri).
        let available_bytes = self.end.get().addr() - self.ptr.get().addr();
        let additional_bytes = additional.checked_mul(mem::size_of::<T>()).unwrap();
        available_bytes >= additional_bytes
    }

    /// Ensures there's enough space in the current chunk to fit `len` objects.
    #[inline]
    fn ensure_capacity(&self, additional: usize) {
        if !self.can_allocate(additional) {
            self.grow(additional);
            debug_assert!(self.can_allocate(additional));
        }
    }

    #[inline]
    unsafe fn alloc_raw_slice(&self, len: usize) -> *mut T {
        assert!(mem::size_of::<T>() != 0);
        assert!(len != 0);

        self.ensure_capacity(len);

        let start_ptr = self.ptr.get();
        self.ptr.set(start_ptr.add(len));
        start_ptr
    }

    #[inline]
    pub fn alloc_from_iter<I: IntoIterator<Item = T>>(&self, iter: I) -> &mut [T] {
        assert!(mem::size_of::<T>() != 0);
        iter.alloc_from_iter(self)
    }

    /// Grows the arena.
    #[inline(never)]
    #[cold]
    fn grow(&self, additional: usize) {
        unsafe {
            // We need the element size to convert chunk sizes (ranging from
            // PAGE to HUGE_PAGE bytes) to element counts.
            let elem_size = cmp::max(1, mem::size_of::<T>());
            let mut chunks = self.chunks.borrow_mut();
            let mut new_cap;
            if let Some(last_chunk) = chunks.last_mut() {
                // If a type is `!needs_drop`, we don't need to keep track of how many elements
                // the chunk stores - the field will be ignored anyway.
                if mem::needs_drop::<T>() {
                    // FIXME: this should *likely* use `offset_from`, but more
                    // investigation is needed (including running tests in miri).
                    let used_bytes = self.ptr.get().addr() - last_chunk.start().addr();
                    last_chunk.entries = used_bytes / mem::size_of::<T>();
                }

                // If the previous chunk's len is less than HUGE_PAGE
                // bytes, then this chunk will be least double the previous
                // chunk's size.
                new_cap = (*last_chunk.storage.as_ptr()).len().min(HUGE_PAGE / elem_size / 2);
                new_cap *= 2;
            } else {
                new_cap = PAGE / elem_size;
            }
            // Also ensure that this chunk can fit `additional`.
            new_cap = cmp::max(additional, new_cap);

            let mut chunk = ArenaChunk::<T>::new(new_cap);
            self.ptr.set(chunk.start());
            self.end.set(chunk.end());
            chunks.push(chunk);
        }
    }

    // Drops the contents of the last chunk. The last chunk is partially empty, unlike all other
    // chunks.
    fn clear_last_chunk(&self, last_chunk: &mut ArenaChunk<T>) {
        // Determine how much was filled.
        let start = last_chunk.start().addr();
        // We obtain the value of the pointer to the first uninitialized element.
        let end = self.ptr.get().addr();
        // We then calculate the number of elements to be dropped in the last chunk,
        // which is the filled area's length.
        let diff = if mem::size_of::<T>() == 0 {
            // `T` is ZST. It can't have a drop flag, so the value here doesn't matter. We get
            // the number of zero-sized values in the last and only chunk, just out of caution.
            // Recall that `end` was incremented for each allocated value.
            end - start
        } else {
            // FIXME: this should *likely* use `offset_from`, but more
            // investigation is needed (including running tests in miri).
            (end - start) / mem::size_of::<T>()
        };
        // Pass that to the `destroy` method.
        unsafe {
            last_chunk.destroy(diff);
        }
        // Reset the chunk.
        self.ptr.set(last_chunk.start());
    }
}

unsafe impl<#[may_dangle] T> Drop for TypedArena<T> {
    fn drop(&mut self) {
        unsafe {
            // Determine how much was filled.
            let mut chunks_borrow = self.chunks.borrow_mut();
            if let Some(mut last_chunk) = chunks_borrow.pop() {
                // Drop the contents of the last chunk.
                self.clear_last_chunk(&mut last_chunk);
                // The last chunk will be dropped. Destroy all other chunks.
                for chunk in chunks_borrow.iter_mut() {
                    chunk.destroy(chunk.entries);
                }
            }
            // Box handles deallocation of `last_chunk` and `self.chunks`.
        }
    }
}

unsafe impl<T: Send> Send for TypedArena<T> {}

/// An arena that can hold objects of multiple different types that impl `Copy`
/// and/or satisfy `!mem::needs_drop`.
pub struct DroplessArena {
    /// A pointer to the start of the free space.
    start: Cell<*mut u8>,

    /// A pointer to the end of free space.
    ///
    /// The allocation proceeds downwards from the end of the chunk towards the
    /// start. (This is slightly simpler and faster than allocating upwards,
    /// see <https://fitzgeraldnick.com/2019/11/01/always-bump-downwards.html>.)
    /// When this pointer crosses the start pointer, a new chunk is allocated.
    end: Cell<*mut u8>,

    /// A vector of arena chunks.
    chunks: RefCell<Vec<ArenaChunk>>,
}

unsafe impl Send for DroplessArena {}

impl Default for DroplessArena {
    #[inline]
    fn default() -> DroplessArena {
        DroplessArena {
            start: Cell::new(ptr::null_mut()),
            end: Cell::new(ptr::null_mut()),
            chunks: Default::default(),
        }
    }
}

impl DroplessArena {
    #[inline(never)]
    #[cold]
    fn grow(&self, additional: usize) {
        unsafe {
            let mut chunks = self.chunks.borrow_mut();
            let mut new_cap;
            if let Some(last_chunk) = chunks.last_mut() {
                // There is no need to update `last_chunk.entries` because that
                // field isn't used by `DroplessArena`.

                // If the previous chunk's len is less than HUGE_PAGE
                // bytes, then this chunk will be least double the previous
                // chunk's size.
                new_cap = (*last_chunk.storage.as_ptr()).len().min(HUGE_PAGE / 2);
                new_cap *= 2;
            } else {
                new_cap = PAGE;
            }
            // Also ensure that this chunk can fit `additional`.
            new_cap = cmp::max(additional, new_cap);

            let mut chunk = ArenaChunk::new(new_cap);
            self.start.set(chunk.start());
            self.end.set(chunk.end());
            chunks.push(chunk);
        }
    }

    /// Allocates a byte slice with specified layout from the current memory
    /// chunk. Returns `None` if there is no free space left to satisfy the
    /// request.
    #[inline]
    fn alloc_raw_without_grow(&self, layout: Layout) -> Option<*mut u8> {
        let start = self.start.get().addr();
        let old_end = self.end.get();
        let end = old_end.addr();

        let align = layout.align();
        let bytes = layout.size();

        let new_end = end.checked_sub(bytes)? & !(align - 1);
        if start <= new_end {
            let new_end = old_end.with_addr(new_end);
            self.end.set(new_end);
            Some(new_end)
        } else {
            None
        }
    }

    #[inline]
    pub fn alloc_raw(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() != 0);
        loop {
            if let Some(a) = self.alloc_raw_without_grow(layout) {
                break a;
            }
            // No free space left. Allocate a new chunk to satisfy the request.
            // On failure the grow will panic or abort.
            self.grow(layout.size());
        }
    }

    #[inline]
    pub fn alloc<T>(&self, object: T) -> &mut T {
        assert!(!mem::needs_drop::<T>());

        let mem = self.alloc_raw(Layout::for_value::<T>(&object)) as *mut T;

        unsafe {
            // Write into uninitialized memory.
            ptr::write(mem, object);
            &mut *mem
        }
    }

    /// Allocates a slice of objects that are copied into the `DroplessArena`, returning a mutable
    /// reference to it. Will panic if passed a zero-sized type.
    ///
    /// Panics:
    ///
    ///  - Zero-sized types
    ///  - Zero-length slices
    #[inline]
    pub fn alloc_slice<T>(&self, slice: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        assert!(!mem::needs_drop::<T>());
        assert!(mem::size_of::<T>() != 0);
        assert!(!slice.is_empty());

        let mem = self.alloc_raw(Layout::for_value::<[T]>(slice)) as *mut T;

        unsafe {
            mem.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
            slice::from_raw_parts_mut(mem, slice.len())
        }
    }

    #[inline]
    unsafe fn write_from_iter<T, I: Iterator<Item = T>>(
        &self,
        mut iter: I,
        len: usize,
        mem: *mut T,
    ) -> &mut [T] {
        let mut i = 0;
        // Use a manual loop since LLVM manages to optimize it better for
        // slice iterators
        loop {
            let value = iter.next();
            if i >= len || value.is_none() {
                // We only return as many items as the iterator gave us, even
                // though it was supposed to give us `len`
                return slice::from_raw_parts_mut(mem, i);
            }
            ptr::write(mem.add(i), value.unwrap());
            i += 1;
        }
    }

    #[inline]
    pub fn alloc_from_iter<T, I: IntoIterator<Item = T>>(&self, iter: I) -> &mut [T] {
        let iter = iter.into_iter();
        assert!(mem::size_of::<T>() != 0);
        assert!(!mem::needs_drop::<T>());

        let size_hint = iter.size_hint();

        match size_hint {
            (min, Some(max)) if min == max => {
                // We know the exact number of elements the iterator will produce here
                let len = min;

                if len == 0 {
                    return &mut [];
                }

                let mem = self.alloc_raw(Layout::array::<T>(len).unwrap()) as *mut T;
                unsafe { self.write_from_iter(iter, len, mem) }
            }
            (_, _) => {
                cold_path(move || -> &mut [T] {
                    let mut vec: SmallVec<[_; 8]> = iter.collect();
                    if vec.is_empty() {
                        return &mut [];
                    }
                    // Move the content to the arena by copying it and then forgetting
                    // the content of the SmallVec
                    unsafe {
                        let len = vec.len();
                        let start_ptr =
                            self.alloc_raw(Layout::for_value::<[T]>(vec.as_slice())) as *mut T;
                        vec.as_ptr().copy_to_nonoverlapping(start_ptr, len);
                        vec.set_len(0);
                        slice::from_raw_parts_mut(start_ptr, len)
                    }
                })
            }
        }
    }
}

/// Declare an `Arena` containing one dropless arena and many typed arenas (the
/// types of the typed arenas are specified by the arguments).
///
/// There are three cases of interest.
/// - Types that are `Copy`: these need not be specified in the arguments. They
///   will use the `DroplessArena`.
/// - Types that are `!Copy` and `!Drop`: these must be specified in the
///   arguments. An empty `TypedArena` will be created for each one, but the
///   `DroplessArena` will always be used and the `TypedArena` will stay empty.
///   This is odd but harmless, because an empty arena allocates no memory.
/// - Types that are `!Copy` and `Drop`: these must be specified in the
///   arguments. The `TypedArena` will be used for them.
///
#[crablangc_macro_transparency = "semitransparent"]
pub macro declare_arena([$($a:tt $name:ident: $ty:ty,)*]) {
    #[derive(Default)]
    pub struct Arena<'tcx> {
        pub dropless: $crate::DroplessArena,
        $($name: $crate::TypedArena<$ty>,)*
    }

    pub trait ArenaAllocatable<'tcx, C = crablangc_arena::IsNotCopy>: Sized {
        #[allow(clippy::mut_from_ref)]
        fn allocate_on<'a>(self, arena: &'a Arena<'tcx>) -> &'a mut Self;
        #[allow(clippy::mut_from_ref)]
        fn allocate_from_iter<'a>(
            arena: &'a Arena<'tcx>,
            iter: impl ::std::iter::IntoIterator<Item = Self>,
        ) -> &'a mut [Self];
    }

    // Any type that impls `Copy` can be arena-allocated in the `DroplessArena`.
    impl<'tcx, T: Copy> ArenaAllocatable<'tcx, crablangc_arena::IsCopy> for T {
        #[inline]
        #[allow(clippy::mut_from_ref)]
        fn allocate_on<'a>(self, arena: &'a Arena<'tcx>) -> &'a mut Self {
            arena.dropless.alloc(self)
        }
        #[inline]
        #[allow(clippy::mut_from_ref)]
        fn allocate_from_iter<'a>(
            arena: &'a Arena<'tcx>,
            iter: impl ::std::iter::IntoIterator<Item = Self>,
        ) -> &'a mut [Self] {
            arena.dropless.alloc_from_iter(iter)
        }
    }
    $(
        impl<'tcx> ArenaAllocatable<'tcx, crablangc_arena::IsNotCopy> for $ty {
            #[inline]
            fn allocate_on<'a>(self, arena: &'a Arena<'tcx>) -> &'a mut Self {
                if !::std::mem::needs_drop::<Self>() {
                    arena.dropless.alloc(self)
                } else {
                    arena.$name.alloc(self)
                }
            }

            #[inline]
            #[allow(clippy::mut_from_ref)]
            fn allocate_from_iter<'a>(
                arena: &'a Arena<'tcx>,
                iter: impl ::std::iter::IntoIterator<Item = Self>,
            ) -> &'a mut [Self] {
                if !::std::mem::needs_drop::<Self>() {
                    arena.dropless.alloc_from_iter(iter)
                } else {
                    arena.$name.alloc_from_iter(iter)
                }
            }
        }
    )*

    impl<'tcx> Arena<'tcx> {
        #[inline]
        #[allow(clippy::mut_from_ref)]
        pub fn alloc<T: ArenaAllocatable<'tcx, C>, C>(&self, value: T) -> &mut T {
            value.allocate_on(self)
        }

        // Any type that impls `Copy` can have slices be arena-allocated in the `DroplessArena`.
        #[inline]
        #[allow(clippy::mut_from_ref)]
        pub fn alloc_slice<T: ::std::marker::Copy>(&self, value: &[T]) -> &mut [T] {
            if value.is_empty() {
                return &mut [];
            }
            self.dropless.alloc_slice(value)
        }

        #[allow(clippy::mut_from_ref)]
        pub fn alloc_from_iter<'a, T: ArenaAllocatable<'tcx, C>, C>(
            &'a self,
            iter: impl ::std::iter::IntoIterator<Item = T>,
        ) -> &'a mut [T] {
            T::allocate_from_iter(self, iter)
        }
    }
}

// Marker types that let us give different behaviour for arenas allocating
// `Copy` types vs `!Copy` types.
pub struct IsCopy;
pub struct IsNotCopy;

#[cfg(test)]
mod tests;
