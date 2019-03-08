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
  include_remote: bool,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let branches = repo.branches(None).with_context(|_| "couldn't iterate branches")?;

  for branch in branches {
    let (branch, typ) = branch.with_context(|_| "couldn't identify branch")?;
    let name = branch.name().with_context(|_| "couldn't identify branch name")?.unwrap_or("[branch name is invalid utf8]");
    let head_prefix = if branch.is_head() {
      "*"
    } else {
      " "
    };
    match (typ, args.include_remote) {
      (BranchType::Remote, false) => (),
      _ => println!("{} {}", head_prefix, name),
    }
  }

  Ok(())
}

