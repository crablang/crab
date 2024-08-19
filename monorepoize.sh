#!/usr/bin/env bash

###
# converts a project with git submodules to a monorepo by importing all the
# submodules into the main repository.
#
# gets submodule info from .gitmodules file and imports each submodule into
# the main repository.
###

main() {
    # checkout "current" branch and reset to latest changes
    git checkout current >/dev/null 2>&1 || fail "failed to checkout master branch"
    git reset --hard origin/upstream >/dev/null 2>&1 || fail "failed to reset upstream branch"

    if [[ -f .gitmodules ]]; then
        # make sure the .gitmodules file is not empty
        if [[ ! -s .gitmodules ]]; then
            echo ".gitmodules file is empty. nothing to do. exiting..."
        else
            echo "found .gitmodules file. attempting to convert to monorepo..."
            convert
        fi
    else
        echo "no .gitmodules file found. exiting..."
    fi

    # copy over the README from remote master branch
    git checkout origin/master -- README.md >/dev/null 2>&1 || fail "failed to copy README.md from remote master branch"
    git add README.md >/dev/null 2>&1 || fail "failed to add README.md"
    git commit -m "use CrabLang README.md" >/dev/null 2>&1 || fail "failed to commit README.md"

    # force push master branch to remote
    git push -f origin current >/dev/null 2>&1 || fail "failed to push 'current' branch to remote"
}

convert() {
    # read each submodule from .gitmodules file
    while read -r i; do
        if [[ $i == \[submodule* ]]; then
            read -r i # next line is the path
            path=$(echo "$i" | cut -d'=' -f2 | xargs)
            read -r i # next line is the url
            url=$(echo "$i" | cut -d'=' -f2 | xargs)
            # if path equals "src/llvm-project", skip it
            if [[ $path == "src/llvm-project" ]]; then
                echo "skipping LLVM submodule..."
                continue
            fi
            echo "converting $path from $url..."
            # deinitialize and remove the submodule
            git submodule deinit -f "$path" >/dev/null 2>&1 || fail "failed to deinit $path"
            rm -rf .git/modules/"$path"
            git rm -f "$path" >/dev/null 2>&1 || fail "failed to remove $path"
            # clone the submodule and remove its .git directory
            git clone "$url" "$path" >/dev/null 2>&1 || fail "failed to clone $url"
            rm -rf "$path"/.git
            # add the submodule files and commit
            git add "$path" >/dev/null 2>&1 || fail "failed to add $path with git"
            msg="imported $path into main repository"
            echo "$msg"
            git commit -m "$msg" >/dev/null 2>&1 || fail "failed to commit $path"
            echo "successfully converted $path."
        fi
    done <.gitmodules
}

fail() {
    printf '%s\n' "$1" >&2
    exit "${2-1}"
}

try_and_continue() {
    $1 || echo "command failed: $1, continuing..."
}

main "$@" && echo "converted to monorepo successfully."
