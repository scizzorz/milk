use exitfailure::ExitFailure;
use failure::Error;
use structopt::StructOpt;

/// A new front-end for Git.
#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Cli {
  /// Path to the repository root
  #[structopt(long = "repo", short = "p", default_value = ".")]
  repo_path: std::path::PathBuf,

  /// Don't print information
  #[structopt(long = "quiet", short = "q")]
  quiet: bool,

  #[structopt(subcommand)]
  command: Command,
}

#[derive(StructOpt, Debug)]
struct InitCli {
  /// Create a bare repository
  #[structopt(long = "bare")]
  bare: bool,
}

#[derive(StructOpt, Debug)]
struct ListCli {
  /// Milk-style reference label to list
  #[structopt(short = "ref", long = "r", default_value = "/HEAD")]
  ref_name: String,

  /// Subtree path to list
  #[structopt(default_value = "")]
  tree_path: std::path::PathBuf,
}

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
enum Command {
  /// Initialize a new Git repository
  #[structopt(name = "init")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Init(InitCli),

  /// List the contents of a tree
  #[structopt(name = "ls")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  List(ListCli),
}

fn init(args: &InitCli) -> Result<(), Error> {
  println!("init {:?}", args);
  Ok(())
}

fn ls(args: &ListCli) -> Result<(), Error> {
  println!("ls {:?}", args);
  Ok(())
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  println!("{:?}", args);
  let ok = match args.command {
    Command::Init(args) => init(&args),

    Command::List(args) => ls(&args),
  }?;

  Ok(ok)
}
