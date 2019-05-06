use exitfailure::ExitFailure;
use failure::Error;
use failure::ResultExt;
use git2::Repository;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Display status of work tree and index
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
}

fn temporary_editor(path: &Path, contents: &str) -> Result<String, Error> {
  let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open(path)
    .with_context(|_| "couldn't open temp commit message file")?;

  file
    .write_all(contents.as_bytes())
    .with_context(|_| "couldn't write temp commit message")?;
  file
    .sync_all()
    .with_context(|_| "couldn't sync file contents")?;

  // FIXME one of the Err cases here is a non-unicode value... I'd assume you
  // can run a non-unicode command, no?
  let editor = match env::var("VISUAL") {
    Ok(val) => val,
    Err(_) => match env::var("EDITOR") {
      Ok(val) => val,
      Err(_) => panic!("Neither $VISUAL nor $EDITOR is happy."),
    },
  };

  let mut editor_command = Command::new(editor);
  editor_command.arg(&path);

  let mut editor_proc = editor_command
    .spawn()
    .with_context(|_| "couldn't spawn editor")?;

  editor_proc
    .wait()
    .with_context(|_| "editor failed for some reason")?;

  let mut file = File::open(path).with_context(|_| "couldn't re-open file")?;

  let mut contents = String::new();
  file
    .read_to_string(&mut contents)
    .with_context(|_| "couldn't read from file")?;

  Ok(contents)
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let sig = repo
    .signature()
    .with_context(|_| "couldn't obtain signature")?;

  let mut index = repo.index().with_context(|_| "couldn't open index")?;
  let tree_id = index.write_tree().with_context(|_| "couldn't write tree")?;
  let tree = repo
    .find_tree(tree_id)
    .with_context(|_| "couldn't find tree")?;

  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit HEAD")?;

  let parents = [&commit];

  let mut message_file_path = PathBuf::new();
  message_file_path.push(repo.path());
  message_file_path.push("COMMIT_EDITMSG");

  let message =
    temporary_editor(&message_file_path, "").with_context(|_| "couldn't get message")?;

  // FIXME fix commit message
  repo
    .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents)
    .with_context(|_| "couldn't write commit")?;

  Ok(())
}
