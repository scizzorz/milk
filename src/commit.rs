use exitfailure::ExitFailure;
use failure::Error;
use failure::ResultExt;
use git2::Repository;
use milk::highlight_named_oid;
use milk::print_commit;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
/// Create a new commit
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
}

fn temporary_editor(path: &Path, contents: &str) -> Result<String, Error> {
  // FIXME one of the Err cases here is for a non-unicode value... I'd assume you
  // can run a non-unicode command, no?
  let editor = env::var("EDITOR").with_context(|_| "$EDITOR is not defined.")?;

  let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open(path)
    .with_context(|_| "couldn't open $EDITOR file")?;

  file
    .write_all(contents.as_bytes())
    .with_context(|_| "couldn't write $EDITOR file contents")?;

  file
    .sync_all()
    .with_context(|_| "couldn't sync $EDITOR file contents")?;

  let mut editor_command = Command::new(editor);
  editor_command.arg(&path);

  editor_command
    .spawn()
    .and_then(|mut handle| handle.wait())
    .with_context(|_| "$EDITOR failed for some reason")?;

  let mut file = File::open(path).with_context(|_| "couldn't re-open file")?;

  let mut contents = String::new();
  file
    .read_to_string(&mut contents)
    .with_context(|_| "couldn't read from file")?;

  if std::fs::remove_file(&path).is_err() {
    eprintln!("WARNING: Unable to delete {} after use", path.display());
  }

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
  let message = message.trim();

  if message.is_empty() {
    eprintln!("Aborting due to empty commit message.");
    exit(exitcode::DATAERR);
  }

  let new_commit_id = repo
    .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents)
    .with_context(|_| "couldn't write commit")?;

  let new_commit = repo
    .find_commit(new_commit_id)
    .with_context(|_| "couldn't find commit")?;

  let head_name = head.shorthand().unwrap_or("[???]");
  println!("{}", highlight_named_oid(&repo, head_name, commit.id()));
  print_commit(&repo, &new_commit);

  Ok(())
}
