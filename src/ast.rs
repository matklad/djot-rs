use indexmap::IndexMap;

pub type Attrs = IndexMap<String, String>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Tag {
  #[serde(flatten)]
  pub kind: TagKind,
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub children: Vec<Tag>,
}

macro_rules!  tags {
    ($($tag:ident,)*) => {
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

$(
impl Cast<$tag> for Tag {
  fn cast(&mut self) -> &mut $tag {
    match &mut self.kind { TagKind::$tag(it) => it, _ => panic!() }
  }
}
)*
    };
}

tags![
  Doc,
  Heading,
  Para,
  Link,
  Image,
  CodeBlock,
  ReferenceDefinition,
  Strong,
  Emph,
  Span,
  DoubleQuoted,
  Verbatim,
  Softbreak,
  Url,
  Str,
  Emoji,
  ReferenceKey,
  ReferenceValue,
  Insert,
  Delete,
  Mark,
  Superscript,
  Subscript,
  EmDash,
  EnDash,
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct Doc {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Heading {
  pub level: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Para {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Link {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub destination: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reference: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Image {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub destination: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reference: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CodeBlock {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lang: Option<String>,
  pub text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReferenceDefinition {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Softbreak {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Strong {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Emph {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Insert {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Delete {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Mark {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Superscript {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Subscript {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EmDash {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EnDash {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Span {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DoubleQuoted {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReferenceKey {
  pub text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReferenceValue {
  pub text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Url {
  pub destination: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Verbatim {
  pub text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Str {
  pub text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Emoji {
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

  pub(crate) fn cast<T>(&mut self) -> &mut T
  where
    Self: Cast<T>,
  {
    Cast::cast(self)
  }
}

impl Str {
  pub fn new(text: impl Into<String>) -> Str {
    Str { text: text.into() }
  }
}

impl Emoji {
  pub fn new(text: impl Into<String>) -> Emoji {
    Emoji { text: text.into() }
  }
}

pub(crate) trait Cast<T> {
  fn cast(&mut self) -> &mut T;
}
