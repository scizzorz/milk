use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::BranchType;
use git2::Repository;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  #[structopt(long = "remote", short = "r")]
  is_remote: bool,
  #[structopt(long = "force", short = "f")]
  force: bool,
  from_name: String,
  to_name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let typ = if args.is_remote {
    BranchType::Remote
  } else {
    BranchType::Local
  };

  let mut branch = repo
    .find_branch(&args.from_name, typ)
    .with_context(|_| "couldn't find branch")?;

  branch
    .rename(&args.to_name, args.force)
    .with_context(|_| "couldn't rename branch")?;

  Ok(())
}
