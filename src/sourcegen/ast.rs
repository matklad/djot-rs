use crate::{format_to, sourcegen::camel_case};

use crate::sourcegen::ensure_content;

const TAGS: &str = "
heading level: u32
para
link destination: Option<String>, reference: Option<String>
image destination: Option<String>, reference: Option<String>
code_block lang: Option<String>, text: String
strong
emph
insert
delete
mark
superscript
subscript
span
double_quoted
url destination: String

reference_definition destination: String
soft_break
em_dash
en_dash
verbatim text: String
str text: String
emoji alias: String
";

#[test]
fn generate_annotations() {
  let (composites, atoms) = TAGS.trim().split_once("\n\n").unwrap();

  let mut buf = format!("use super::Attrs;\n");
  emit_ast_comp(&mut buf, composites);
  emit_ast_atom(&mut buf, atoms);
  emit_ast_tag(&mut buf, composites, atoms);
  ensure_content("src/ast/generated.rs", &buf);
}

fn emit_ast_comp(buf: &mut String, composites: &str) {
  for comp in composites.lines() {
    let (ident, fields) = comp.split_once(" ").unwrap_or((comp, ""));
    let fields = if fields.is_empty() {
      String::new()
    } else {
      fields.split(", ").map(|it| format!("pub {it},\n")).collect::<String>()
    };

    format_to! {buf, "
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct {} {{
  #[serde(skip_serializing_if = \"Attrs::is_empty\")]
  pub attrs: Attrs,
  pub children: Vec<Tag>,
  {fields}
}}
", camel_case(ident)}
  }
}

fn emit_ast_atom(buf: &mut String, atoms: &str) {
  for atom in atoms.lines() {
    let (ident, fields) = atom.split_once(" ").unwrap_or((atom, ""));
    let fields = if fields.is_empty() {
      String::new()
    } else {
      fields.split(", ").map(|it| format!("pub {it},\n")).collect::<String>()
    };
    format_to! {buf, "
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct {} {{
  #[serde(skip_serializing_if = \"Attrs::is_empty\")]
  pub attrs: Attrs,
  {fields}
}}
", camel_case(ident)}
  }
}

fn emit_ast_tag(buf: &mut String, composites: &str, atoms: &str) {
  let mut variants = String::new();
  for comp in composites.lines() {
    let ident = comp.split_once(" ").map_or(comp, |it| it.0);
    let camel = camel_case(ident);
    format_to!(variants, "  {camel}({camel}),\n");
  }
  for atom in atoms.lines() {
    let ident = atom.split_once(" ").map_or(atom, |it| it.0);
    let camel = camel_case(ident);
    format_to!(variants, "  {camel}({camel}),\n");
  }
  format_to!(
    buf,
    "
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = \"tag\", rename_all = \"snake_case\")]
pub enum Tag {{ {variants} }}
"
  )
}
