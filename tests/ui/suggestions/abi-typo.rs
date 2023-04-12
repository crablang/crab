// run-crablangfix
extern "cdedl" fn cdedl() {} //~ ERROR invalid ABI

fn main() {
    cdedl();
}
