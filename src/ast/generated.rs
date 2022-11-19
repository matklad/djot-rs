use super::Attrs;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Heading {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
  pub level: u32,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Para {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Link {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
  pub destination: Option<String>,
  pub reference: Option<String>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Image {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
  pub destination: Option<String>,
  pub reference: Option<String>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct CodeBlock {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
  pub lang: Option<String>,
  pub text: String,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Strong {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Emph {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Insert {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Delete {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Mark {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Superscript {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Subscript {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Span {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct DoubleQuoted {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Url {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
  pub destination: String,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct SoftBreak {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct EmDash {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct EnDash {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Verbatim {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub text: String,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Str {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub text: String,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Emoji {
  #[serde(skip_serializing_if = "Attrs::is_empty")]
  pub attrs: Attrs,
  pub alias: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "tag", rename_all = "snake_case")]
pub enum Tag {
  Heading(Heading),
  Para(Para),
  Link(Link),
  Image(Image),
  CodeBlock(CodeBlock),
  Strong(Strong),
  Emph(Emph),
  Insert(Insert),
  Delete(Delete),
  Mark(Mark),
  Superscript(Superscript),
  Subscript(Subscript),
  Span(Span),
  DoubleQuoted(DoubleQuoted),
  Url(Url),
  SoftBreak(SoftBreak),
  EmDash(EmDash),
  EnDash(EnDash),
  Verbatim(Verbatim),
  Str(Str),
  Emoji(Emoji),
}
