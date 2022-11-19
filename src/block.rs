use std::{fmt::Write, ops::Range};

use crate::{
  annot::{Annot, Atom, Comp},
  inline,
  patterns::{find, find_at, PatMatch},
  Match, ParseOpts,
};

#[derive(Default)]
pub struct Tokenizer {
  pub subject: String,
  indent: usize,
  startline: usize,
  starteol: usize,
  endeol: usize,
  pub(crate) matches: Vec<Match>,
  pos: usize,
  last_matched_container: usize,
  opts: ParseOpts,
  finished_line: bool,

  pub(crate) debug: String,
}

trait Container {
  fn content(&self) -> &'static str;
  fn inline_parser(&mut self) -> Option<&mut inline::Tokenizer> {
    None
  }
  fn restore_indent(&self) -> Option<usize> {
    None
  }
  fn open(p: &mut Tokenizer) -> Option<Box<dyn Container>>
  where
    Self: Sized;
  fn cont(&mut self, p: &mut Tokenizer) -> bool;
  fn close(self: Box<Self>, p: &mut Tokenizer);
}

const CONTAINERS: &[fn(&mut Tokenizer) -> Option<Box<dyn Container>>] =
  &[Para::open, CodeBlock::open, ReferenceDefinition::open];

struct Para {
  inline_parser: inline::Tokenizer,
}

impl Container for Para {
  fn content(&self) -> &'static str {
    "inline"
  }
  fn inline_parser(&mut self) -> Option<&mut inline::Tokenizer> {
    Some(&mut self.inline_parser)
  }
  fn open(p: &mut Tokenizer) -> Option<Box<dyn Container>>
  where
    Self: Sized,
  {
    p.add_match(p.pos..p.pos, Comp::Para.add());
    Some(Box::new(Para {
      inline_parser: inline::Tokenizer::new(p.subject.clone(), p.opts.clone()),
    }))
  }

  fn cont(&mut self, p: &mut Tokenizer) -> bool {
    p.find("^%S").is_match
  }

  fn close(mut self: Box<Self>, p: &mut Tokenizer) {
    p.matches.extend(self.inline_parser.get_matches());
    p.add_match(p.pos - 1..p.pos - 1, Comp::Para.sub())
  }
}

struct CodeBlock {
  border: char,
  indent: usize,
}

impl Container for CodeBlock {
  fn content(&self) -> &'static str {
    "text"
  }
  fn restore_indent(&self) -> Option<usize> {
    Some(self.indent)
  }
  fn open(p: &mut Tokenizer) -> Option<Box<dyn Container>>
  where
    Self: Sized,
  {
    let mut border = '`';
    let mut m = p.find("^```([ \t]*)([^%s`]*)[ \t]*[\r\n]");
    if !m.is_match {
      border = '~';
      m = p.find("^~~~([ \t]*)([^%s`]*)[ \t]*[\r\n]");
    }
    if !m.is_match {
      return None;
    }
    let lang = m.cap2;

    p.add_match(p.pos..p.pos + 3, Comp::CodeBlock.add());
    if !lang.is_empty() {
      p.add_match(lang.start..lang.end, Atom::CodeLanguage)
    }

    p.pos = p.pos + 2;
    p.finished_line = true;
    Some(Box::new(CodeBlock { border, indent: p.indent }))
  }

  fn cont(&mut self, p: &mut Tokenizer) -> bool {
    let m =
      if self.border == '`' { p.find("^(```)[ \t]*[\r\n]") } else { p.find("^(~~~)[ \t]*[\r\n]") };
    if m.is_match {
      p.pos = m.end - 1;
      p.finished_line = true;
      false
    } else {
      true
    }
  }

  fn close(self: Box<Self>, p: &mut Tokenizer) {
    p.add_match(p.pos - 3..p.pos, Comp::CodeBlock.sub());
  }
}

struct ReferenceDefinition {
  indent: usize,
}

impl Container for ReferenceDefinition {
  fn content(&self) -> &'static str {
    ""
  }

  fn open(p: &mut Tokenizer) -> Option<Box<dyn Container>>
  where
    Self: Sized,
  {
    let m = p.find("^[[]([^\r\n]*)%]:[ \t]*(%S*)");
    if !m.is_match {
      return None;
    }
    p.add_match(m.start..m.start, Comp::ReferenceDefinition.add());
    p.add_match(m.start..m.start + m.cap1.len() + 1, Atom::ReferenceKey);
    p.add_match(m.end - m.cap2.len()..m.end, Atom::ReferenceValue);
    p.pos = m.end;
    Some(Box::new(ReferenceDefinition { indent: p.indent }))
  }

  fn cont(&mut self, p: &mut Tokenizer) -> bool {
    if self.indent >= p.indent {
      return false;
    }
    let m = p.find("^(%S+)");
    if m.is_match {
      p.add_match(m.cap1.start..m.cap1.end, Atom::ReferenceValue);
      p.pos = m.end;
    }
    true
  }

  fn close(self: Box<Self>, p: &mut Tokenizer) {
    p.add_match(p.pos..p.pos, Comp::ReferenceDefinition.sub())
  }

  fn inline_parser(&mut self) -> Option<&mut inline::Tokenizer> {
    None
  }
}

impl Tokenizer {
  pub fn new(mut subject: String, opts: ParseOpts) -> Tokenizer {
    if !find(&subject, "[\r\n]$").is_match {
      subject.push('\n');
    }
    let mut res = Tokenizer::default();
    res.subject = subject;
    res.opts = opts;
    res
  }

  fn find(&self, pat: &'static str) -> PatMatch {
    find_at(&self.subject, pat, self.pos)
  }

  fn add_match(&mut self, range: Range<usize>, annot: impl Into<Annot>) {
    self.matches.push(Match::new(range, annot))
  }

  //  fn add_container(&mut self, container: Container) {
  //    let last_matched = self.last_matched_container;
  //    while containers.len() > last_matched
  //      || (containers.len() > 0 && containers.last().unwrap().spec.content != "block")
  //    {
  //      containers.last().unwrap().spec.close(self)
  //    }
  //    containers.push(container)
  //  }

  fn skip_space(&mut self) {
    let m = find_at(&self.subject, "[^ \t]", self.pos);
    if m.is_match {
      self.indent = m.start - self.startline;
      self.pos = m.start;
    }
  }

  fn get_eol(&mut self) {
    let mut m = find_at(&self.subject, "[\r]?[\n]", self.pos);
    if !m.is_match {
      (m.start, m.end) = (self.subject.len(), self.subject.len());
    }
    self.starteol = m.start;
    self.endeol = m.end;
  }

  pub fn parse(&mut self) {
    let mut containers: Vec<Box<dyn Container>> = Vec::new();

    let subjectlen = self.subject.len();
    while self.pos < subjectlen {
      self.indent = 0;
      self.startline = self.pos;
      self.finished_line = false;
      self.get_eol();

      // check open containers for continuation
      self.last_matched_container = 0;
      for idx in 0..containers.len() {
        // skip any indentation
        self.skip_space();
        if containers[idx].cont(self) {
          self.last_matched_container = idx + 1
        } else {
          break;
        }
      }

      // if we hit a close fence, we can move to next line
      if self.finished_line {
        while containers.len() > self.last_matched_container {
          containers.pop().unwrap().close(self)
        }
      }

      if !self.finished_line {
        // check for new containers
        self.skip_space();
        let mut is_blank = self.pos == self.starteol;

        let mut new_starts = false;
        let last_match = containers[..self.last_matched_container].first();
        let mut check_starts = !is_blank
          && !matches!(last_match, Some(c) if c.content() != "block")
          && !self.find("^%a+%s").is_match; // optimization

        while check_starts {
          check_starts = false;
          for i in 1..CONTAINERS.len() {
            let open = CONTAINERS[i];
            if let Some(cont) = open(self) {
              let content = cont.content();
              containers.push(cont);
              self.last_matched_container = containers.len();
              if self.finished_line {
                check_starts = false
              } else {
                self.skip_space();
                new_starts = true;
                check_starts = content != "text"
              }
              break;
            }
          }
        }

        if !self.finished_line {
          // handle remaining content
          self.skip_space();

          is_blank = self.pos == self.starteol;

          let is_lazy = !is_blank
            && !new_starts
            && self.last_matched_container < containers.len()
            && containers.last().unwrap().content() == "inline";

          if !is_lazy && self.last_matched_container < containers.len() {
            while containers.len() > self.last_matched_container {
              containers.pop().unwrap().close(self);
            }
          }

          // add para by default if there's text
          if !matches!(containers.last(), Some(c) if c.content() != "block") {
            if is_blank {
              if !new_starts {
                // need to track these for tight/loose lists
                self.add_match(self.pos..self.endeol, Atom::Blankline);
              }
            } else {
              let para = CONTAINERS[0](self).unwrap();
              containers.push(para);
            }
          }

          if let Some(tip) = containers.last_mut() {
            if let Some(tip_indent) = tip.restore_indent() {
              let mut startpos = self.pos;
              if self.indent > tip_indent {
                // get back the leading spaces we gobbled
                startpos = startpos - (self.indent - tip_indent)
              }
              self.add_match(startpos..self.endeol, Atom::Str)
            } else if let Some(inline_parser) = tip.inline_parser() {
              if !is_blank {
                inline_parser.feed(self.pos, self.endeol)
              }
            }
          }
        }
      }

      self.pos = self.endeol;
    }
    self.finish(containers)
  }

  fn finish(&mut self, mut containers: Vec<Box<dyn Container>>) {
    // close unmatched containers
    while let Some(cont) = containers.pop() {
      cont.close(self)
    }
    if self.opts.debug_matches {
      for &m in &self.matches {
        let ms = format!("{} {}-{}", m.a, m.s + 1, if m.e == m.s { m.e + 1 } else { m.e });
        writeln!(self.debug, "{ms:<20} {:?}", self.subject.get(m.s..m.e).unwrap_or_default())
          .expect("str format can't fail");
      }
    }
  }
}
