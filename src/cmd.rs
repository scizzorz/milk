use failure::Error;
use colored::*;
use failure::ResultExt;
use git2::ObjectType;
use git2::Odb;
use git2::Oid;
use git2::Repository;
use git2::RepositoryInitOptions;
use git2::StatusOptions;
use git2::build::CheckoutBuilder;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use super::cli::BranchCommand;
use super::cli::Command;
use super::cli;
use super::highlight_named_oid;
use super::print_commit;

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

pub fn main(args: cli::Root) -> Result<(), Error> {
  match args.command {
    Command::Branch(cmd_args) => {
      println!("branch {:?}", cmd_args);
      match cmd_args.command {
        BranchCommand::Ls(_subcmd_args) => (),
        BranchCommand::Mv(_subcmd_args) => (),
        BranchCommand::New(_subcmd_args) => (),
        BranchCommand::Rename(_subcmd_args) => (),
        BranchCommand::Rm(_subcmd_args) => (),
      }
      Ok(())
    }
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

pub fn clean(globals: cli::Global, args: cli::Clean) -> Result<(), Error> {
  let repo = Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;
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

pub fn commit(_globals: cli::Global, _args: cli::Commit) -> Result<(), Error> {
  Ok(())
}

pub fn diff(_globals: cli::Global, _args: cli::Diff) -> Result<(), Error> {
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

pub fn ls(_globals: cli::Global, _args: cli::Ls) -> Result<(), Error> {
  Ok(())
}

pub fn me(globals: cli::Global, _args: cli::Me) -> Result<(), Error> {
  let repo = Repository::discover(globals.repo_path).with_context(|_| "couldn't open repository")?;

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

pub fn restore(_globals: cli::Global, _args: cli::Restore) -> Result<(), Error> {
  Ok(())
}

pub fn show(_globals: cli::Global, _args: cli::Show) -> Result<(), Error> {
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

pub fn status(_globals: cli::Global, _args: cli::Status) -> Result<(), Error> {
  Ok(())
}

pub fn tag(_globals: cli::Global, _args: cli::Tag) -> Result<(), Error> {
  Ok(())
}

pub fn unstage(_globals: cli::Global, _args: cli::Unstage) -> Result<(), Error> {
  Ok(())
}

pub fn where_(_globals: cli::Global, _args: cli::Where) -> Result<(), Error> {
  Ok(())
}
