use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use git2::Status;
use git2::StatusOptions;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  #[structopt(long = "hide-untracked", short = "u")]
  hide_untracked: bool,
  #[structopt(long = "show-ignored", short = "i")]
  show_ignored: bool,
}

fn get_status_string(status: Status) -> String {
  let index_string = if status.is_index_new() {
    "new".cyan()
  } else if status.is_index_modified() {
    "mod".green()
  } else if status.is_index_deleted() {
    "del".red()
  } else if status.is_index_renamed() {
    "ren".blue()
  } else if status.is_index_typechange() {
    "typ".blue()
  } else {
    "   ".normal()
  };

  let working_string = if status.is_wt_new() {
    "new".bright_cyan()
  } else if status.is_wt_modified() {
    "mod".bright_green()
  } else if status.is_wt_deleted() {
    "del".bright_red()
  } else if status.is_wt_renamed() {
    "ren".bright_blue()
  } else if status.is_wt_typechange() {
    "typ".bright_blue()
  } else {
    "   ".normal()
  };

  if status.is_ignored() {
    format!("{}", " ignored".white())
  } else if status.is_conflicted() {
    format!("{}", "conflict".red())
  } else {
    format!(" {} {}", index_string, working_string)
  }
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let mut status_opts = StatusOptions::new();
  status_opts.include_untracked(!args.hide_untracked);
  status_opts.include_ignored(args.show_ignored);

  let repo = Repository::discover(args.repo_path).with_context(|_| "couldn't open repository")?;

  let statuses = repo
    .statuses(Some(&mut status_opts))
    .with_context(|_| "couldn't open status")?;

  for entry in statuses.iter() {
    let path = entry.path().unwrap_or("[invalid utf-8]");
    let status = entry.status();
    let status_string = get_status_string(status);

    println!("{} {}", status_string, path);
  }

  Ok(())
}
