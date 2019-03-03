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

  #[structopt(default_value = ".")]
  path: std::path::PathBuf,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let mut repo_opts = RepositoryInitOptions::new();
  repo_opts.bare(args.bare);
  repo_opts.no_reinit(true);

  let _repo = Repository::init_opts(args.path, &repo_opts)
    .with_context(|_| format!("couldn't initialize repository"))?;

  if !args.quiet {
    println!("Initialized repository.");
  }

  Ok(())
}
