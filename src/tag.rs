use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use milk::find_from_name;
use milk::print_object;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Create a new tag
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// Milk-style reference label to tag
  #[structopt(short = "ref", long = "r", default_value = "/HEAD")]
  ref_name: String,

  /// Name of created tag
  tag_name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(&args.repo_path).with_context(|_| "couldn't open repository")?;

  let object = find_from_name(&repo, &args.ref_name).with_context(|_| "couldn't look up object")?;
  print_object(&repo, &object);

  repo
    .tag_lightweight(&args.tag_name, &object, false)
    .with_context(|_| "couldn't create tag")?;

  Ok(())
}
