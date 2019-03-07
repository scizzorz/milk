use milk::highlight_named_oid;
use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use milk::get_short_id;
use milk::git_to_chrono;
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
    "{}",
    highlight_named_oid(&repo, head_name, commit.id())
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
