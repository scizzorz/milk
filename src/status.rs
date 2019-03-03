use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use git2::Status;
use git2::StatusOptions;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(default_value = ".")]
  path: std::path::PathBuf,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let status_opts = StatusOptions::new();

  let repo = Repository::open(args.path).with_context(|_| "couldn't open repository")?;
  let _state = repo.state();

  println!("path  index  working");
  for entry in repo.statuses(None).with_context(|_| "couldn't open status")?.iter() {
    let path = entry.path().unwrap();
    let status = entry.status();
    let index_string = if status.is_index_new() {
      "new"
    } else if status.is_index_modified() {
      "modified"
    } else if status.is_index_deleted() {
      "deleted"
    } else if status.is_index_renamed() {
      "renamed"
    } else if status.is_index_typechange() {
      "typechange"
    } else {
      "    "
    };
    let working_string = if status.is_wt_new() {
      "new"
    } else if status.is_wt_modified() {
      "modified"
    } else if status.is_wt_deleted() {
      "deleted"
    } else if status.is_wt_renamed() {
      "renamed"
    } else if status.is_wt_typechange() {
      "typechange"
    } else {
      "    "
    };

    let status_string = if status.is_ignored() {
      format!("{} ignored", path)
    } else if status.is_conflicted() {
      format!("{} conflicted", path)
    } else {
      format!("{} {} {}", path, index_string, working_string)
    };

    println!("{}", status_string);
  }

  Ok(())
}
