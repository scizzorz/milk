use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::ObjectType;
use git2::Repository;
use milk::get_short_id;
use milk::git_to_chrono;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  #[structopt(short = "ref", long = "r", default_value = "HEAD")]
  ref_name: String,
  tag_name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(&args.repo_path).with_context(|_| "couldn't open repository")?;

  // FIXME this isn't a good way to look up references
  let ref_ = repo
    .find_reference(&args.ref_name)
    .with_context(|_| format!("couldn't find ref `{}`", args.ref_name))?;

  let object = ref_
    .peel(ObjectType::Any)
    .with_context(|_| "couldn't peel ref")?;
  println!("{}", get_short_id(&repo, object.id()).bright_black());

  // TODO add annotated tags
  repo
    .tag_lightweight(&args.tag_name, &object, false)
    .with_context(|_| "couldn't create tag")?;

  Ok(())
}
