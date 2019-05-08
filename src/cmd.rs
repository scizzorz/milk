use super::cli;
use super::cli::Command;
use failure::Error;

pub fn main(args: cli::Root) -> Result<(), Error> {
  let ok = match args.command {
    Command::Init(args) => init(&args),
    Command::List(args) => ls(&args),
  }?;

  Ok(ok)
}

pub fn init(args: &cli::Init) -> Result<(), Error> {
  println!("init {:?}", args);
  Ok(())
}

pub fn ls(args: &cli::List) -> Result<(), Error> {
  println!("ls {:?}", args);
  Ok(())
}
