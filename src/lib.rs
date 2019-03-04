use git2::Repository;
use git2::Oid;

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
