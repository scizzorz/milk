use chrono::offset::FixedOffset;
use chrono::offset::TimeZone;
use chrono::DateTime;
use colored::*;
use git2::Blob;
use git2::Commit;
use git2::Error;
use git2::Object;
use git2::ObjectType;
use git2::Oid;
use git2::Repository;
use git2::Tag;
use git2::Time;
use git2::Tree;
use std::io;
use std::io::Write;

pub mod cli;
pub mod cmd;

pub fn print_commit(repo: &Repository, commit: &Commit) {
  let author = commit.author();
  let author_name = author.name().unwrap_or("[???]");
  let author_email = author.email().unwrap_or("[???]");
  let author_time = git_to_chrono(&author.when());

  let committer = commit.committer();
  let committer_name = committer.name().unwrap_or("[???]");
  let committer_email = committer.email().unwrap_or("[???]");
  let committer_time = git_to_chrono(&committer.when());

  println!("{}", highlight_named_oid(repo, "tree", commit.tree_id()));

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

pub fn print_tree(repo: &Repository, tree: &Tree) {
  for entry in tree.iter() {
    let raw_name = entry.name().unwrap_or("[invalid utf-8]");
    let name = match entry.kind() {
      Some(ObjectType::Tree) => format!(
        "{}/ {}",
        raw_name.blue(),
        get_short_id(repo, entry.id()).bright_black()
      ),
      Some(ObjectType::Commit) => format!(
        "@{} {}",
        raw_name.bright_red(),
        get_short_id(repo, entry.id()).bright_black()
      ),
      Some(ObjectType::Tag) => format!(
        "#{} {}",
        raw_name.bright_cyan(),
        get_short_id(repo, entry.id()).bright_black()
      ),
      _ => format!(
        "{} {}",
        raw_name,
        get_short_id(repo, entry.id()).bright_black()
      ),
    };

    println!("{}", name);
  }
}

pub fn print_blob(_repo: &Repository, blob: &Blob) {
  // _repo is unused here, but I'm keeping it to maintain the API that the
  // rest of the print_* functions have
  let mut stdout = io::stdout();

  // what happens on failure?
  match stdout.write(blob.content()) {
    _ => (),
  }
}

pub fn print_tag(repo: &Repository, tag: &Tag) {
  println!("{}", highlight_named_oid(repo, "target", tag.target_id()));

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

pub fn print_object(repo: &Repository, object: &Object) {
  match object.kind() {
    Some(ObjectType::Blob) => {
      println!("{}", highlight_named_oid(&repo, "blob", object.id()));
      let blob = object.as_blob().unwrap();
      print_blob(repo, &blob);
    }
    Some(ObjectType::Tree) => {
      println!("{}", highlight_named_oid(&repo, "tree", object.id()));
      let tree = object.as_tree().unwrap();
      print_tree(repo, &tree);
    }
    Some(ObjectType::Commit) => {
      println!("{}", highlight_named_oid(&repo, "commit", object.id()));
      let commit = object.as_commit().unwrap();
      print_commit(repo, &commit);
    }
    Some(ObjectType::Tag) => {
      println!("{}", highlight_named_oid(&repo, "tag", object.id()));
      let tag = object.as_tag().unwrap();
      print_tag(repo, &tag);
    }
    _ => {
      println!("{}", highlight_named_oid(&repo, "unknown", object.id()));
    }
  }
}

pub fn highlight_named_oid(repo: &Repository, name: &str, oid: Oid) -> String {
  format!("{} {}", name.cyan(), get_short_id(repo, oid).bright_black())
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

pub fn find_from_refname<'repo>(
  repo: &'repo Repository,
  name: &str,
) -> Result<Object<'repo>, Error> {
  let oid = repo.refname_to_id(name)?;
  repo.find_object(oid, Some(ObjectType::Any))
}

pub fn find_from_name<'repo>(repo: &'repo Repository, name: &str) -> Result<Object<'repo>, Error> {
  let mut iter = name.chars();
  let head = iter.next();
  let tail: String = iter.collect();

  if head.is_none() {
    find_from_refname(repo, "HEAD")
  } else if let Some('#') = head {
    find_from_refname(repo, &format!("refs/tags/{}", tail))
  } else if let Some('@') = head {
    if tail.is_empty() {
      find_from_refname(repo, "HEAD")
    } else {
      find_from_refname(repo, &format!("refs/heads/{}", tail))
    }
  } else if let Some('/') = head {
    find_from_refname(repo, &tail)
  } else {
    let odb = repo.odb()?;
    let short_oid = Oid::from_str(name)?;
    let oid = odb.exists_prefix(short_oid, name.len())?;
    repo.find_object(oid, Some(ObjectType::Any))
  }
}
