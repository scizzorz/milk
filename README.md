# Milk

A modernized porcelain for git.

## Why

Apparently terminals are going through some sort of modern renaissance where
old, antiquated tools are being replaced with hip, fresh tools. I'm hopping on
the bandwagon.

See:

* [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep)
* [jakubroztocil/httpie](https://github.com/jakubroztocil/httpie)
* [ogham/exa](https://github.com/ogham/exa)
* [sharkdp/bat](https://github.com/sharkdp/bat)
* [sharkdp/fd](https://github.com/sharkdp/fd)


Anyway, I've always found git's standard porcelain to be... confusing. I'm sure
many newer users may also feel that way, so I'd like to see if I can do
something about it. Maybe I can't. Whatever.

The original idea credit goes to [eevee/tcup](https://github.com/eevee/tcup),
but that project seems to be have been long abandoned. Many moons ago it was
even written in Rust 0.6.

### Why is it called that

Because short words are good for CLI tools and the phrase 'git milk?' was
funny to me. Whatever.

## Design

As it turns out, eevee has already done a fair amount of inspirational
[planning](https://github.com/eevee/tcup/wiki/Planning). Alas, this is not
`tcup`! This is `milk`. My plan is slightly different, but the general idea
remains the same. Several pieces are taken from eevee's, but a few are just
things that I wish were easier to get out of git.

* I appreciate eevee's `--batch` and `--json` ideas, but `milk` is initially
  intended to just improve the human experience of interacting with git
  repositories, not the machine experience.
* Not sure I agree with being able to do everything that git can do; there's a
  lot of dark magic hidden away in git's plumbing that is so arcane that it
  almost never comes up. I just want to support the majority of common
  operations.

### Commands

If there's a checkbox next to it, it means it has at least minimal
functionality. It does not mean it's completely finished.

#### Inspection

* [x] `ls` - I like being able to browse the clean git tree like I would browse the
  dirty working tree
* [x] `show` - Like `git cat-file -p <id>` but better.
* [x] `me` - Funny, but also helpful when you may have multiple identities for
  various repos (eg, personal / work emails)
* [x] `head` - Just display the current `HEAD`. This is probably obsolete because of
  `show` defaulting to `HEAD`.
* [x] `where` - Show the base directory for the working repo
* [x] `status` - Obvious
* [x] `diff` - Obvious
* [ ] `log` - Obvious

#### File operations

* [ ] `track [paths]` - Start tracking things? Does this make sense? I've always
  found it weird that `git add` will move something from untracked to staged or
  from modified to staged. Feels like there's a missing step but I don't know
  if it actually makes any sense to break it up like this.
* [x] `ignore [paths]` - Add things to `.gitignore`
* [x] `stage [paths]` - Stage files.
* [x] `unstage [paths]` - Unstage files. Like `git reset --mixed`, but you can't
  move your HEAD at the same time. That's weird.
* [x] `clean [paths]` - Clean all local modifications. Like `git reset --hard`.
  This adds all dirty files to the ODB and prints out the OIDs , just in case
  you really oof yourself and need to get them back. See `restore` below to
  restore oopsied files.
* [x] `restore <blob> <path>` - Place the contents of `<blob>` from the ODB
  into a file at `<path>`.

#### Repo operations

* [ ] `switch` - Switch HEAD to something else
* [ ] `update` - Try to pull new changes from a remote, including
  fastforwarding local branches and stuff
* [ ] `sync` - Try to push/pull new changes from a remote, prompting the user
  to rebase if there's not a fast-forward avaialble
* [x] `commit` - Obvious
* [ ] `merge` - Obvious, but with a change: when merging two branches, *both*
  branches will update to the merge commit. I find too often that I `checkout
  master; merge dev; checkout dev; merge master` to get a clean branching point
  for `dev`.

#### Branch operations

* [x] `branch new` - Create a branch
* [x] `branch rm` - Remove a branch
* [x] `branch ls` - List all branches
* [x] `branch rename` - Rename a branch
* [x] `branch mv` - Move a branch from its current location to a new one

## Gripes with Git

I don't have a really solid vision aside from "easier to use", but here are
some areas I've noted in git's existing porcelain:

* Many commands are extremely overloaded, which leads to some context-sensitive
  ambiguity. These commands can often do unexpected things or are intended to
  do things completely different from the way they're actually used. Notably,
  I've been using `git reset` as a way to unstage all changes, or unstage
  changes for a particular file via `git reset <path>`. I recently learned
  `git reset` *actually* sets the `HEAD` to a specific revision and sets the
  index to the state of that revision. Effectively, this *does* unstage
  things... but it also does a lot more than that, which is rather unintuitive.
  Also, it has some odd ambiguous cases for arguments - `git reset <tree-ish>`
  and `git reset <path>`. If a user ever tries to use an ambiguous command, it
  will notify them - but it's still surprising. I don't like being surprised by
  my tooling.
* `git checkout` is similarly overloaded. Also, both of these don't really have
  names that represent what I actually want to do. `reset` is actually used to
  *unstage*, and `checkout` is used to both *switch branches* and to *undo
  local changes*. Weird stuff.
* `git branch` is super overloaded. Enumerating, creating, renaming, moving,
  deleting, and copying are all the same command, just with a different flag.
  I'd like to normalize this with subcommands instead of flags, eg:
  * `git-branch -v` -> `milk branch ls`
  * `git-branch <branch>` -> `milk branch new <branch>`
  * `git branch -m <old-name> <new-name>` -> `milk branch rename <old> <new>`
  * `git-branch -f <branch> <commit-ish>` -> `milk branch mv <branch> <commit-ish>`
  * `git branch -d <branch>` -> `milk branch rm <branch>`
* `git stash` is also fairly overloaded, but with subcommands instead. That's
  weird, but I guess it makes sense sometimes? I don't have many ideas to fix
  this one.
* `git add .` and `git commit -a` make it too easy to intentionally add garbage
  that shouldn't be tracked. `git add -p` is excellent, but it makes the
  process of committing a bit more of a chore. How can this be streamlined so
  users are encouraged to make small, logical commits instead of large, lumpy
  commits?
* `git commit -m` encourages users to write short, unhelpful messages. It's
  GONE.
* `commit-ish`, `tree-ish`, and their ilk are sometimes ambiguous and
  confusing. I'd like to create some notation to clearly indicate what an
  entity is expected to be, for example `beef` may refer to commit, `@beef` may
  refer to a branch, and `#beef` may refer to a tag. Dunno yet.
* Probably other stuff too. Dunno.
* eevee also lists many more [gripes](https://github.com/eevee/tcup/wiki/Git-gripes)
