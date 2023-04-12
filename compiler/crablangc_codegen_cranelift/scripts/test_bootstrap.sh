#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/../"

source ./scripts/setup_crablang_fork.sh

echo "[TEST] Bootstrap of crablangc"
pushd crablang
rm -r compiler/crablangc_codegen_cranelift/{Cargo.*,src}
cp ../Cargo.* compiler/crablangc_codegen_cranelift/
cp -r ../src compiler/crablangc_codegen_cranelift/src

./x.py build --stage 1 library/std
popd
