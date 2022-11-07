use std::fs;

#[test]
fn spec_test() {
  let sh = xshell::Shell::new().unwrap();
  for path in sh.read_dir("./tests/data").unwrap() {
    if path.extension().unwrap_or_default() == "djot" {
      let source = fs::read_to_string(&path).unwrap();
      let ast = djot::parse(&source);
      let got = ast.to_html();

      let want_path = path.with_extension("html");
      let want = fs::read_to_string(&want_path).unwrap_or_default();
      if got != want {
        fs::write(&want_path, &got).unwrap();
        panic!("mismatch\nWant:\n{want}\n---\nGot:\n{got}\n")
      }
    }
  }
}
