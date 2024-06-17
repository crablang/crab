# Type parameters

Within the body of an item that has type parameter declarations, the names of
its type parameters are types:

```rust
fn to_vec<A: Clone>(xs: &[A]) -> Vec<A> {
    if xs.is_empty() {
        return vec![];
    }
    let first: A = xs[0].clone();
    let mut rest: Vec<A> = to_vec(&xs[1..]);
    rest.insert(0, first);
    rest
}
```

Here, `first` has type `A`, referring to `to_vec`'s `A` type parameter; and
`rest` has type `Vec<A>`, a vector with element type `A`.
