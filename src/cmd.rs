use super::cli;
use super::cli::Command;
use failure::Error;

pub fn main(args: cli::Root) -> Result<(), Error> {
  match args.command {
    Command::Init(cmd_args) => init(&args.globals, &cmd_args),
    Command::List(cmd_args) => ls(&args.globals, &cmd_args),
    _ => {
      println!("Can't run mystery command.");
      Ok(())
    }
  }?;

  Ok(())
}

pub fn init(globals: &cli::Global, args: &cli::Init) -> Result<(), Error> {
  println!("init {:?} + {:?}", globals, args);
  Ok(())
}

pub fn ls(globals: &cli::Global, args: &cli::List) -> Result<(), Error> {
  println!("ls {:?} + {:?}", globals, args);
  Ok(())
}
