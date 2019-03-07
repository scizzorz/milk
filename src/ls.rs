use colored::*;
use exitcode;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::ObjectType;
use git2::Oid;
use milk::highlight_named_oid;
use git2::Repository;
use git2::Tree;
use milk::get_short_id;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  #[structopt(short = "ref", long = "r", default_value = "HEAD")]
  ref_name: String,
  #[structopt(default_value = "")]
  tree_path: std::path::PathBuf,
}

fn print_tree(repo: &Repository, tree: &Tree) {
  for entry in tree.iter() {
    let raw_name = entry.name().unwrap_or("[???]");
    let name = match entry.kind() {
      Some(ObjectType::Tree) => format!(
        "{}/ {}",
        raw_name.blue(),
        get_short_id(repo, entry.id()).bright_black()
      ),
      Some(ObjectType::Commit) => format!(
        "@{} {}",
        raw_name.bright_red(),
        get_short_id(repo, entry.id()).bright_black()
      ),
      Some(ObjectType::Tag) => format!(
        "#{} {}",
        raw_name.bright_cyan(),
        get_short_id(repo, entry.id()).bright_black()
      ),
      _ => format!(
        "{} {}",
        raw_name,
        get_short_id(repo, entry.id()).bright_black()
      ),
    };

    println!("{}", name);
  }
}

fn find_subtree(tree: &Tree, name: &str) -> Option<Oid> {
  for entry in tree.iter() {
    let raw_name = entry.name().unwrap_or("[???]");
    if raw_name == name {
      return Some(entry.id());
    }
  }
  None
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(&args.repo_path).with_context(|_| "couldn't open repository")?;

  // FIXME this isn't a good way to look up references
  let ref_ = repo
    .find_reference(&args.ref_name)
    .with_context(|_| format!("couldn't find ref `{}`", args.ref_name))?;

  let commit = ref_
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit")?;

  // tf do I do if these aren't UTF-8? Quit?
  let head_name = ref_.shorthand().unwrap_or("[???]");

  println!(
    "{}",
    highlight_named_oid(&repo, head_name, commit.id())
  );

  if args.tree_path.is_absolute() {
    eprintln!("Tree path must be relative");
    exit(exitcode::USAGE);
  }

  let mut tree = commit.tree().with_context(|_| "couldn't find tree")?;

  for frag in args.tree_path.iter() {
    let frag_name = match frag.to_str() {
      Some(x) => x,
      None => {
        eprintln!("Tree path must be valid UTF-8");
        exit(exitcode::USAGE);
      }
    };

    match find_subtree(&tree, &frag_name) {
      Some(next_tree_id) => {
        println!(
          "{}/ {}",
          frag_name.cyan(),
          get_short_id(&repo, next_tree_id).bright_black()
        );
        tree = repo
          .find_tree(next_tree_id)
          .with_context(|_| "couldn't find tree")?;
      }
      None => {
        eprintln!("Subtree `{}` did not exist", frag_name);
        exit(exitcode::USAGE);
      }
    };
  }

  print_tree(&repo, &tree);

  Ok(())
}
