use std::ops::Range;

use crate::{
  annot::{Annot, Atom},
  patterns::find_at,
  Match,
};

#[derive(Default)]
pub(crate) struct Tokenizer {
  subject: String,
  state: State,
  begin: usize,
  lastpos: usize,
  matches: Vec<Match>,
}

#[derive(Default)]
enum State {
  Scanning,
  ScanningId,
  ScanningClass,
  ScanningKey,
  ScanningValue,
  ScanningBareValue,
  ScanningQuotedValue,
  ScanningEscaped,
  ScanningComment,
  Fail,
  Done,
  #[default]
  Start,
}

pub(crate) enum Status {
  Done,
  Fail,
  Continue,
}

impl Tokenizer {
  pub(crate) fn new(subject: String) -> Tokenizer {
    let mut res = Tokenizer::default();
    res.subject = subject;
    res
  }

  fn add_match(&mut self, range: Range<usize>, annot: impl Into<Annot>) {
    self.matches.push(Match::new(range, annot))
  }

  pub(crate) fn get_matches(&mut self) -> Vec<Match> {
    std::mem::take(&mut self.matches)
  }

  // Feed tokenizer a slice of text from the subject, between
  // startpos and endpos inclusive.  Return status, position,
  // where status is either "done" (position should point to
  // final '}'), "fail" (position should point to first character
  // that could not be tokenized), or "continue" (position should
  // point to last character parsed).
  pub(crate) fn feed(&mut self, startpos: usize, endpos: usize) -> (Status, usize) {
    let mut pos = startpos;
    while pos <= endpos {
      self.state = self.step(pos);
      match self.state {
        State::Done => return (Status::Done, pos),
        State::Fail => {
          self.lastpos = pos + 1;
          return (Status::Fail, pos);
        }
        _ => {
          self.lastpos = pos + 1;
          pos = pos + 1
        }
      }
    }
    (Status::Continue, pos)
  }

  fn step(&mut self, pos: usize) -> State {
    match self.state {
      State::Start => {
        if find_at(&self.subject, "^{", pos).is_match {
          State::Scanning
        } else {
          State::Fail
        }
      }
      State::Fail => State::Fail,
      State::Done => State::Done,
      State::Scanning => match self.subject.as_bytes()[pos] {
        b' ' | b'\t' | b'\n' | b'\r' => State::Scanning,
        b'}' => State::Done,
        b'#' => {
          self.begin = pos;
          State::ScanningId
        }
        b'%' => {
          self.begin = pos;
          State::ScanningComment
        }
        b'.' => {
          self.begin = pos;
          State::ScanningClass
        }
        _ => {
          if find_at(&self.subject, "^[%a%d_:-]", pos).is_match {
            self.begin = pos;
            State::ScanningKey
          } else {
            State::Fail
          }
        }
      },
      State::ScanningComment => {
        if self.subject.as_bytes()[pos] == b'%' {
          State::Scanning
        } else {
          State::ScanningComment
        }
      }
      State::ScanningId => self.step_ident(pos, Atom::Id, State::ScanningId),
      State::ScanningClass => self.step_ident(pos, Atom::Class, State::ScanningClass),
      State::ScanningKey => {
        let c = self.subject.as_bytes()[pos];
        if c == b'=' {
          self.add_match(self.begin..self.lastpos, Atom::Key);
          self.begin = !0;
          State::ScanningValue
        } else if find_at(&self.subject, "^[%a%d_:-]", pos).is_match {
          State::ScanningKey
        } else {
          State::Fail
        }
      }
      State::ScanningValue => {
        let c = self.subject.as_bytes()[pos];
        if c == b'"' {
          self.begin = pos;
          State::ScanningQuotedValue
        } else if find_at(&self.subject, "^[%a%d_:-]", pos).is_match {
          self.begin = pos;
          State::ScanningBareValue
        } else {
          State::Fail
        }
      }
      State::ScanningBareValue => {
        let c = self.subject.as_bytes()[pos];
        if find_at(&self.subject, "^[%a%d_:-]", pos).is_match {
          State::ScanningBareValue
        } else if c == b'}' {
          self.add_match(self.begin..self.lastpos, Atom::Value);
          self.begin = !0;
          State::Done
        } else if find_at(&self.subject, "^%s", pos).is_match {
          self.add_match(self.begin..self.lastpos, Atom::Value);
          self.begin = !0;
          State::Scanning
        } else {
          State::Fail
        }
      }
      State::ScanningEscaped => State::ScanningQuotedValue,
      State::ScanningQuotedValue => {
        let c = self.subject.as_bytes()[pos];
        match c {
          b'"' => {
            self.add_match(self.begin + 1..self.lastpos, Atom::Value);
            self.begin = !0;
            State::Scanning
          }
          b'\\' => State::ScanningEscaped,
          b'{' | b'}' => State::Fail,
          b'\n' => {
            self.add_match(self.begin + 1..self.lastpos, Atom::Value);
            State::ScanningQuotedValue
          }
          _ => State::ScanningQuotedValue,
        }
      }
    }
  }

  fn step_ident(&mut self, pos: usize, atom: Atom, state: State) -> State {
    let c = self.subject.as_bytes()[pos];
    match c {
      b'_' | b'-' | b':' => state,
      b'}' => {
        if self.lastpos > self.begin + 1 {
          self.add_match(self.begin + 1..self.lastpos, atom)
        }
        self.begin = !0;
        State::Done
      }
      _ => {
        if find_at(&self.subject, "^[^%s%p]", pos).is_match {
          state
        } else if find_at(&self.subject, "^%s", pos).is_match {
          if self.lastpos > self.begin {
            self.add_match(self.begin + 1..self.lastpos, atom)
          }
          self.begin = !0;
          State::Scanning
        } else {
          State::Fail
        }
      }
    }
  }
}
