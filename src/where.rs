use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  match repo.workdir() {
    Some(path) => match path.to_str() {
      Some(path_str) => println!("{}", path_str),
      None => println!("Path is not UTF-8"),
    },
    None => println!("Repository is bare."),
  }

  Ok(())
}
