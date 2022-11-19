use std::{
  collections::{BTreeMap, HashMap},
  ops::Range,
};

use crate::{
  annot::{Annot, Atom, Comp},
  patterns::{find_at, is_space, PatMatch},
  Match, ParseOpts,
};

#[derive(Default)]
pub struct Tokenizer {
  opts: ParseOpts,
  subject: String,
  matches: BTreeMap<usize, Match>,
  openers: HashMap<u8, Vec<Opener>>,
  verbatim: usize,
  verbatim_type: Comp,
  destination: bool,
  firstpos: usize,
  lastpos: usize,
}

#[derive(Debug, Clone, Copy)]
struct Opener {
  spos: usize,
  epos: usize,
  annot: &'static str,
  subspos: usize,
  subepos: usize,
}

impl Opener {
  fn new(spos: usize, epos: usize) -> Self {
    Self { spos, epos, annot: "", subspos: 0, subepos: 0 }
  }
}

// allow up to 3 captures...
fn bounded_find(subj: &str, patt: &'static str, startpos: usize, endpos: usize) -> PatMatch {
  let mut m = find_at(subj, patt, startpos);
  if m.end > endpos {
    m = PatMatch::default()
  }
  m
}

impl Tokenizer {
  pub fn new(subject: String, opts: ParseOpts) -> Tokenizer {
    let mut res = Tokenizer::default();
    res.subject = subject;
    res.opts = opts;
    res
  }

  fn add_match(&mut self, range: Range<usize>, annotation: impl Into<Annot>) {
    let m = Match::new(range.clone(), annotation);
    self.matches.insert(range.start, m);
  }

  fn add_opener(&mut self, name: u8, opener: Opener) {
    self.openers.entry(name).or_default().push(opener)
  }

  fn clear_openers(&mut self, startpos: usize, endpos: usize) {
    for v in self.openers.values_mut() {
      v.retain(|it| !(startpos <= it.spos && it.epos <= endpos))
    }
  }

  fn str_matches(&mut self, startpos: usize, endpos: usize) {
    for i in startpos..endpos {
      if let Some(m) = self.matches.get_mut(&i) {
        if m.is_not(Atom::Str) && m.is_not(Atom::Escape) {
          m.a = Atom::Str.into();
        }
      }
    }
  }

  fn between_matched(&mut self, pos: usize, c: u8, annotation: Comp, defaultmatch: Atom) -> usize {
    self.between_matched_impl(
      pos,
      c,
      annotation,
      defaultmatch,
      Option::<fn(&str, usize) -> PatMatch>::None,
    )
  }

  fn between_matched_with_open_test(
    &mut self,
    pos: usize,
    c: u8,
    annotation: Comp,
    defaultmatch: Atom,
    open_test: impl FnOnce(&str, usize) -> PatMatch,
  ) -> usize {
    self.between_matched_impl(pos, c, annotation, defaultmatch, Some(open_test))
  }

  fn between_matched_impl(
    &mut self,
    pos: usize,
    c: u8,
    annotation: Comp,
    mut defaultmatch: Atom,
    opentest: Option<impl FnOnce(&str, usize) -> PatMatch>,
  ) -> usize {
    let mut can_open = find_at(&self.subject, "^%S", pos + 1).is_match;
    let mut can_close = !self.subject[..pos].ends_with(is_space);
    let has_open_marker =
      pos != 0 && self.matches.get(&(pos - 1)).map_or(false, |it| it.is(Atom::OpenMarker));
    let has_close_marker = self.subject.as_bytes()[pos + 1] == b'}';
    let mut endcloser = pos;
    let mut startopener = pos;

    if let Some(opentest) = opentest {
      can_open = can_open && opentest(&self.subject, pos).is_match;
    }

    // allow explicit open/close markers to override:
    if has_open_marker {
      can_open = true;
      can_close = false;
      startopener = pos - 1;
    }
    if !has_open_marker && has_close_marker {
      can_close = true;
      can_open = false;
      endcloser = pos + 1;
    }

    if has_open_marker && defaultmatch.is_right_atom() {
      defaultmatch = defaultmatch.corresponding_left_atom();
    } else if has_close_marker && defaultmatch.is_left_atom() {
      defaultmatch = defaultmatch.corresponding_right_atom();
    }

    let openers = self.openers.entry(c).or_default();
    if can_close && openers.len() > 0 {
      // check openers for a match
      let opener = *openers.last().unwrap();
      if opener.epos != pos - 1 {
        // exclude empty emph
        self.clear_openers(opener.spos, pos);
        self.add_match(opener.spos..opener.epos, Annot::Add(annotation));
        self.add_match(pos..endcloser, Annot::Sub(annotation));
        return endcloser + 1;
      }
    }
    // if we get here, we didn't match an opener
    if can_open {
      self.add_opener(c, Opener::new(startopener, pos));
      self.add_match(startopener..pos + 1, defaultmatch);
      pos + 1
    } else {
      self.add_match(startopener..endcloser + 1, defaultmatch);
      endcloser + 1
    }
  }

  fn matchers(&mut self, c: u8, pos: usize, endpos: usize) -> Option<usize> {
    match c {
      b'`' => {
        let m = bounded_find(&self.subject, "^`*", pos, endpos);
        if !m.is_match {
          return None;
        }
        // TODO: display/inline math

        self.add_match(pos..m.end, Annot::Add(Comp::Verbatim));
        self.verbatim_type = Comp::Verbatim;

        self.verbatim = m.end - pos;
        return Some(m.end);
      }
      b'\\' => {
        let m = bounded_find(&self.subject, "^[ \t]*\r?\n", pos + 1, endpos);
        self.add_match(pos..pos + 1, Atom::Escape);

        if m.is_match {
          // see f there were preceding spaces
          if let Some((_, &mm)) = self.matches.iter().rev().next() {
            let sp = mm.s;
            let mut ep = mm.e;
            if mm.is(Atom::Str) {
              while self.subject.as_bytes()[ep] == b' ' || self.subject.as_bytes()[ep] == b'\t' {
                ep = ep - 1
              }
              if sp == ep {
                self.matches.remove(&sp);
              } else {
                self.add_match(sp..ep, Atom::Str)
              }
            }
          }
          self.add_match(pos + 1..m.end, Atom::Hardbreak);
          return Some(m.end);
        } else {
          let m = bounded_find(&self.subject, "^[%p ]", pos + 1, endpos);
          if !m.is_match {
            self.add_match(pos..pos + 1, Atom::Str);
            return Some(pos + 1);
          } else {
            self.add_match(pos..pos + 1, Atom::Escape);
            if find_at(&self.subject, "^ ", pos + 1).is_match {
              self.add_match(pos + 1..m.end, Atom::Nbsp)
            } else {
              self.add_match(pos + 1..m.end, Atom::Str)
            }
            return Some(m.end);
          }
        }
      }
      b'<' => {
        let url = bounded_find(&self.subject, "^%<[^<>%s]+%>", pos, endpos);
        if url.is_match {
          let is_url = bounded_find(&self.subject, "^%a+:", pos + 1, url.end).is_match;
          let is_email = bounded_find(&self.subject, "^[^:]+%@", pos + 1, url.end).is_match;
          if is_email {
            self.add_match(url.start..url.start + 1, Comp::Email.add());
            self.add_match(url.start + 1..url.end - 1, Atom::Str);
            self.add_match(url.end - 1..url.end, Comp::Email.sub());
            return Some(url.end);
          } else if is_url {
            self.add_match(url.start..url.start + 1, Comp::Url.add());
            self.add_match(url.start + 1..url.end - 1, Atom::Str);
            self.add_match(url.end - 1..url.end, Comp::Url.sub());
            return Some(url.end);
          }
        }
        return None;
      }
      b'~' => Some(self.between_matched(pos, b'~', Comp::Subscript, Atom::Str)),
      b'^' => Some(self.between_matched(pos, b'^', Comp::Superscript, Atom::Str)),
      b'[' => {
        let m = bounded_find(&self.subject, "^%^([^]]+)%]", pos + 1, endpos);
        if m.is_match {
          self.add_match(pos..m.end, Atom::FootnoteReference);
          return Some(m.end);
        } else {
          self.add_opener(b'[', Opener::new(pos, pos + 1));
          self.add_match(pos..pos + 1, Atom::Str);
          return Some(pos + 1);
        }
      }
      b']' => {
        let openers = self.openers.entry(b'[').or_default();
        if openers.len() > 0 {
          let opener = openers.last_mut().unwrap();
          if opener.annot == "reference_link" {
            let opener = *opener;
            // found a reference link
            // add the matches
            let is_image = self.subject[..opener.spos].ends_with('!')
              && !self.subject[..opener.spos].ends_with("[]");
            if is_image {
              self.add_match(opener.spos - 1..opener.spos, Atom::ImageMarker);
              self.add_match(opener.spos..opener.epos, Comp::Imagetext.add());
              self.add_match(opener.subspos..opener.subepos, Comp::Imagetext.sub());
            } else {
              self.add_match(opener.spos..opener.epos, Comp::Linktext.add());
              self.add_match(opener.subspos..opener.subepos, Comp::Linktext.sub());
            }
            self.add_match(opener.subepos - 1..opener.subepos, Comp::Reference.add());
            self.add_match(pos..pos, Comp::Reference.sub());
            // convert all matches to str
            self.str_matches(opener.subepos + 1, pos);
            // remove from openers
            self.clear_openers(opener.spos, pos);
            return Some(pos + 1);
          } else if bounded_find(&self.subject, "^[%[]", pos + 1, endpos).is_match {
            opener.annot = "reference_link";
            opener.subspos = pos; // intermediate ]
            opener.subepos = pos + 2; // intermediate [
            self.add_match(pos..pos + 2, Atom::Str);
            return Some(pos + 2);
          } else if bounded_find(&self.subject, "^[(]", pos + 1, endpos).is_match {
            opener.annot = "explicit_link";
            opener.subspos = pos; // intermediate ]
            opener.subepos = pos + 2; // intermediate (
            self.openers.remove(&b'('); // clear ( openers
            self.destination = true;
            self.add_match(pos..pos + 2, Atom::Str);
            return Some(pos + 2);
          }
        }
        return None;
      }
      b'(' => {
        if !self.destination {
          return None;
        }
        self.add_opener(b'(', Opener::new(pos, pos + 1));
        self.add_match(pos..pos + 1, Atom::Str);
        return Some(pos + 1);
      }
      b')' => {
        if !self.destination {
          return None;
        }
        let parens = self.openers.entry(b'(').or_default();
        if parens.len() > 0 {
          // TODO?
          parens.pop();
          self.add_match(pos..pos + 1, Atom::Str);
          return Some(pos + 1);
        } else {
          let openers = &self.openers.entry(b'[').or_default().clone();
          if let Some(&opener) = openers.last() {
            if opener.annot == "explicit_link" {
              let (startdest, enddest) = (opener.subepos - 1, pos);
              // we have inline link
              let is_image = self.subject[..opener.spos].ends_with('!')
                && !self.subject[..opener.spos].ends_with("[]");
              if is_image {
                self.add_match(opener.spos - 1..opener.spos, Atom::ImageMarker);
                self.add_match(opener.spos..opener.epos, Comp::Imagetext.add());
                self.add_match(opener.subspos..opener.subepos, Comp::Imagetext.sub());
              } else {
                self.add_match(opener.spos..opener.epos, Comp::Linktext.add());
                self.add_match(opener.subspos..opener.subepos, Comp::Linktext.sub());
              }
              self.add_match(startdest..startdest + 1, Comp::Destination.add());
              self.add_match(enddest..enddest + 1, Comp::Destination.sub());
              self.destination = false;
              // convert all matches to str
              self.str_matches(opener.subepos + 1, pos);
              // remove from openers
              self.clear_openers(opener.spos, pos);
              return Some(enddest + 1);
            }
          }
          return None;
        }
      }
      b'_' => Some(self.between_matched(pos, b'_', Comp::Emph, Atom::Str)),
      b'*' => Some(self.between_matched(pos, b'*', Comp::Strong, Atom::Str)),
      b'{' => {
        if self.subject[pos + 1..endpos].starts_with(|c: char| "_*~^+='\"-".contains(c)) {
          self.add_match(pos..pos + 1, Atom::OpenMarker);
          return Some(pos + 1);
        } else {
          // TODO: attributes
          self.add_match(pos..pos + 1, Atom::Str);
          return Some(pos + 1);
        }
      }
      b':' => {
        let m = bounded_find(&self.subject, "^%:[%w_+-]+%:", pos, endpos);
        if m.is_match {
          self.add_match(m.start..m.end, Atom::Emoji);
          return Some(m.end);
        } else {
          self.add_match(pos..pos + 1, Atom::Str);
          return Some(pos + 1);
        }
      }
      b'+' => Some(self.between_matched_with_open_test(
        pos,
        b'+',
        Comp::Insert,
        Atom::Str,
        |subject, pos| {
          find_at(subject, "^%{", pos - 1).or_else(|| find_at(subject, "^%}", pos + 1))
        },
      )),
      b'=' => Some(self.between_matched_with_open_test(
        pos,
        b'=',
        Comp::Mark,
        Atom::Str,
        |subject, pos| {
          find_at(subject, "^%{", pos - 1).or_else(|| find_at(subject, "^%}", pos + 1))
        },
      )),
      b'\'' => todo!(),
      b'"' => Some(self.between_matched(pos, b'"', Comp::DoubleQuoted, Atom::LeftDoubleQuote)),
      b'-' => {
        let subject = &self.subject[..];
        if subject.as_bytes().get(pos - 1) == Some(&b'{')
          || subject.as_bytes().get(pos + 1) == Some(&b'}')
        {
          return Some(self.between_matched_with_open_test(
            pos,
            b'-',
            Comp::Delete,
            Atom::Str,
            |subject, pos| {
              find_at(subject, "^%{", pos - 1).or_else(|| find_at(subject, "^%}", pos + 1))
            },
          ));
        }

        let ep = find_at(subject, "^%-*", pos).end.min(endpos);
        let mut hyphens = ep - pos;
        if subject.as_bytes().get(ep) == Some(&b'}') {
          // last hyphen is close del
          hyphens -= 1;
        }
        if hyphens == 0 {
          self.add_match(pos..pos + 2, Atom::Str);
          return Some(pos + 2);
        }
        let mut pos = pos;
        let all_em = hyphens % 3 == 0;
        let all_en = hyphens % 2 == 0;
        while hyphens > 0 {
          if all_em {
            self.add_match(pos..pos + 3, Atom::EmDash);
            pos += 3;
            hyphens -= 3;
          } else if all_en {
            self.add_match(pos..pos + 2, Atom::EnDash);
            pos += 2;
            hyphens -= 2;
          } else if hyphens >= 3 && (hyphens % 2 != 0 || hyphens > 4) {
            self.add_match(pos..pos + 3, Atom::EmDash);
            pos += 3;
            hyphens -= 3;
          } else if hyphens >= 2 {
            self.add_match(pos..pos + 2, Atom::EnDash);
            pos += 2;
            hyphens -= 2;
          } else {
            self.add_match(pos..pos + 1, Atom::Str);
            pos += 1;
            hyphens -= 1;
          }
        }
        Some(pos)
      }
      b'.' => {
        if bounded_find(&self.subject, "^%.%.", pos + 1, endpos).is_match {
          self.add_match(pos..pos + 3, Atom::Ellipses);
          return Some(pos + 3);
        }
        return None;
      }
      _ => return None,
    }
  }
  fn single_char(&mut self, pos: usize) -> usize {
    self.add_match(pos..pos + 1, Atom::Str);
    pos + 1
  }

  // Feed a slice to the parser, updating state.
  pub fn feed(&mut self, spos: usize, endpos: usize) {
    let special = "[%]%[\\`{}_*()!<>~^:=+$\r\n'\".-]";
    let subject = self.subject.clone();
    if spos < self.firstpos {
      self.firstpos = spos
    }
    if endpos > self.lastpos {
      self.lastpos = endpos
    }
    let mut pos = spos;
    while pos < endpos {
      if false {
        // TODO: attributes
      } else {
        // find next interesting character:
        let newpos = bounded_find(&subject, special, pos, endpos).or(endpos);
        if newpos > pos {
          self.add_match(pos..newpos, Atom::Str);
          pos = newpos;
          if pos > endpos {
            break; // otherwise, fall through:
          }
        }
        // if we get here, then newpos = pos,
        // i.e. we have something interesting at pos
        let c = subject.as_bytes()[pos];
        if c == b'\r' || c == b'\n' {
          if c == b'\r' && bounded_find(&subject, "^[%n]", pos + 1, endpos).is_match {
            self.add_match(pos..pos + 2, Atom::Softbreak);
            pos = pos + 2
          } else {
            self.add_match(pos..pos + 1, Atom::Softbreak);
            pos = pos + 1
          }
        } else if self.verbatim > 0 {
          if c == b'`' {
            let m = bounded_find(&subject, "^`+", pos, endpos);
            if m.is_match && m.end - pos == self.verbatim {
              // TODO: Check for raw attributes
              self.add_match(pos..m.end, self.verbatim_type.sub());
              pos = m.end;
              self.verbatim = 0;
              self.verbatim_type = Comp::default();
            } else {
              let endchar = m.end_or(endpos);
              self.add_match(pos..endchar, Atom::Str);
              pos = endchar
            }
          } else {
            self.add_match(pos..pos + 1, Atom::Str);
            pos = pos + 1
          }
        } else {
          pos = self.matchers(c, pos, endpos).unwrap_or_else(|| self.single_char(pos))
        }
      }
    }
  }

  pub(crate) fn get_matches(&mut self) -> Vec<Match> {
    let mut sorted: Vec<Match> = Vec::new();
    let mut m_last = Match::new(0..0, Atom::Ellipses); // TODO
    for i in self.firstpos..=self.lastpos {
      if let Some(&m) = self.matches.get(&i) {
        if m.is(Atom::Str) && m_last.is(Atom::Str) && m_last.e == m.s {
          (*sorted.last_mut().unwrap()).e = m.e;
          m_last.e = m.e;
        } else {
          sorted.push(m);
          m_last = m
        }
      }
    }
    if sorted.len() > 0 {
      if sorted.last().unwrap().is(Atom::Softbreak) {
        // remove final softbreak
        sorted.pop();
      }
      if self.verbatim > 0 {
        // unclosed verbatim
        let e = sorted.last().unwrap().e;
        sorted.push(Match::new(e..e, self.verbatim_type.sub()))
      }
    }
    sorted
  }
}
