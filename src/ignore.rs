use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Ignore files or patterns
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// Interpret paths as glob patterns and add them to .gitignore unmodified
  #[structopt(long = "pattern", short = "-P")]
  is_pattern: bool,

  /// The file or pattern to ignore
  pattern: String,
}

fn handle_file(
  repo: &Repository,
  filepath: String,
  workdir: &Path,
) -> Result<Option<String>, ExitFailure> {
  let path = Path::new(&filepath);

  if !path.exists() {
    print!("File '{}' does not exist, still ignore? [Y/n] ", filepath);
    io::stdout().flush().context("Could not flush stdout")?;

    let mut input = String::new();
    io::stdin()
      .read_line(&mut input)
      .context("Could not read stdin")?;

    match input.trim_end() {
      "y" | "Y" | "" => (),
      _ => return Ok(None),
    }
  }

  // Try to transform path into its canonical path
  // from the workdir
  let path = match path.canonicalize() {
    Ok(abs_path) => match abs_path.strip_prefix(workdir) {
      Ok(rel_path) => rel_path.to_path_buf(),
      _ => path.to_path_buf(),
    },
    _ => path.to_path_buf(),
  };

  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit")?;
  let tree = commit.tree().with_context(|_| "couldn't locate tree")?;

  if tree.get_path(&path).is_ok() {
    println!("Warning: file {} is currently tracked by git", filepath);
  };

  let final_filepath = path
    .to_str()
    .ok_or_else(|| failure::err_msg("Path is not UTF-8"))?;
  Ok(Some(String::from(final_filepath)))
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).context("Couldn't open repository")?;

  let workdir_bytes = repo
    .workdir()
    .ok_or_else(|| failure::err_msg("repository is bare"))?;
  let workdir = Path::new(
    workdir_bytes
      .to_str()
      .ok_or_else(|| failure::err_msg("path is not utf-8"))?,
  );

  let to_ignore = if args.is_pattern {
    Some(args.pattern)
  } else {
    handle_file(&repo, args.pattern, &workdir)?
  };

  if let Some(to_ignore) = to_ignore {
    let gitignore_path = workdir.join(".gitignore");

    let mut gitignore = OpenOptions::new()
      .create(!gitignore_path.exists())
      .append(true)
      .open(workdir.join(".gitignore"))
      .context("Couldn't open .gitignore file")?;

    println!("Adding {} to .gitignore", to_ignore);
    writeln!(gitignore, "{}", to_ignore).context("Couldn't write to .gitignore file")?;
  };

  Ok(())
}
