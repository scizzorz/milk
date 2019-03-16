use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::build::CheckoutBuilder;
use git2::ObjectType;
use git2::Odb;
use git2::Oid;
use git2::Repository;
use git2::StatusOptions;
use milk::highlight_named_oid;
use std::fs::File;
use std::io::Read;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// The paths to clean
  #[structopt(raw())]
  paths: Vec<String>,
}

fn write_blob(odb: &Odb, path: &str) -> Result<Oid, ExitFailure> {
  let mut handle = File::open(path)?;
  let mut bytes = Vec::new();
  let _size = handle.read_to_end(&mut bytes)?;
  let oid = odb.write(ObjectType::Blob, &bytes)?;
  Ok(oid)
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;
  let odb = repo.odb().with_context(|_| "couldn't open odb")?;

  let mut checkout = CheckoutBuilder::new();
  checkout.force();

  for path in &args.paths {
    checkout.path(path);
  }

  if !args.paths.is_empty() {
    for path in &args.paths {
      let oid = write_blob(&odb, path)?;
      println!("{}", highlight_named_oid(&repo, path, oid));
    }
  } else {
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(false);
    status_opts.include_ignored(false);

    let statuses = repo
      .statuses(Some(&mut status_opts))
      .with_context(|_| "couldn't open status")?;

    for entry in statuses.iter() {
      if let Some(path) = entry.path() {
        let status = entry.status();
        if status.is_wt_modified() || status.is_index_modified() {
          let oid = write_blob(&odb, path)?;
          println!("{}", highlight_named_oid(&repo, path, oid));
        }
      }
    }
  }

  repo
    .checkout_head(Some(&mut checkout))
    .with_context(|_| "couldn't checkout")?;

  Ok(())
}
