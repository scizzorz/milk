use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::ObjectType;
use git2::Repository;
use milk::find_from_name;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  #[structopt(long = "ref", short = "r", default_value = "/HEAD")]
  ref_name: String,
  #[structopt(long = "force", short = "f")]
  force: bool,
  name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;
  let object = find_from_name(&repo, &args.ref_name).with_context(|_| "couldn't look up ref")?;

  if let Some(ObjectType::Commit) = object.kind() {
    let commit = object.into_commit().unwrap();

    repo
      .branch(&args.name, &commit, args.force)
      .with_context(|_| "couldn't create branch")?;
  } else {
    Err(failure::err_msg("object was not a commit"))?;
  }

  Ok(())
}
