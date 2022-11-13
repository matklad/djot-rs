use crate::{
  inline,
  patterns::{find, find_at, PatMatch},
  Match, ParseOpts, Warn,
};

struct Container {
  spec: &'static Spec,
  inline_parser: inline::Parser,
  indent: usize,
}

#[derive(Default)]
pub struct Parser {
  warn: Option<Warn>,
  pub subject: String,
  indent: usize,
  startline: usize,
  starteol: usize,
  endeol: usize,
  pub matches: Vec<Match>,
  containers: Vec<Container>,
  pos: usize,
  last_matched_container: usize,
  opts: ParseOpts,
  finished_line: bool,
}

struct Spec {
  #[allow(unused)]
  name: &'static str,
  is_para: bool,
  content: &'static str,
  cont: fn(&mut Parser) -> bool,
  open: fn(&mut Parser) -> bool,
  close: fn(&mut Parser),
}

impl Spec {
  fn cont(&self, p: &mut Parser) -> bool {
    (self.cont)(p)
  }
  fn open(&self, p: &mut Parser) -> bool {
    (self.open)(p)
  }
  fn close(&self, p: &mut Parser) {
    (self.close)(p)
  }
}

static SPECS: &[Spec] = &[
  Spec {
    name: "para",
    is_para: true,
    content: "inline",
    cont: |p| p.find("^%S").is_match,
    open: |p| {
      p.add_container(Container {
        spec: &SPECS[0],
        inline_parser: inline::Parser::new(p.subject.clone(), p.opts.clone(), p.warn.clone()),
        indent: 0,
      });
      p.add_match(p.pos, p.pos, "+para");
      true
    },
    close: |p| {
      p.get_inline_matches();
      p.add_match(p.pos - 1, p.pos - 1, "-para");
      p.containers.pop();
    },
  },
  Spec {
    name: "code_block",
    is_para: false,
    content: "text",
    cont: |p| {
      let m = p.find("^(```)[ \t]*[\r\n]");
      if m.is_match {
        p.pos = m.end - 1;
        p.finished_line = true;
        false
      } else {
        true
      }
    },
    open: |p| {
      if !p.subject[p.pos..].starts_with("```") {
        return false;
      }
      p.add_container(Container {
        spec: &SPECS[1],
        inline_parser: inline::Parser::new(p.subject.clone(), p.opts.clone(), p.warn.clone()),
        indent: 0,
      });
      p.add_match(p.pos, p.pos + 3, "+code_block");
      p.pos = p.pos + 2;
      p.finished_line = true;
      true
    },
    close: |p| {
      p.add_match(p.pos - 3, p.pos, "-code_block");
      p.containers.pop();
    },
  },
];

impl Parser {
  pub fn new(mut subject: String, opts: ParseOpts, warn: Option<Warn>) -> Parser {
    if !find(&subject, "[\r\n]$").is_match {
      subject.push('\n');
    }
    let mut res = Parser::default();
    res.subject = subject;
    res.opts = opts;
    res.warn = warn;
    res
  }

  fn get_inline_matches(&mut self) {
    let matches = self.containers.last_mut().unwrap().inline_parser.get_matches();
    self.matches.extend(matches);
  }

  fn find(&self, pat: &'static str) -> PatMatch<'static> {
    find_at(&self.subject, pat, self.pos)
  }

  fn add_match(&mut self, startpos: usize, endpos: usize, annotation: &'static str) {
    self.matches.push((startpos, endpos, annotation))
  }

  fn add_container(&mut self, container: Container) {
    let last_matched = self.last_matched_container;
    while self.containers.len() > last_matched
      || (self.containers.len() > 0 && self.containers.last().unwrap().spec.content != "block")
    {
      self.containers.last().unwrap().spec.close(self)
    }
    self.containers.push(container)
  }

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
    let specs = SPECS;
    let para_spec = &specs[0];
    let subjectlen = self.subject.len();
    while self.pos < subjectlen {
      self.indent = 0;
      self.startline = self.pos;
      self.finished_line = false;
      self.get_eol();

      // check open containers for continuation
      self.last_matched_container = 0;
      for idx in 0..self.containers.len() {
        // skip any indentation
        self.skip_space();
        let container = self.containers[idx].spec;
        if container.cont(self) {
          self.last_matched_container = idx + 1
        } else {
          break;
        }
      }

      // if we hit a close fence, we can move to next line
      if self.finished_line {
        while self.containers.len() > self.last_matched_container {
          self.containers.last().unwrap().spec.close(self)
        }
      }

      if !self.finished_line {
        // check for new containers
        self.skip_space();
        let mut is_blank = self.pos == self.starteol;

        let mut new_starts = false;
        let last_match = self.containers[..self.last_matched_container].first();
        let mut check_starts = !is_blank
          && !matches!(last_match, Some(c) if c.spec.content != "block")
          && !self.find("^%a+%s").is_match; // optimization

        while check_starts {
          check_starts = false;
          for i in 0..specs.len() {
            let spec = &specs[i];
            if !spec.is_para {
              if spec.open(self) {
                self.last_matched_container = self.containers.len();
                if self.finished_line {
                  check_starts = false
                } else {
                  self.skip_space();
                  new_starts = true;
                  check_starts = spec.content != "text"
                }
                break;
              }
            }
          }
        }

        if !self.finished_line {
          // handle remaining content
          self.skip_space();

          is_blank = self.pos == self.starteol;

          let is_lazy = !is_blank
            && !new_starts
            && self.last_matched_container < self.containers.len()
            && self.containers.last().unwrap().spec.content == "inline";

          if !is_lazy && self.last_matched_container < self.containers.len() {
            while self.containers.len() > self.last_matched_container {
              self.containers.last().unwrap().spec.close(self);
            }
          }

          // add para by default if there's text
          if !matches!(self.containers.last(), Some(c) if c.spec.content != "block") {
            if is_blank {
              if !new_starts {
                // need to track these for tight/loose lists
                self.add_match(self.pos, self.endeol, "blankline");
              }
            } else {
              para_spec.open(self);
            }
          }

          if let Some(tip) = self.containers.last_mut() {
            if tip.spec.content == "text" {
              let mut startpos = self.pos;
              if self.indent > tip.indent {
                // get back the leading spaces we gobbled
                startpos = startpos - (self.indent - tip.indent)
              }
              self.add_match(startpos, self.endeol, "str")
            } else if tip.spec.content == "inline" {
              if !is_blank {
                tip.inline_parser.feed(self.pos, self.endeol)
              }
            }
          }
        }
      }

      self.pos = self.endeol;
    }
    self.finish()
  }

  fn finish(&mut self) {
    // close unmatched containers
    while let Some(cont) = self.containers.last() {
      cont.spec.close(self)
    }
    if self.opts.debug_matches {
      for &(s, e, a) in &self.matches {
        let m = format!("{a} {}-{}", s + 1, if e == s { e + 1 } else { e });
        eprintln!("{m:<20} {:?}", self.subject.get(s..e).unwrap_or_default())
      }
    }
  }
}
