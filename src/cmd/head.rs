use chrono::offset::FixedOffset;
use chrono::offset::TimeZone;
use chrono::DateTime;
use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use git2::Status;
use git2::StatusOptions;
use git2::Time;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(default_value = ".")]
  path: std::path::PathBuf,
}

fn git_to_chrono(sig: &Time) -> DateTime<FixedOffset> {
  let timestamp = sig.seconds();
  let offset_sec = sig.offset_minutes() * 60;
  let fixed_offset = FixedOffset::east(offset_sec);
  fixed_offset.timestamp(timestamp, 0)
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let mut status_opts = StatusOptions::new();
  status_opts.include_untracked(true);

  let repo = Repository::open(args.path).with_context(|_| "couldn't open repository")?;
  let head = repo.head().with_context(|_| "couldn't locate HEAD")?;
  let commit = head
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit HEAD")?;

  // tf do I do if these aren't UTF-8? Quit?
  let head_name = head.shorthand().unwrap_or("[???]");

  let author = commit.author();
  let author_name = author.name().unwrap_or("[???]");
  let author_email = author.email().unwrap_or("[???]");
  let author_time = git_to_chrono(&author.when());

  let committer = commit.committer();
  let committer_name = committer.name().unwrap_or("[???]");
  let committer_email = committer.email().unwrap_or("[???]");
  let committer_time = git_to_chrono(&committer.when());

  println!(
    "{} {}",
    head_name.cyan(),
    commit.id().to_string().bright_black()
  );
  println!(
    "{} {} {}",
    author_name.cyan(),
    author_email.bright_black(),
    author_time.to_string().bright_blue()
  );

  if author_name != committer_name || author_email != committer_email {
    println!(
      "committed by {} {} {}",
      committer_name.cyan(),
      committer_email.bright_black(),
      committer_time.to_string().bright_blue()
    );
  }

  println!("{}", commit.message().unwrap_or(""));

  Ok(())
}
