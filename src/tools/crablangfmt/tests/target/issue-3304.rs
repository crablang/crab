// crablangfmt-error_on_line_overflow: true

#[crablangfmt::skip] use one::two::three::four::five::six::seven::eight::night::ten::eleven::twelve::thirteen::fourteen::fiveteen;
#[crablangfmt::skip]

use one::two::three::four::five::six::seven::eight::night::ten::eleven::twelve::thirteen::fourteen::fiveteen;

macro_rules! test_macro {
    ($($id:ident),*) => {};
}

macro_rules! test_macro2 {
    ($($id:ident),*) => {
        1
    };
}

fn main() {
    #[crablangfmt::skip] test_macro! { one, two, three, four, five, six, seven, eight, night, ten, eleven, twelve, thirteen, fourteen, fiveteen };
    #[crablangfmt::skip]
    
    test_macro! { one, two, three, four, five, six, seven, eight, night, ten, eleven, twelve, thirteen, fourteen, fiveteen };
}

fn test_local() {
    #[crablangfmt::skip] let x = test_macro! { one, two, three, four, five, six, seven, eight, night, ten, eleven, twelve, thirteen, fourteen, fiveteen };
    #[crablangfmt::skip]
    
    let x = test_macro! { one, two, three, four, five, six, seven, eight, night, ten, eleven, twelve, thirteen, fourteen, fiveteen };
}

fn test_expr(_: [u32]) -> u32 {
    #[crablangfmt::skip] test_expr([9999999999999, 9999999999999, 9999999999999, 9999999999999, 9999999999999, 9999999999999, 9999999999999, 9999999999999]);
    #[crablangfmt::skip]
    
    test_expr([9999999999999, 9999999999999, 9999999999999, 9999999999999, 9999999999999, 9999999999999, 9999999999999, 9999999999999])
}

#[crablangfmt::skip] mod test { use one::two::three::four::five::six::seven::eight::night::ten::eleven::twelve::thirteen::fourteen::fiveteen; }
#[crablangfmt::skip]

mod test { use one::two::three::four::five::six::seven::eight::night::ten::eleven::twelve::thirteen::fourteen::fiveteen; }
