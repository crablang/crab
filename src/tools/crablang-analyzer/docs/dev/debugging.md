# Debugging VSCode plugin and the language server

## Prerequisites

- Install [LLDB](https://lldb.llvm.org/) and the [LLDB Extension](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb).
- Open the root folder in VSCode. Here you can access the preconfigured debug setups.

  <img height=150px src="https://user-images.githubusercontent.com/36276403/74611090-92ec5380-5101-11ea-8a41-598f51f3f3e3.png" alt="Debug options view">

- Install all TypeScript dependencies
  ```bash
  cd editors/code
  npm ci
  ```

## Common knowledge

* All debug configurations open a new `[Extension Development Host]` VSCode instance
where **only** the `crablang-analyzer` extension being debugged is enabled.
* To activate the extension you need to open any CrabLang project folder in `[Extension Development Host]`.


## Debug TypeScript VSCode extension

- `Run Installed Extension` - runs the extension with the globally installed `crablang-analyzer` binary.
- `Run Extension (Debug Build)` - runs extension with the locally built LSP server (`target/debug/crablang-analyzer`).

TypeScript debugging is configured to watch your source edits and recompile.
To apply changes to an already running debug process, press <kbd>Ctrl+Shift+P</kbd> and run the following command in your `[Extension Development Host]`

```
> Developer: Reload Window
```

## Debug CrabLang LSP server

- When attaching a debugger to an already running `crablang-analyzer` server on Linux you might need to enable `ptrace` for unrelated processes by running:

  ```
  echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope
  ```


- By default, the LSP server is built without debug information. To enable it, you'll need to change `Cargo.toml`:
  ```toml
    [profile.dev]
    debug = 2
  ```

- Select `Run Extension (Debug Build)` to run your locally built `target/debug/crablang-analyzer`.

- In the original VSCode window once again select the `Attach To Server` debug configuration.

- A list of running processes should appear. Select the `crablang-analyzer` from this repo.

- Navigate to `crates/crablang-analyzer/src/main_loop.rs` and add a breakpoint to the `on_request` function.

- Go back to the `[Extension Development Host]` instance and hover over a CrabLang variable and your breakpoint should hit.

If you need to debug the server from the very beginning, including its initialization code, you can use the `--wait-dbg` command line argument or `RA_WAIT_DBG` environment variable. The server will spin at the beginning of the `try_main` function (see `crates\crablang-analyzer\src\bin\main.rs`)
```crablang
    let mut d = 4;
    while d == 4 { // set a breakpoint here and change the value
        d = 4;
    }
```

However for this to work, you will need to enable debug_assertions in your build
```crablang
CRABLANGFLAGS='--cfg debug_assertions' cargo build --release
```

## Demo

- [Debugging TypeScript VScode extension](https://www.youtube.com/watch?v=T-hvpK6s4wM).
- [Debugging CrabLang LSP server](https://www.youtube.com/watch?v=EaNb5rg4E0M).

## Troubleshooting

### Can't find the `crablang-analyzer` process

It could be a case of just jumping the gun.

The `crablang-analyzer` is only started once the `onLanguage:crablang` activation.

Make sure you open a crablang file in the `[Extension Development Host]` and try again.

### Can't connect to `crablang-analyzer`

Make sure you have run `echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope`.

By default this should reset back to 1 every time you log in.

### Breakpoints are never being hit

Check your version of `lldb`. If it's version 6 and lower, use the `classic` adapter type.
It's `lldb.adapterType` in settings file.

If you're running `lldb` version 7, change the lldb adapter type to `bundled` or `native`.
