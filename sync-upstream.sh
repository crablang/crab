#!/usr/bin/env bash

# check if we're on the "upstream" branch, and if not, switch to it
upstream_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$upstream_branch" != "upstream" ]; then
    # check if the branch already exists locally
    git branch | grep -q upstream
    if [ $? -ne 0 ]; then
        git checkout master
      git checkout -b upstream
    else
      git checkout upstream
    fi
fi

# add the remote "upstream" if it doesn't exist
git remote | grep -q upstream
if [ $? -ne 0 ]; then
  git remote add upstream https://github.com/rust-lang/rust.git
fi

# hard reset to the remote "upstream" and branch "master"
git fetch upstream
git reset --hard upstream/master

# check if we're on the "current" branch, and if not, switch to it
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "current" ]; then
    # check if the branch already exists locally
    git branch | grep -q current
    if [ $? -ne 0 ]; then
      git checkout master
      git checkout -b current
    else
      git checkout current
      git reset --hard master
    fi
fi

# copy the README.md file from the local master branch
# git checkout master -- README.md CODE_OF_CONDUCT.md CONTRIBUTING.md sync-upstream.sh
git rebase -Xtheirs upstream

# add it to git in the current branch and create a commit
# git add --all
# git commit -m "incorporate stable changes"

# git push git paid
# git push --force origin current
