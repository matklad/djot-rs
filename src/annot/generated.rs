#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Comp {
  Doc,
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
}
