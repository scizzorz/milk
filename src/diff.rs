use colored::*;
use exitfailure::ExitFailure;
use failure::format_err;
use failure::Error;
use failure::ResultExt;
use git2::Diff;
use git2::DiffOptions;
use git2::Repository;
use git2::Tree;
use milk::find_from_name;
use structopt::StructOpt;

/// Create a new commit
#[derive(StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// Milk-style reference label to "old" tree-ish
  ///
  /// Includes special /INDEX and /WORK options to represent the work tree and
  /// the index, respectively.
  #[structopt(default_value = "/INDEX")]
  old_tree: String,

  /// Milk-style reference label to "new" tree-ish
  #[structopt(default_value = "/WORK")]
  new_tree: String,
}

enum DiffTarget<'a> {
  WorkingTree,
  Index,
  Name(&'a str),
}

impl<'a> DiffTarget<'a> {
  fn from_str(s: &str) -> DiffTarget {
    match s {
      "/WORK" => DiffTarget::WorkingTree,
      "/INDEX" => DiffTarget::Index,
      _ => DiffTarget::Name(s),
    }
  }
}

fn name_to_tree<'repo>(repo: &'repo Repository, s: &str) -> Result<Tree<'repo>, Error> {
  let object = find_from_name(repo, s).with_context(|_| "couldn't find refname")?;
  let tree = object
    .peel_to_tree()
    .with_context(|_| "couldn't peel to commit HEAD")?;
  Ok(tree)
}

fn make_diff<'repo>(
  repo: &'repo Repository,
  old_target: DiffTarget,
  new_target: DiffTarget,
) -> Result<Diff<'repo>, Error> {
  let mut options = DiffOptions::new();

  match (old_target, new_target) {
    // tree..
    (DiffTarget::Name(old), DiffTarget::WorkingTree) => {
      let old_tree = name_to_tree(&repo, old).with_context(|_| "couldn't look up old tree")?;

      let diff = repo
        .diff_tree_to_workdir(Some(&old_tree), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }

    (DiffTarget::Name(old), DiffTarget::Name(new)) => {
      let old_tree = name_to_tree(&repo, old).with_context(|_| "couldn't look up old tree")?;
      let new_tree = name_to_tree(&repo, new).with_context(|_| "couldn't look up new tree")?;

      let diff = repo
        .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }

    (DiffTarget::Name(old), DiffTarget::Index) => {
      let old_tree = name_to_tree(&repo, old).with_context(|_| "couldn't look up old tree")?;
      let index = repo.index().with_context(|_| "couldn't read index")?;

      let diff = repo
        .diff_tree_to_index(Some(&old_tree), Some(&index), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }

    // index..
    (DiffTarget::Index, DiffTarget::WorkingTree) => {
      let index = repo.index().with_context(|_| "couldn't read index")?;
      let diff = repo
        .diff_index_to_workdir(Some(&index), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;

      Ok(diff)
    }

    (DiffTarget::Index, DiffTarget::Name(new)) => {
      let index = repo.index().with_context(|_| "couldn't read index")?;
      let new_tree = name_to_tree(&repo, new).with_context(|_| "couldn't look up new tree")?;
      options.reverse(true);

      let diff = repo
        .diff_tree_to_index(Some(&new_tree), Some(&index), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }

    (DiffTarget::Index, DiffTarget::Index) => {
      // FIXME why? it probably works...
      Err(format_err!("Cannot diff between identical targets"))
    }

    // working..
    (DiffTarget::WorkingTree, DiffTarget::WorkingTree) => {
      // FIXME why? it probably works...
      Err(format_err!("Cannot diff between identical targets"))
    }
    (DiffTarget::WorkingTree, DiffTarget::Name(new)) => {
      let new_tree = name_to_tree(&repo, new).with_context(|_| "couldn't look up new tree")?;
      options.reverse(true);

      let diff = repo
        .diff_tree_to_workdir(Some(&new_tree), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;
      Ok(diff)
    }
    (DiffTarget::WorkingTree, DiffTarget::Index) => {
      let index = repo.index().with_context(|_| "couldn't read index")?;
      options.reverse(true);
      let diff = repo
        .diff_index_to_workdir(Some(&index), Some(&mut options))
        .with_context(|_| "couldn't generate diff")?;

      Ok(diff)
    }
  }
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let old_target = DiffTarget::from_str(&args.old_tree);
  let new_target = DiffTarget::from_str(&args.new_tree);

  let diff = make_diff(&repo, old_target, new_target).with_context(|_| "failed to diff")?;

  // this API is literally insane
  // example code yanked from here:
  //   https://github.com/rust-lang/git2-rs/blob/master/examples/diff.rs#L153-L179
  diff
    .print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
      let display = std::str::from_utf8(line.content()).unwrap();
      match line.origin() {
        '+' => print!("{}{}", "+".green(), display.green()),
        '-' => print!("{}{}", "-".red(), display.red()),
        ' ' => print!(" {}", display.white()),
        _ => print!("{}", display.cyan()),
      }
      true
    })
    .with_context(|_| "failed to print diff")?;

  Ok(())
}
