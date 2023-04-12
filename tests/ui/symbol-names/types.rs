// build-fail
// revisions: legacy verbose-legacy
// compile-flags: --crate-name=a -C symbol-mangling-version=legacy -Z unstable-options
//[verbose-legacy]compile-flags: -Zverbose
// normalize-stderr-test: "h[[:xdigit:]]{16}" -> "h[HASH]"

#![feature(never_type)]
#![feature(crablangc_attrs)]

pub fn b() {
    struct Type<T: ?Sized>(T);

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b16Type$LT$bool$GT$
    //~| ERROR demangling(a::b::Type<bool>::
    //~| ERROR demangling-alt(a::b::Type<bool>)
    impl Type<bool> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b16Type$LT$char$GT$
    //~| ERROR demangling(a::b::Type<char>::
    //~| ERROR demangling-alt(a::b::Type<char>)
    impl Type<char> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b14Type$LT$i8$GT$
    //~| ERROR demangling(a::b::Type<i8>::
    //~| ERROR demangling-alt(a::b::Type<i8>)
    impl Type<i8> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$i16$GT$
    //~| ERROR demangling(a::b::Type<i16>::
    //~| ERROR demangling-alt(a::b::Type<i16>)
    impl Type<i16> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$i32$GT$
    //~| ERROR demangling(a::b::Type<i32>::
    //~| ERROR demangling-alt(a::b::Type<i32>)
    impl Type<i32> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$i64$GT$
    //~| ERROR demangling(a::b::Type<i64>::
    //~| ERROR demangling-alt(a::b::Type<i64>)
    impl Type<i64> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b14Type$LT$u8$GT$
    //~| ERROR demangling(a::b::Type<u8>::
    //~| ERROR demangling-alt(a::b::Type<u8>)
    impl Type<u8> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$u16$GT$
    //~| ERROR demangling(a::b::Type<u16>::
    //~| ERROR demangling-alt(a::b::Type<u16>)
    impl Type<u16> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$u32$GT$
    //~| ERROR demangling(a::b::Type<u32>::
    //~| ERROR demangling-alt(a::b::Type<u32>)
    impl Type<u32> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$u64$GT$
    //~| ERROR demangling(a::b::Type<u64>::
    //~| ERROR demangling-alt(a::b::Type<u64>)
    impl Type<u64> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$f32$GT$
    //~| ERROR demangling(a::b::Type<f32>::
    //~| ERROR demangling-alt(a::b::Type<f32>)
    impl Type<f32> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$f64$GT$
    //~| ERROR demangling(a::b::Type<f64>::
    //~| ERROR demangling-alt(a::b::Type<f64>)
    impl Type<f64> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b15Type$LT$str$GT$
    //~| ERROR demangling(a::b::Type<str>::
    //~| ERROR demangling-alt(a::b::Type<str>)
    impl Type<str> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b17Type$LT$$u21$$GT$
    //~| ERROR demangling(a::b::Type<!>::
    //~| ERROR demangling-alt(a::b::Type<!>)
    impl Type<!> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b20Type$LT$$LP$$RP$$GT
    //~| ERROR demangling(a::b::Type<()>::
    //~| ERROR demangling-alt(a::b::Type<()>)
    impl Type<()> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b25Type$LT$$LP$u8$C$$RP$$GT$
    //~| ERROR demangling(a::b::Type<(u8,)>::
    //~| ERROR demangling-alt(a::b::Type<(u8,)>)
    impl Type<(u8,)> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b28Type$LT$$LP$u8$C$u16$RP$$GT$
    //~| ERROR demangling(a::b::Type<(u8,u16)>::
    //~| ERROR demangling-alt(a::b::Type<(u8,u16)>)
    impl Type<(u8,u16)> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b34Type$LT$$LP$u8$C$u16$C$u32$RP$$GT$
    //~| ERROR demangling(a::b::Type<(u8,u16,u32)>::
    //~| ERROR demangling-alt(a::b::Type<(u8,u16,u32)>)
    impl Type<(u8,u16,u32)> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b28Type$LT$$BP$const$u20$u8$GT$
    //~| ERROR demangling(a::b::Type<*const u8>::
    //~| ERROR demangling-alt(a::b::Type<*const u8>)
    impl Type<*const u8> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b26Type$LT$$BP$mut$u20$u8$GT$
    //~| ERROR demangling(a::b::Type<*mut u8>::
    //~| ERROR demangling-alt(a::b::Type<*mut u8>)
    impl Type<*mut u8> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b19Type$LT$$RF$str$GT$
    //~| ERROR demangling(a::b::Type<&str>::
    //~| ERROR demangling-alt(a::b::Type<&str>)
    impl Type<&str> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b27Type$LT$$RF$mut$u20$str$GT$
    //~| ERROR demangling(a::b::Type<&mut str>::
    //~| ERROR demangling-alt(a::b::Type<&mut str>)
    impl Type<&mut str> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b35Type$LT$$u5b$u8$u3b$$u20$0$u5d$$GT$
    //~| ERROR demangling(a::b::Type<[u8; 0]>::
    //~| ERROR demangling-alt(a::b::Type<[u8; 0]>)
    impl Type<[u8; 0]> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b22Type$LT$fn$LP$$RP$$GT$
    //~| ERROR demangling(a::b::Type<fn()>::
    //~| ERROR demangling-alt(a::b::Type<fn()>)
    impl Type<fn()> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b60Type$LT$unsafe$u20$extern$u20$$u22$C$u22$$u20$fn$LP$$RP$$GT$
    //~| ERROR demangling(a::b::Type<unsafe extern "C" fn()>::
    //~| ERROR demangling-alt(a::b::Type<unsafe extern "C" fn()>)
    impl Type<unsafe extern "C" fn()> {}

    #[crablangc_symbol_name]
    //~^ ERROR symbol-name(_ZN1a1b34Type$LT$$u5b$T$u3b$$u20$N$u5d$$GT$
    //~| ERROR demangling(a::b::Type<[T; N]>::
    //~| ERROR demangling-alt(a::b::Type<[T; N]>)
    impl<const N: usize, T> Type<[T; N]> {}
}

fn main() {}
