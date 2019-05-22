use super::cli;
use super::cli::BranchCommand;
use super::cli::Command;
use super::editor;
use super::find_subtree;
use super::get_status_string;
use super::DiffTarget;
use super::MilkRepo;
use colored::*;
use exitcode;
use failure::Error;
use failure::ResultExt;
use git2::build::CheckoutBuilder;
use git2::BranchType;
use git2::ObjectType;
use git2::Repository;
use git2::RepositoryInitOptions;
use git2::ResetType;
use git2::StatusOptions;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

pub fn main(args: cli::Root) -> Result<(), Error> {
  match args.command {
    Command::Branch(cmd_args) => match cmd_args.command {
      BranchCommand::Ls(subcmd_args) => branch_ls(args.globals, subcmd_args),
      BranchCommand::Mv(subcmd_args) => branch_mv(args.globals, subcmd_args),
      BranchCommand::New(subcmd_args) => branch_new(args.globals, subcmd_args),
      BranchCommand::Rename(subcmd_args) => branch_rename(args.globals, subcmd_args),
      BranchCommand::Rm(subcmd_args) => branch_rm(args.globals, subcmd_args),
      BranchCommand::Switch(subcmd_args) => branch_switch(args.globals, subcmd_args),
    },
    Command::Clean(cmd_args) => clean(args.globals, cmd_args),
    Command::Commit(cmd_args) => commit(args.globals, cmd_args),
    Command::Diff(cmd_args) => diff(args.globals, cmd_args),
    Command::Head(cmd_args) => head(args.globals, cmd_args),
    Command::Ignore(cmd_args) => ignore(args.globals, cmd_args),
    Command::Init(cmd_args) => init(args.globals, cmd_args),
    Command::Ls(cmd_args) => ls(args.globals, cmd_args),
    Command::Me(cmd_args) => me(args.globals, cmd_args),
    Command::Restore(cmd_args) => restore(args.globals, cmd_args),
    Command::Show(cmd_args) => show(args.globals, cmd_args),
    Command::Stage(cmd_args) => stage(args.globals, cmd_args),
    Command::Status(cmd_args) => status(args.globals, cmd_args),
    Command::Tag(cmd_args) => tag(args.globals, cmd_args),
    Command::Unstage(cmd_args) => unstage(args.globals, cmd_args),
    Command::Where(cmd_args) => where_(args.globals, cmd_args),
  }?;

  Ok(())
}

pub fn branch_ls(globals: cli::Global, args: cli::BranchLs) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let branches = repo
    .branches(None)
    .with_context(|_| "couldn't iterate branches")?;

  for branch in branches {
    let (branch, typ) = branch.with_context(|_| "couldn't identify branch")?;
    let name = branch
      .name()
      .with_context(|_| "couldn't identify branch name")?
      .unwrap_or("[branch name is invalid utf8]");
    let head_prefix = if branch.is_head() { "*" } else { " " };
    match (typ, args.include_remote) {
      (BranchType::Remote, false) => (),
      _ => println!("{} {}", head_prefix, name),
    }
  }

  Ok(())
}

pub fn branch_mv(globals: cli::Global, args: cli::BranchMv) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let _src_branch = repo
    .find_branch(&args.src_name, BranchType::Local)
    .with_context(|_| "couldn't find source branch")?;

  let dest_object = repo
    .find_from_name(&args.dest_ref)
    .with_context(|_| "couldn't look up dest ref")?;

  if let Some(ObjectType::Commit) = dest_object.kind() {
    let commit = dest_object.into_commit().unwrap();

    repo
      .branch(&args.src_name, &commit, true)
      .with_context(|_| "couldn't move branch")?;

    println!("Moved branch");
    println!("{}", repo.highlight_named_oid(&args.src_name, commit.id()));
    repo.print_commit(&commit);
  } else {
    Err(failure::err_msg("dest object was not a commit"))?;
  }

  Ok(())
}

pub fn branch_new(globals: cli::Global, args: cli::BranchNew) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;
  let object = repo
    .find_from_name(&args.ref_name)
    .with_context(|_| "couldn't look up ref")?;

  if let Some(ObjectType::Commit) = object.kind() {
    let commit = object.into_commit().unwrap();

    repo
      .branch(&args.name, &commit, false)
      .with_context(|_| "couldn't create branch")?;

    println!("Created branch");
    println!("{}", repo.highlight_named_oid(&args.name, commit.id()));

    repo.print_commit(&commit);
  } else {
    Err(failure::err_msg("object was not a commit"))?;
  }

  Ok(())
}

pub fn branch_switch(globals: cli::Global, args: cli::BranchSwitch) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  // find the destination branch's git-style ref name
  let branch = repo
    .find_branch(&args.name, BranchType::Local)
    .with_context(|_| "couldn't locate target branch")?;
  let ref_ = branch.into_reference();
  let ref_name = ref_
    .name()
    .ok_or_else(|| failure::err_msg("ref name is invalid UTF-8"))?;

  // we don't want to switch branches if the index doesn't match HEAD
  let diff = repo
    .make_diff(DiffTarget::Index, DiffTarget::Name("/HEAD"))
    .with_context(|_| "failed to diff index to working tree")?;
  let stats = diff
    .stats()
    .with_context(|_| "couldn't compute diff stats")?;
  if stats.files_changed() > 0 {
    eprintln!(
      "{}: please unstage all changes before switching branches",
      "error".red()
    );
    exit(exitcode::DATAERR);
  }

  if !args.no_stash {
    // FIXME do something to save uncommitted changes here. it looks like the
    // git2 stash_* functions kinda suck, so...  maybe don't use those?
  }

  // move HEAD
  repo
    .set_head(&ref_name)
    .with_context(|_| "couldn't change HEAD")?;

  // force checkout the working tree
  let mut checkout = CheckoutBuilder::new();
  checkout.force();
  repo
    .checkout_head(Some(&mut checkout))
    .with_context(|_| "couldn't checkout HEAD")?;

  // FIXME check if this branch had a Milk-generated stash, and if so, check
  // those out instead of the HEAD tree

  Ok(())
}

pub fn branch_rename(globals: cli::Global, args: cli::BranchRename) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let typ = if args.is_remote {
    BranchType::Remote
  } else {
    BranchType::Local
  };

  let mut branch = repo
    .find_branch(&args.src_name, typ)
    .with_context(|_| "couldn't find branch")?;

  branch
    .rename(&args.dest_name, args.force)
    .with_context(|_| "couldn't rename branch")?;

  let target = branch
    .get()
    .target()
    .ok_or_else(|| failure::err_msg("couldn't get branch target"))?;

  let commit = repo
    .find_commit(target)
    .with_context(|_| "couldn't find commit")?;

  println!("Renamed branch {} => {}", args.src_name, args.dest_name);
  println!("{}", repo.highlight_named_oid(&args.dest_name, target));
  repo.print_commit(&commit);

  Ok(())
}

pub fn branch_rm(globals: cli::Global, args: cli::BranchRm) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let typ = if args.is_remote {
    BranchType::Remote
  } else {
    BranchType::Local
  };

  let mut branch = repo
    .find_branch(&args.name, typ)
    .with_context(|_| "couldn't find branch")?;

  branch.delete().with_context(|_| "couldn't delete branch")?;

  let target = branch
    .get()
    .target()
    .ok_or_else(|| failure::err_msg("couldn't get removed branch target"))?;
  let commit = repo
    .find_commit(target)
    .with_context(|_| "couldn't find commit")?;

  println!("Removed branch");
  println!("{}", repo.highlight_named_oid(&args.name, target));
  repo.print_commit(&commit);

  Ok(())
}

pub fn clean(globals: cli::Global, args: cli::Clean) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let mut checkout = CheckoutBuilder::new();
  checkout.force();

  for path in &args.paths {
    checkout.path(path);
  }

  if !args.paths.is_empty() {
    for path in &args.paths {
      let oid = repo.write_blob(Path::new(path))?;
      println!("{}", repo.highlight_named_oid(path, oid));
    }
  } else {
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(false);
    status_opts.include_ignored(false);

    let statuses = repo
      .statuses(Some(&mut status_opts))
      .with_context(|_| "couldn't open status")?;

    for entry in statuses.iter() {
      if let Some(path) = entry.path() {
        let status = entry.status();
        if status.is_wt_modified() || status.is_index_modified() {
          let oid = repo.write_blob(Path::new(path))?;
          println!("{}", repo.highlight_named_oid(path, oid));
        }
      }
    }
  }

  repo
    .checkout_head(Some(&mut checkout))
    .with_context(|_| "couldn't checkout")?;

  Ok(())
}

pub fn commit(globals: cli::Global, _args: cli::Commit) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let sig = repo
    .signature()
    .with_context(|_| "couldn't obtain signature")?;

  let mut index = repo.index().with_context(|_| "couldn't open index")?;
  let tree_id = index.write_tree().with_context(|_| "couldn't write tree")?;
  let tree = repo
    .find_tree(tree_id)
    .with_context(|_| "couldn't find tree")?;

  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit HEAD")?;

  let parents = [&commit];

  let mut message_file_path = PathBuf::new();
  message_file_path.push(repo.path());
  message_file_path.push("COMMIT_EDITMSG");

  let message = editor(&message_file_path, "").with_context(|_| "couldn't get message")?;
  let message = message.trim();

  if message.is_empty() {
    eprintln!("Aborting due to empty commit message.");
    exit(exitcode::DATAERR);
  }

  let new_commit_id = repo
    .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents)
    .with_context(|_| "couldn't write commit")?;

  let new_commit = repo
    .find_commit(new_commit_id)
    .with_context(|_| "couldn't find commit")?;

  let head_name = head.shorthand().unwrap_or("[???]");
  println!("{}", repo.highlight_named_oid(head_name, commit.id()));
  repo.print_commit(&new_commit);

  Ok(())
}

pub fn diff(globals: cli::Global, args: cli::Diff) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let old_target = DiffTarget::from_str(&args.old_tree);
  let new_target = DiffTarget::from_str(&args.new_tree);

  let diff = repo
    .make_diff(old_target, new_target)
    .with_context(|_| "failed to diff")?;

  // this API is literally insane
  // example code yanked from here:
  //   https://github.com/rust-lang/git2-rs/blob/master/examples/diff.rs#L153-L179
  diff
    .print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
      let display = std::str::from_utf8(line.content()).unwrap();
      match line.origin() {
        '+' => print!("{}{}", "+".green(), display.green()),
        '-' => print!("{}{}", "-".red(), display.red()),
        ' ' => print!(" {}", display.white()),
        _ => print!("{}", display.cyan()),
      }
      true
    })
    .with_context(|_| "failed to print diff")?;

  Ok(())
}

pub fn head(globals: cli::Global, _args: cli::Head) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;
  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit HEAD")?;

  // tf do I do if these aren't UTF-8? Quit?
  let head_name = head.shorthand().unwrap_or("[???]");
  println!("{}", repo.highlight_named_oid(head_name, commit.id()));

  repo.print_commit(&commit);

  Ok(())
}

pub fn ignore(globals: cli::Global, args: cli::Ignore) -> Result<(), Error> {
  let repo = Repository::discover(globals.repo_path).context("Couldn't open repository")?;

  if args.is_pattern {
    repo.ignore_string(&args.pattern)
  } else {
    repo.ignore_file(&Path::new(&args.pattern))
  }
}

pub fn init(globals: cli::Global, args: cli::Init) -> Result<(), Error> {
  let mut repo_opts = RepositoryInitOptions::new();
  repo_opts.bare(args.bare);
  repo_opts.no_reinit(true);

  let _repo = Repository::init_opts(globals.repo_path, &repo_opts)
    .with_context(|_| "couldn't initialize repository")?;

  if !globals.quiet {
    println!("Initialized repository.");
  }

  Ok(())
}

pub fn ls(globals: cli::Global, args: cli::Ls) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let object = repo
    .find_from_name(&args.ref_name)
    .with_context(|_| "couldn't find refname")?;

  let commit = match object.into_commit() {
    Ok(commit) => commit,
    Err(_) => {
      return Err(failure::err_msg("refname didn't point to commit")).context("...?")?;
    }
  };

  println!("{}", repo.highlight_named_oid(&args.ref_name, commit.id()));

  if args.tree_path.is_absolute() {
    eprintln!("Tree path must be relative");
    exit(exitcode::USAGE);
  }

  let mut tree = commit.tree().with_context(|_| "couldn't find tree")?;

  for frag in args.tree_path.iter() {
    let frag_name = match frag.to_str() {
      Some(x) => x,
      None => {
        eprintln!("Tree path must be valid UTF-8");
        exit(exitcode::USAGE);
      }
    };

    match find_subtree(&tree, &frag_name) {
      Some(next_tree_id) => {
        println!(
          "{}/ {}",
          frag_name.cyan(),
          repo.get_short_id(next_tree_id).bright_black()
        );
        tree = repo
          .find_tree(next_tree_id)
          .with_context(|_| "couldn't find tree")?;
      }
      None => {
        eprintln!("Subtree `{}` did not exist", frag_name);
        exit(exitcode::USAGE);
      }
    };
  }

  repo.print_tree(&tree);

  Ok(())
}

pub fn me(globals: cli::Global, _args: cli::Me) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  // I don't know why this has to be this way
  // if you don't do the snapshot(), it crashes when reading a string
  // with an obscure error that's hard to Google:
  // "get_string called on a live config object; class=Config (7)"
  let mut config = repo.config().with_context(|_| "couldn't open config")?;
  let config = config
    .snapshot()
    .with_context(|_| "couldn't snapshot config")?;

  // read user name and email
  let user_name = config
    .get_str("user.name")
    .with_context(|_| "couldn't read user.name")?;
  let user_email = config
    .get_str("user.email")
    .with_context(|_| "couldn't read user.email")?;

  // print info
  println!("{} {}", user_name.cyan(), user_email.bright_black(),);

  Ok(())
}

pub fn restore(globals: cli::Global, args: cli::Restore) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;
  let object = repo
    .find_from_name(&args.object_name)
    .with_context(|_| "couldn't look up object")?;

  let blob = match object.into_blob() {
    Ok(blob) => blob,
    Err(_) => {
      return Err(failure::err_msg("name didn't point to blob")).context("...?")?;
    }
  };

  let bytes = blob.content();

  let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open(args.path)
    .with_context(|_| "couldn't open file for writing")?;

  file
    .write_all(bytes)
    .with_context(|_| "couldn't write to file")?;

  Ok(())
}

pub fn show(globals: cli::Global, args: cli::Show) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let object = repo
    .find_from_name(&args.name)
    .with_context(|_| "couldn't look up object")?;

  repo.print_object(&object);

  Ok(())
}

pub fn stage(globals: cli::Global, args: cli::Stage) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let mut index = repo.index().with_context(|_| "couldn't open index")?;

  for path in args.paths {
    index
      .add_path(Path::new(&path))
      .with_context(|_| "couldn't add path")?;
    println!("Staged {}", path);
  }

  index.write().with_context(|_| "couldn't write index")?;

  Ok(())
}

pub fn status(globals: cli::Global, args: cli::Status) -> Result<(), Error> {
  let mut status_opts = StatusOptions::new();
  status_opts.include_untracked(!args.hide_untracked);
  status_opts.include_ignored(args.show_ignored);

  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let statuses = repo
    .statuses(Some(&mut status_opts))
    .with_context(|_| "couldn't open status")?;

  for entry in statuses.iter() {
    let path = entry.path().unwrap_or("[invalid utf-8]");
    let status = entry.status();
    let status_string = get_status_string(status);

    println!("{} {}", status_string, path);
  }

  Ok(())
}

pub fn tag(globals: cli::Global, args: cli::Tag) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let object = repo
    .find_from_name(&args.ref_name)
    .with_context(|_| "couldn't look up object")?;
  repo.print_object(&object);

  repo
    .tag_lightweight(&args.tag_name, &object, false)
    .with_context(|_| "couldn't create tag")?;

  Ok(())
}

pub fn unstage(globals: cli::Global, args: cli::Unstage) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel(ObjectType::Any)
    .with_context(|_| "couldn't peel to commit HEAD")?;

  if !args.paths.is_empty() {
    repo
      .reset_default(Some(&commit), args.paths)
      .with_context(|_| "could not reset paths")?;
  } else {
    repo
      .reset(&commit, ResetType::Mixed, None)
      .with_context(|_| "could not reset to HEAD")?;
  }

  Ok(())
}

pub fn where_(globals: cli::Global, _args: cli::Where) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  match repo.workdir() {
    Some(path) => match path.to_str() {
      Some(path_str) => println!("{}", path_str),
      None => println!("Path is not UTF-8"),
    },
    None => println!("Repository is bare."),
  }

  Ok(())
}
