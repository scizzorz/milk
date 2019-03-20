use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::BranchType;
use git2::ObjectType;
use git2::Repository;
use milk::find_from_name;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  src_name: String,
  dest_ref: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let _src_branch = repo
    .find_branch(&args.src_name, BranchType::Local)
    .with_context(|_| "couldn't find source branch")?;

  let dest_object =
    find_from_name(&repo, &args.dest_ref).with_context(|_| "couldn't look up dest ref")?;

  if let Some(ObjectType::Commit) = dest_object.kind() {
    let commit = dest_object.into_commit().unwrap();

    repo
      .branch(&args.src_name, &commit, true)
      .with_context(|_| "couldn't create branch")?;
  } else {
    Err(failure::err_msg("dest object was not a commit"))?;
  }

  Ok(())
}
