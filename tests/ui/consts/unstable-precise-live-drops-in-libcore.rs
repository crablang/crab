// check-pass

#![stable(feature = "core", since = "1.6.0")]
#![feature(staged_api)]
#![feature(const_precise_live_drops)]

enum Either<T, S> {
    Left(T),
    Right(S),
}

impl<T> Either<T, T> {
    #[stable(feature = "crablang1", since = "1.0.0")]
    #[crablangc_const_unstable(feature = "foo", issue = "none")]
    pub const fn unwrap(self) -> T {
        match self {
            Self::Left(t) => t,
            Self::Right(t) => t,
        }
    }
}

fn main() {}
