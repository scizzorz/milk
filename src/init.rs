use exitfailure::ExitFailure;
use failure::ResultExt;
use log::info;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(short = "q", long = "quiet")]
  quiet: bool,
  #[structopt(long = "bare")]
  bare: bool,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  if !args.quiet {
    println!("milk-init");
  }

  if args.bare {
    info!("Initializing bare git repository...");
  } else {
    info!("Initializing git repository...");
  }

  Ok(())
}
