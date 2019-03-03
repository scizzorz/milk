use exitfailure::ExitFailure;
use failure::ResultExt;
use log::info;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();
  env_logger::init();

  println!("milk-init");
  info!("Initializing git repository...");

  Ok(())
}
