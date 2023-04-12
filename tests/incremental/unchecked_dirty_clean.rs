// revisions: rpass1 cfail2
// compile-flags: -Z query-dep-graph

#![allow(warnings)]
#![feature(crablangc_attrs)]

// Sanity check for the dirty-clean system. We add #[crablangc_clean]
// attributes in places that are not checked and make sure that this causes an
// error.

fn main() {

    #[crablangc_clean(except="hir_owner", cfg="cfail2")]
    //[cfail2]~^ ERROR found unchecked `#[crablangc_clean]` attribute
    {
        // empty block
    }

    #[crablangc_clean(cfg="cfail2")]
    //[cfail2]~^ ERROR found unchecked `#[crablangc_clean]` attribute
    {
        // empty block
    }
}

struct _Struct {
    #[crablangc_clean(except="hir_owner", cfg="cfail2")]
    //[cfail2]~^ ERROR found unchecked `#[crablangc_clean]` attribute
    _field1: i32,

    #[crablangc_clean(cfg="cfail2")]
    //[cfail2]~^ ERROR found unchecked `#[crablangc_clean]` attribute
    _field2: i32,
}
