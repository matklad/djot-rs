use std::{ops::Range, rc::Rc};

use annot::Annot;

mod annot;
mod patterns;
mod block;
mod inline;
pub mod ast;
mod to_ast;
mod to_html;

pub fn parse(text: &str) -> ast::Tag {
  parse_opts(ParseOpts::default(), text).ast
}

#[derive(Default, Clone)]
pub struct ParseOpts {
  pub debug_matches: bool,
}

pub struct Parse {
  pub ast: ast::Tag,
  pub debug: String,
}

pub fn parse_opts(opts: ParseOpts, text: &str) -> Parse {
  let mut p = block::Parser::new(text.to_string(), opts, None);
  p.parse();
  let debug = p.debug.clone();
  let ast = p.to_ast();
  Parse { ast, debug }
}

pub fn to_html(tag: &ast::Tag) -> String {
  to_html_opts(&HtmlOpts::default(), tag)
}

#[derive(Default, Clone)]
pub struct HtmlOpts {}

pub fn to_html_opts(opts: &HtmlOpts, tag: &ast::Tag) -> String {
  to_html::to_html(opts, tag)
}

pub type Warn = Rc<dyn Fn()>;

#[derive(Debug, Clone, Copy)]
struct Match {
  s: usize,
  e: usize,
  a: Annot,
}

impl Match {
  fn new(range: Range<usize>, a: impl Into<Annot>) -> Match {
    Match { s: range.start, e: range.end, a: a.into() }
  }
  fn is(&self, annot: impl Into<Annot>) -> bool {
    self.a == annot.into()
  }
  fn is_not(&self, annot: impl Into<Annot>) -> bool {
    !self.is(annot)
  }
}
