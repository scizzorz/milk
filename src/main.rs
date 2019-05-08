use exitfailure::ExitFailure;
use failure::Error;
use structopt::StructOpt;
use milk::cli;
use milk::cli::Command;

fn init(args: &cli::Init) -> Result<(), Error> {
  println!("init {:?}", args);
  Ok(())
}

fn ls(args: &cli::List) -> Result<(), Error> {
  println!("ls {:?}", args);
  Ok(())
}

fn main() -> Result<(), ExitFailure> {
  let args = cli::Root::from_args();
  env_logger::init();

  println!("{:?}", args);
  let ok = match args.command {
    Command::Init(args) => init(&args),

    Command::List(args) => ls(&args),
  }?;

  Ok(ok)
}
