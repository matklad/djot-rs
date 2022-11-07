use std::borrow::Cow;

use crate::ast::{Tag, TagKind};

impl Tag {
  pub fn to_html(&self) -> String {
    let mut ctx = Ctx::default();
    ctx.render(self);
    ctx.res
  }
}

#[derive(Default)]
struct Ctx {
  res: String,
}
impl Ctx {
  fn render(&mut self, tag: &Tag) {
    match &tag.kind {
      TagKind::Doc(_doc) => self.render_children(tag),
      TagKind::Heading(_) => todo!(),
      TagKind::Para(_para) => {
        self.render_tag("p", tag);
        self.render_children(tag);
        self.out("</p>");
        self.out("\n")
      }
      TagKind::Str(str) => self.out_escape_html(&str.text),
    }
  }

  fn render_children(&mut self, tag: &Tag) {
    for child in &tag.children {
      self.render(child)
    }
  }

  fn render_tag(&mut self, tag_name: &str, _tag: &Tag) {
    self.out("<");
    self.out(tag_name);
    self.out(">");
  }

  fn out(&mut self, s: &str) {
    self.res.push_str(s)
  }
  fn out_escape_html(&mut self, s: &str) {
    self.res.push_str(s)
  }
}
