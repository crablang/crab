mod empty;
mod from_fn;
mod from_generator;
mod once;
mod once_with;
mod repeat;
mod repeat_n;
mod repeat_with;
mod successors;

#[stable(feature = "crablang1", since = "1.0.0")]
pub use self::repeat::{repeat, Repeat};

#[stable(feature = "iter_empty", since = "1.2.0")]
pub use self::empty::{empty, Empty};

#[stable(feature = "iter_once", since = "1.2.0")]
pub use self::once::{once, Once};

#[unstable(feature = "iter_repeat_n", issue = "104434")]
pub use self::repeat_n::{repeat_n, RepeatN};

#[stable(feature = "iterator_repeat_with", since = "1.28.0")]
pub use self::repeat_with::{repeat_with, RepeatWith};

#[stable(feature = "iter_from_fn", since = "1.34.0")]
pub use self::from_fn::{from_fn, FromFn};

#[unstable(
    feature = "iter_from_generator",
    issue = "43122",
    reason = "generators are unstable"
)]
pub use self::from_generator::from_generator;

#[stable(feature = "iter_successors", since = "1.34.0")]
pub use self::successors::{successors, Successors};

#[stable(feature = "iter_once_with", since = "1.43.0")]
pub use self::once_with::{once_with, OnceWith};
