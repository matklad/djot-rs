use std::path::PathBuf;

use anyhow::Context;
use lexopt::{Arg::Long, Arg::Short, Arg::Value};

fn main() -> anyhow::Result<()> {
  let mut matches = false;
  let mut ast = false;
  let mut files = Vec::new();

  let mut parser = lexopt::Parser::from_env();
  while let Some(arg) = parser.next()? {
    match arg {
      Short('m') | Long("matches") => matches = true,
      Short('a') | Long("ast") => ast = true,
      Value(val) => files.push(val),
      _ => Err(arg.unexpected())?,
    }
  }

  let mut inputs = Vec::new();
  if files.is_empty() {
    let content = std::io::read_to_string(std::io::stdin()).context("failed to read stdin")?;
    inputs.push(content)
  } else {
    for file in files {
      let path = PathBuf::from(file);
      let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
      inputs.push(content)
    }
  }

  let opts = djot::ParseOpts { debug_matches: matches };
  for content in inputs {
    let doc = djot::Document::parse_opts(opts.clone(), &content);
    if matches {
      println!("{}", doc.debug)
    } else if ast {
      println!("{}", doc.to_json())
    } else {
      println!("{}", doc.to_html())
    }
  }

  Ok(())
}
