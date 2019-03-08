use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  /// Unstage all files
  #[structopt(long = "all", short = "a")]
  all: bool,
  /// The paths to unstage
  #[structopt(raw())]
  paths: Vec<String>,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel(git2::ObjectType::Any)
    .with_context(|_| "couldn't peel to commit HEAD")?;

  if args.all {
    repo
      .reset(&commit, git2::ResetType::Mixed, None)
      .with_context(|_| "could not reset to HEAD")?;
  } else if args.paths.len() > 0 {
    repo
      .reset_default(Some(&commit), args.paths)
      .with_context(|_| "could not reset paths")?;
  } else {
    println!("Doing nothing\nPass -a to unstage all files")
  }

  Ok(())
}
