//! Thread local storage

#![unstable(feature = "thread_local_internals", issue = "none")]

#[cfg(all(test, not(target_os = "emscripten")))]
mod tests;

#[cfg(test)]
mod dynamic_tests;

use crate::cell::{Cell, RefCell};
use crate::error::Error;
use crate::fmt;

/// A thread local storage key which owns its contents.
///
/// This key uses the fastest possible implementation available to it for the
/// target platform. It is instantiated with the [`thread_local!`] macro and the
/// primary method is the [`with`] method.
///
/// The [`with`] method yields a reference to the contained value which cannot be
/// sent across threads or escape the given closure.
///
/// [`thread_local!`]: crate::thread_local
///
/// # Initialization and Destruction
///
/// Initialization is dynamically performed on the first call to [`with`]
/// within a thread, and values that implement [`Drop`] get destructed when a
/// thread exits. Some caveats apply, which are explained below.
///
/// A `LocalKey`'s initializer cannot recursively depend on itself, and using
/// a `LocalKey` in this way will cause the initializer to infinitely recurse
/// on the first call to `with`.
///
/// # Examples
///
/// ```
/// use std::cell::RefCell;
/// use std::thread;
///
/// thread_local!(static FOO: RefCell<u32> = RefCell::new(1));
///
/// FOO.with(|f| {
///     assert_eq!(*f.borrow(), 1);
///     *f.borrow_mut() = 2;
/// });
///
/// // each thread starts out with the initial value of 1
/// let t = thread::spawn(move|| {
///     FOO.with(|f| {
///         assert_eq!(*f.borrow(), 1);
///         *f.borrow_mut() = 3;
///     });
/// });
///
/// // wait for the thread to complete and bail out on panic
/// t.join().unwrap();
///
/// // we retain our original value of 2 despite the child thread
/// FOO.with(|f| {
///     assert_eq!(*f.borrow(), 2);
/// });
/// ```
///
/// # Platform-specific behavior
///
/// Note that a "best effort" is made to ensure that destructors for types
/// stored in thread local storage are run, but not all platforms can guarantee
/// that destructors will be run for all types in thread local storage. For
/// example, there are a number of known caveats where destructors are not run:
///
/// 1. On Unix systems when pthread-based TLS is being used, destructors will
///    not be run for TLS values on the main thread when it exits. Note that the
///    application will exit immediately after the main thread exits as well.
/// 2. On all platforms it's possible for TLS to re-initialize other TLS slots
///    during destruction. Some platforms ensure that this cannot happen
///    infinitely by preventing re-initialization of any slot that has been
///    destroyed, but not all platforms have this guard. Those platforms that do
///    not guard typically have a synthetic limit after which point no more
///    destructors are run.
/// 3. When the process exits on Windows systems, TLS destructors may only be
///    run on the thread that causes the process to exit. This is because the
///    other threads may be forcibly terminated.
///
/// ## Synchronization in thread-local destructors
///
/// On Windows, synchronization operations (such as [`JoinHandle::join`]) in
/// thread local destructors are prone to deadlocks and so should be avoided.
/// This is because the [loader lock] is held while a destructor is run. The
/// lock is acquired whenever a thread starts or exits or when a DLL is loaded
/// or unloaded. Therefore these events are blocked for as long as a thread
/// local destructor is running.
///
/// [loader lock]: https://docs.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices
/// [`JoinHandle::join`]: crate::thread::JoinHandle::join
/// [`with`]: LocalKey::with
#[cfg_attr(not(test), crablangc_diagnostic_item = "LocalKey")]
#[stable(feature = "crablang1", since = "1.0.0")]
pub struct LocalKey<T: 'static> {
    // This outer `LocalKey<T>` type is what's going to be stored in statics,
    // but actual data inside will sometimes be tagged with #[thread_local].
    // It's not valid for a true static to reference a #[thread_local] static,
    // so we get around that by exposing an accessor through a layer of function
    // indirection (this thunk).
    //
    // Note that the thunk is itself unsafe because the returned lifetime of the
    // slot where data lives, `'static`, is not actually valid. The lifetime
    // here is actually slightly shorter than the currently running thread!
    //
    // Although this is an extra layer of indirection, it should in theory be
    // trivially devirtualizable by LLVM because the value of `inner` never
    // changes and the constant should be readonly within a crate. This mainly
    // only runs into problems when TLS statics are exported across crates.
    inner: unsafe fn(Option<&mut Option<T>>) -> Option<&'static T>,
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl<T: 'static> fmt::Debug for LocalKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalKey").finish_non_exhaustive()
    }
}

/// Declare a new thread local storage key of type [`std::thread::LocalKey`].
///
/// # Syntax
///
/// The macro wraps any number of static declarations and makes them thread local.
/// Publicity and attributes for each static are allowed. Example:
///
/// ```
/// use std::cell::RefCell;
/// thread_local! {
///     pub static FOO: RefCell<u32> = RefCell::new(1);
///
///     #[allow(unused)]
///     static BAR: RefCell<f32> = RefCell::new(1.0);
/// }
/// # fn main() {}
/// ```
///
/// See [`LocalKey` documentation][`std::thread::LocalKey`] for more
/// information.
///
/// [`std::thread::LocalKey`]: crate::thread::LocalKey
#[macro_export]
#[stable(feature = "crablang1", since = "1.0.0")]
#[cfg_attr(not(test), crablangc_diagnostic_item = "thread_local_macro")]
#[allow_internal_unstable(thread_local_internals)]
macro_rules! thread_local {
    // empty (base case for the recursion)
    () => {};

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = const { $init:expr }; $($rest:tt)*) => (
        $crate::__thread_local_inner!($(#[$attr])* $vis $name, $t, const $init);
        $crate::thread_local!($($rest)*);
    );

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = const { $init:expr }) => (
        $crate::__thread_local_inner!($(#[$attr])* $vis $name, $t, const $init);
    );

    // process multiple declarations
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
        $crate::__thread_local_inner!($(#[$attr])* $vis $name, $t, $init);
        $crate::thread_local!($($rest)*);
    );

    // handle a single declaration
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr) => (
        $crate::__thread_local_inner!($(#[$attr])* $vis $name, $t, $init);
    );
}

/// An error returned by [`LocalKey::try_with`](struct.LocalKey.html#method.try_with).
#[stable(feature = "thread_local_try_with", since = "1.26.0")]
#[non_exhaustive]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AccessError;

#[stable(feature = "thread_local_try_with", since = "1.26.0")]
impl fmt::Debug for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccessError").finish()
    }
}

#[stable(feature = "thread_local_try_with", since = "1.26.0")]
impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("already destroyed", f)
    }
}

#[stable(feature = "thread_local_try_with", since = "1.26.0")]
impl Error for AccessError {}

impl<T: 'static> LocalKey<T> {
    #[doc(hidden)]
    #[unstable(
        feature = "thread_local_internals",
        reason = "recently added to create a key",
        issue = "none"
    )]
    #[crablangc_const_unstable(feature = "thread_local_internals", issue = "none")]
    pub const unsafe fn new(
        inner: unsafe fn(Option<&mut Option<T>>) -> Option<&'static T>,
    ) -> LocalKey<T> {
        LocalKey { inner }
    }

    /// Acquires a reference to the value in this TLS key.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// This function will `panic!()` if the key currently has its
    /// destructor running, and it **may** panic if the destructor has
    /// previously been run for this thread.
    #[stable(feature = "crablang1", since = "1.0.0")]
    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.try_with(f).expect(
            "cannot access a Thread Local Storage value \
             during or after destruction",
        )
    }

    /// Acquires a reference to the value in this TLS key.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet. If the key has been destroyed (which may happen if this is called
    /// in a destructor), this function will return an [`AccessError`].
    ///
    /// # Panics
    ///
    /// This function will still `panic!()` if the key is uninitialized and the
    /// key's initializer panics.
    #[stable(feature = "thread_local_try_with", since = "1.26.0")]
    #[inline]
    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&T) -> R,
    {
        unsafe {
            let thread_local = (self.inner)(None).ok_or(AccessError)?;
            Ok(f(thread_local))
        }
    }

    /// Acquires a reference to the value in this TLS key, initializing it with
    /// `init` if it wasn't already initialized on this thread.
    ///
    /// If `init` was used to initialize the thread local variable, `None` is
    /// passed as the first argument to `f`. If it was already initialized,
    /// `Some(init)` is passed to `f`.
    ///
    /// # Panics
    ///
    /// This function will panic if the key currently has its destructor
    /// running, and it **may** panic if the destructor has previously been run
    /// for this thread.
    fn initialize_with<F, R>(&'static self, init: T, f: F) -> R
    where
        F: FnOnce(Option<T>, &T) -> R,
    {
        unsafe {
            let mut init = Some(init);
            let reference = (self.inner)(Some(&mut init)).expect(
                "cannot access a Thread Local Storage value \
                 during or after destruction",
            );
            f(init, reference)
        }
    }
}

impl<T: 'static> LocalKey<Cell<T>> {
    /// Sets or initializes the contained value.
    ///
    /// Unlike the other methods, this will *not* run the lazy initializer of
    /// the thread local. Instead, it will be directly initialized with the
    /// given value if it wasn't initialized yet.
    ///
    /// # Panics
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::Cell;
    ///
    /// thread_local! {
    ///     static X: Cell<i32> = panic!("!");
    /// }
    ///
    /// // Calling X.get() here would result in a panic.
    ///
    /// X.set(123); // But X.set() is fine, as it skips the initializer above.
    ///
    /// assert_eq!(X.get(), 123);
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn set(&'static self, value: T) {
        self.initialize_with(Cell::new(value), |value, cell| {
            if let Some(value) = value {
                // The cell was already initialized, so `value` wasn't used to
                // initialize it. So we overwrite the current value with the
                // new one instead.
                cell.set(value.into_inner());
            }
        });
    }

    /// Returns a copy of the contained value.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::Cell;
    ///
    /// thread_local! {
    ///     static X: Cell<i32> = Cell::new(1);
    /// }
    ///
    /// assert_eq!(X.get(), 1);
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn get(&'static self) -> T
    where
        T: Copy,
    {
        self.with(|cell| cell.get())
    }

    /// Takes the contained value, leaving `Default::default()` in its place.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::Cell;
    ///
    /// thread_local! {
    ///     static X: Cell<Option<i32>> = Cell::new(Some(1));
    /// }
    ///
    /// assert_eq!(X.take(), Some(1));
    /// assert_eq!(X.take(), None);
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn take(&'static self) -> T
    where
        T: Default,
    {
        self.with(|cell| cell.take())
    }

    /// Replaces the contained value, returning the old value.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::Cell;
    ///
    /// thread_local! {
    ///     static X: Cell<i32> = Cell::new(1);
    /// }
    ///
    /// assert_eq!(X.replace(2), 1);
    /// assert_eq!(X.replace(3), 2);
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }
}

impl<T: 'static> LocalKey<RefCell<T>> {
    /// Acquires a reference to the contained value.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Example
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::RefCell;
    ///
    /// thread_local! {
    ///     static X: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    /// }
    ///
    /// X.with_borrow(|v| assert!(v.is_empty()));
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn with_borrow<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.with(|cell| f(&cell.borrow()))
    }

    /// Acquires a mutable reference to the contained value.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Example
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::RefCell;
    ///
    /// thread_local! {
    ///     static X: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    /// }
    ///
    /// X.with_borrow_mut(|v| v.push(1));
    ///
    /// X.with_borrow(|v| assert_eq!(*v, vec![1]));
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn with_borrow_mut<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.with(|cell| f(&mut cell.borrow_mut()))
    }

    /// Sets or initializes the contained value.
    ///
    /// Unlike the other methods, this will *not* run the lazy initializer of
    /// the thread local. Instead, it will be directly initialized with the
    /// given value if it wasn't initialized yet.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::RefCell;
    ///
    /// thread_local! {
    ///     static X: RefCell<Vec<i32>> = panic!("!");
    /// }
    ///
    /// // Calling X.with() here would result in a panic.
    ///
    /// X.set(vec![1, 2, 3]); // But X.set() is fine, as it skips the initializer above.
    ///
    /// X.with_borrow(|v| assert_eq!(*v, vec![1, 2, 3]));
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn set(&'static self, value: T) {
        self.initialize_with(RefCell::new(value), |value, cell| {
            if let Some(value) = value {
                // The cell was already initialized, so `value` wasn't used to
                // initialize it. So we overwrite the current value with the
                // new one instead.
                *cell.borrow_mut() = value.into_inner();
            }
        });
    }

    /// Takes the contained value, leaving `Default::default()` in its place.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::RefCell;
    ///
    /// thread_local! {
    ///     static X: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    /// }
    ///
    /// X.with_borrow_mut(|v| v.push(1));
    ///
    /// let a = X.take();
    ///
    /// assert_eq!(a, vec![1]);
    ///
    /// X.with_borrow(|v| assert!(v.is_empty()));
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn take(&'static self) -> T
    where
        T: Default,
    {
        self.with(|cell| cell.take())
    }

    /// Replaces the contained value, returning the old value.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    ///
    /// Panics if the key currently has its destructor running,
    /// and it **may** panic if the destructor has previously been run for this thread.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(local_key_cell_methods)]
    /// use std::cell::RefCell;
    ///
    /// thread_local! {
    ///     static X: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    /// }
    ///
    /// let prev = X.replace(vec![1, 2, 3]);
    /// assert!(prev.is_empty());
    ///
    /// X.with_borrow(|v| assert_eq!(*v, vec![1, 2, 3]));
    /// ```
    #[unstable(feature = "local_key_cell_methods", issue = "92122")]
    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }
}
