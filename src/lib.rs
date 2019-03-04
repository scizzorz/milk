use git2::Repository;
use git2::Oid;
use chrono::offset::FixedOffset;
use git2::Time;
use chrono::offset::TimeZone;
use chrono::DateTime;

pub fn get_short_id(repo: &Repository, oid: Oid) -> String {
  // wtf
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
