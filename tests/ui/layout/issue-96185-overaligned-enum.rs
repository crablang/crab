// normalize-stderr-test "pref: Align\([1-8] bytes\)" -> "pref: $$PREF_ALIGN"
#![crate_type = "lib"]
#![feature(crablangc_attrs)]

// This cannot use `Scalar` abi since there is padding.
#[crablangc_layout(debug)]
#[repr(align(8))]
pub enum Aligned1 { //~ ERROR: layout_of
    Zero = 0,
    One = 1,
}

// This should use `Scalar` abi.
#[crablangc_layout(debug)]
#[repr(align(1))]
pub enum Aligned2 { //~ ERROR: layout_of
    Zero = 0,
    One = 1,
}
