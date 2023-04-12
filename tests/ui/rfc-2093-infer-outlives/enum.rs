#![feature(crablangc_attrs)]

// Needs an explicit where clause stating outlives condition. (RFC 2093)

// Type T needs to outlive lifetime 'a.
#[crablangc_outlives]
enum Foo<'a, T> { //~ ERROR crablangc_outlives
    One(Bar<'a, T>)
}

// Type U needs to outlive lifetime 'b
#[crablangc_outlives]
struct Bar<'b, U> { //~ ERROR crablangc_outlives
    field2: &'b U
}

// Type K needs to outlive lifetime 'c.
#[crablangc_outlives]
enum Ying<'c, K> { //~ ERROR crablangc_outlives
    One(&'c Yang<K>)
}

struct Yang<V> {
    field2: V
}

fn main() {}
