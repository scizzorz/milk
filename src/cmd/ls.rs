use colored::*;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Repository;
use git2::ObjectType;
use git2::StatusOptions;
use git2::Time;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,
  #[structopt(short = "ref", long = "r", default_value = "HEAD")]
  ref_name: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  let mut status_opts = StatusOptions::new();
  status_opts.include_untracked(true);

  let repo = Repository::open(&args.repo_path).with_context(|_| "couldn't open repository")?;

  // FIXME this isn't a good way to look up references
  let ref_ = repo.find_reference(&args.ref_name).with_context(|_| format!("couldn't find ref `{}`", args.ref_name))?;
  let commit = ref_
    .peel_to_commit()
    .with_context(|_| "couldn't peel to commit")?;

  // tf do I do if these aren't UTF-8? Quit?
  let head_name = ref_.shorthand().unwrap_or("[???]");

  println!(
    "{} {}",
    head_name.cyan(),
    commit.id().to_string().bright_black()
  );

  // FIXME if `args.path` is provided, it needs to be used
  let tree = commit.tree().with_context(|_| "couldn't find tree")?;

  for entry in tree.iter() {
    let raw_name = entry.name().unwrap_or("[???]");
    let name = match entry.kind() {
      Some(ObjectType::Tree) => format!("{}/ {}", raw_name.blue(), entry.id().to_string().bright_black()),
      Some(ObjectType::Commit) => format!("@{} {}", raw_name.bright_red(), entry.id().to_string().bright_black()),
      Some(ObjectType::Tag) => format!("#{} {}", raw_name.bright_cyan(), entry.id().to_string().bright_black()),
      _ => format!("{} {}", raw_name, entry.id().to_string().bright_black()),
    };

    println!("{}", name);
  }

  Ok(())
}
