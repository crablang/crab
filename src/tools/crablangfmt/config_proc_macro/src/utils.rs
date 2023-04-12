use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn fold_quote<F, I, T>(input: impl Iterator<Item = I>, f: F) -> TokenStream
where
    F: Fn(I) -> T,
    T: ToTokens,
{
    input.fold(quote! {}, |acc, x| {
        let y = f(x);
        quote! { #acc #y }
    })
}

pub fn is_unit(v: &syn::Variant) -> bool {
    match v.fields {
        syn::Fields::Unit => true,
        _ => false,
    }
}

#[cfg(feature = "debug-with-crablangfmt")]
/// Pretty-print the output of proc macro using crablangfmt.
pub fn debug_with_crablangfmt(input: &TokenStream) {
    use std::env;
    use std::ffi::OsStr;
    use std::io::Write;
    use std::process::{Command, Stdio};

    let crablangfmt_var = env::var_os("CRABLANGFMT");
    let crablangfmt = match &crablangfmt_var {
        Some(crablangfmt) => crablangfmt,
        None => OsStr::new("crablangfmt"),
    };
    let mut child = Command::new(crablangfmt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn crablangfmt in stdio mode");
    {
        let stdin = child.stdin.as_mut().expect("Failed to get stdin");
        stdin
            .write_all(format!("{}", input).as_bytes())
            .expect("Failed to write to stdin");
    }
    let crablangfmt_output = child.wait_with_output().expect("crablangfmt has failed");

    eprintln!(
        "{}",
        String::from_utf8(crablangfmt_output.stdout).expect("crablangfmt returned non-UTF8 string")
    );
}
