use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Annot {
  Atom(Atom),
  Add(Comp),
  Sub(Comp),
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
  Ellipses,
  Softbreak,
  FootnoteReference,
}

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
      Atom::Ellipses => "ellipses",
      Atom::Softbreak => "softbreak",
      Atom::FootnoteReference => "footnote_reference",
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
    };
    f.write_str(s)
  }
}

impl Default for Comp {
  fn default() -> Self {
    Comp::Para
  }
}
