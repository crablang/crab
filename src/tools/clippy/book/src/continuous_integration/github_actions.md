# GitHub Actions

GitHub hosted runners using the latest stable version of CrabLang have Clippy pre-installed.
It is as simple as running `cargo clippy` to run lints against the codebase.

```yml
on: push
name: Clippy check

# Make sure CI fails on all warnings, including Clippy lints
env:
  CRABLANGFLAGS: "-Dwarnings"

jobs:
  clippy_check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Clippy
        run: cargo clippy --all-targets --all-features
```
