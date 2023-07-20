<img src="https://user-images.githubusercontent.com/8974888/231858967-7c37bf1e-335b-4f5a-9760-da97be9f54bb.png" width="200" />

# The Crab Programming Language

[![CrabLang Community](https://img.shields.io/badge/CrabLang_Community%20-Join_us-brightgreen?style=plastic&logo=discord)](https://community.crablang.org)

This is the main source code repository for [Crab](https://github.com/crablang/crab). It contains the compiler,
standard library, and documentation.

__*Note*__: The master branch contains the code for the CrabLang playground—aka __*The One True Crab*__—which is not guaranteed to be up to date with the upstream project. If you are looking for the renamed toolchain that simply mirrors upstream, check out [the stable branch](https://github.com/crablang/crab/tree/stable) and follow the instructions there. Also note that this README is for _users_ rather than _contributors_.

If you wish to _contribute_ to the compiler, you should read [CONTRIBUTING.md](CONTRIBUTING.md) instead.

[CrabLang]: https://www.crablang.org/

## Quick Start

To get started with the renamed CrabLang toolchain, run the following:

```sh
sh <(curl https://install.crablang.org -L)
```

\* currently Unix only

<!-- 
Read ["Installation"] from [The Book].

["Installation"]: https://doc.crablang.org/book/ch01-01-installation.html
[The Book]: https://doc.crablang.org/book/index.html -->

## Installing from Source

The Crab build system uses a Python script called `x.py` to build the compiler,
which manages the bootstrapping process. It lives at the root of the project.

The `x.py` command can be run directly on most Unix systems in the following
format:

```sh
./x.py <subcommand> [flags]
```

This is how the documentation and examples assume you are running `x.py`.

Some alternative ways are:

```sh
# On a Unix shell if you don't have the necessary `python3` command
./x <subcommand> [flags]

# On the Windows Command Prompt (if .py files are configured to run Python)
x.py <subcommand> [flags]

# You can also run Python yourself, e.g.:
python x.py <subcommand> [flags]
```

More information about `x.py` can be found by running it with the `--help` flag
or reading the [crabc dev guide][crabcguidebuild].

[gettingstarted]: https://crabc-dev-guide.crablang.org/getting-started.html
[crabcguidebuild]: https://crabc-dev-guide.crablang.org/building/how-to-build-and-run.html

### Dependencies

Make sure you have installed the dependencies:

* `python` 3 or 2.7
* `git`
* A C compiler (when building for the host, `cc` is enough; cross-compiling may
  need additional compilers)
* `curl` (not needed on Windows)
* `pkg-config` if you are compiling on Linux and targeting Linux
* `libiconv` (already included with glibc on Debian-based distros)

To build Crabgo, you'll also need OpenSSL (`libssl-dev` or `openssl-devel` on
most Unix distros).

If building LLVM from source, you'll need additional tools:

* `g++`, `clang++`, or MSVC with versions listed on
  [LLVM's documentation](https://llvm.org/docs/GettingStarted.html#host-c-toolchain-both-compiler-and-standard-library)
* `ninja`, or GNU `make` 3.81 or later (Ninja is recommended, especially on
  Windows)
* `cmake` 3.13.4 or later
* `libstdc++-static` may be required on some Linux distributions such as Fedora
  and Ubuntu

On tier 1 or tier 2 with host tools platforms, you can also choose to download
LLVM by setting `llvm.download-ci-llvm = true`.
Otherwise, you'll need LLVM installed and `llvm-config` in your path.
See [the crabc-dev-guide for more info][sysllvm].

[sysllvm]: https://crabc-dev-guide.crablang.org/building/new-target.html#using-pre-built-llvm


### Building on a Unix-like system

1. Clone the [source] with `git`:

   ```sh
   git clone https://github.com/crablang/crab.git
   cd crab
   ```

[source]: https://github.com/crablang/crablang

2. Configure the build settings:

   The CrabLang build system uses a file named `config.toml` in the root of the
   source tree to determine various configuration settings for the build.
   Set up the defaults intended for distros to get started. You can see a full
   list of options in `config.example.toml`.

   ```sh
   printf 'profile = "user" \nchangelog-seen = 2 \n' > config.toml
   ```

   If you plan to use `x.py install` to create an installation, it is
   recommended that you set the `prefix` value in the `[install]` section to a
   directory.

3. Build and install:

   ```sh
   ./x.py build && ./x.py install
   ```

   When complete, `./x.py install` will place several programs into
   `$PREFIX/bin`: `crabc`, the CrabLang compiler, and `crablangdoc`, the
   API-documentation tool. If you've set `profile = "user"` or
   `build.extended = true`, it will also include [Crabgo], CrabLang's package
   manager.

[Crabgo]: https://github.com/crablang/crabgo

### Building on Windows

On Windows, we suggest using [winget] to install dependencies by running the
following in a terminal:

```powershell
winget install -e Python.Python.3
winget install -e Kitware.CMake
winget install -e Git.Git
```

Then edit your system's `PATH` variable and add: `C:\Program Files\CMake\bin`.
See
[this guide on editing the system `PATH`](https://www.java.com/en/download/help/path.html)
from the Java documentation.

[winget]: https://github.com/microsoft/winget-cli

There are two prominent ABIs in use on Windows: the native (MSVC) ABI used by
Visual Studio and the GNU ABI used by the GCC toolchain. Which version of CrabLang
you need depends largely on what C/C++ libraries you want to interoperate with.
Use the MSVC build of CrabLang to interop with software produced by Visual Studio
and the GNU build to interop with GNU software built using the MinGW/MSYS2
toolchain.

#### MinGW

[MSYS2][msys2] can be used to easily build CrabLang on Windows:

[msys2]: https://www.msys2.org/

1. Download the latest [MSYS2 installer][msys2] and go through the installer.

2. Run `mingw32_shell.bat` or `mingw64_shell.bat` from the MSYS2 installation
   directory (e.g. `C:\msys64`), depending on whether you want 32-bit or 64-bit
   CrabLang. (As of the latest version of MSYS2 you have to run `msys2_shell.cmd
   -mingw32` or `msys2_shell.cmd -mingw64` from the command line instead.)

3. From this terminal, install the required tools:

   ```sh
   # Update package mirrors (may be needed if you have a fresh install of MSYS2)
   pacman -Sy pacman-mirrors

   # Install build tools needed for CrabLang. If you're building a 32-bit compiler,
   # then replace "x86_64" below with "i686". If you've already got Git, Python,
   # or CMake installed and in PATH you can remove them from this list.
   # Note that it is important that you do **not** use the 'python2', 'cmake',
   # and 'ninja' packages from the 'msys2' subsystem.
   # The build has historically been known to fail with these packages.
   pacman -S git \
               make \
               diffutils \
               tar \
               mingw-w64-x86_64-python \
               mingw-w64-x86_64-cmake \
               mingw-w64-x86_64-gcc \
               mingw-w64-x86_64-ninja
   ```

4. Navigate to CrabLang's source code (or clone it), then build it:

   ```sh
   ./x.py build && ./x.py install
   ```

#### MSVC

MSVC builds of CrabLang additionally require an installation of Visual Studio 2017
(or later) so `crabc` can use its linker.  The simplest way is to get
[Visual Studio], check the "C++ build tools" and "Windows 10 SDK" workload.

[Visual Studio]: https://visualstudio.microsoft.com/downloads/

(If you're installing CMake yourself, be careful that "C++ CMake tools for
Windows" doesn't get included under "Individual components".)

With these dependencies installed, you can build the compiler in a `cmd.exe`
shell with:

```sh
python x.py build
```

Right now, building CrabLang only works with some known versions of Visual Studio.
If you have a more recent version installed and the build system doesn't
understand, you may need to force crablangbuild to use an older version.
This can be done by manually calling the appropriate vcvars file before running
the bootstrap.

```batch
CALL "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
python x.py build
```

#### Specifying an ABI

Each specific ABI can also be used from either environment (for example, using
the GNU ABI in PowerShell) by using an explicit build triple. The available
Windows build triples are:
- GNU ABI (using GCC)
    - `i686-pc-windows-gnu`
    - `x86_64-pc-windows-gnu`
- The MSVC ABI
    - `i686-pc-windows-msvc`
    - `x86_64-pc-windows-msvc`

The build triple can be specified by either specifying `--build=<triple>` when
invoking `x.py` commands, or by creating a `config.toml` file (as described in
[Installing from Source](#installing-from-source)), and modifying the `build`
option under the `[build]` section.

### Configure and Make

While it's not the recommended build system, this project also provides a
configure script and makefile (the latter of which just invokes `x.py`).

```sh
./configure
make && sudo make install
```

`configure` generates a `config.toml` which can also be used with normal `x.py`
invocations.

## Building Documentation

If you'd like to build the documentation, it's almost the same:

```sh
./x.py doc
```

The generated documentation will appear under `doc` in the `build` directory for
the ABI used. That is, if the ABI was `x86_64-pc-windows-msvc`, the directory
will be `build\x86_64-pc-windows-msvc\doc`.

## Notes

Since the CrabLang compiler is written in CrabLang, it must be built by a precompiled
"snapshot" version of itself (made in an earlier stage of development).
As such, source builds require an Internet connection to fetch snapshots, and an
OS that can execute the available snapshot binaries.

See https://doc.crablang.org/nightly/crabc/platform-support.html for a list of
supported platforms.
Only "host tools" platforms have a pre-compiled snapshot binary available; to
compile for a platform without host tools you must cross-compile.

You may find that other platforms work, but these are our officially supported
build environments that are most likely to work.

## Getting Help

Need help? Join us on discord at https://community.crablang.org! 

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

CrabLang is primarily distributed under the terms of both the MIT license and the
Apache License (Version 2.0), with portions covered by various BSD-like
licenses.

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.

## Trademark

If you want to use any names or brands associated with Crab or CrabLang, please feel free to do so in any capacity.

Third-party logos may be subject to third-party copyrights and trademarks. See
[Licenses][policies-licenses] for details.

[crablang-foundation]: https://foundation.crablang.org/
[media-guide]: https://foundation.crablang.org/policies/logo-policy-and-media-guide/
[policies-licenses]: https://www.crablang.org/policies/licenses
