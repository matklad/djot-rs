use crate::{
  format_to,
  sourcegen::{camel_case, ensure_content},
};

const ANNOTATIONS: &str = "
verbatim
email
url
subscript
superscript
para
code_block
imagetext
linktext
reference
destination
emph
strong
span
double_quoted
reference_definition
insert
delete
mark
attributes

str
escape
hardbreak
nbsp
blankline
image_marker
left_double_quote
right_double_quote
ellipses
softbreak
footnote_reference
open_marker
emoji
reference_key
reference_value
code_language
em_dash
en_dash
id
key
value
class
";

#[test]
fn generate_annotations() {
  let (composites, atoms) = ANNOTATIONS.trim().split_once("\n\n").unwrap();

  let mut buf = "\
use std::fmt;
"
  .to_string();

  emit_comp(&mut buf, composites);
  emit_atom(&mut buf, atoms);
  ensure_content("src/annot/generated.rs", &buf);
}

fn emit_comp(buf: &mut String, composites: &str) {
  format_to!(
    buf,
    "\
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Comp {{
"
  );
  for ident in composites.lines() {
    format_to!(buf, "  {},\n", camel_case(ident))
  }
  format_to!(buf, "}}\n");

  let mut display_arms = String::new();
  for ident in composites.lines() {
    format_to!(display_arms, "      Comp::{} => \"{ident}\",\n", camel_case(ident))
  }

  format_to!(
    buf,
    "
impl fmt::Display for Comp {{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
    f.write_str(match self {{
      {display_arms}
    }})
  }}
}}
    "
  );
}

fn emit_atom(buf: &mut String, atoms: &str) {
  let mut variants = String::new();
  for ident in atoms.lines() {
    format_to!(variants, "  {},\n", camel_case(ident))
  }

  format_to!(
    buf,
    "
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Atom {{
  {variants}
}}
"
  );

  let mut left_atoms = String::new();
  let mut right_atoms = String::new();
  let mut ltr = String::new();
  let mut rtl = String::new();
  for ident in atoms.lines() {
    if ident.starts_with("left_") {
      format_to!(left_atoms, " | Atom::{}", camel_case(ident));
      let rident = &ident.replace("left", "right");
      format_to!(ltr, "Atom::{} => Atom::{},\n", camel_case(ident), camel_case(rident));
      format_to!(rtl, "Atom::{} => Atom::{},\n", camel_case(rident), camel_case(ident));
    }
    if ident.starts_with("right_") {
      format_to!(right_atoms, " | Atom::{}", camel_case(ident))
    }
  }

  format_to!(
    buf,
    "
impl Atom {{
  pub(crate) fn is_left_atom(self) -> bool {{
    matches!(self, {left_atoms})
  }}
  pub(crate) fn is_right_atom(self) -> bool {{
    matches!(self, {right_atoms})
  }}
  pub(crate) fn corresponding_left_atom(self) -> Atom {{
    match self {{
      {rtl}
      _ => self
    }}
  }}
  pub(crate) fn corresponding_right_atom(self) -> Atom {{
    match self {{
      {ltr}
      _ => self
    }}
  }}
}}
"
  );

  let mut display_arms = String::new();
  for ident in atoms.lines() {
    format_to!(display_arms, "      Atom::{} => \"{ident}\",\n", camel_case(ident))
  }

  format_to!(
    buf,
    "
impl fmt::Display for Atom {{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
    f.write_str(match self {{
      {display_arms}
    }})
  }}
}}
    "
  );
}
