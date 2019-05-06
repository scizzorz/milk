use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use milk::find_from_name;
use milk::print_object;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Display the contents of an object
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// Milk-style reference label to object
  #[structopt(default_value = "/HEAD")]
  name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let object = find_from_name(&repo, &args.name).with_context(|_| "couldn't look up object")?;

  print_object(&repo, &object);

  Ok(())
}
