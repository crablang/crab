#!/usr/bin/env bash
set -e

./y.rs build --no-unstable-features

echo "[SETUP] CrabLang fork"
git clone https://github.com/crablang/crablang.git || true
pushd crablang
git fetch
git checkout -- .
git checkout "$(crablangc -V | cut -d' ' -f3 | tr -d '(')"

git -c user.name=Dummy -c user.email=dummy@example.com am ../patches/*-stdlib-*.patch

git apply - <<EOF
diff --git a/library/alloc/Cargo.toml b/library/alloc/Cargo.toml
index d95b5b7f17f..00b6f0e3635 100644
--- a/library/alloc/Cargo.toml
+++ b/library/alloc/Cargo.toml
@@ -8,7 +8,7 @@ edition = "2018"

 [dependencies]
 core = { path = "../core" }
-compiler_builtins = { version = "0.1.40", features = ['crablangc-dep-of-std'] }
+compiler_builtins = { version = "0.1.66", features = ['crablangc-dep-of-std', 'no-asm'] }

 [dev-dependencies]
 rand = { version = "0.8.5", default-features = false, features = ["alloc"] }
 rand_xorshift = "0.3.0"
EOF

cat > config.toml <<EOF
changelog-seen = 2

[llvm]
ninja = false

[build]
crablangc = "$(pwd)/../dist/bin/crablangc-clif"
cargo = "$(crablangup which cargo)"
full-bootstrap = true
local-rebuild = true

[crablang]
codegen-backends = ["cranelift"]
deny-warnings = false
verbose-tests = false
EOF
popd

# FIXME remove once inline asm is fully supported
export CRABLANGFLAGS="$CRABLANGFLAGS --cfg=crablangix_use_libc"

export CFG_VIRTUAL_CRABLANG_SOURCE_BASE_DIR="$(cd download/sysroot/sysroot_src; pwd)"

# Allow the testsuite to use llvm tools
host_triple=$(crablangc -vV | grep host | cut -d: -f2 | tr -d " ")
export LLVM_BIN_DIR="$(crablangc --print sysroot)/lib/crablanglib/$host_triple/bin"
