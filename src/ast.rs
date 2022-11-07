use std::collections::BTreeMap;

use crate::{patterns::capture2, Match};

type Attrs = BTreeMap<String, String>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Tag {
  #[serde(flatten)]
  pub kind: TagKind,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  pub attrs: Attrs,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub children: Vec<Tag>,
}

macro_rules!  tags {
    ($($tag:ident),*) => {
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "tag", rename_all = "lowercase")]
pub enum TagKind {$(
  $tag($tag)
),*}

$(
impl From<$tag> for TagKind {
  fn from(kind: $tag) -> TagKind {
    TagKind::$tag(kind)
  }
}
)*
    };
}

tags![Doc, Heading, Para, Str];

#[derive(Debug, Clone, serde::Serialize)]
pub struct Doc {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Heading {
  pub level: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Para {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Str {
  pub text: String,
}

impl Tag {
  pub fn new(kind: impl Into<TagKind>) -> Tag {
    Tag { kind: kind.into(), attrs: Attrs::new(), children: Vec::new() }
  }

  pub fn with_attrs(mut self, attrs: Attrs) -> Tag {
    self.attrs = attrs;
    self
  }

  pub fn with_children(mut self, children: Vec<Tag>) -> Tag {
    self.children = children;
    self
  }

  pub fn to_json(&self) -> String {
    serde_json::to_string_pretty(self).unwrap()
  }
}

impl Str {
  pub fn new(text: impl Into<String>) -> Str {
    Str { text: text.into() }
  }
}

#[test]
fn serialization() {
  let doc = Tag::new(Doc {}).with_children(vec![
    Tag::new(Heading { level: 1 })
      .with_attrs(Attrs::from([("id".to_string(), "Try-djot".to_string())]))
      .with_children(vec![Tag::new(Str::new("Try djot"))]),
    Tag::new(Para {}).with_children(vec![Tag::new(Str::new("Hello"))]),
  ]);

  eprintln!("{}", doc.to_json())
}
