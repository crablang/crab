#!/bin/bash

# check if we're on the "current" branch, and if not, switch to it
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "current" ]; then
  git checkout current
fi

# hard reset to the remote "upstream" and branch "master"
git fetch upstream
git reset --hard upstream/master

# copy the README.md file from the local master branch
git checkout master -- README.md

# add it to git in the current branch and create a commit
git add README.md
git commit -m "use crablang readme"

# git push git paid
