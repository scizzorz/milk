use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  file_path: std::path::PathBuf,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let mut index = repo.index().with_context(|_| "couldn't open index")?;

  index.add_path(&args.file_path).with_context(|_| "couldn't add path")?;

  index.write().with_context(|_| "couldn't write index")?;

  let printable_path = args.file_path.to_str().unwrap_or("path is not valid UTF-8");

  println!("Staged {}", printable_path);

  Ok(())
}
