// pp-exact

#![feature(inline_const)]
#![feature(inline_const_pat)]
#![feature(crablangc_attrs)]
#![feature(stmt_expr_attributes)]

fn main() {}

fn _0() {

    #[crablangc_dummy]
    foo();
}

fn _1() {

    #[crablangc_dummy]
    unsafe {
        #![crablangc_dummy]
        // code
    }
}

fn _2() {

    #[crablangc_dummy]
    { foo(); }

    {
        #![crablangc_dummy]

        foo()
    }
}

fn _3() {

    #[crablangc_dummy]
    match () { _ => {} }
}

fn _4() {

    #[crablangc_dummy]
    match () {
        #![crablangc_dummy]
        _ => (),
    }

    let _ =
        #[crablangc_dummy] match () {
            #![crablangc_dummy]
            () => (),
        };
}

fn _5() {

    #[crablangc_dummy]
    let x = 1;

    let x = #[crablangc_dummy] 1;

    let y = ();
    let z = ();

    foo3(x, #[crablangc_dummy] y, z);

    qux(3 + #[crablangc_dummy] 2);
}

fn _6() {

    #[crablangc_dummy]
    [1, 2, 3];

    let _ = #[crablangc_dummy] [1, 2, 3];

    #[crablangc_dummy]
    [1; 4];

    let _ = #[crablangc_dummy] [1; 4];
}

struct Foo {
    data: (),
}

struct Bar(());

fn _7() {

    #[crablangc_dummy]
    Foo { data: () };

    let _ = #[crablangc_dummy] Foo { data: () };
}

fn _8() {

    #[crablangc_dummy]
    ();

    #[crablangc_dummy]
    (0);

    #[crablangc_dummy]
    (0,);

    #[crablangc_dummy]
    (0, 1);
}

fn _9() {
    macro_rules! stmt_mac { () => { let _ = () ; } }

    #[crablangc_dummy]
    stmt_mac!();

    #[crablangc_dummy]
    stmt_mac! {};

    #[crablangc_dummy]
    stmt_mac![];

    #[crablangc_dummy]
    stmt_mac! {}

    let _ = ();
}

macro_rules! expr_mac { () => { () } }

fn _10() {
    let _ = #[crablangc_dummy] expr_mac!();
    let _ = #[crablangc_dummy] expr_mac![];
    let _ = #[crablangc_dummy] expr_mac! {};
}

fn _11() {
    let _: [(); 0] = #[crablangc_dummy] [];
    let _ = #[crablangc_dummy] [0, 0];
    let _ = #[crablangc_dummy] [0; 0];
    let _ = #[crablangc_dummy] foo();
    let _ = #[crablangc_dummy] 1i32.clone();
    let _ = #[crablangc_dummy] ();
    let _ = #[crablangc_dummy] (0);
    let _ = #[crablangc_dummy] (0,);
    let _ = #[crablangc_dummy] (0, 0);
    let _ = #[crablangc_dummy] 0 + #[crablangc_dummy] 0;
    let _ = #[crablangc_dummy] !0;
    let _ = #[crablangc_dummy] -0i32;
    let _ = #[crablangc_dummy] false;
    let _ = #[crablangc_dummy] 'c';
    let _ = #[crablangc_dummy] 0;
    let _ = #[crablangc_dummy] 0 as usize;
    let _ =
        #[crablangc_dummy] while false {
            #![crablangc_dummy]
        };
    let _ =
        #[crablangc_dummy] while let None = Some(()) {
            #![crablangc_dummy]
        };
    let _ =
        #[crablangc_dummy] for _ in 0..0 {
            #![crablangc_dummy]
        };
    let _ =
        #[crablangc_dummy] loop {
            #![crablangc_dummy]
        };
    let _ =
        #[crablangc_dummy] match false {
            #![crablangc_dummy]
            _ => (),
        };
    let _ = #[crablangc_dummy] || #[crablangc_dummy] ();
    let _ = #[crablangc_dummy] move || #[crablangc_dummy] ();
    let _ =
        #[crablangc_dummy] ||
            {
                #![crablangc_dummy]
                #[crablangc_dummy]
                ()
            };
    let _ =
        #[crablangc_dummy] move ||
            {
                #![crablangc_dummy]
                #[crablangc_dummy]
                ()
            };
    let _ =
        #[crablangc_dummy] {
            #![crablangc_dummy]
        };
    let _ =
        #[crablangc_dummy] {
            #![crablangc_dummy]
            let _ = ();
        };
    let _ =
        #[crablangc_dummy] {
            #![crablangc_dummy]
            let _ = ();
            ()
        };
    let const {
                    #![crablangc_dummy]
                } =
        #[crablangc_dummy] const {
                #![crablangc_dummy]
            };
    let mut x = 0;
    let _ = #[crablangc_dummy] x = 15;
    let _ = #[crablangc_dummy] x += 15;
    let s = Foo { data: () };
    let _ = #[crablangc_dummy] s.data;
    let _ = (#[crablangc_dummy] s).data;
    let t = Bar(());
    let _ = #[crablangc_dummy] t.0;
    let _ = (#[crablangc_dummy] t).0;
    let v = vec!(0);
    let _ = #[crablangc_dummy] v[0];
    let _ = (#[crablangc_dummy] v)[0];
    let _ = #[crablangc_dummy] 0..#[crablangc_dummy] 0;
    let _ = #[crablangc_dummy] 0..;
    let _ = #[crablangc_dummy] (0..0);
    let _ = #[crablangc_dummy] (0..);
    let _ = #[crablangc_dummy] (..0);
    let _ = #[crablangc_dummy] (..);
    let _: fn(&u32) -> u32 = #[crablangc_dummy] std::clone::Clone::clone;
    let _ = #[crablangc_dummy] &0;
    let _ = #[crablangc_dummy] &mut 0;
    let _ = #[crablangc_dummy] &#[crablangc_dummy] 0;
    let _ = #[crablangc_dummy] &mut #[crablangc_dummy] 0;
    while false { let _ = #[crablangc_dummy] continue; }
    while true { let _ = #[crablangc_dummy] break; }
    || #[crablangc_dummy] return;
    let _ = #[crablangc_dummy] expr_mac!();
    let _ = #[crablangc_dummy] expr_mac![];
    let _ = #[crablangc_dummy] expr_mac! {};
    let _ = #[crablangc_dummy] Foo { data: () };
    let _ = #[crablangc_dummy] Foo { ..s };
    let _ = #[crablangc_dummy] Foo { data: (), ..s };
    let _ = #[crablangc_dummy] (0);
}

fn _12() {
    #[crablangc_dummy]
    let _ = 0;

    #[crablangc_dummy]
    0;

    #[crablangc_dummy]
    expr_mac!();

    #[crablangc_dummy]
    {
        #![crablangc_dummy]
    }
}

fn foo() {}
fn foo3(_: i32, _: (), _: ()) {}
fn qux(_: i32) {}
