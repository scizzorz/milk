use colored::*;
use exitcode;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Oid;
use git2::Repository;
use git2::Tree;
use milk::find_from_name;
use milk::get_short_id;
use milk::highlight_named_oid;
use milk::print_tree;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt)]
/// List the contents of a tree
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// Milk-style reference label to list
  #[structopt(short = "ref", long = "r", default_value = "/HEAD")]
  ref_name: String,

  /// Subtree path to list
  #[structopt(default_value = "")]
  tree_path: std::path::PathBuf,
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

  let object = find_from_name(&repo, &args.ref_name).with_context(|_| "couldn't find refname")?;

  let commit = match object.into_commit() {
    Ok(commit) => commit,
    Err(_) => {
      return Err(failure::err_msg("refname didn't point to commit")).context("...?")?;
    }
  };

  println!(
    "{}",
    highlight_named_oid(&repo, &args.ref_name, commit.id())
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
