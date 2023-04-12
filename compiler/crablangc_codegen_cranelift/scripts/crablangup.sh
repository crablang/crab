#!/usr/bin/env bash

set -e

TOOLCHAIN=${TOOLCHAIN:-$(date +%Y-%m-%d)}

function check_git_fixed_subtree() {
    if [[ ! -e ./git-fixed-subtree.sh ]]; then
        echo "Missing git-fixed-subtree.sh. Please run the following commands to download it:"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/bjorn3/git/tqc-subtree-portable/contrib/subtree/git-subtree.sh -o git-fixed-subtree.sh"
        echo "chmod u+x git-fixed-subtree.sh"
        exit 1
    fi
    if [[ ! -x ./git-fixed-subtree.sh ]]; then
        echo "git-fixed-subtree.sh is not executable. Please run the following command to make it executable:"
        echo "chmod u+x git-fixed-subtree.sh"
        exit 1
    fi
}

case $1 in
    "prepare")
        echo "=> Installing new nightly"
        crablangup toolchain install --profile minimal "nightly-${TOOLCHAIN}" # Sanity check to see if the nightly exists
        sed -i "s/\"nightly-.*\"/\"nightly-${TOOLCHAIN}\"/" crablang-toolchain
        crablangup component add crablangfmt || true

        echo "=> Uninstalling all old nightlies"
        for nightly in $(crablangup toolchain list | grep nightly | grep -v "$TOOLCHAIN" | grep -v nightly-x86_64); do
            crablangup toolchain uninstall "$nightly"
        done

        ./clean_all.sh

        ./y.rs prepare

        (cd download/sysroot && cargo update && cargo fetch && cp Cargo.lock ../../build_sysroot/)
        ;;
    "commit")
        git add crablang-toolchain build_sysroot/Cargo.lock
        git commit -m "CrabLangup to $(crablangc -V)"
        ;;
    "push")
        check_git_fixed_subtree

        cg_clif=$(pwd)
        pushd ../crablang
        git pull origin master
        branch=sync_cg_clif-$(date +%Y-%m-%d)
        git checkout -b "$branch"
        "$cg_clif/git-fixed-subtree.sh" pull --prefix=compiler/crablangc_codegen_cranelift/ https://github.com/bjorn3/crablangc_codegen_cranelift.git master
        git push -u my "$branch"

        # immediately merge the merge commit into cg_clif to prevent merge conflicts when syncing
        # from crablang/crablang later
        "$cg_clif/git-fixed-subtree.sh" push --prefix=compiler/crablangc_codegen_cranelift/ "$cg_clif" sync_from_crablang
        popd
        git merge sync_from_crablang
	;;
    "pull")
        check_git_fixed_subtree

        CRABLANG_VERS=$(curl "https://static.crablang.org/dist/$TOOLCHAIN/channel-crablang-nightly-git-commit-hash.txt")
        echo "Pulling $CRABLANG_VERS ($TOOLCHAIN)"

        cg_clif=$(pwd)
        pushd ../crablang
        git fetch origin master
        git checkout "$CRABLANG_VERS"
        "$cg_clif/git-fixed-subtree.sh" push --prefix=compiler/crablangc_codegen_cranelift/ "$cg_clif" sync_from_crablang
        popd
        git merge sync_from_crablang -m "Sync from crablang $CRABLANG_VERS"
        git branch -d sync_from_crablang
        ;;
    *)
        echo "Unknown command '$1'"
        echo "Usage: ./crablangup.sh prepare|commit"
        ;;
esac
