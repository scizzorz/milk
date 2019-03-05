use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use milk::find_from_name;
use milk::get_short_id;
use milk::git_to_chrono;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let object = find_from_name(&repo, &args.name);
  Ok(())
}
