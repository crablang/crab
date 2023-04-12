This crate is regularly synced with its mirror in the crablangc repo at `compiler/crablangc_smir`.

We use `git subtree` for this to preserve commits and allow the crablangc repo to
edit these crates without having to touch this repo. This keeps the crates compiling
while allowing us to independently work on them here. The effort of keeping them in
sync is pushed entirely onto us, without affecting crablangc workflows negatively.
This may change in the future, but changes to policy should only be done via a
compiler team MCP.

## Instructions for working on this crate locally

Since the crate is the same in the crablangc repo and here, the dependencies on crablangc_* crates
will only either work here or there, but never in both places at the same time. Thus we use
optional dependencies on the crablangc_* crates, requiring local development to use

```
cargo build --no-default-features -Zavoid-dev-deps
```

in order to compile successfully.

## Instructions for syncing

### Updating this repository

In the crablangc repo, execute

```
git subtree push --prefix=compiler/crablangc_smir url_to_your_fork_of_project_stable_mir some_feature_branch
```

and then open a PR of your `some_feature_branch` against https://github.com/crablang/project-stable-mir

### Updating the crablangc library

First we need to bump our stack limit, as the crablangc repo otherwise quickly hits that:

```
ulimit -s 60000
```

#### Maximum function recursion depth (1000) reached

Then we need to disable `dash` as the default shell for sh scripts, as otherwise we run into a
hard limit of a recursion depth of 1000:

```
sudo dpkg-reconfigure dash
```

and then select `No` to disable dash.


#### Patching your `git worktree`

The regular git worktree does not scale to repos of the size of the crablangc repo.
So download the `git-subtree.sh` from https://github.com/gitgitgadget/git/pull/493/files and run

```
sudo cp --backup /path/to/patched/git-subtree.sh /usr/lib/git-core/git-subtree
sudo chmod --reference=/usr/lib/git-core/git-subtree~ /usr/lib/git-core/git-subtree
sudo chown --reference=/usr/lib/git-core/git-subtree~ /usr/lib/git-core/git-subtree
```

#### Actually doing a sync

In the crablangc repo, execute

```
git subtree pull --prefix=compiler/crablangc_smir https://github.com/crablang/project-stable-mir smir
```

Note: only ever sync to crablangc from the project-stable-mir's `smir` branch. Do not sync with your own forks.

Then open a PR against crablangc just like a regular PR.

## Stable MIR Design

The stable-mir will follow a similar approach to proc-macro2. It’s
implementation will eventually be broken down into two main crates:

- `stable_mir`: Public crate, to be published on crates.io, which will contain
the stable data structure as well as proxy APIs to make calls to the
compiler.
- `crablangc_smir`: The compiler crate that will translate from internal MIR to
SMIR. This crate will also implement APIs that will be invoked by
stable-mir to query the compiler for more information.

This will help tools to communicate with the crablang compiler via stable APIs. Tools will depend on
`stable_mir` crate, which will invoke the compiler using APIs defined in `crablangc_smir`. I.e.:

```
    ┌──────────────────────────────────┐           ┌──────────────────────────────────┐
    │   External Tool     ┌──────────┐ │           │ ┌──────────┐   CrabLang Compiler     │
    │                     │          │ │           │ │          │                     │
    │                     │stable_mir| │           │ │crablangc_smir│                     │
    │                     │          │ ├──────────►| │          │                     │
    │                     │          │ │◄──────────┤ │          │                     │
    │                     │          │ │           │ │          │                     │
    │                     │          │ │           │ │          │                     │
    │                     └──────────┘ │           │ └──────────┘                     │
    └──────────────────────────────────┘           └──────────────────────────────────┘
```

More details can be found here:
https://hackmd.io/XhnYHKKuR6-LChhobvlT-g?view

For now, the code for these two crates are in separate modules of this crate.
The modules have the same name for simplicity. We also have a third module,
`crablangc_internal` which will expose APIs and definitions that allow users to
gather information from internal MIR constructs that haven't been exposed in
the `stable_mir` module.
