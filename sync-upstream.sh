#!/usr/bin/env bash

# check if we're on the "upstream" branch, and if not, switch to it
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "upstream" ]; then
    # check if the branch already exists locally
    if ! git branch | grep -q upstream; then
        git checkout -b upstream || echo "Failed to create and checkout upstream" && exit 1
    else
        git checkout upstream || echo "Failed to checkout upstream" && exit 1
    fi
fi

# add the remote "upstream" if it doesn't exist
if ! git remote | grep -q upstream; then
    git remote add upstream https://github.com/rust-lang/rust.git || echo "Failed to add upstream remote" && exit 1
fi

# hard reset to the remote "upstream" and branch "master"
git fetch upstream || echo "Failed to fetch upstream" && exit 1
git reset --hard upstream/master || echo "Failed to reset to upstream/master" && exit 1

# check if we're on the "current" branch, and if not, switch to it
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "current" ]; then
    # check if the branch already exists locally
    if ! git branch | grep -q current; then
        git checkout -b current || echo "Failed to create and checkout current" && exit 1
    else
        git checkout current || echo "Failed to checkout current" && exit 1
    fi
fi

git reset --hard master || echo "Failed to reset \"current\" to local \"master\"" && exit 1

# rebase the local "current" branch to the local "upstream" branch, favoring the "current" branch
git rebase -Xtheirs upstream || echo "Failed to rebase \"current\" with \"upstream\"" && exit 1


# make sure we are on the "current" branch
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "current" ]; then
    echo "Not on \"current\" branch, aborting"
    exit 1
fi

# push the changes to the remote "current" branch
git push --force origin current || echo "Failed to push \"current\" to remote" && exit 1

