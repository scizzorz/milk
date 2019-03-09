use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::ObjectType;
use git2::Repository;
use git2::ResetType;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

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
    .peel(ObjectType::Any)
    .with_context(|_| "couldn't peel to commit HEAD")?;

  if args.paths.len() > 0 {
    println!("clean {:?}", args.paths);
  } else {
    println!("clean");
  }

  Ok(())
}
