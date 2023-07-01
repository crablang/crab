#!/usr/bin/env bash

fail() {
    printf '%s\n' "$1" >&2
    exit "${2-1}"
}

# make sure we're in the root directory of the repository
if [ ! -f "Cargo.toml" ]; then
    fail "Failed to find \"Cargo.toml\", are you sure you're in the root directory of the repository?"
fi

cwd=$(pwd)

# update all submodules
git submodule update --init --recursive

# array of submodule paths and their corresponding branch names
submodules=()

# get the path and the branch name from the .gitmodules file
while IFS=$'\n' read -r line; do
    if [[ "$line" =~ ^.*path\ =.*$ ]]; then
        path=${line#*= }
    elif [[ "$line" =~ ^.*branch\ =.*$ ]]; then
        branch=${line#*= }
        submodules+=("$path:$branch")
    fi
done < .gitmodules

# iterate over the array and update each submodule
for i in "${submodules[@]}"; do
    name=${i%%:*}
    branch=${i#*:}
    echo "Updating $name to $branch"
    # change directory to the submodule
    cd "$name" || fail "Failed to change directory into submodule \"$name\""
    # fetch the latest changes from the remote
    git fetch || fail "Failed to fetch latest changes for submodule \"$name\""
    # checkout the specified branch
    git checkout "$branch" || fail "Failed to checkout branch \"$branch\" for submodule \"$name\""
    # pull the latest changes from the remote
    git pull || fail "Failed to pull latest changes for submodule \"$name\""
    # change directory back to the root of the repository
    cd "$cwd" || fail "Failed to change directory back to root of repository"
done
