use std::ops::Range;

#[derive(Debug, Default)]
pub struct PatMatch {
  pub is_match: bool,
  pub start: usize,
  pub end: usize,
  pub cap1: Range<usize>,
  pub cap2: Range<usize>,
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
  PatMatch { start: range.start + start, end: range.end + start, is_match, cap1: 0..0, cap2: 0..0 }
}

pub fn capture_at<'a>(subject: &'a str, pat: &'static str, start: usize) -> PatMatch {
  let mut pat = lua_patterns::LuaPattern::new(pat);
  let is_match = pat.matches(&subject[start..]);
  let range = pat.range();
  let cap1 = pat.capture(1);
  let cap2 = pat.capture(2);
  PatMatch { start: range.start + start, end: range.end + start, is_match, cap1, cap2 }
}

pub(crate) fn is_space(c: char) -> bool {
  " \n\t".contains(c)
}
