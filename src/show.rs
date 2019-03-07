use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::ObjectType;
use git2::Repository;
use milk::find_from_name;
use milk::get_short_id;
use milk::highlight_named_oid;
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
      println!("{}", highlight_named_oid(&repo, "blob", object.id()));
    }
    Some(ObjectType::Tree) => {
      println!("{}", highlight_named_oid(&repo, "tree", object.id()));
    }
    Some(ObjectType::Commit) => {
      println!("{}", highlight_named_oid(&repo, "commit", object.id()));
    }
    Some(ObjectType::Tag) => {
      println!("{}", highlight_named_oid(&repo, "tag", object.id()));
    }
    _ => {
      println!("{}", highlight_named_oid(&repo, "unknown", object.id()));
    }
  }

  Ok(())
}
