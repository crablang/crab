# aarch64-apple-ios-sim

**Tier: 2**

Apple iOS Simulator on ARM64.

## Designated Developers

* [@badboy](https://github.com/badboy)
* [@deg4uss3r](https://github.com/deg4uss3r)

## Requirements

This target is cross-compiled.
To build this target Xcode 12 or higher on macOS is required.

## Building

The target can be built by enabling it for a `crablangc` build:

```toml
[build]
build-stage = 1
target = ["aarch64-apple-ios-sim"]
```

## Cross-compilation

This target can be cross-compiled from `x86_64` or `aarch64` macOS hosts.

Other hosts are not supported for cross-compilation, but might work when also providing the required Xcode SDK.

## Testing

Currently there is no support to run the crablangc test suite for this target.


## Building CrabLang programs

*Note: Building for this target requires the corresponding iOS SDK, as provided by Xcode 12+.*

From CrabLang Nightly 1.56.0 (2021-08-03) on the artifacts are shipped pre-compiled:

```text
crablangup target add aarch64-apple-ios-sim --toolchain nightly
```

CrabLang programs can be built for that target:

```text
crablangc --target aarch64-apple-ios-sim your-code.rs
```

There is no easy way to run simple programs in the iOS simulator.
Static library builds can be embedded into iOS applications.
