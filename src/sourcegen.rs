//! Generates matches and ast structures
mod annot;
mod ast;

use std::path::Path;

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
