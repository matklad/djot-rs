pub mod ast;

mod annot;
mod patterns;
mod block;
mod inline;
mod tree;
mod html;

use std::ops::Range;

use annot::Annot;

#[derive(Debug, Clone)]
pub struct Document {
  pub children: Vec<ast::Tag>,
  pub debug: String,
}

impl Document {
  pub fn parse(text: &str) -> Document {
    Document::parse_opts(ParseOpts::default(), text)
  }

  pub fn parse_opts(opts: ParseOpts, text: &str) -> Document {
    let mut p = block::Parser::new(text.to_string(), opts);
    p.parse();
    tree::build(p)
  }

  pub fn to_html(&self) -> String {
    self.to_html_opts(&HtmlOpts::default())
  }

  pub fn to_html_opts(&self, opts: &HtmlOpts) -> String {
    html::convert(opts, self)
  }

  pub fn to_json(&self) -> String {
    #[derive(serde::Serialize)]
    struct DocRepr<'a> {
      tag: &'static str,
      children: &'a [ast::Tag],
    }
    serde_json::to_string_pretty(&DocRepr { tag: "doc", children: self.children.as_slice() })
      .unwrap()
  }
}

#[derive(Default, Clone)]
pub struct ParseOpts {
  pub debug_matches: bool,
}

#[derive(Default, Clone)]
pub struct HtmlOpts {}

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
