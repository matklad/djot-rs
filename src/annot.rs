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

impl Comp {
  pub(crate) fn add(self) -> Annot {
    Annot::Add(self)
  }
  pub(crate) fn sub(self) -> Annot {
    Annot::Sub(self)
  }
}

impl Default for Comp {
  fn default() -> Self {
    Comp::Para
  }
}
