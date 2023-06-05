#!/usr/bin/env bash

fail() {
    printf '%s\n' "$1" >&2
    exit "${2-1}"
}

# check if we're on the "upstream" branch, and if not, switch to it
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "upstream" ]; then
    # check if the branch already exists locally
    if ! git branch | grep -q upstream; then
        git checkout -b upstream || fail "Failed to create and checkout upstream"
    else
        git checkout upstream || fail "Failed to checkout upstream"
    fi
fi

# add the remote "upstream" if it doesn't exist
if ! git remote | grep -q upstream; then
    git remote add upstream https://github.com/rust-lang/rust.git || fail "Failed to add upstream remote"
fi

# add the remote "origin" if it doesn't exist
if ! git remote | grep -q origin; then
    git remote add origin https://github.com/crablang/crab.git || fail "Failed to add origin remote"
fi

# hard reset to the remote "upstream" and branch "master"
git fetch upstream || fail "Failed to fetch upstream"
git reset --hard upstream/master || fail "Failed to reset to upstream/master"

# check if we're on the "current" branch, and if not, switch to it
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "current" ]; then
    # check if the branch already exists locally
    if ! git branch | grep -q current; then
        git checkout -b current || fail "Failed to create and checkout current"
    else
        git checkout current || fail "Failed to checkout current"
    fi
fi

git reset --hard origin/master || fail "Failed to reset local \"current\" to remote crab \"master\""

# rebase the local "current" branch to the local "upstream" branch, favoring the "current" branch
git rebase -Xtheirs upstream || fail "Failed to rebase \"current\" with \"upstream\""

# push the changes to the remote "current" branch
git push --force origin current || fail "Failed to push \"current\" to remote"
