// This test can't work with run-crablangfix because it needs two passes of test+fix

#[warn(clippy::deref_addrof)]
#[allow(unused_variables, unused_mut)]
fn main() {
    let a = 10;

    //This produces a suggestion of 'let b = *&a;' which
    //will trigger the 'clippy::deref_addrof' lint again
    let b = **&&a;

    {
        let mut x = 10;
        let y = *&mut x;
    }

    {
        //This produces a suggestion of 'let y = *&mut x' which
        //will trigger the 'clippy::deref_addrof' lint again
        let mut x = 10;
        let y = **&mut &mut x;
    }
}
