use structopt::StructOpt;

/// A new front-end for Git
#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub struct Root {
  #[structopt(flatten)]
  pub globals: Global,

  #[structopt(subcommand)]
  pub command: Command,
}

#[derive(StructOpt, Debug)]
pub struct Global {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  pub repo_path: std::path::PathBuf,

  /// Don't print information
  #[structopt(long = "quiet", short = "q")]
  pub quiet: bool,
}

// FIXME surely there's a way to propagate ColoredHelp to all members...?
#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub enum Command {
  /// Operate on branches
  #[structopt(name = "branch")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Branch(Branch),

  /// Reset untracked modifications to files
  #[structopt(name = "clean")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Clean(Clean),

  /// Create a new commit
  #[structopt(name = "commit")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Commit(Commit),

  /// Print a diff between two trees
  #[structopt(name = "diff")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Diff(Diff),

  /// Print information about HEAD
  #[structopt(name = "head")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Head(Head),

  /// Ignore files or patterns
  #[structopt(name = "ignore")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Ignore(Ignore),

  /// Initialize a new Git repository
  #[structopt(name = "init")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Init(Init),

  /// List the contents of a tree
  #[structopt(name = "ls")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Ls(Ls),

  /// Display the current committing user
  #[structopt(name = "me")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Me(Me),

  /// Dump contents of an object into a file
  #[structopt(name = "restore")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Restore(Restore),

  /// Display the contents of an object
  #[structopt(name = "show")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Show(Show),

  /// Stage files from the index
  #[structopt(name = "stage")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Stage(Stage),

  /// Display status of work tree and index
  #[structopt(name = "status")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Status(Status),

  /// Create a new tag
  #[structopt(name = "tag")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Tag(Tag),

  /// Unstage files from the index
  #[structopt(name = "unstage")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Unstage(Unstage),

  /// Print out the working tree location of a repository
  #[structopt(name = "where")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Where(Where),
}

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub enum BranchCommand {
  /// List all branches
  #[structopt(name = "ls")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Ls(BranchLs),

  /// Change what a branch points to
  #[structopt(name = "mv")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Mv(BranchMv),

  /// Create a new branch
  #[structopt(name = "new")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  New(BranchNew),

  /// Rename a branch
  #[structopt(name = "rename")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Rename(BranchRename),

  /// Remove a branch
  #[structopt(name = "rm")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Rm(BranchRm),
}

#[derive(StructOpt, Debug)]
pub struct Branch {
  #[structopt(subcommand)]
  pub command: BranchCommand,
}

#[derive(StructOpt, Debug)]
pub struct BranchLs {
  /// Include remote branches in the list
  #[structopt(long = "remote", short = "r")]
  pub include_remote: bool,
}

#[derive(StructOpt, Debug)]
pub struct BranchMv {
  /// Branch to be moved
  src_name: String,

  /// Milk-style label of the destination reference
  dest_ref: String,
}

#[derive(StructOpt, Debug)]
pub struct BranchNew {
  /// Milk-style label of the destination reference
  #[structopt(long = "ref", short = "r", default_value = "/HEAD")]
  ref_name: String,

  /// Name of the new branch
  name: String,
}

#[derive(StructOpt, Debug)]
pub struct BranchRename {
  /// Whether the branch is remote or not
  #[structopt(long = "remote", short = "r")]
  is_remote: bool,

  /// Whether an existing branch with the destination name should be overridden
  #[structopt(long = "force", short = "f")]
  force: bool,

  /// Branch to be renamed
  src_name: String,

  /// New name of the branch
  dest_name: String,
}

#[derive(StructOpt, Debug)]
pub struct BranchRm {
  /// Whether the branch is remote
  #[structopt(long = "remote", short = "r")]
  is_remote: bool,

  /// Branch to be removed
  name: String,
}

#[derive(StructOpt, Debug)]
pub struct Clean {
  /// Paths to clean
  pub paths: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Commit {}

#[derive(StructOpt, Debug)]
pub struct Diff {
  /// Milk-style reference label to "old" tree-ish
  ///
  /// Includes special /INDEX and /WORK options to represent the work tree and
  /// the index, respectively.
  #[structopt(default_value = "/INDEX")]
  pub old_tree: String,

  /// Milk-style reference label to "new" tree-ish
  #[structopt(default_value = "/WORK")]
  pub new_tree: String,
}

#[derive(StructOpt, Debug)]
pub struct Head {}

#[derive(StructOpt, Debug)]
pub struct Ignore {
  /// Interpret paths as glob patterns and add them to .gitignore unmodified
  #[structopt(long = "pattern", short = "-P")]
  pub is_pattern: bool,

  /// The file or pattern to ignore
  pub pattern: String,
}

#[derive(StructOpt, Debug)]
pub struct Init {
  /// Create a bare repository
  #[structopt(long = "bare")]
  pub bare: bool,
}

#[derive(StructOpt, Debug)]
pub struct Ls {
  /// Milk-style reference label to list
  #[structopt(short = "ref", long = "r", default_value = "/HEAD")]
  pub ref_name: String,

  /// Subtree path to list
  #[structopt(default_value = "")]
  pub tree_path: std::path::PathBuf,
}

#[derive(StructOpt, Debug)]
pub struct Me {}

#[derive(StructOpt, Debug)]
pub struct Restore {
  /// Object ID to read contents from
  pub object_name: String,

  /// File path to write object
  pub path: std::path::PathBuf,
}

#[derive(StructOpt, Debug)]
pub struct Show {
  /// Milk-style reference label to object
  #[structopt(default_value = "/HEAD")]
  pub name: String,
}

#[derive(StructOpt, Debug)]
pub struct Stage {
  /// Paths to stage
  pub paths: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Status {
  /// Whether untracked files should be hidden or not
  #[structopt(long = "hide-untracked", short = "u")]
  pub hide_untracked: bool,

  /// Whether ignored files should be shown or not
  #[structopt(long = "show-ignored", short = "i")]
  pub show_ignored: bool,
}

#[derive(StructOpt, Debug)]
pub struct Tag {
  /// Milk-style reference label to tag
  #[structopt(short = "ref", long = "r", default_value = "/HEAD")]
  pub ref_name: String,

  /// Name of created tag
  pub tag_name: String,
}

#[derive(StructOpt, Debug)]
pub struct Unstage {
  /// Paths to unstage
  pub paths: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Where {}
