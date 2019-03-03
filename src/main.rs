use clap::crate_version;
use clap::App;
use clap::AppSettings;
use exitfailure::ExitFailure;
use failure::ResultExt;
use log::info;
use std::process::exit;
use std::process::Command;
use std::process::Stdio;

fn main() -> Result<(), ExitFailure> {
  env_logger::init();

  let args = App::new("milk")
    .version(crate_version!())
    .setting(AppSettings::AllowExternalSubcommands)
    .setting(AppSettings::ColoredHelp)
    .get_matches();

  match args.subcommand() {
    (subcommand, Some(scmd)) => {
      let subcommand_args: Vec<&str> = match scmd.values_of("") {
        Some(values) => values.collect(),
        None => Vec::new(),
      };

      info!("Running subcommand: `milk-{}`", subcommand);

      let exit_status = Command::new(format!("milk-{}", subcommand))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args(&subcommand_args[..])
        .spawn()
        .and_then(|mut handle| handle.wait())
        .with_context(|_| format!("couldn't execute command: `milk-{}`", subcommand))?;

      if !exit_status.success() {
        eprintln!("{} exited with non-zero exit code", subcommand);
        exit(exit_status.code().unwrap_or(1));
      }
    }

    _ => {}
  }

  Ok(())
}
