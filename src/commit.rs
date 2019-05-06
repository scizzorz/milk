use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Display status of work tree and index
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let sig = repo.signature().with_context(|_| "couldn't obtain signature")?;

  let mut index = repo.index().with_context(|_| "couldn't open index")?;
  let tree_id = index.write_tree().with_context(|_| "couldn't write tree")?;
  let tree = repo.find_tree(tree_id).with_context(|_| "couldn't find tree")?;

  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit HEAD")?;

  let parents = [&commit];

  let message = "HEHH";

  // FIXME fix commit message
  repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents).with_context(|_| "couldn't write commit")?;

  Ok(())
}
