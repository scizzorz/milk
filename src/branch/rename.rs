use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::BranchType;
use git2::Repository;
use milk::highlight_named_oid;
use milk::print_commit;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  #[structopt(long = "remote", short = "r")]
  is_remote: bool,
  #[structopt(long = "force", short = "f")]
  force: bool,
  src_name: String,
  dest_name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let typ = if args.is_remote {
    BranchType::Remote
  } else {
    BranchType::Local
  };

  let mut branch = repo
    .find_branch(&args.src_name, typ)
    .with_context(|_| "couldn't find branch")?;

  branch
    .rename(&args.dest_name, args.force)
    .with_context(|_| "couldn't rename branch")?;

  let target = branch
    .get()
    .target()
    .ok_or_else(|| failure::err_msg("couldn't get branch target"))?;

  let commit = repo
    .find_commit(target)
    .with_context(|_| "couldn't find commit")?;

  println!("Renamed branch {}", args.src_name);
  println!("{}", highlight_named_oid(&repo, &args.dest_name, target));
  print_commit(&repo, &commit);

  Ok(())
}
