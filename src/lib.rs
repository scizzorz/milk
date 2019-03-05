use chrono::DateTime;
use chrono::offset::FixedOffset;
use chrono::offset::TimeZone;
use clap::App;
use clap::AppSettings;
use clap::crate_version;
use exitfailure::ExitFailure;
use failure::ResultExt;
use git2::Object;
use git2::Oid;
use git2::Repository;
use git2::Time;
use std::process::Command;
use std::process::Stdio;
use std::process::exit;

pub fn run_supercommand(prefix: &str) -> Result<(), ExitFailure> {
  let args = App::new(prefix)
    .version(crate_version!())
    .setting(AppSettings::AllowExternalSubcommands)
    .setting(AppSettings::ColoredHelp)
    .get_matches();


  match args.subcommand() {
    (subcommand, Some(scmd)) => {
      let command = format!("{}-{}", prefix, subcommand);

      let subcommand_args: Vec<&str> = match scmd.values_of("") {
        Some(values) => values.collect(),
        None => Vec::new(),
      };

      let exit_status = Command::new(&command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args(&subcommand_args[..])
        .spawn()
        .and_then(|mut handle| handle.wait())
        .with_context(|_| format!("couldn't execute command: `{}`", command))?;

      if !exit_status.success() {
        // FIXME this should probably return an Err(...)?
        eprintln!("{} exited with non-zero exit code", command);
        exit(exit_status.code().unwrap_or(exitcode::SOFTWARE));
      }
    }

    _ => {}
  }

  Ok(())
}

pub fn get_short_id(repo: &Repository, oid: Oid) -> String {
  // wtf is the better Rust pattern for this?
  match repo.find_object(oid, None) {
    Ok(object) => match object.short_id() {
      Ok(buf) => match buf.as_str() {
        Some(res) => res.to_string(),
        _ => oid.to_string(),
      },
      _ => oid.to_string(),
    },
    _ => oid.to_string(),
  }
}

pub fn git_to_chrono(sig: &Time) -> DateTime<FixedOffset> {
  let timestamp = sig.seconds();
  let offset_sec = sig.offset_minutes() * 60;
  let fixed_offset = FixedOffset::east(offset_sec);
  fixed_offset.timestamp(timestamp, 0)
}

pub fn find_from_name<'repo>(repo: &'repo Repository, name: &str) -> Option<Object<'repo>> {
  /* wip
  if name.get(0u) == "#" {
  }
  else if name[0] == "@" {
  }
  else {
  }
  */

  None
}
