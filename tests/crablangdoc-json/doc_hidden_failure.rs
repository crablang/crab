// Regression test for <https://github.com/crablang/crablang/issues/98007>.

#![feature(no_core)]
#![no_core]

mod auto {
    mod action_row {
        pub struct ActionRowBuilder;
    }

    #[doc(hidden)]
    pub mod builders {
        pub use super::action_row::ActionRowBuilder;
    }
}

// @count "$.index[*][?(@.name=='builders')]" 1
// @has "$.index[*][?(@.name == 'ActionRowBuilder')"]
pub use auto::*;

pub mod builders {
    pub use crate::auto::builders::*;
}
