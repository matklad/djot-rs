use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Comp {
  Verbatim,
  Email,
  Url,
  Subscript,
  Superscript,
  Para,
  CodeBlock,
  Imagetext,
  Linktext,
  Reference,
  Destination,
  Emph,
  Strong,
  DoubleQuoted,
  ReferenceDefinition,
  Insert,
  Delete,
  Mark,
}

impl fmt::Display for Comp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
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
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Atom {
  Str,
  Escape,
  Hardbreak,
  Nbsp,
  Blankline,
  ImageMarker,
  LeftDoubleQuote,
  RightDoubleQuote,
  Ellipses,
  Softbreak,
  FootnoteReference,
  OpenMarker,
  Emoji,
  ReferenceKey,
  ReferenceValue,
  CodeLanguage,
  EmDash,
  EnDash,
}

impl Atom {
  pub(crate) fn is_left_atom(self) -> bool {
    matches!(self,  | Atom::LeftDoubleQuote)
  }
  pub(crate) fn is_right_atom(self) -> bool {
    matches!(self,  | Atom::RightDoubleQuote)
  }
  pub(crate) fn corresponding_left_atom(self) -> Atom {
    match self {
      Atom::RightDoubleQuote => Atom::LeftDoubleQuote,

      _ => self,
    }
  }
  pub(crate) fn corresponding_right_atom(self) -> Atom {
    match self {
      Atom::LeftDoubleQuote => Atom::RightDoubleQuote,

      _ => self,
    }
  }
}

impl fmt::Display for Atom {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
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
    })
  }
}
