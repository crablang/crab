#!/usr/bin/env bash

if [ -z $CHANNEL ]; then
export CHANNEL='debug'
fi

pushd $(dirname "$0") >/dev/null
source config.sh

# read nightly compiler from crablang-toolchain file
TOOLCHAIN=$(cat crablang-toolchain | grep channel | sed 's/channel = "\(.*\)"/\1/')

popd >/dev/null

if [[ $(crablangc -V) != $(crablangc +${TOOLCHAIN} -V) ]]; then
    echo "crablangc_codegen_gcc is build for $(crablangc +${TOOLCHAIN} -V) but the default crablangc version is $(crablangc -V)."
    echo "Using $(crablangc +${TOOLCHAIN} -V)."
fi

cmd=$1
shift

CRABLANGDOCFLAGS="$CRABLANGFLAGS" cargo +${TOOLCHAIN} $cmd $@
