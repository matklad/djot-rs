use std::rc::Rc;

mod patterns;
mod block;
mod inline;
pub mod ast;
mod to_ast;
mod to_html;

pub fn parse(text: &str) -> ast::Tag {
  parse_opts(Opts::default(), text)
}

pub fn parse_opts(opts: Opts, text: &str) -> ast::Tag {
  let mut p = block::Parser::new(text.to_string(), opts, None);
  p.parse();
  p.to_ast()
}

#[derive(Default, Clone)]
pub struct Opts {
  pub debug_matches: bool,
}

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

