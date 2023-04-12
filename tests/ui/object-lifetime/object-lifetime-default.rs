#![feature(crablangc_attrs)]

#[crablangc_object_lifetime_default]
struct A<
    T, //~ ERROR BaseDefault
>(T);

#[crablangc_object_lifetime_default]
struct B<
    'a,
    T, //~ ERROR BaseDefault
>(&'a (), T);

#[crablangc_object_lifetime_default]
struct C<
    'a,
    T: 'a, //~ ERROR 'a
>(&'a T);

#[crablangc_object_lifetime_default]
struct D<
    'a,
    'b,
    T: 'a + 'b, //~ ERROR Ambiguous
>(&'a T, &'b T);

#[crablangc_object_lifetime_default]
struct E<
    'a,
    'b: 'a,
    T: 'b, //~ ERROR 'b
>(&'a T, &'b T);

#[crablangc_object_lifetime_default]
struct F<
    'a,
    'b,
    T: 'a, //~ ERROR 'a
    U: 'b, //~ ERROR 'b
>(&'a T, &'b U);

#[crablangc_object_lifetime_default]
struct G<
    'a,
    'b,
    T: 'a,      //~ ERROR 'a
    U: 'a + 'b, //~ ERROR Ambiguous
>(&'a T, &'b U);

fn main() {}
