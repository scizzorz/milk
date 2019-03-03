use exitfailure::ExitFailure;
use failure::ResultExt;
use std::process::exit;
use std::process::Command;
use std::process::Stdio;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  command: String,
}

fn main() -> Result<(), ExitFailure> {
  let args = Cli::from_args();

  let exit_status = Command::new(format!("milk-{}", args.command))
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    //.args(&subcommand_args[..])
    .spawn()
    .and_then(|mut handle| handle.wait())
    .with_context(|_| format!("couldn't execute command: `{}`", args.command))?;

  if !exit_status.success() {
    println!("{} exited with non-zero exit code", args.command);
    exit(exit_status.code().unwrap_or(1));
  }

  Ok(())
}
