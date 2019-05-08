use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::DiffOptions;
use git2::Repository;
use milk::find_from_name;
use structopt::StructOpt;

/// Create a new commit
#[derive(StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// Milk-style reference label to diff
  #[structopt(default_value = "/HEAD")]
  ref_name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let mut options = DiffOptions::new();

  let object = find_from_name(&repo, &args.ref_name).with_context(|_| "couldn't find refname")?;
  let tree = object
    .peel_to_tree()
    .with_context(|_| "couldn't peel to commit HEAD")?;

  let diff = repo
    .diff_tree_to_workdir(Some(&tree), Some(&mut options))
    .with_context(|_| "failed to diff")?;

  // this API is literally insane
  // example code yanked from here: https://github.com/rust-lang/git2-rs/blob/master/examples/diff.rs#L153-L179
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
