use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use git2::RepositoryInitOptions;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(short = "q", long = "quiet")]
  quiet: bool,

  #[structopt(long = "bare")]
  bare: bool,

  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let mut repo_opts = RepositoryInitOptions::new();
  repo_opts.bare(args.bare);
  repo_opts.no_reinit(true);

  let _repo = Repository::init_opts(args.repo_path, &repo_opts)
    .with_context(|_| "couldn't initialize repository")?;

  if !args.quiet {
    println!("Initialized repository.");
  }

  Ok(())
}
