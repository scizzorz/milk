use super::cli;
use super::cli::BranchCommand;
use super::cli::Command;
use super::find_from_name;
use super::get_short_id;
use super::highlight_named_oid;
use super::print_commit;
use super::print_object;
use super::print_tree;
use colored::*;
use exitcode;
use failure::format_err;
use failure::Error;
use failure::ResultExt;
use git2::build::CheckoutBuilder;
use git2::BranchType;
use git2::Diff;
use git2::DiffOptions;
use git2::ObjectType;
use git2::Odb;
use git2::Oid;
use git2::Repository;
use git2::RepositoryInitOptions;
use git2::ResetType;
use git2::Status;
use git2::StatusOptions;
use git2::Tree;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::exit;

// used by commit
fn temporary_editor(path: &Path, contents: &str) -> Result<String, Error> {
  // FIXME one of the Err cases here is for a non-unicode value... I'd assume you
  // can run a non-unicode command, no?
  let editor = env::var("EDITOR").with_context(|_| "$EDITOR is not defined.")?;

  let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open(path)
    .with_context(|_| "couldn't open $EDITOR file")?;

  file
    .write_all(contents.as_bytes())
    .with_context(|_| "couldn't write $EDITOR file contents")?;

  file
    .sync_all()
    .with_context(|_| "couldn't sync $EDITOR file contents")?;

  let mut editor_command = process::Command::new(editor);
  editor_command.arg(&path);

  editor_command
    .spawn()
    .and_then(|mut handle| handle.wait())
    .with_context(|_| "$EDITOR failed for some reason")?;

  let mut file = File::open(path).with_context(|_| "couldn't re-open file")?;

  let mut contents = String::new();
  file
    .read_to_string(&mut contents)
    .with_context(|_| "couldn't read from file")?;

  if std::fs::remove_file(&path).is_err() {
    eprintln!("WARNING: Unable to delete {} after use", path.display());
  }

  Ok(contents)
}

// used by ls
fn find_subtree(tree: &Tree, name: &str) -> Option<Oid> {
  for entry in tree.iter() {
    let raw_name = entry.name().unwrap_or("[???]");
    if raw_name == name {
      return Some(entry.id());
    }
  }
  None
}

// used by ignore
fn handle_file(
  repo: &Repository,
  filepath: String,
  workdir: &Path,
) -> Result<Option<String>, Error> {
  let path = Path::new(&filepath);

  if !path.exists() {
    print!("File '{}' does not exist, still ignore? [Y/n] ", filepath);
    io::stdout().flush().context("Could not flush stdout")?;

    let mut input = String::new();
    io::stdin()
      .read_line(&mut input)
      .context("Could not read stdin")?;

    match input.trim_end() {
      "y" | "Y" | "" => (),
      _ => return Ok(None),
    }
  }

  // Try to transform path into its canonical path
  // from the workdir
  let path = match path.canonicalize() {
    Ok(abs_path) => match abs_path.strip_prefix(workdir) {
      Ok(rel_path) => rel_path.to_path_buf(),
      _ => path.to_path_buf(),
    },
    _ => path.to_path_buf(),
  };

  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit")?;
  let tree = commit.tree().with_context(|_| "couldn't locate tree")?;

  if tree.get_path(&path).is_ok() {
    println!("Warning: file {} is currently tracked by git", filepath);
  };

  let final_filepath = path
    .to_str()
    .ok_or_else(|| failure::err_msg("Path is not UTF-8"))?;
  Ok(Some(String::from(final_filepath)))
}

// used by clean
fn write_blob(odb: &Odb, path: &str) -> Result<Oid, Error> {
  let mut handle = File::open(path)?;
  let mut bytes = Vec::new();
  let _size = handle.read_to_end(&mut bytes)?;
  let oid = odb.write(ObjectType::Blob, &bytes)?;
  Ok(oid)
}

// used by diff
enum DiffTarget<'a> {
  WorkingTree,
  Index,
  Name(&'a str),
}

// used by diff
impl<'a> DiffTarget<'a> {
  fn from_str(s: &str) -> DiffTarget {
    match s {
      "/WORK" => DiffTarget::WorkingTree,
      "/INDEX" => DiffTarget::Index,
      _ => DiffTarget::Name(s),
    }
  }
}

// used by diff
fn name_to_tree<'repo>(repo: &'repo Repository, s: &str) -> Result<Tree<'repo>, Error> {
  let object = find_from_name(repo, s).with_context(|_| "couldn't find refname")?;
  let tree = object
    .peel_to_tree()
    .with_context(|_| "couldn't peel to commit HEAD")?;
  Ok(tree)
}

// used by diff
fn make_diff<'repo>(
  repo: &'repo Repository,
  old_target: DiffTarget,
  new_target: DiffTarget,
) -> Result<Diff<'repo>, Error> {
  let mut options = DiffOptions::new();

  match (old_target, new_target) {
    // tree..
    (DiffTarget::Name(old), DiffTarget::WorkingTree) => {
      let old_tree = name_to_tree(&repo, old).with_context(|_| "couldn't look up old tree")?;

      let diff = repo
        .diff_tree_to_workdir(Some(&old_tree), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }

    (DiffTarget::Name(old), DiffTarget::Name(new)) => {
      let old_tree = name_to_tree(&repo, old).with_context(|_| "couldn't look up old tree")?;
      let new_tree = name_to_tree(&repo, new).with_context(|_| "couldn't look up new tree")?;

      let diff = repo
        .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }

    (DiffTarget::Name(old), DiffTarget::Index) => {
      let old_tree = name_to_tree(&repo, old).with_context(|_| "couldn't look up old tree")?;
      let index = repo.index().with_context(|_| "couldn't read index")?;

      let diff = repo
        .diff_tree_to_index(Some(&old_tree), Some(&index), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }

    // index..
    (DiffTarget::Index, DiffTarget::WorkingTree) => {
      let index = repo.index().with_context(|_| "couldn't read index")?;
      let diff = repo
        .diff_index_to_workdir(Some(&index), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;

      Ok(diff)
    }

    (DiffTarget::Index, DiffTarget::Name(new)) => {
      let index = repo.index().with_context(|_| "couldn't read index")?;
      let new_tree = name_to_tree(&repo, new).with_context(|_| "couldn't look up new tree")?;
      options.reverse(true);

      let diff = repo
        .diff_tree_to_index(Some(&new_tree), Some(&index), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }

    (DiffTarget::Index, DiffTarget::Index) => {
      // FIXME why? it probably works...
      Err(format_err!("Cannot diff between identical targets"))
    }

    // working..
    (DiffTarget::WorkingTree, DiffTarget::WorkingTree) => {
      // FIXME why? it probably works...
      Err(format_err!("Cannot diff between identical targets"))
    }
    (DiffTarget::WorkingTree, DiffTarget::Name(new)) => {
      let new_tree = name_to_tree(&repo, new).with_context(|_| "couldn't look up new tree")?;
      options.reverse(true);

      let diff = repo
        .diff_tree_to_workdir(Some(&new_tree), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }
    (DiffTarget::WorkingTree, DiffTarget::Index) => {
      let index = repo.index().with_context(|_| "couldn't read index")?;
      options.reverse(true);
      let diff = repo
        .diff_index_to_workdir(Some(&index), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;

      Ok(diff)
    }
  }
}

// used by status
fn get_status_string(status: Status) -> String {
  let index_string = if status.is_index_new() {
    "new".cyan()
  } else if status.is_index_modified() {
    "mod".green()
  } else if status.is_index_deleted() {
    "del".red()
  } else if status.is_index_renamed() {
    "ren".blue()
  } else if status.is_index_typechange() {
    "typ".blue()
  } else {
    "   ".normal()
  };

  let working_string = if status.is_wt_new() {
    "new".bright_cyan()
  } else if status.is_wt_modified() {
    "mod".bright_green()
  } else if status.is_wt_deleted() {
    "del".bright_red()
  } else if status.is_wt_renamed() {
    "ren".bright_blue()
  } else if status.is_wt_typechange() {
    "typ".bright_blue()
  } else {
    "   ".normal()
  };

  if status.is_ignored() {
    format!("{}", " ignored".white())
  } else if status.is_conflicted() {
    format!("{}", "conflict".red())
  } else {
    format!(" {} {}", index_string, working_string)
  }
}

pub fn main(args: cli::Root) -> Result<(), Error> {
  match args.command {
    Command::Branch(cmd_args) => match cmd_args.command {
      BranchCommand::Ls(subcmd_args) => branch_ls(args.globals, subcmd_args),
      BranchCommand::Mv(_subcmd_args) => Ok(()),
      BranchCommand::New(_subcmd_args) => Ok(()),
      BranchCommand::Rename(_subcmd_args) => Ok(()),
      BranchCommand::Rm(_subcmd_args) => Ok(()),
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

pub fn clean(globals: cli::Global, args: cli::Clean) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;
  let odb = repo.odb().with_context(|_| "couldn't open odb")?;

  let mut checkout = CheckoutBuilder::new();
  checkout.force();

  for path in &args.paths {
    checkout.path(path);
  }

  if !args.paths.is_empty() {
    for path in &args.paths {
      let oid = write_blob(&odb, path)?;
      println!("{}", highlight_named_oid(&repo, path, oid));
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
          let oid = write_blob(&odb, path)?;
          println!("{}", highlight_named_oid(&repo, path, oid));
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

  let message =
    temporary_editor(&message_file_path, "").with_context(|_| "couldn't get message")?;
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
  println!("{}", highlight_named_oid(&repo, head_name, commit.id()));
  print_commit(&repo, &new_commit);

  Ok(())
}

pub fn diff(globals: cli::Global, args: cli::Diff) -> Result<(), Error> {
  let repo =
    Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

  let old_target = DiffTarget::from_str(&args.old_tree);
  let new_target = DiffTarget::from_str(&args.new_tree);

  let diff = make_diff(&repo, old_target, new_target).with_context(|_| "failed to diff")?;

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
  println!("{}", highlight_named_oid(&repo, head_name, commit.id()));

  print_commit(&repo, &commit);

  Ok(())
}

pub fn ignore(globals: cli::Global, args: cli::Ignore) -> Result<(), Error> {
  let repo = Repository::discover(globals.repo_path).context("Couldn't open repository")?;

  let workdir_bytes = repo
    .workdir()
    .ok_or_else(|| failure::err_msg("repository is bare"))?;
  let workdir = Path::new(
    workdir_bytes
      .to_str()
      .ok_or_else(|| failure::err_msg("path is not utf-8"))?,
  );

  let to_ignore = if args.is_pattern {
    Some(args.pattern)
  } else {
    handle_file(&repo, args.pattern, &workdir)?
  };

  if let Some(to_ignore) = to_ignore {
    let gitignore_path = workdir.join(".gitignore");

    let mut gitignore = OpenOptions::new()
      .create(!gitignore_path.exists())
      .append(true)
      .open(workdir.join(".gitignore"))
      .context("Couldn't open .gitignore file")?;

    println!("Adding {} to .gitignore", to_ignore);
    writeln!(gitignore, "{}", to_ignore).context("Couldn't write to .gitignore file")?;
  };

  Ok(())
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

  let object = find_from_name(&repo, &args.ref_name).with_context(|_| "couldn't find refname")?;

  let commit = match object.into_commit() {
    Ok(commit) => commit,
    Err(_) => {
      return Err(failure::err_msg("refname didn't point to commit")).context("...?")?;
    }
  };

  println!(
    "{}",
    highlight_named_oid(&repo, &args.ref_name, commit.id())
  );

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
          get_short_id(&repo, next_tree_id).bright_black()
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

  print_tree(&repo, &tree);

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
  let object =
    find_from_name(&repo, &args.object_name).with_context(|_| "couldn't look up object")?;

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

  let object = find_from_name(&repo, &args.name).with_context(|_| "couldn't look up object")?;

  print_object(&repo, &object);

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

  let object = find_from_name(&repo, &args.ref_name).with_context(|_| "couldn't look up object")?;
  print_object(&repo, &object);

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
