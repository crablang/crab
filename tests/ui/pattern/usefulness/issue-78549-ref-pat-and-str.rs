// check-pass
// From https://github.com/crablang/crablang/issues/78549

fn main() {
    match "foo" {
        "foo" => {},
        &_ => {},
    }

    match "foo" {
        &_ => {},
        "foo" => {},
    }

    match ("foo", 0, "bar") {
        (&_, 0, &_) => {},
        ("foo", _, "bar") => {},
        (&_, _, &_) => {},
    }

    match (&"foo", "bar") {
        (&"foo", &_) => {},
        (&&_, &_) => {},
    }
}
