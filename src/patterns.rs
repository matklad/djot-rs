#[derive(Debug, Default)]
pub struct PatMatch<'a> {
  pub is_match: bool,
  pub start: usize,
  pub end: usize,
  pub cap1: &'a str,
  pub cap2: &'a str,
}

impl<'a> PatMatch<'a> {
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

pub fn find<'a>(subject: &'a str, pat: &'static str) -> PatMatch<'static> {
  find_at(subject, pat, 0)
}

pub fn find_at<'a>(subject: &'a str, pat: &'static str, start: usize) -> PatMatch<'static> {
  let mut pat = lua_patterns::LuaPattern::new(pat);
  let is_match = pat.matches(&subject[start..]);
  let range = pat.range();
  PatMatch { start: range.start + start, end: range.end + start, is_match, cap1: "", cap2: "" }
}

pub(crate) fn is_space(c: char) -> bool {
  " \n\t".contains(c)
}
