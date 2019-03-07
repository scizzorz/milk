use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::ObjectType;
use git2::Repository;
use milk::find_from_name;
use milk::get_short_id;
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

  let object = find_from_name(&repo, &args.name).with_context(|_| "couldn't look up object")?;

  match object.kind() {
    Some(ObjectType::Blob) => {
      println!("{} {}", "blob".cyan(), get_short_id(&repo, object.id()).bright_black());
    }
    Some(ObjectType::Tree) => {
      println!("{} {}", "tree".cyan(), get_short_id(&repo, object.id()).bright_black());
    }
    Some(ObjectType::Commit) => {
      println!("{} {}", "commit".cyan(), get_short_id(&repo, object.id()).bright_black());
    }
    Some(ObjectType::Tag) => {
      println!("{} {}", "tag".cyan(), get_short_id(&repo, object.id()).bright_black());
    }
    _ => {
      println!("{} {}", "unknown".cyan(), get_short_id(&repo, object.id()).bright_black());
    }
  }

  Ok(())
}
