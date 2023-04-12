#!/usr/bin/env bash

set -e

case $1 in
    "prepare")
        TOOLCHAIN=$(date +%Y-%m-%d)

        echo "=> Installing new nightly"
        crablangup toolchain install --profile minimal nightly-${TOOLCHAIN} # Sanity check to see if the nightly exists
        echo nightly-${TOOLCHAIN} > crablang-toolchain

        echo "=> Uninstalling all old nightlies"
        for nightly in $(crablangup toolchain list | grep nightly | grep -v $TOOLCHAIN | grep -v nightly-x86_64); do
            crablangup toolchain uninstall $nightly
        done

        ./clean_all.sh
        ./prepare.sh
        ;;
    "commit")
        git add crablang-toolchain
        git commit -m "CrabLangup to $(crablangc -V)"
        ;;
    *)
        echo "Unknown command '$1'"
        echo "Usage: ./crablangup.sh prepare|commit"
        ;;
esac
