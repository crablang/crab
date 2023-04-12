// aux-build:a.rs
// aux-build:b.rs
// revisions:rpass1 rpass2

#![feature(crablangc_attrs)]


#[cfg(rpass1)]
extern crate a;
#[cfg(rpass1)]
extern crate b;

#[cfg(rpass2)]
extern crate b;
#[cfg(rpass2)]
extern crate a;

use a::A;
use b::B;

//? #[crablangc_clean(label="typeck", cfg="rpass2")]
pub fn main() {
    A + B;
}
