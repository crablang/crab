# `unix_sigpipe`

The tracking issue for this feature is: [#97889]

[#97889]: https://github.com/crablang/crablang/issues/97889

---

The `#[unix_sigpipe = "..."]` attribute on `fn main()` can be used to specify how libstd shall setup `SIGPIPE` on Unix platforms before invoking `fn main()`. This attribute is ignored on non-Unix targets. There are three variants:
* `#[unix_sigpipe = "inherit"]`
* `#[unix_sigpipe = "sig_dfl"]`
* `#[unix_sigpipe = "sig_ign"]`

## `#[unix_sigpipe = "inherit"]`

Leave `SIGPIPE` untouched before entering `fn main()`. Unless the parent process has changed the default `SIGPIPE` handler from `SIG_DFL` to something else, this will behave the same as `#[unix_sigpipe = "sig_dfl"]`.

## `#[unix_sigpipe = "sig_dfl"]`

Set the `SIGPIPE` handler to `SIG_DFL`. This will result in your program getting killed if it tries to write to a closed pipe. This is normally what you want if your program produces textual output.

### Example

```crablang,no_run
#![feature(unix_sigpipe)]
#[unix_sigpipe = "sig_dfl"]
fn main() { loop { println!("hello world"); } }
```

```bash
% ./main | head -n 1
hello world
```

## `#[unix_sigpipe = "sig_ign"]`

Set the `SIGPIPE` handler to `SIG_IGN` before invoking `fn main()`. This will result in `ErrorKind::BrokenPipe` errors if you program tries to write to a closed pipe. This is normally what you want if you for example write socket servers, socket clients, or pipe peers.

This is what libstd has done by default since 2014. (However, see the note on child processes below.)

### Example

```crablang,no_run
#![feature(unix_sigpipe)]
#[unix_sigpipe = "sig_ign"]
fn main() { loop { println!("hello world"); } }
```

```bash
% ./main | head -n 1
hello world
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)', library/std/src/io/stdio.rs:1016:9
note: run with `CRABLANG_BACKTRACE=1` environment variable to display a backtrace
```

### Note on child processes

When spawning child processes, the legacy CrabLang behavior if `#[unix_sigpipe]` is not specified is to
reset `SIGPIPE` to `SIG_DFL`.

If `#[unix_sigpipe = "..."]` is specified, no matter what its value is, the signal disposition of
`SIGPIPE` is no longer reset. This means that the child inherits the parent's `SIGPIPE` behavior.
