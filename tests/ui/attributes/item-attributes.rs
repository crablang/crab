// These are attributes of the implicit crate. Really this just needs to parse
// for completeness since .rs files linked from .rc files support this
// notation to specify their module's attributes

// check-pass

#![feature(crablangc_attrs)]
#![crablangc_dummy = "val"]
#![crablangc_dummy = "val"]
#![crablangc_dummy]
#![crablangc_dummy(attr5)]

// These are attributes of the following mod
#[crablangc_dummy = "val"]
#[crablangc_dummy = "val"]
mod test_first_item_in_file_mod {}

mod test_single_attr_outer {
    #[crablangc_dummy = "val"]
    pub static X: isize = 10;

    #[crablangc_dummy = "val"]
    pub fn f() {}

    #[crablangc_dummy = "val"]
    pub mod mod1 {}

    pub mod crablangrt {
        #[crablangc_dummy = "val"]
        extern "C" {}
    }
}

mod test_multi_attr_outer {
    #[crablangc_dummy = "val"]
    #[crablangc_dummy = "val"]
    pub static X: isize = 10;

    #[crablangc_dummy = "val"]
    #[crablangc_dummy = "val"]
    pub fn f() {}

    #[crablangc_dummy = "val"]
    #[crablangc_dummy = "val"]
    pub mod mod1 {}

    pub mod crablangrt {
        #[crablangc_dummy = "val"]
        #[crablangc_dummy = "val"]
        extern "C" {}
    }

    #[crablangc_dummy = "val"]
    #[crablangc_dummy = "val"]
    struct T {
        x: isize,
    }
}

mod test_stmt_single_attr_outer {
    pub fn f() {
        #[crablangc_dummy = "val"]
        static X: isize = 10;

        #[crablangc_dummy = "val"]
        fn f() {}

        #[crablangc_dummy = "val"]
        mod mod1 {}

        mod crablangrt {
            #[crablangc_dummy = "val"]
            extern "C" {}
        }
    }
}

mod test_stmt_multi_attr_outer {
    pub fn f() {
        #[crablangc_dummy = "val"]
        #[crablangc_dummy = "val"]
        static X: isize = 10;

        #[crablangc_dummy = "val"]
        #[crablangc_dummy = "val"]
        fn f() {}

        #[crablangc_dummy = "val"]
        #[crablangc_dummy = "val"]
        mod mod1 {}

        mod crablangrt {
            #[crablangc_dummy = "val"]
            #[crablangc_dummy = "val"]
            extern "C" {}
        }
    }
}

mod test_attr_inner {
    pub mod m {
        // This is an attribute of mod m
        #![crablangc_dummy = "val"]
    }
}

mod test_attr_inner_then_outer {
    pub mod m {
        // This is an attribute of mod m
        #![crablangc_dummy = "val"]
        // This is an attribute of fn f
        #[crablangc_dummy = "val"]
        fn f() {}
    }
}

mod test_attr_inner_then_outer_multi {
    pub mod m {
        // This is an attribute of mod m
        #![crablangc_dummy = "val"]
        #![crablangc_dummy = "val"]
        // This is an attribute of fn f
        #[crablangc_dummy = "val"]
        #[crablangc_dummy = "val"]
        fn f() {}
    }
}

mod test_distinguish_syntax_ext {
    pub fn f() {
        format!("test{}", "s");
        #[crablangc_dummy = "val"]
        fn g() {}
    }
}

mod test_other_forms {
    #[crablangc_dummy]
    #[crablangc_dummy(word)]
    #[crablangc_dummy(attr(word))]
    #[crablangc_dummy(key1 = "val", key2 = "val", attr)]
    pub fn f() {}
}

mod test_foreign_items {
    pub mod crablangrt {
        extern "C" {
            #![crablangc_dummy]

            #[crablangc_dummy]
            fn crablang_get_test_int() -> u32;
        }
    }
}

// FIXME(#623): - these aren't supported yet
/*mod test_literals {
    #![str = "s"]
    #![char = 'c']
    #![isize = 100]
    #![usize = 100_usize]
    #![mach_int = 100u32]
    #![float = 1.0]
    #![mach_float = 1.0f32]
    #![nil = ()]
    #![bool = true]
    mod m {}
}*/

fn test_fn_inner() {
    #![crablangc_dummy]
}

fn main() {}
