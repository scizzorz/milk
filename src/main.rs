use exitfailure::ExitFailure;
use failure::ResultExt;
use milk::cli;
use milk::cmd;
use structopt::StructOpt;

fn main() -> Result<(), ExitFailure> {
  let args = cli::Root::from_args();
  env_logger::init();
  cmd::main(args).with_context(|_| "couldn't execute command")?;
  Ok(())
}
