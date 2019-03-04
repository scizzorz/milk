use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use git2::StatusOptions;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let mut status_opts = StatusOptions::new();
  status_opts.include_untracked(true);

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  // I don't know why this has to be this way
  // if you don't do the snapshot(), it crashes when reading a string
  // with an obscure error that's hard to Google:
  // "get_string called on a live config object; class=Config (7)"
  let mut config = repo.config().with_context(|_| "couldn't open config")?;
  let config = config
    .snapshot()
    .with_context(|_| "couldn't snapshot config")?;

  // read user name and email
  let user_name = config
    .get_str("user.name")
    .with_context(|_| "couldn't read user.name")?;
  let user_email = config
    .get_str("user.email")
    .with_context(|_| "couldn't read user.email")?;

  // print info
  println!("{} {}", user_name.cyan(), user_email.bright_black(),);
  Ok(())
}
