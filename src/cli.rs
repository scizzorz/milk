use structopt::StructOpt;

/// A new front-end for Git.
#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub struct Root {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  pub repo_path: std::path::PathBuf,

  /// Don't print information
  #[structopt(long = "quiet", short = "q")]
  pub quiet: bool,

  #[structopt(subcommand)]
  pub command: Command,
}

#[derive(StructOpt, Debug)]
pub struct Init {
  /// Create a bare repository
  #[structopt(long = "bare")]
  pub bare: bool,
}

#[derive(StructOpt, Debug)]
pub struct List {
  /// Milk-style reference label to list
  #[structopt(short = "ref", long = "r", default_value = "/HEAD")]
  pub ref_name: String,

  /// Subtree path to list
  #[structopt(default_value = "")]
  pub tree_path: std::path::PathBuf,
}

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub enum Command {
  /// Initialize a new Git repository
  #[structopt(name = "init")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Init(Init),

  /// List the contents of a tree
  #[structopt(name = "ls")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  List(List),
}
