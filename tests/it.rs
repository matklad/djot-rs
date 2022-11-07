use std::fs;

#[test]
fn spec_test() {
  let opts = djot::Opts { debug_matches: true };
  let only = "fe";

  let sh = xshell::Shell::new().unwrap();
  for path in sh.read_dir("./tests/data").unwrap() {
    if path.extension().unwrap_or_default() == "djot" {
      if !only.is_empty() && !path.to_str().unwrap_or_default().contains(&only) {
        continue;
      }
      if true {
        eprintln!("{}:", path.file_stem().unwrap().to_str().unwrap())
      }
      let source = fs::read_to_string(&path).unwrap();
      let ast = djot::parse_opts(opts.clone(), &source);
      let got = ast.to_html();

      let want_path = path.with_extension("html");
      let want = to_ref_html(&source);
      let want2 = std::fs::read_to_string(&want_path).unwrap_or_default();
      if want2.trim() != want.trim() {
        std::fs::write(&want_path, &want).unwrap()
      }
      if got.trim() != want.trim() {
        panic!("Mismatch in {}\nWant:\n{want}\n---\nGot:\n{got}\n", path.display())
      }
    }
  }
}

fn to_ref_html(source: &str) -> String {
  let sh = xshell::Shell::new().unwrap();
  if !sh.path_exists("ref") {
    xshell::cmd!(sh, "git clone https://github.com/jgm/djot ref").run().unwrap();
  }
  sh.change_dir("ref");
  let html = xshell::cmd!(sh, "lua ./bin/main.lua").stdin(source).read().unwrap();
  html
}
