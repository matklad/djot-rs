mod generated;

use std::fmt;

pub(crate) use self::generated::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Annot {
  Atom(Atom),
  Add(Comp),
  Sub(Comp),
}

impl PartialEq<Atom> for Annot {
  fn eq(&self, other: &Atom) -> bool {
    match self {
      Annot::Atom(it) => it == other,
      _ => false,
    }
  }
}

impl From<Atom> for Annot {
  fn from(value: Atom) -> Annot {
    Annot::Atom(value)
  }
}

impl fmt::Display for Annot {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Annot::Atom(it) => write!(f, "{it}"),
      Annot::Add(it) => write!(f, "+{it}"),
      Annot::Sub(it) => write!(f, "-{it}"),
    }
  }
}

impl Atom {
  pub(crate) fn is_left_atom(self) -> bool {
    match self {
      Atom::Str
      | Atom::Escape
      | Atom::Hardbreak
      | Atom::Nbsp
      | Atom::Blankline
      | Atom::ImageMarker
      | Atom::Ellipses
      | Atom::Softbreak
      | Atom::FootnoteReference
      | Atom::OpenMarker
      | Atom::Emoji
      | Atom::ReferenceKey
      | Atom::ReferenceValue
      | Atom::CodeLanguage
      | Atom::EmDash
      | Atom::EnDash => false,
      Atom::LeftDoubleQuote => true,
      Atom::RightDoubleQuote => false,
    }
  }

  pub(crate) fn is_right_atom(self) -> bool {
    match self {
      Atom::Str
      | Atom::Escape
      | Atom::Hardbreak
      | Atom::Nbsp
      | Atom::Blankline
      | Atom::ImageMarker
      | Atom::Ellipses
      | Atom::Softbreak
      | Atom::FootnoteReference
      | Atom::OpenMarker
      | Atom::Emoji
      | Atom::ReferenceKey
      | Atom::ReferenceValue
      | Atom::CodeLanguage
      | Atom::EmDash
      | Atom::EnDash => false,
      Atom::LeftDoubleQuote => false,
      Atom::RightDoubleQuote => true,
    }
  }

  pub(crate) fn corresponding_left_atom(self) -> Self {
    match self {
      Atom::Str
      | Atom::Escape
      | Atom::Hardbreak
      | Atom::Nbsp
      | Atom::Blankline
      | Atom::ImageMarker
      | Atom::Ellipses
      | Atom::Softbreak
      | Atom::FootnoteReference
      | Atom::OpenMarker
      | Atom::Emoji
      | Atom::ReferenceKey
      | Atom::ReferenceValue
      | Atom::CodeLanguage
      | Atom::EmDash
      | Atom::EnDash => self,
      Atom::LeftDoubleQuote => self,
      Atom::RightDoubleQuote => Atom::LeftDoubleQuote,
    }
  }

  pub(crate) fn corresponding_right_atom(self) -> Self {
    match self {
      Atom::Str
      | Atom::Escape
      | Atom::Hardbreak
      | Atom::Nbsp
      | Atom::Blankline
      | Atom::ImageMarker
      | Atom::Ellipses
      | Atom::Softbreak
      | Atom::FootnoteReference
      | Atom::OpenMarker
      | Atom::Emoji
      | Atom::ReferenceKey
      | Atom::ReferenceValue
      | Atom::CodeLanguage
      | Atom::EmDash
      | Atom::EnDash => self,
      Atom::LeftDoubleQuote => Atom::RightDoubleQuote,
      Atom::RightDoubleQuote => self,
    }
  }
}

impl fmt::Display for Atom {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Atom::Str => "str",
      Atom::Escape => "escape",
      Atom::Hardbreak => "hardbreak",
      Atom::Nbsp => "nbsp",
      Atom::Blankline => "blankline",
      Atom::ImageMarker => "image_marker",
      Atom::LeftDoubleQuote => "left_double_quote",
      Atom::RightDoubleQuote => "right_double_quote",
      Atom::Ellipses => "ellipses",
      Atom::Softbreak => "softbreak",
      Atom::FootnoteReference => "footnote_reference",
      Atom::OpenMarker => "open_marker",
      Atom::Emoji => "emoji",
      Atom::ReferenceKey => "reference_key",
      Atom::ReferenceValue => "reference_value",
      Atom::CodeLanguage => "code_language",
      Atom::EmDash => "em_dash",
      Atom::EnDash => "en_dash",
    };
    f.write_str(s)
  }
}

impl Comp {
  pub(crate) fn add(self) -> Annot {
    Annot::Add(self)
  }
  pub(crate) fn sub(self) -> Annot {
    Annot::Sub(self)
  }
}

impl fmt::Display for Comp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Comp::Doc => "doc",
      Comp::Verbatim => "verbatim",
      Comp::Email => "email",
      Comp::Url => "url",
      Comp::Subscript => "subscript",
      Comp::Superscript => "superscript",
      Comp::Para => "para",
      Comp::CodeBlock => "code_block",
      Comp::Imagetext => "imagetext",
      Comp::Linktext => "linktext",
      Comp::Reference => "reference",
      Comp::Destination => "destination",
      Comp::Emph => "emph",
      Comp::Strong => "strong",
      Comp::DoubleQuoted => "double_quoted",
      Comp::ReferenceDefinition => "reference_definition",
      Comp::Insert => "insert",
      Comp::Delete => "delete",
      Comp::Mark => "mark",
    };
    f.write_str(s)
  }
}

impl Default for Comp {
  fn default() -> Self {
    Comp::Para
  }
}
