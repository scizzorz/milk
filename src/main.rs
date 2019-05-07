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
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
enum Command {
  /// Initialize a new Git repository
  #[structopt(name = "init")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  Init {
    /// Create a bare repository
    #[structopt(long = "bare")]
    bare: bool,
  },

  /// List the contents of a tree
  #[structopt(name = "ls")]
  #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
  List {
    /// Milk-style reference label to list
    #[structopt(short = "ref", long = "r", default_value = "/HEAD")]
    ref_name: String,

    /// Subtree path to list
    #[structopt(default_value = "")]
    tree_path: std::path::PathBuf,
  }
}

fn init(bare: bool) -> Result<(), Error> {
  println!("init --bare={}", bare);
  Ok(())
}

fn ls(ref_name: &str, tree_path: std::path::PathBuf) -> Result<(), Error> {
  println!("ls -r {} {}", ref_name, tree_path.display());
  Ok(())
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();
  //run_supercommand("milk")
  println!("{:?}", args);
  let ok = match args.command {
    Command::Init {bare} => {
      init(bare)
    },

    Command::List {ref_name, tree_path} => {
      ls(&ref_name, tree_path)
    }
  }?;

  Ok(ok)
}
