use crate::{
  patterns::{find, find_at},
  Match, annot::Atom,
};

#[derive(Default)]
pub(crate) struct Tokenizer {
  subject: String,
  state: State,
  begin: usize,
  failed: bool,
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

impl Tokenizer {
  pub(crate) fn new(subject: String) -> Tokenizer {
    let mut res = Tokenizer::default();
    res.subject = subject;
    res
  }
  // Feed tokenizer a slice of text from the subject, between
  // startpos and endpos inclusive.  Return status, position,
  // where status is either "done" (position should point to
  // final '}'), "fail" (position should point to first character
  // that could not be tokenized), or "continue" (position should
  // point to last character parsed).
  pub(crate) fn feed(&mut self, startpos: usize, endpos: usize) -> Result<Option<usize>, usize> {
    let mut pos = startpos;
    while pos <= endpos {
      self.state = self.step(pos);
      match self.state {
        State::Done => return Ok(Some(pos)),
        State::Fail => {
          self.lastpos = pos;
          return Err(pos);
        }
        _ => {
          self.lastpos = pos;
          pos = pos + 1
        }
      }
    }
    Ok(None)
  }

  fn step(&mut self, pos: usize) -> State {
    match self.state {
      State::Start => {
        if find_at(&self.subject, "^{", pos) {
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
      State::ScanningId => {
        let c = self.subject.as_bytes()[pos];
        match c {
          b'_' | b'-' | b':' => State::ScanningId,
          b'}' => {
            if self.lastpos > self.begin + 1 {
                self.add_match(self.begin + 1..self.lastpos, Atom::Id)
            }
            self.begin = !0;
            State::Done
          }
        }
      }
      State::ScanningClass => todo!(),
      State::ScanningKey => todo!(),
      State::ScanningValue => todo!(),
      State::ScanningBareValue => todo!(),
      State::ScanningQuotedValue => todo!(),
      State::ScanningEscaped => todo!(),
    }
  }
}
