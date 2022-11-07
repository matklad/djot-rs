use std::rc::Rc;

mod patterns;
mod block;
mod inline;
pub mod ast;

#[derive(Default, Clone)]
pub struct Opts {}

pub type Warn = Rc<dyn Fn()>;

pub type Match = (usize, usize, &'static str);

fn matches_pattern(m: Option<&Match>, pat: &'static str) -> bool {
  let Some(&(_, _, annotation)) = m else { return false; };
  annotation == pat
}

fn plus(a: &'static str) -> &'static str {
  match a {
    "emph" => "+emph",
    "strong" => "+strong",
    _ => panic!("{a}"),
  }
}

fn minus(a: &'static str) -> &'static str {
  match a {
    "emph" => "-emph",
    "strong" => "-strong",
    _ => panic!("{a}"),
  }
}

#[test]
fn smoke() {
  let text = "[![image](img.jpg)](url)";
  let mut p = block::Parser::new(text.to_string(), Opts::default(), None);
  p.parse();
  let mut got = String::new();
  for (s, e, a) in p.matches {
    let m = format!("{a} {}-{}", s + 1, if e == s { e + 1 } else { e });
    got.push_str(&m);
    got.push('\n');
    eprintln!("{m:<20} {:?}", text.get(s..e).unwrap_or_default())
  }

  let sh = xshell::Shell::new().unwrap();
  if !sh.path_exists("ref") {
    xshell::cmd!(sh, "git clone https://github.com/jgm/djot ref").run().unwrap();
  }
  sh.change_dir("ref");
  let want = xshell::cmd!(sh, "lua ./bin/main.lua -m").stdin(text).read().unwrap();
  if want.trim() == got.trim() {
    eprintln!("ok!")
  } else {
    eprintln!("\nERROR:");
    eprintln!("{want}")
  }
}
