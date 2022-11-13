use crate::{
  ast::{Attrs, Tag, TagKind},
  to_ast::get_string_content,
  HtmlOpts,
};

pub(crate) fn to_html(opts: &HtmlOpts, tag: &Tag) -> String {
  let mut ctx = Ctx { opts, res: String::new() };
  ctx.render(tag);
  ctx.res
}

struct Ctx<'a> {
  #[allow(unused)]
  opts: &'a HtmlOpts,
  res: String,
}
impl<'a> Ctx<'a> {
  fn render(&mut self, tag: &Tag) {
    match &tag.kind {
      TagKind::Doc(_doc) => self.render_children(tag),
      TagKind::Heading(_) => todo!(),
      TagKind::Para(_para) => {
        self.render_tag("p", &tag.attrs);
        self.render_children(tag);
        self.out("</p>");
        self.out("\n")
      }
      TagKind::Link(image) => {
        let mut attrs = Attrs::new();
        attrs.insert("href".to_string(), image.destination.clone());
        self.render_tag("a", &attrs);
        self.render_children(tag);
        self.out("</a>");
      }
      TagKind::Image(image) => {
        let mut attrs = Attrs::new();
        let alt_text = get_string_content(tag);
        if !alt_text.is_empty() {
          attrs.insert("alt".to_string(), alt_text);
        }
        attrs.insert("src".to_string(), image.destination.clone());
        self.render_tag("img", &attrs)
      }
      TagKind::CodeBlock(code_block) => {
        self.render_tag("pre", &tag.attrs);
        self.render_tag("code", &Attrs::default());
        self.out_escape_html(&code_block.text);
        self.out("</code></pre>");
      }
      TagKind::Strong(_) => {
        self.render_tag("strong", &tag.attrs);
        self.render_children(tag);
        self.out("</strong>");
      }
      TagKind::Emph(_) => {
        self.render_tag("em", &tag.attrs);
        self.render_children(tag);
        self.out("</em>");
      }
      TagKind::SoftBreak(_) => self.out("\n"),
      TagKind::Str(str) => self.out_escape_html(&str.text),
    }
  }

  fn render_children(&mut self, tag: &Tag) {
    for child in &tag.children {
      self.render(child)
    }
  }

  fn render_tag(&mut self, tag_name: &str, attrs: &Attrs) {
    self.out("<");
    self.out(tag_name);
    for (k, v) in attrs {
      self.out(" ");
      self.out(k);
      self.out("=");
      self.out(&format!("{v:?}"));
    }
    self.out(">");
  }

  fn out(&mut self, s: &str) {
    self.res.push_str(s)
  }
  fn out_escape_html(&mut self, s: &str) {
    self.res.push_str(s)
  }
}
