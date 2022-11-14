use std::{fs, path::PathBuf};

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
  debug_ast: bool,
  ref_matches: bool,
  parse: djot::ParseOpts,
}

#[test]
fn ref_tests() {
  let opts = TestOpts {
    debug_ast: false,
    ref_matches: true,
    parse: djot::ParseOpts { debug_matches: true },
  };

  let mut last_fail = LastFail::load();
  let sh = xshell::Shell::new().unwrap();
  let mut total = 0;
  for path in sh.read_dir("./tests/data").unwrap() {
    if path.extension().unwrap_or_default() == "test" {
      let file_stem = path.file_stem().unwrap_or_default().to_str().unwrap_or_default();
      let source = fs::read_to_string(&path).unwrap();
      for (i, test_case) in parse_test(source.as_str()).into_iter().enumerate() {
        if last_fail.skip(file_stem, i) {
          continue;
        }
        let mut debug = String::new();
        let doc = djot::Document::parse_opts(opts.parse.clone(), &test_case.djot);
        debug.push_str(&doc.debug);
        if opts.debug_ast {
          debug.push_str(&doc.to_json());
        }
        let got = doc.to_html();
        let want = test_case.html.as_str();
        let ref_html = to_ref_html(&test_case.djot, false);
        if opts.ref_matches {
          debug.push_str(&format!("Ref Matches:\n{}-----", to_ref_html(&test_case.djot, true)));
        }
        if want != ref_html.as_str() {
          panic!(
            "\nReference mismatch in {}\nRef:\n{ref_html}-----\nWant:\n{want}-----\n",
            file_stem
          )
        }
        if got.as_str() != want {
          let mut msg = format!(
            "\nMismatch in {}\nSource:\n{}-----\nWant:\n{want}-----\nGot:\n{got}-----\n",
            file_stem, test_case.djot,
          );
          if !debug.is_empty() {
            msg = format!("{msg}Debug:\n{debug}-----\n")
          }
          panic!("{msg}")
        }
        last_fail.test_ok();
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

  res
}

fn parse_fence(line: &str) -> Option<usize> {
  if line.bytes().all(|it| it == b'`') && line.len() > 0 {
    Some(line.len())
  } else {
    None
  }
}

struct LastFail {
  loaded: Option<(String, usize)>,
  current: Option<(String, usize)>,
}

impl LastFail {
  fn load() -> LastFail {
    let mut loaded = None;
    if let Ok(text) = fs::read_to_string(fail_file()) {
      let (name, pos) = text.split_once(':').unwrap_or_else(|| panic!("bad fail file {text:?}"));
      let idx = pos.parse::<usize>().unwrap_or_else(|_| panic!("bad fail file {text:?}"));
      eprintln!("loaded fail {name}:{idx}");
      loaded = Some((name.to_string(), idx))
    }
    LastFail { loaded, current: None }
  }
  fn skip(&mut self, name: &str, pos: usize) -> bool {
    self.current = Some((name.to_string(), pos));
    if let Some(loaded) = &self.loaded {
      return !(loaded.0 == name && loaded.1 == pos);
    }
    false
  }
  fn test_ok(&mut self) {
    if let Some((name, pos)) = &self.loaded {
      eprintln!("{}:{} is now ok!", name, pos);
      let _ = fs::remove_file(&fail_file());
      self.loaded = None;
    }
    self.current = None
  }
}

impl Drop for LastFail {
  fn drop(&mut self) {
    if let Some((name, pos)) = &self.current {
      eprintln!("saved fail {name}:{pos}");
      let _ = fs::write(fail_file(), format!("{name}:{pos}"));
    }
  }
}

fn fail_file() -> PathBuf {
  PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("fail")
}
