use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::ObjectType;
use git2::Repository;
use git2::ResetType;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Unstage files from the index
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// Paths to unstage
  #[structopt(raw())]
  paths: Vec<String>,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

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
