use std::ops::Range;

#[derive(Debug)]
pub struct PatMatch {
  pub is_match: bool,
  pub start: usize,
  pub end: usize,
}
impl PatMatch {
  pub(crate) fn or(&self, endpos: usize) -> usize {
    if self.is_match {
      self.start
    } else {
      endpos
    }
  }

  pub(crate) fn end_or(&self, endpos: usize) -> usize {
    if self.is_match {
      self.end
    } else {
      endpos
    }
  }
}

pub fn find(subject: &str, pat: &'static str) -> PatMatch {
  find_at(subject, pat, 0)
}

pub fn find_at(subject: &str, pat: &'static str, start: usize) -> PatMatch {
  let mut pat = lua_patterns::LuaPattern::new(pat);
  let is_match = pat.matches(&subject[start..]);
  let range = pat.range();
  PatMatch { start: range.start + start, end: range.end + start, is_match }
}
