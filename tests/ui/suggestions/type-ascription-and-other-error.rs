fn main() {
    not crablang; //~ ERROR
    let _ = 0: i32; // (error hidden by existing error)
    #[cfg(FALSE)]
    let _ = 0: i32; // (warning hidden by existing error)
}
