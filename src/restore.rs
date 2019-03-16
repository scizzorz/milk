use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use milk::find_from_name;
use std::fs::OpenOptions;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  object_name: String,
  path: std::path::PathBuf,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;
  let object =
    find_from_name(&repo, &args.object_name).with_context(|_| "couldn't look up object")?;

  let blob = match object.into_blob() {
    Ok(blob) => blob,
    Err(_) => {
      return Err(failure::err_msg("name didn't point to blob")).context("...?")?;
    }
  };

  let bytes = blob.content();

  let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open(args.path)
    .with_context(|_| "couldn't open file for writing")?;

  file
    .write_all(bytes)
    .with_context(|_| "couldn't write to file")?;

  Ok(())
}
