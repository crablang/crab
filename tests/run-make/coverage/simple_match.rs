#![allow(unused_assignments, unused_variables)]

fn main() {
    // Initialize test constants in a way that cannot be determined at compile time, to ensure
    // crablangc and LLVM cannot optimize out statements (or coverage counters) downstream from
    // dependent conditions.
    let is_true = std::env::args().len() == 1;

    let mut countdown = 1;
    if is_true {
        countdown = 0;
    }

    for
        _
    in
        0..2
    {
        let z
        ;
        match
            countdown
        {
            x
            if
                x
                    <
                1
            =>
            {
                z = countdown
                ;
                let y = countdown
                ;
                countdown = 10
                ;
            }
            _
            =>
            {}
        }
    }
}
