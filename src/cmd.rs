use super::cli;
use super::cli::Command;
use super::highlight_named_oid;
use super::print_commit;
use failure::Error;
use failure::ResultExt;
use git2::Repository;
use git2::RepositoryInitOptions;
use std::path::Path;

pub fn main(args: cli::Root) -> Result<(), Error> {
  match args.command {
    Command::Clean(cmd_args) => clean(&args.globals, &cmd_args),
    Command::Commit(cmd_args) => commit(&args.globals, &cmd_args),
    Command::Diff(cmd_args) => diff(&args.globals, &cmd_args),
    Command::Head(cmd_args) => head(&args.globals, &cmd_args),
    Command::Ignore(cmd_args) => ignore(&args.globals, &cmd_args),
    Command::Init(cmd_args) => init(&args.globals, &cmd_args),
    Command::Ls(cmd_args) => ls(&args.globals, &cmd_args),
    Command::Me(cmd_args) => me(&args.globals, &cmd_args),
    Command::Restore(cmd_args) => restore(&args.globals, &cmd_args),
    Command::Show(cmd_args) => show(&args.globals, &cmd_args),
    Command::Stage(cmd_args) => stage(&args.globals, &cmd_args),
    Command::Status(cmd_args) => status(&args.globals, &cmd_args),
    Command::Tag(cmd_args) => tag(&args.globals, &cmd_args),
    Command::Unstage(cmd_args) => unstage(&args.globals, &cmd_args),
    Command::Where(cmd_args) => where_(&args.globals, &cmd_args),
  }?;

  Ok(())
}

pub fn clean(_globals: &cli::Global, _args: &cli::Clean) -> Result<(), Error> {
  Ok(())
}

pub fn commit(_globals: &cli::Global, _args: &cli::Commit) -> Result<(), Error> {
  Ok(())
}

pub fn diff(_globals: &cli::Global, _args: &cli::Diff) -> Result<(), Error> {
  Ok(())
}

pub fn head(globals: &cli::Global, _args: &cli::Head) -> Result<(), Error> {
  let repo =
    Repository::discover(&globals.repo_path).with_context(|_| "couldn't open repository")?;
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

pub fn ignore(_globals: &cli::Global, _args: &cli::Ignore) -> Result<(), Error> {
  Ok(())
}

pub fn init(globals: &cli::Global, args: &cli::Init) -> Result<(), Error> {
  let mut repo_opts = RepositoryInitOptions::new();
  repo_opts.bare(args.bare);
  repo_opts.no_reinit(true);

  let _repo = Repository::init_opts(&globals.repo_path, &repo_opts)
    .with_context(|_| "couldn't initialize repository")?;

  if !globals.quiet {
    println!("Initialized repository.");
  }

  Ok(())
}

pub fn ls(_globals: &cli::Global, _args: &cli::Ls) -> Result<(), Error> {
  Ok(())
}

pub fn me(_globals: &cli::Global, _args: &cli::Me) -> Result<(), Error> {
  Ok(())
}

pub fn restore(_globals: &cli::Global, _args: &cli::Restore) -> Result<(), Error> {
  Ok(())
}

pub fn show(_globals: &cli::Global, _args: &cli::Show) -> Result<(), Error> {
  Ok(())
}

pub fn stage(globals: &cli::Global, args: &cli::Stage) -> Result<(), Error> {
  let repo =
    Repository::discover(&globals.repo_path).with_context(|_| "couldn't open repository")?;

  let mut index = repo.index().with_context(|_| "couldn't open index")?;

  for path in &args.paths {
    index
      .add_path(Path::new(&path))
      .with_context(|_| "couldn't add path")?;
    println!("Staged {}", path);
  }

  index.write().with_context(|_| "couldn't write index")?;

  Ok(())
}

pub fn status(_globals: &cli::Global, _args: &cli::Status) -> Result<(), Error> {
  Ok(())
}

pub fn tag(_globals: &cli::Global, _args: &cli::Tag) -> Result<(), Error> {
  Ok(())
}

pub fn unstage(_globals: &cli::Global, _args: &cli::Unstage) -> Result<(), Error> {
  Ok(())
}

pub fn where_(_globals: &cli::Global, _args: &cli::Where) -> Result<(), Error> {
  Ok(())
}
