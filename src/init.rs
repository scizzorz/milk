use exitfailure::ExitFailure;
use failure::ResultExt;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  println!("milk-init");
  Ok(())
}
