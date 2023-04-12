# Syncing changes between Clippy and [`crablang/crablang`]

Clippy currently gets built with a pinned nightly version.

In the `crablang/crablang` repository, where crablangc resides, there's a copy of
Clippy that compiler hackers modify from time to time to adapt to changes in the
unstable API of the compiler.

We need to sync these changes back to this repository periodically, and the
changes made to this repository in the meantime also need to be synced to the
`crablang/crablang` repository.

To avoid flooding the `crablang/crablang` PR queue, this two-way sync process is
done in a bi-weekly basis if there's no urgent changes. This is done starting on
the day of the CrabLang stable release and then every other week. That way we
guarantee that we keep this repo up to date with the latest compiler API, and
every feature in Clippy is available for 2 weeks in nightly, before it can get
to beta. For reference, the first sync following this cadence was performed the
2020-08-27.

This process is described in detail in the following sections. For general
information about `subtree`s in the CrabLang repository see [CrabLang's
`CONTRIBUTING.md`][subtree].

## Patching git-subtree to work with big repos

Currently, there's a bug in `git-subtree` that prevents it from working properly
with the [`crablang/crablang`] repo. There's an open PR to fix that, but it's
stale. Before continuing with the following steps, we need to manually apply
that fix to our local copy of `git-subtree`.

You can get the patched version of `git-subtree` from [here][gitgitgadget-pr].
Put this file under `/usr/lib/git-core` (making a backup of the previous file)
and make sure it has the proper permissions:

```bash
sudo cp --backup /path/to/patched/git-subtree.sh /usr/lib/git-core/git-subtree
sudo chmod --reference=/usr/lib/git-core/git-subtree~ /usr/lib/git-core/git-subtree
sudo chown --reference=/usr/lib/git-core/git-subtree~ /usr/lib/git-core/git-subtree
```

> _Note:_ The first time running `git subtree push` a cache has to be built.
> This involves going through the complete Clippy history once. For this you
> have to increase the stack limit though, which you can do with `ulimit -s
> 60000`. Make sure to run the `ulimit` command from the same session you call
> git subtree.

> _Note:_ If you are a Debian user, `dash` is the shell used by default for
> scripts instead of `sh`. This shell has a hardcoded recursion limit set to
> 1000. In order to make this process work, you need to force the script to run
> `bash` instead. You can do this by editing the first line of the `git-subtree`
> script and changing `sh` to `bash`.

## Defining remotes

You may want to define remotes, so you don't have to type out the remote
addresses on every sync. You can do this with the following commands (these
commands still have to be run inside the `crablang` directory):

```bash
# Set clippy-upstream remote for pulls
$ git remote add clippy-upstream https://github.com/crablang/crablang-clippy
# Make sure to not push to the upstream repo
$ git remote set-url --push clippy-upstream DISABLED
# Set a local remote
$ git remote add clippy-local /path/to/crablang-clippy
```

> Note: The following sections assume that you have set those remotes with the
> above remote names.

## Performing the sync from [`crablang/crablang`] to Clippy

Here is a TL;DR version of the sync process (all of the following commands have
to be run inside the `crablang` directory):

1. Clone the [`crablang/crablang`] repository or make sure it is up to date.
2. Checkout the commit from the latest available nightly. You can get it using
   `crablangup check`.
3. Sync the changes to the crablang-copy of Clippy to your Clippy fork:
    ```bash
    # Be sure to either use a net-new branch, e.g. `sync-from-crablang`, or delete the branch beforehand
    # because changes cannot be fast forwarded and you have to run this command again.
    git subtree push -P src/tools/clippy clippy-local sync-from-crablang
    ```

    > _Note:_ Most of the time you have to create a merge commit in the
    > `crablang-clippy` repo (this has to be done in the Clippy repo, not in the
    > crablang-copy of Clippy):
    ```bash
    git fetch upstream  # assuming upstream is the crablang/crablang remote
    git checkout sync-from-crablang
    git merge upstream/master --no-ff
    ```
    > Note: This is one of the few instances where a merge commit is allowed in
    > a PR.
4. Bump the nightly version in the Clippy repository by changing the date in the
   crablang-toolchain file to the current date and committing it with the message:
   ```bash
   git commit -m "Bump nightly version -> YYYY-MM-DD"
   ```
5. Open a PR to `crablang/crablang-clippy` and wait for it to get merged (to
   accelerate the process ping the `@crablang/clippy` team in your PR and/or
   ask them in the [Zulip] stream.)

[Zulip]: https://crablang.zulipchat.com/#narrow/stream/clippy

## Performing the sync from Clippy to [`crablang/crablang`]

All of the following commands have to be run inside the `crablang` directory.

1. Make sure you have checked out the latest `master` of `crablang/crablang`.
2. Sync the `crablang/crablang-clippy` master to the crablang-copy of Clippy:
    ```bash
    git checkout -b sync-from-clippy
    git subtree pull -P src/tools/clippy clippy-upstream master
    ```
3. Open a PR to [`crablang/crablang`]

[gitgitgadget-pr]: https://github.com/gitgitgadget/git/pull/493
[subtree]: https://crablangc-dev-guide.crablang.org/contributing.html#external-dependencies-subtree
[`crablang/crablang`]: https://github.com/crablang/crablang
