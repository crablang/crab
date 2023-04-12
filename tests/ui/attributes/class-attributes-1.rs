// build-pass (FIXME(62277): could be check-pass?)
// pp-exact - Make sure we actually print the attributes

#![feature(crablangc_attrs)]

struct Cat {
    name: String,
}

impl Drop for Cat {
    #[crablangc_dummy]
    fn drop(&mut self) { println!("{} landed on hir feet" , self . name); }
}


#[crablangc_dummy]
fn cat(name: String) -> Cat { Cat{name: name,} }

fn main() { let _kitty = cat("Spotty".to_string()); }
