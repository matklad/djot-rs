// TODO: re-export everything.
pub mod ast;

mod annot;
mod patterns;
mod block;
mod inline;
mod attribute;
mod tree;
mod emoji;
mod html;
#[cfg(test)]
mod sourcegen;

use std::{collections::BTreeMap, ops::Range};

use crate::annot::Annot;

#[derive(Debug, Default, Clone)]
pub struct Document {
  pub children: Vec<ast::Tag>,
  pub references: BTreeMap<String, ast::ReferenceDefinition>,
  pub debug: String,
}

#[derive(Default, Clone)]
pub struct ParseOpts {
  pub debug_matches: bool,
}

#[derive(Default, Clone)]
pub struct HtmlOpts {}

impl Document {
  pub fn parse(text: &str) -> Document {
    Document::parse_opts(ParseOpts::default(), text)
  }

  pub fn parse_opts(opts: ParseOpts, text: &str) -> Document {
    let mut p = block::Tokenizer::new(text.to_string(), opts);
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
      references: &'a BTreeMap<String, ast::ReferenceDefinition>,
    }
    serde_json::to_string_pretty(&DocRepr {
      tag: "doc",
      children: self.children.as_slice(),
      references: &self.references,
    })
    .unwrap()
  }
}

#[derive(Debug, Clone)]
struct Match {
  range: Range<usize>,
  a: Annot,
}

impl Match {
  fn new(range: Range<usize>, a: impl Into<Annot>) -> Match {
    Match { range, a: a.into() }
  }
  fn is(&self, annot: impl Into<Annot>) -> bool {
    self.a == annot.into()
  }
  fn is_not(&self, annot: impl Into<Annot>) -> bool {
    !self.is(annot)
  }
}

/// Appends formatted string to a `String`.
macro_rules! _format_to {
    ($buf:expr) => ();
    ($buf:expr, $lit:literal $($arg:tt)*) => {
        { use ::std::fmt::Write as _; let _ = ::std::write!($buf, $lit $($arg)*); }
    };
}
pub(crate) use _format_to as format_to;
