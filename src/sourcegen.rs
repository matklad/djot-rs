//! Generates matches and ast structures
use std::path::Path;

use crate::format_to;

const ANNOTATIONS: &str = "
doc
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
double_quoted
reference_definition
insert
delete
mark

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
";

#[test]
fn generate_annotations() {
  let (composites, atoms) = ANNOTATIONS.trim().split_once("\n\n").unwrap();
  let mut buf = String::new();
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
}

fn emit_atom(buf: &mut String, atoms: &str) {
  let mut left_atoms = String::new();
  let mut right_atoms = String::new();
  for ident in atoms.lines() {
    if ident.starts_with("left_") {
      format_to!(left_atoms, " | Atom::{}", camel_case(ident))
    }
    if ident.starts_with("right_") {
      format_to!(right_atoms, " | Atom::{}", camel_case(ident))
    }
  }

  format_to!(
    buf,
    "
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Atom {{
  "
  );
  for ident in atoms.lines() {
    format_to!(buf, "  {},\n", camel_case(ident))
  }
  format_to!(buf, "}}\n");
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
}}
"
  );
}

fn camel_case(ident: &str) -> String {
  ident
    .split('_')
    .flat_map(|word| {
      word.chars().next().map(|it| it.to_ascii_uppercase()).into_iter().chain(word.chars().skip(1))
    })
    .collect()
}

fn ensure_content(path: &str, content: &str) {
  let base = Path::new(env!("CARGO_MANIFEST_DIR"));
  let path = base.join(path);
  let old = std::fs::read_to_string(&path).unwrap_or_default();
  if normalize(&old) == normalize(content) {
    return;
  }
  std::fs::write(&path, content)
    .unwrap_or_else(|err| panic!("can't write {}: {err}", path.display()));
}

fn normalize(s: &str) -> String {
  s.split_ascii_whitespace().flat_map(|it| it.split(',')).collect()
}
