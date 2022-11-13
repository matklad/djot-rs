use std::fs;

#[allow(unused)]
fn to_ref_html(source: &str, matches: bool) -> String {
  let sh = xshell::Shell::new().unwrap();
  if !sh.path_exists("ref") {
    xshell::cmd!(sh, "git clone https://github.com/jgm/djot ref").run().unwrap();
  }
  sh.change_dir("ref");
  let matches = if matches { Some("-m") } else { None };
  let mut html = xshell::cmd!(sh, "lua ./bin/main.lua {matches...}").stdin(source).read().unwrap();
  html.push('\n');
  html
}

struct TestOpts {
  only: &'static str,
  only_nth: usize,
  debug_ast: bool,
  ref_matches: bool,
  parse: djot::ParseOpts,
}

#[test]
fn ref_tests() {
  let opts = TestOpts {
    only: "",
    only_nth: !0,
    debug_ast: false,
    ref_matches: true,
    parse: djot::ParseOpts { debug_matches: true },
  };

  let sh = xshell::Shell::new().unwrap();
  let mut total = 0;
  for path in sh.read_dir("./tests/data").unwrap() {
    if path.extension().unwrap_or_default() == "test" {
      if !opts.only.is_empty() && !path.to_str().unwrap_or_default().contains(&opts.only) {
        continue;
      }
      let source = fs::read_to_string(&path).unwrap();
      for (i, test_case) in parse_test(source.as_str()).into_iter().enumerate() {
        if !opts.only.is_empty() && opts.only_nth != !0 && i != opts.only_nth {
          continue;
        }
        let mut debug = String::new();
        let parse = djot::parse_opts(opts.parse.clone(), &test_case.djot);
        debug.push_str(&parse.debug);
        if opts.debug_ast {
          debug.push_str(&parse.ast.to_json());
        }
        let got = djot::to_html(&parse.ast);
        let want = test_case.html.as_str();
        let ref_html = to_ref_html(&test_case.djot, false);
        if opts.ref_matches {
          debug.push_str(&format!("Ref Matches:\n{}-----", to_ref_html(&test_case.djot, true)));
        }
        if want != ref_html.as_str() {
          panic!(
            "\nReference mismatch in {}\nRef:\n{ref_html}-----\nWant:\n{want}-----\n",
            path.display()
          )
        }
        if got.as_str() != want {
          let mut msg = format!(
            "\nMismatch in {}\nSource:\n{}-----\nWant:\n{want}-----\nGot:\n{got}-----\n",
            path.display(),
            test_case.djot,
          );
          if !debug.is_empty() {
            msg = format!("{msg}Debug:\n{debug}-----\n")
          }
          panic!("{msg}")
        }
        total += 1;
      }
    }
  }
  eprintln!("total tests: {total}");
}

#[derive(Debug, Default)]
struct TestCase {
  djot: String,
  html: String,
}

#[derive(Debug)]
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
      ParseState::Init if line == "STOP" => {
        break;
      }
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
  if line.bytes().all(|it| it == b'`') && line.len() > 0 {
    Some(line.len())
  } else {
    None
  }
}
