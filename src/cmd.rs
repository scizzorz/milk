use super::cli;
use super::cli::Command;
use failure::Error;

pub fn main(args: cli::Root) -> Result<(), Error> {
  match args.command {
    Command::Clean(cmd_args) => clean(&args.globals, &cmd_args),
    Command::Commit(cmd_args) => commit(&args.globals, &cmd_args),
    Command::Diff(cmd_args) => diff(&args.globals, &cmd_args),
    Command::Head(cmd_args) => head(&args.globals, &cmd_args),
    Command::Ignore(cmd_args) => ignore(&args.globals, &cmd_args),
    Command::Init(cmd_args) => init(&args.globals, &cmd_args),
    Command::Ls(cmd_args) => ls(&args.globals, &cmd_args),
    Command::Me(cmd_args) => me(&args.globals, &cmd_args),
    Command::Restore(cmd_args) => restore(&args.globals, &cmd_args),
    Command::Show(cmd_args) => show(&args.globals, &cmd_args),
    Command::Stage(cmd_args) => stage(&args.globals, &cmd_args),
    Command::Status(cmd_args) => status(&args.globals, &cmd_args),
    Command::Tag(cmd_args) => tag(&args.globals, &cmd_args),
    Command::Unstage(cmd_args) => unstage(&args.globals, &cmd_args),
    Command::Where(cmd_args) => where_(&args.globals, &cmd_args),
  }?;

  Ok(())
}

pub fn clean(globals: &cli::Global, args: &cli::Clean) -> Result<(), Error> {
  Ok(())
}

pub fn commit(globals: &cli::Global, args: &cli::Commit) -> Result<(), Error> {
  Ok(())
}

pub fn diff(globals: &cli::Global, args: &cli::Diff) -> Result<(), Error> {
  Ok(())
}

pub fn head(globals: &cli::Global, args: &cli::Head) -> Result<(), Error> {
  Ok(())
}

pub fn ignore(globals: &cli::Global, args: &cli::Ignore) -> Result<(), Error> {
  Ok(())
}

pub fn init(globals: &cli::Global, args: &cli::Init) -> Result<(), Error> {
  Ok(())
}

pub fn ls(globals: &cli::Global, args: &cli::Ls) -> Result<(), Error> {
  Ok(())
}

pub fn me(globals: &cli::Global, args: &cli::Me) -> Result<(), Error> {
  Ok(())
}

pub fn restore(globals: &cli::Global, args: &cli::Restore) -> Result<(), Error> {
  Ok(())
}

pub fn show(globals: &cli::Global, args: &cli::Show) -> Result<(), Error> {
  Ok(())
}

pub fn stage(globals: &cli::Global, args: &cli::Stage) -> Result<(), Error> {
  Ok(())
}

pub fn status(globals: &cli::Global, args: &cli::Status) -> Result<(), Error> {
  Ok(())
}

pub fn tag(globals: &cli::Global, args: &cli::Tag) -> Result<(), Error> {
  Ok(())
}

pub fn unstage(globals: &cli::Global, args: &cli::Unstage) -> Result<(), Error> {
  Ok(())
}

pub fn where_(globals: &cli::Global, args: &cli::Where) -> Result<(), Error> {
  Ok(())
}
