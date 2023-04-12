//! Internal interface for communicating between a `proc_macro` client
//! (a proc macro crate) and a `proc_macro` server (a compiler front-end).
//!
//! Serialization (with C ABI buffers) and unique integer handles are employed
//! to allow safely interfacing between two copies of `proc_macro` built
//! (from the same source) by different compilers with potentially mismatching
//! CrabLang ABIs (e.g., stage0/bin/crablangc vs stage1/bin/crablangc during bootstrap).

#![deny(unsafe_code)]

pub use super::{Delimiter, Level, LineColumn, Spacing};
use std::fmt;
use std::hash::Hash;
use std::marker;
use std::mem;
use std::ops::Bound;
use std::panic;
use std::sync::atomic::AtomicUsize;
use std::sync::Once;
use std::thread;

/// Higher-order macro describing the server RPC API, allowing automatic
/// generation of type-safe CrabLang APIs, both client-side and server-side.
///
/// `with_api!(MySelf, my_self, my_macro)` expands to:
/// ```crablang,ignore (pseudo-code)
/// my_macro! {
///     // ...
///     Literal {
///         // ...
///         fn character(ch: char) -> MySelf::Literal;
///         // ...
///         fn span(my_self: &MySelf::Literal) -> MySelf::Span;
///         fn set_span(my_self: &mut MySelf::Literal, span: MySelf::Span);
///     },
///     // ...
/// }
/// ```
///
/// The first two arguments serve to customize the arguments names
/// and argument/return types, to enable several different usecases:
///
/// If `my_self` is just `self`, then each `fn` signature can be used
/// as-is for a method. If it's anything else (`self_` in practice),
/// then the signatures don't have a special `self` argument, and
/// can, therefore, have a different one introduced.
///
/// If `MySelf` is just `Self`, then the types are only valid inside
/// a trait or a trait impl, where the trait has associated types
/// for each of the API types. If non-associated types are desired,
/// a module name (`self` in practice) can be used instead of `Self`.
macro_rules! with_api {
    ($S:ident, $self:ident, $m:ident) => {
        $m! {
            FreeFunctions {
                fn drop($self: $S::FreeFunctions);
                fn track_env_var(var: &str, value: Option<&str>);
                fn track_path(path: &str);
            },
            TokenStream {
                fn drop($self: $S::TokenStream);
                fn clone($self: &$S::TokenStream) -> $S::TokenStream;
                fn is_empty($self: &$S::TokenStream) -> bool;
                fn expand_expr($self: &$S::TokenStream) -> Result<$S::TokenStream, ()>;
                fn from_str(src: &str) -> $S::TokenStream;
                fn to_string($self: &$S::TokenStream) -> String;
                fn from_token_tree(
                    tree: TokenTree<$S::Group, $S::Punct, $S::Ident, $S::Literal>,
                ) -> $S::TokenStream;
                fn concat_trees(
                    base: Option<$S::TokenStream>,
                    trees: Vec<TokenTree<$S::Group, $S::Punct, $S::Ident, $S::Literal>>,
                ) -> $S::TokenStream;
                fn concat_streams(
                    base: Option<$S::TokenStream>,
                    streams: Vec<$S::TokenStream>,
                ) -> $S::TokenStream;
                fn into_trees(
                    $self: $S::TokenStream
                ) -> Vec<TokenTree<$S::Group, $S::Punct, $S::Ident, $S::Literal>>;
            },
            Group {
                fn drop($self: $S::Group);
                fn clone($self: &$S::Group) -> $S::Group;
                fn new(delimiter: Delimiter, stream: Option<$S::TokenStream>) -> $S::Group;
                fn delimiter($self: &$S::Group) -> Delimiter;
                fn stream($self: &$S::Group) -> $S::TokenStream;
                fn span($self: &$S::Group) -> $S::Span;
                fn span_open($self: &$S::Group) -> $S::Span;
                fn span_close($self: &$S::Group) -> $S::Span;
                fn set_span($self: &mut $S::Group, span: $S::Span);
            },
            Punct {
                fn new(ch: char, spacing: Spacing) -> $S::Punct;
                fn as_char($self: $S::Punct) -> char;
                fn spacing($self: $S::Punct) -> Spacing;
                fn span($self: $S::Punct) -> $S::Span;
                fn with_span($self: $S::Punct, span: $S::Span) -> $S::Punct;
            },
            Ident {
                fn new(string: &str, span: $S::Span, is_raw: bool) -> $S::Ident;
                fn span($self: $S::Ident) -> $S::Span;
                fn with_span($self: $S::Ident, span: $S::Span) -> $S::Ident;
            },
            Literal {
                fn drop($self: $S::Literal);
                fn clone($self: &$S::Literal) -> $S::Literal;
                fn from_str(s: &str) -> Result<$S::Literal, ()>;
                fn to_string($self: &$S::Literal) -> String;
                fn debug_kind($self: &$S::Literal) -> String;
                fn symbol($self: &$S::Literal) -> String;
                fn suffix($self: &$S::Literal) -> Option<String>;
                fn integer(n: &str) -> $S::Literal;
                fn typed_integer(n: &str, kind: &str) -> $S::Literal;
                fn float(n: &str) -> $S::Literal;
                fn f32(n: &str) -> $S::Literal;
                fn f64(n: &str) -> $S::Literal;
                fn string(string: &str) -> $S::Literal;
                fn character(ch: char) -> $S::Literal;
                fn byte_string(bytes: &[u8]) -> $S::Literal;
                fn span($self: &$S::Literal) -> $S::Span;
                fn set_span($self: &mut $S::Literal, span: $S::Span);
                fn subspan(
                    $self: &$S::Literal,
                    start: Bound<usize>,
                    end: Bound<usize>,
                ) -> Option<$S::Span>;
            },
            SourceFile {
                fn drop($self: $S::SourceFile);
                fn clone($self: &$S::SourceFile) -> $S::SourceFile;
                fn eq($self: &$S::SourceFile, other: &$S::SourceFile) -> bool;
                fn path($self: &$S::SourceFile) -> String;
                fn is_real($self: &$S::SourceFile) -> bool;
            },
            MultiSpan {
                fn drop($self: $S::MultiSpan);
                fn new() -> $S::MultiSpan;
                fn push($self: &mut $S::MultiSpan, span: $S::Span);
            },
            Diagnostic {
                fn drop($self: $S::Diagnostic);
                fn new(level: Level, msg: &str, span: $S::MultiSpan) -> $S::Diagnostic;
                fn sub(
                    $self: &mut $S::Diagnostic,
                    level: Level,
                    msg: &str,
                    span: $S::MultiSpan,
                );
                fn emit($self: $S::Diagnostic);
            },
            Span {
                fn debug($self: $S::Span) -> String;
                fn def_site() -> $S::Span;
                fn call_site() -> $S::Span;
                fn mixed_site() -> $S::Span;
                fn source_file($self: $S::Span) -> $S::SourceFile;
                fn parent($self: $S::Span) -> Option<$S::Span>;
                fn source($self: $S::Span) -> $S::Span;
                fn start($self: $S::Span) -> LineColumn;
                fn end($self: $S::Span) -> LineColumn;
                fn before($self: $S::Span) -> $S::Span;
                fn after($self: $S::Span) -> $S::Span;
                fn join($self: $S::Span, other: $S::Span) -> Option<$S::Span>;
                fn resolved_at($self: $S::Span, at: $S::Span) -> $S::Span;
                fn source_text($self: $S::Span) -> Option<String>;
                fn save_span($self: $S::Span) -> usize;
                fn recover_proc_macro_span(id: usize) -> $S::Span;
            },
        }
    };
}

// FIXME(eddyb) this calls `encode` for each argument, but in reverse,
// to match the ordering in `reverse_decode`.
macro_rules! reverse_encode {
    ($writer:ident;) => {};
    ($writer:ident; $first:ident $(, $rest:ident)*) => {
        reverse_encode!($writer; $($rest),*);
        $first.encode(&mut $writer, &mut ());
    }
}

// FIXME(eddyb) this calls `decode` for each argument, but in reverse,
// to avoid borrow conflicts from borrows started by `&mut` arguments.
macro_rules! reverse_decode {
    ($reader:ident, $s:ident;) => {};
    ($reader:ident, $s:ident; $first:ident: $first_ty:ty $(, $rest:ident: $rest_ty:ty)*) => {
        reverse_decode!($reader, $s; $($rest: $rest_ty),*);
        let $first = <$first_ty>::decode(&mut $reader, $s);
    }
}

#[allow(unsafe_code)]
mod buffer;
#[forbid(unsafe_code)]
pub mod client;
#[allow(unsafe_code)]
mod closure;
#[forbid(unsafe_code)]
mod handle;
#[macro_use]
#[forbid(unsafe_code)]
mod rpc;
#[allow(unsafe_code)]
mod scoped_cell;
#[allow(unsafe_code)]
mod selfless_reify;
#[forbid(unsafe_code)]
pub mod server;

use buffer::Buffer;
pub use rpc::PanicMessage;
use rpc::{Decode, DecodeMut, Encode, Reader, Writer};

/// An active connection between a server and a client.
/// The server creates the bridge (`Bridge::run_server` in `server.rs`),
/// then passes it to the client through the function pointer in the `run`
/// field of `client::Client`. The client holds its copy of the `Bridge`
/// in TLS during its execution (`Bridge::{enter, with}` in `client.rs`).
#[repr(C)]
pub struct Bridge<'a> {
    /// Reusable buffer (only `clear`-ed, never shrunk), primarily
    /// used for making requests, but also for passing input to client.
    cached_buffer: Buffer,

    /// Server-side function that the client uses to make requests.
    dispatch: closure::Closure<'a, Buffer, Buffer>,

    /// If 'true', always invoke the default panic hook
    force_show_panics: bool,

    // Prevent Send and Sync impls. `!Send`/`!Sync` is the usual way of doing
    // this, but that requires unstable features. crablang-analyzer uses this code
    // and avoids unstable features.
    _marker: marker::PhantomData<*mut ()>,
}

#[forbid(unsafe_code)]
#[allow(non_camel_case_types)]
mod api_tags {
    use super::rpc::{DecodeMut, Encode, Reader, Writer};

    macro_rules! declare_tags {
        ($($name:ident {
            $(fn $method:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret_ty:ty)*;)*
        }),* $(,)?) => {
            $(
                pub(super) enum $name {
                    $($method),*
                }
                rpc_encode_decode!(enum $name { $($method),* });
            )*

            pub(super) enum Method {
                $($name($name)),*
            }
            rpc_encode_decode!(enum Method { $($name(m)),* });
        }
    }
    with_api!(self, self, declare_tags);
}

/// Helper to wrap associated types to allow trait impl dispatch.
/// That is, normally a pair of impls for `T::Foo` and `T::Bar`
/// can overlap, but if the impls are, instead, on types like
/// `Marked<T::Foo, Foo>` and `Marked<T::Bar, Bar>`, they can't.
trait Mark {
    type Unmarked;
    fn mark(unmarked: Self::Unmarked) -> Self;
}

/// Unwrap types wrapped by `Mark::mark` (see `Mark` for details).
trait Unmark {
    type Unmarked;
    fn unmark(self) -> Self::Unmarked;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Marked<T, M> {
    value: T,
    _marker: marker::PhantomData<M>,
}

impl<T, M> Mark for Marked<T, M> {
    type Unmarked = T;
    fn mark(unmarked: Self::Unmarked) -> Self {
        Marked { value: unmarked, _marker: marker::PhantomData }
    }
}
impl<T, M> Unmark for Marked<T, M> {
    type Unmarked = T;
    fn unmark(self) -> Self::Unmarked {
        self.value
    }
}
impl<'a, T, M> Unmark for &'a Marked<T, M> {
    type Unmarked = &'a T;
    fn unmark(self) -> Self::Unmarked {
        &self.value
    }
}
impl<'a, T, M> Unmark for &'a mut Marked<T, M> {
    type Unmarked = &'a mut T;
    fn unmark(self) -> Self::Unmarked {
        &mut self.value
    }
}

impl<T: Mark> Mark for Vec<T> {
    type Unmarked = Vec<T::Unmarked>;
    fn mark(unmarked: Self::Unmarked) -> Self {
        // Should be a no-op due to std's in-place collect optimizations.
        unmarked.into_iter().map(T::mark).collect()
    }
}
impl<T: Unmark> Unmark for Vec<T> {
    type Unmarked = Vec<T::Unmarked>;
    fn unmark(self) -> Self::Unmarked {
        // Should be a no-op due to std's in-place collect optimizations.
        self.into_iter().map(T::unmark).collect()
    }
}

macro_rules! mark_noop {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Mark for $ty {
                type Unmarked = Self;
                fn mark(unmarked: Self::Unmarked) -> Self {
                    unmarked
                }
            }
            impl Unmark for $ty {
                type Unmarked = Self;
                fn unmark(self) -> Self::Unmarked {
                    self
                }
            }
        )*
    }
}
mark_noop! {
    (),
    bool,
    char,
    &'_ [u8],
    &'_ str,
    String,
    usize,
    Delimiter,
    Level,
    LineColumn,
    Spacing,
}

rpc_encode_decode!(
    enum Delimiter {
        Parenthesis,
        Brace,
        Bracket,
        None,
    }
);
rpc_encode_decode!(
    enum Level {
        Error,
        Warning,
        Note,
        Help,
    }
);
rpc_encode_decode!(struct LineColumn { line, column });
rpc_encode_decode!(
    enum Spacing {
        Alone,
        Joint,
    }
);

macro_rules! mark_compound {
    (enum $name:ident <$($T:ident),+> { $($variant:ident $(($field:ident))?),* $(,)? }) => {
        impl<$($T: Mark),+> Mark for $name <$($T),+> {
            type Unmarked = $name <$($T::Unmarked),+>;
            fn mark(unmarked: Self::Unmarked) -> Self {
                match unmarked {
                    $($name::$variant $(($field))? => {
                        $name::$variant $((Mark::mark($field)))?
                    })*
                }
            }
        }

        impl<$($T: Unmark),+> Unmark for $name <$($T),+> {
            type Unmarked = $name <$($T::Unmarked),+>;
            fn unmark(self) -> Self::Unmarked {
                match self {
                    $($name::$variant $(($field))? => {
                        $name::$variant $((Unmark::unmark($field)))?
                    })*
                }
            }
        }
    }
}

macro_rules! compound_traits {
    ($($t:tt)*) => {
        rpc_encode_decode!($($t)*);
        mark_compound!($($t)*);
    };
}

compound_traits!(
    enum Bound<T> {
        Included(x),
        Excluded(x),
        Unbounded,
    }
);

compound_traits!(
    enum Option<T> {
        Some(t),
        None,
    }
);

compound_traits!(
    enum Result<T, E> {
        Ok(t),
        Err(e),
    }
);

#[derive(Clone)]
pub enum TokenTree<G, P, I, L> {
    Group(G),
    Punct(P),
    Ident(I),
    Literal(L),
}

compound_traits!(
    enum TokenTree<G, P, I, L> {
        Group(tt),
        Punct(tt),
        Ident(tt),
        Literal(tt),
    }
);
