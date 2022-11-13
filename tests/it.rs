use std::fs;

#[allow(unused)]
fn to_ref_html(source: &str) -> String {
  let sh = xshell::Shell::new().unwrap();
  if !sh.path_exists("ref") {
    xshell::cmd!(sh, "git clone https://github.com/jgm/djot ref").run().unwrap();
  }
  sh.change_dir("ref");
  let mut html = xshell::cmd!(sh, "lua ./bin/main.lua").stdin(source).read().unwrap();
  html.push('\n');
  html
}

#[test]
fn ref_tests() {
  let opts = djot::ParseOpts { debug_matches: false };
  let only = "";

  let sh = xshell::Shell::new().unwrap();
  for path in sh.read_dir("./tests/data").unwrap() {
    if path.extension().unwrap_or_default() == "test" {
      if !only.is_empty() && !path.to_str().unwrap_or_default().contains(&only) {
        continue;
      }
      let source = fs::read_to_string(&path).unwrap();
      for test_case in parse_test(source.as_str()) {
        let ast = djot::parse_opts(opts.clone(), &test_case.djot);
        let got = djot::to_html(&ast);
        let want = test_case.html.as_str();
        let ref_html = to_ref_html(&test_case.djot);
        if want != ref_html.as_str() {
          panic!(
            "Reference mismatch in {}\nRef:\n{ref_html}-----\nWant:\n{want}-----",
            path.display()
          )
        }
        if got.as_str() != want {
          panic!(
            "Mismatch in {}\nSource:\n{source}-----\nWant:\n{want}-----\nGot:\n{got}-----",
            path.display()
          )
        }
      }
    }
  }
}

#[derive(Default)]
struct TestCase {
  djot: String,
  html: String,
}

enum ParseState {
  Init,
  Djot(TestCase, usize),
  Html(TestCase, usize),
}

fn parse_test(source: &str) -> Vec<TestCase> {
  let mut res = Vec::new();
  let mut state = ParseState::Init;
  for line in source.lines() {
    state = match state {
      ParseState::Init => match parse_fence(line) {
        Some(fence) => ParseState::Djot(TestCase::default(), fence),
        None => ParseState::Init,
      },
      ParseState::Djot(mut test_case, test_case_fence) => {
        if line == "." {
          ParseState::Html(test_case, test_case_fence)
        } else {
          test_case.djot.push_str(line);
          test_case.djot.push('\n');
          ParseState::Djot(test_case, test_case_fence)
        }
      }
      ParseState::Html(mut test_case, test_case_fence) => match parse_fence(line) {
        Some(fence) if fence == test_case_fence => {
          res.push(test_case);
          ParseState::Init
        }
        _ => {
          test_case.html.push_str(line);
          test_case.html.push('\n');
          ParseState::Html(test_case, test_case_fence)
        }
      },
    };
  }

  assert!(!res.is_empty(), "empty test case:\n{source}-----");
  res
}

fn parse_fence(line: &str) -> Option<usize> {
  if line.bytes().all(|it| it == b'`') {
    Some(line.len())
  } else {
    None
  }
}
