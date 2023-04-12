// See #3657 - https://github.com/crablang/crablangfmt/issues/3657

#![feature(associated_type_bounds)]

fn f<I: Iterator<Item: Clone>>() {}

fn g<I: Iterator<Item: Clone>>() {}

fn h<I: Iterator<Item: Clone>>() {}

fn i<I: Iterator<Item: Clone>>() {}

fn j<I: Iterator<Item: Clone + 'a>>() {}
