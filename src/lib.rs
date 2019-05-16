use chrono::offset::FixedOffset;
use chrono::offset::TimeZone;
use chrono::DateTime;
use colored::*;
use failure::Error;
use failure::ResultExt;
use git2::Blob;
use git2::Commit;
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
use std::process;

pub mod cli;
pub mod cmd;

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
}

pub fn git_to_chrono(sig: &Time) -> DateTime<FixedOffset> {
  let timestamp = sig.seconds();
  let offset_sec = sig.offset_minutes() * 60;
  let fixed_offset = FixedOffset::east(offset_sec);
  fixed_offset.timestamp(timestamp, 0)
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
