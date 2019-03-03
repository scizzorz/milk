use ansi_term::Colour::Blue;
use ansi_term::Colour::Cyan;
use ansi_term::Colour::Green;
use ansi_term::Colour::Red;
use ansi_term::Colour::Yellow;
use ansi_term::Style;
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

fn get_status_string(status: &Status) -> String {
  let index_string = if status.is_index_new() {
    Green.paint("new")
  } else if status.is_index_modified() {
    Green.paint("mod")
  } else if status.is_index_deleted() {
    Red.paint("del")
  } else if status.is_index_renamed() {
    Blue.bold().paint("ren")
  } else if status.is_index_typechange() {
    Blue.paint("typ")
  } else {
    Style::new().paint("   ")
  };

  let working_string = if status.is_wt_new() {
    Green.bold().paint("new")
  } else if status.is_wt_modified() {
    Green.bold().paint("mod")
  } else if status.is_wt_deleted() {
    Red.bold().paint("del")
  } else if status.is_wt_renamed() {
    Blue.bold().paint("ren")
  } else if status.is_wt_typechange() {
    Blue.bold().paint("typ")
  } else {
    Style::new().paint("   ")
  };

  if status.is_ignored() {
    format!("{}", Cyan.paint(" ignored"))
  } else if status.is_conflicted() {
    format!("{}", Red.paint("conflict"))
  } else {
    format!(" {} {}", index_string, working_string)
  }
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let mut status_opts = StatusOptions::new();
  status_opts.include_untracked(true);

  let repo = Repository::open(args.path).with_context(|_| "couldn't open repository")?;
  let _state = repo.state();

  for entry in repo
    .statuses(Some(&mut status_opts))
    .with_context(|_| "couldn't open status")?
    .iter()
  {
    let path = entry.path().unwrap();
    let status = entry.status();
    let status_string = get_status_string(&status);

    println!("{} {}", status_string, path);
  }

  Ok(())
}
