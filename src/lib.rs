use chrono::offset::FixedOffset;
use chrono::offset::TimeZone;
use chrono::DateTime;
use colored::*;
use failure::format_err;
use failure::Error;
use failure::ResultExt;
use git2::Blob;
use git2::Commit;
use git2::Diff;
use git2::DiffOptions;
use git2::Object;
use git2::ObjectType;
use git2::Oid;
use git2::Repository;
use git2::Tag;
use git2::Time;
use git2::Tree;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;

pub mod cli;
pub mod cmd;

pub enum RepoPath {
  Path(PathBuf),
  NotFound,
  NotRepo,
}

pub trait MilkRepo {
  fn print_commit(&self, commit: &Commit);
  fn print_tree(&self, tree: &Tree);
  fn print_blob(&self, blob: &Blob);
  fn print_tag(&self, tag: &Tag);
  fn print_object(&self, object: &Object);
  fn highlight_named_oid(&self, name: &str, oid: Oid) -> String;
  fn get_short_id(&self, oid: Oid) -> String;
  fn find_from_refname<'repo>(&'repo self, name: &str) -> Result<Object<'repo>, Error>;
  fn find_from_name<'repo>(&'repo self, name: &str) -> Result<Object<'repo>, Error>;
  fn write_blob(&self, path: &Path) -> Result<Oid, Error>;
  fn name_to_tree<'repo>(&'repo self, s: &str) -> Result<Tree<'repo>, Error>;
  fn make_diff<'repo>(
    &'repo self,
    old_target: DiffTarget,
    new_target: DiffTarget,
  ) -> Result<Diff<'repo>, Error>;
  fn canonicalize_path(&self, path: &Path) -> Result<RepoPath, Error>;
  fn ignore_string(&self, line: &str) -> Result<(), Error>;
  fn ignore_file(&self, path: &Path) -> Result<(), Error>;
}

impl MilkRepo for Repository {
  fn print_commit(&self, commit: &Commit) {
    let author = commit.author();
    let author_name = author.name().unwrap_or("[???]");
    let author_email = author.email().unwrap_or("[???]");
    let author_time = git_to_chrono(&author.when());

    let committer = commit.committer();
    let committer_name = committer.name().unwrap_or("[???]");
    let committer_email = committer.email().unwrap_or("[???]");
    let committer_time = git_to_chrono(&committer.when());

    println!("{}", self.highlight_named_oid("tree", commit.tree_id()));

    println!(
      "{} {} {}",
      author_name.cyan(),
      author_email.bright_black(),
      author_time.to_string().bright_blue()
    );

    if author_name != committer_name || author_email != committer_email {
      println!(
        "committed by {} {} {}",
        committer_name.cyan(),
        committer_email.bright_black(),
        committer_time.to_string().bright_blue()
      );
    }

    println!("{}", commit.message().unwrap_or(""));
  }

  fn print_tree(&self, tree: &Tree) {
    for entry in tree.iter() {
      let raw_name = entry.name().unwrap_or("[invalid utf-8]");
      let name = match entry.kind() {
        Some(ObjectType::Tree) => format!(
          "{}/ {}",
          raw_name.blue(),
          self.get_short_id(entry.id()).bright_black()
        ),
        Some(ObjectType::Commit) => format!(
          "@{} {}",
          raw_name.bright_red(),
          self.get_short_id(entry.id()).bright_black()
        ),
        Some(ObjectType::Tag) => format!(
          "#{} {}",
          raw_name.bright_cyan(),
          self.get_short_id(entry.id()).bright_black()
        ),
        _ => format!(
          "{} {}",
          raw_name,
          self.get_short_id(entry.id()).bright_black()
        ),
      };

      println!("{}", name);
    }
  }

  fn print_blob(&self, blob: &Blob) {
    let mut stdout = io::stdout();

    // what happens on failure?
    match stdout.write(blob.content()) {
      _ => (),
    }
  }

  fn print_tag(&self, tag: &Tag) {
    println!("{}", self.highlight_named_oid("target", tag.target_id()));

    let author = tag.tagger();
    if let Some(author) = author {
      let author_name = author.name().unwrap_or("[???]");
      let author_email = author.email().unwrap_or("[???]");
      let author_time = git_to_chrono(&author.when());

      println!(
        "{} {} {}",
        author_name.cyan(),
        author_email.bright_black(),
        author_time.to_string().bright_blue()
      );
    }

    println!("{}", tag.message().unwrap_or(""));
  }

  fn print_object(&self, object: &Object) {
    match object.kind() {
      Some(ObjectType::Blob) => {
        println!("{}", self.highlight_named_oid("blob", object.id()));
        let blob = object.as_blob().unwrap();
        self.print_blob(&blob);
      }
      Some(ObjectType::Tree) => {
        println!("{}", self.highlight_named_oid("tree", object.id()));
        let tree = object.as_tree().unwrap();
        self.print_tree(&tree);
      }
      Some(ObjectType::Commit) => {
        println!("{}", self.highlight_named_oid("commit", object.id()));
        let commit = object.as_commit().unwrap();
        self.print_commit(&commit);
      }
      Some(ObjectType::Tag) => {
        println!("{}", self.highlight_named_oid("tag", object.id()));
        let tag = object.as_tag().unwrap();
        self.print_tag(&tag);
      }
      _ => {
        println!("{}", self.highlight_named_oid("unknown", object.id()));
      }
    }
  }

  fn highlight_named_oid(&self, name: &str, oid: Oid) -> String {
    format!("{} {}", name.cyan(), self.get_short_id(oid).bright_black())
  }

  fn get_short_id(&self, oid: Oid) -> String {
    // wtf is the better Rust pattern for this?
    match self.find_object(oid, None) {
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

  fn find_from_refname<'repo>(&'repo self, name: &str) -> Result<Object<'repo>, Error> {
    let oid = self.refname_to_id(name)?;
    let ok = self
      .find_object(oid, Some(ObjectType::Any))
      .with_context(|_| "couldn't find object with that oid")?;
    Ok(ok)
  }

  fn find_from_name<'repo>(&'repo self, name: &str) -> Result<Object<'repo>, Error> {
    let mut iter = name.chars();
    let head = iter.next();
    let tail: String = iter.collect();

    if head.is_none() {
      self.find_from_refname("HEAD")
    } else if let Some('#') = head {
      self.find_from_refname(&format!("refs/tags/{}", tail))
    } else if let Some('@') = head {
      if tail.is_empty() {
        self.find_from_refname("HEAD")
      } else {
        self.find_from_refname(&format!("refs/heads/{}", tail))
      }
    } else if let Some('/') = head {
      self.find_from_refname(&tail)
    } else {
      let odb = self.odb()?;
      let short_oid = Oid::from_str(name)?;
      let oid = odb.exists_prefix(short_oid, name.len())?;
      let ok = self.find_object(oid, Some(ObjectType::Any))?;
      Ok(ok)
    }
  }

  fn write_blob(&self, path: &Path) -> Result<Oid, Error> {
    let odb = self.odb().with_context(|_| "couldn't open ODB")?;
    let mut handle = File::open(path)?;
    let mut bytes = Vec::new();
    let _size = handle.read_to_end(&mut bytes)?;
    let oid = odb.write(ObjectType::Blob, &bytes)?;
    Ok(oid)
  }

  fn name_to_tree<'repo>(&'repo self, s: &str) -> Result<Tree<'repo>, Error> {
    let tree = self
      .find_from_name(s)
      .with_context(|_| "couldn't find refname")?
      .peel_to_tree()
      .with_context(|_| "couldn't peel to commit HEAD")?;
    Ok(tree)
  }

  fn make_diff<'repo>(
    &'repo self,
    old_target: DiffTarget,
    new_target: DiffTarget,
  ) -> Result<Diff<'repo>, Error> {
    let mut options = DiffOptions::new();

    match (old_target, new_target) {
      // tree..
      (DiffTarget::Name(old), DiffTarget::WorkingTree) => {
        let old_tree = self
          .name_to_tree(old)
          .with_context(|_| "couldn't look up old tree")?;

        let diff = self
          .diff_tree_to_workdir(Some(&old_tree), Some(&mut options))
          .with_context(|_| "couldn't generate diff")?;
        Ok(diff)
      }

      (DiffTarget::Name(old), DiffTarget::Name(new)) => {
        let old_tree = self
          .name_to_tree(old)
          .with_context(|_| "couldn't look up old tree")?;
        let new_tree = self
          .name_to_tree(new)
          .with_context(|_| "couldn't look up new tree")?;

        let diff = self
          .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut options))
          .with_context(|_| "couldn't generate diff")?;
        Ok(diff)
      }

      (DiffTarget::Name(old), DiffTarget::Index) => {
        let old_tree = self
          .name_to_tree(old)
          .with_context(|_| "couldn't look up old tree")?;
        let index = self.index().with_context(|_| "couldn't read index")?;

        let diff = self
          .diff_tree_to_index(Some(&old_tree), Some(&index), Some(&mut options))
          .with_context(|_| "couldn't generate diff")?;
        Ok(diff)
      }

      // index..
      (DiffTarget::Index, DiffTarget::WorkingTree) => {
        let index = self.index().with_context(|_| "couldn't read index")?;
        let diff = self
          .diff_index_to_workdir(Some(&index), Some(&mut options))
          .with_context(|_| "couldn't generate diff")?;

        Ok(diff)
      }

      (DiffTarget::Index, DiffTarget::Name(new)) => {
        let index = self.index().with_context(|_| "couldn't read index")?;
        let new_tree = self
          .name_to_tree(new)
          .with_context(|_| "couldn't look up new tree")?;
        options.reverse(true);

        let diff = self
          .diff_tree_to_index(Some(&new_tree), Some(&index), Some(&mut options))
          .with_context(|_| "couldn't generate diff")?;
        Ok(diff)
      }

      (DiffTarget::Index, DiffTarget::Index) => {
        // FIXME why? it probably works...
        Err(format_err!("Cannot diff between identical targets"))
      }

      // working..
      (DiffTarget::WorkingTree, DiffTarget::WorkingTree) => {
        // FIXME why? it probably works...
        Err(format_err!("Cannot diff between identical targets"))
      }
      (DiffTarget::WorkingTree, DiffTarget::Name(new)) => {
        let new_tree = self
          .name_to_tree(new)
          .with_context(|_| "couldn't look up new tree")?;
        options.reverse(true);

        let diff = self
          .diff_tree_to_workdir(Some(&new_tree), Some(&mut options))
          .with_context(|_| "couldn't generate diff")?;
        Ok(diff)
      }
      (DiffTarget::WorkingTree, DiffTarget::Index) => {
        let index = self.index().with_context(|_| "couldn't read index")?;
        options.reverse(true);
        let diff = self
          .diff_index_to_workdir(Some(&index), Some(&mut options))
          .with_context(|_| "couldn't generate diff")?;

        Ok(diff)
      }
    }
  }

  fn canonicalize_path(&self, path: &Path) -> Result<RepoPath, Error> {
    let workdir = self
      .workdir()
      .ok_or_else(|| failure::err_msg("repository is bare"))?;

    // Try to transform path into its canonical path from the workdir
    let path = match path.canonicalize() {
      Ok(abs_path) => match abs_path.strip_prefix(workdir) {
        Ok(rel_path) => RepoPath::Path(rel_path.to_path_buf()),
        Err(_) => RepoPath::NotRepo,
      },
      Err(_) => RepoPath::NotFound,
    };
    Ok(path)
  }

  fn ignore_string(&self, line: &str) -> Result<(), Error> {
    let workdir = self
      .workdir()
      .ok_or_else(|| failure::err_msg("repository is bare"))?;

    let gitignore_path = workdir.join(".gitignore");

    let mut gitignore = OpenOptions::new()
      .create(!gitignore_path.exists())
      .append(true)
      .open(workdir.join(".gitignore"))
      .context("Couldn't open .gitignore file")?;

    println!("{}: {} to .gitignore", "added".green(), line);
    writeln!(gitignore, "{}", line).context("Couldn't write to .gitignore file")?;

    Ok(())
  }

  fn ignore_file(&self, path: &Path) -> Result<(), Error> {
    let canon_path = self
      .canonicalize_path(&path)
      .with_context(|_| "couldn't canonicalize path")?;

    let path = match canon_path {
      RepoPath::Path(x) => x,
      RepoPath::NotFound => {
        return Err(format_err!("{:?} does not exist", path));
      }
      RepoPath::NotRepo => {
        return Err(format_err!("{:?} is not in this repo", path));
      }
    };

    let tree = self
      .head()
      .with_context(|_| "couldn't locate HEAD")?
      .peel_to_commit()
      .with_context(|_| "couldn't peel to commit")?
      .tree()
      .with_context(|_| "couldn't locate tree")?;

    if tree.get_path(&path).is_ok() {
      eprintln!(
        "{}: file {:?} is currently tracked by git",
        "warning".red(),
        path
      );
    };

    let final_filepath = path
      .to_str()
      .ok_or_else(|| failure::err_msg("path is not utf-8"))?;

    self.ignore_string(final_filepath)
  }
}

pub enum DiffTarget<'a> {
  WorkingTree,
  Index,
  Name(&'a str),
}

impl<'a> DiffTarget<'a> {
  fn from_str(s: &str) -> DiffTarget {
    match s {
      "/WORK" => DiffTarget::WorkingTree,
      "/INDEX" => DiffTarget::Index,
      _ => DiffTarget::Name(s),
    }
  }
}

pub fn git_to_chrono(sig: &Time) -> DateTime<FixedOffset> {
  let timestamp = sig.seconds();
  let offset_sec = sig.offset_minutes() * 60;
  let fixed_offset = FixedOffset::east(offset_sec);
  fixed_offset.timestamp(timestamp, 0)
}

/// `options` will be interspersed with forward slashes and presented to the
/// user. For example, with `options = "Yn?"`, the user will be prompted for
/// `[Y/n/?]`
pub fn prompt_char(msg: &str, options: &str) -> Result<char, Error> {
  // FIXME what if multiples of the same option char?

  let option_chars: Vec<_> = options.chars().collect();
  let mut option_display = String::new();
  let mut sep = "";
  for arg in option_chars {
    option_display.push_str(sep);
    option_display.push(arg);
    sep = "/";
  }
  eprint!("{} [{}] ", msg, option_display);

  io::stdout().flush().context("Could not flush stdout")?;

  // getting this to only accept a single character without requiring the user
  // to press enter is a giant nuisace, so I'm skipping it for now
  let mut input = [0; 1];
  io::stdin()
    .read_exact(&mut input[..])
    .context("Could not read stdin")?;

  Ok(input[0] as char)
}

pub fn editor(path: &Path, contents: &str) -> Result<String, Error> {
  // FIXME one of the Err cases here is for a non-unicode value... I'd assume you
  // can run a non-unicode command, no?
  let editor = env::var("EDITOR").with_context(|_| "$EDITOR is not defined.")?;

  let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open(path)
    .with_context(|_| "couldn't open $EDITOR file")?;

  file
    .write_all(contents.as_bytes())
    .with_context(|_| "couldn't write $EDITOR file contents")?;

  file
    .sync_all()
    .with_context(|_| "couldn't sync $EDITOR file contents")?;

  let mut editor_command = process::Command::new(editor);
  editor_command.arg(&path);

  editor_command
    .spawn()
    .and_then(|mut handle| handle.wait())
    .with_context(|_| "$EDITOR failed for some reason")?;

  let mut file = File::open(path).with_context(|_| "couldn't re-open file")?;

  let mut contents = String::new();
  file
    .read_to_string(&mut contents)
    .with_context(|_| "couldn't read from file")?;

  if std::fs::remove_file(&path).is_err() {
    eprintln!("WARNING: Unable to delete {} after use", path.display());
  }

  Ok(contents)
}
