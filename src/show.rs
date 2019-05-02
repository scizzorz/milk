use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::ObjectType;
use git2::Repository;
use milk::find_from_name;
use milk::highlight_named_oid;
use milk::print_blob;
use milk::print_commit;
use milk::print_tag;
use milk::print_tree;
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

  match object.kind() {
    Some(ObjectType::Blob) => {
      println!("{}", highlight_named_oid(&repo, "blob", object.id()));
      let blob = object.into_blob().unwrap();
      print_blob(&repo, &blob);
    }
    Some(ObjectType::Tree) => {
      println!("{}", highlight_named_oid(&repo, "tree", object.id()));
      let tree = object.into_tree().unwrap();
      print_tree(&repo, &tree);
    }
    Some(ObjectType::Commit) => {
      println!("{}", highlight_named_oid(&repo, "commit", object.id()));
      let commit = object.into_commit().unwrap();
      print_commit(&repo, &commit);
    }
    Some(ObjectType::Tag) => {
      println!("{}", highlight_named_oid(&repo, "tag", object.id()));
      let tag = object.into_tag().unwrap();
      print_tag(&repo, &tag);
    }
    _ => {
      println!("{}", highlight_named_oid(&repo, "unknown", object.id()));
    }
  }

  Ok(())
}
