use std::collections::BTreeMap;

use crate::{
  ast::{self, Attrs, Tag},
  tree::get_string_content,
  Document, HtmlOpts,
};

pub(crate) fn convert(opts: &HtmlOpts, doc: &Document) -> String {
  let refs = &doc.references;
  let mut ctx = Ctx { opts, refs, res: String::new() };
  ctx.render_doc(doc);
  ctx.res
}

struct Ctx<'a> {
  #[allow(unused)]
  opts: &'a HtmlOpts,
  refs: &'a BTreeMap<String, ast::ReferenceDefinition>,
  res: String,
}
impl<'a> Ctx<'a> {
  fn render_doc(&mut self, doc: &Document) {
    for child in &doc.children {
      self.render(child)
    }
  }
  fn render(&mut self, tag: &Tag) {
    match tag {
      Tag::Heading(_) => todo!(),
      Tag::Para(para) => {
        self.render_tag("p", &para.attrs);
        self.render_children(&para.children);
        self.out("</p>");
        self.out("\n")
      }
      Tag::Link(link) => {
        let mut attrs = Attrs::new();
        let dest = self.resolve_reference(link.destination.as_deref(), link.reference.as_deref());
        if let Some(dest) = dest {
          attrs.insert("href".to_string(), dest);
        }
        self.render_tag("a", &attrs);
        self.render_children(&link.children);
        self.out("</a>");
      }
      Tag::Image(image) => {
        let mut attrs = Attrs::new();
        let alt_text = get_string_content(&image.children);
        if !alt_text.is_empty() {
          attrs.insert("alt".to_string(), alt_text);
        }
        let dest = self.resolve_reference(image.destination.as_deref(), image.reference.as_deref());
        if let Some(dest) = dest {
          attrs.insert("src".to_string(), dest);
        }
        self.render_tag("img", &attrs)
      }
      Tag::CodeBlock(code_block) => {
        self.render_tag("pre", &code_block.attrs);
        let mut attrs = Attrs::default();
        if let Some(lang) = &code_block.lang {
          attrs.insert("class".to_string(), format!("language-{lang}"));
        }
        self.render_tag("code", &attrs);
        self.out_escape_html(&code_block.text);
        self.out("</code></pre>\n");
      }
      Tag::Strong(strong) => {
        self.render_tag("strong", &strong.attrs);
        self.render_children(&strong.children);
        self.out("</strong>");
      }
      Tag::Emph(emph) => {
        self.render_tag("em", &emph.attrs);
        self.render_children(&emph.children);
        self.out("</em>");
      }
      Tag::DoubleQuoted(double_quoted) => {
        self.out("&ldquo;");
        self.render_children(&double_quoted.children);
        self.out("&rdquo;");
      }
      Tag::SoftBreak(_) => self.out("\n"),
      Tag::Url(url) => {
        let mut attrs = Attrs::new();
        attrs.insert("href".to_string(), url.destination.clone());
        self.render_tag("a", &attrs);
        self.out_escape_html(&url.destination);
        self.out("</a>");
      }
      Tag::Str(str) => {
        if str.attrs.is_empty() {
          self.out_escape_html(&str.text);
        } else {
          self.render_tag("span", &str.attrs);
          self.out_escape_html(&str.text);
          self.out("</span>")
        }
      }
      Tag::Emoji(emoji) => {
        if let Some(emoji) = crate::emoji::find_emoji(&emoji.alias) {
          self.out(emoji);
        } else {
          self.out(&format!(":{}:", emoji.alias));
        }
      }
      Tag::Verbatim(verbatim) => {
        self.render_tag("code", &verbatim.attrs);
        self.out_escape_html(&verbatim.text);
        self.out("</code>");
      }
      Tag::Span(span) => {
        self.render_tag("span", &span.attrs);
        self.render_children(&span.children);
        self.out("</span>");
      }
      Tag::Insert(insert) => {
        self.render_tag("ins", &insert.attrs);
        self.render_children(&insert.children);
        self.out("</ins>");
      }
      Tag::Delete(delete) => {
        self.render_tag("del", &delete.attrs);
        self.render_children(&delete.children);
        self.out("</del>");
      }
      Tag::Mark(mark) => {
        self.render_tag("mark", &mark.attrs);
        self.render_children(&mark.children);
        self.out("</mark>");
      }
      Tag::Superscript(superscript) => {
        self.render_tag("sup", &superscript.attrs);
        self.render_children(&superscript.children);
        self.out("</sup>");
      }
      Tag::Subscript(subscript) => {
        self.render_tag("sub", &subscript.attrs);
        self.render_children(&subscript.children);
        self.out("</sub>");
      }
      Tag::EmDash(_) => self.out("&mdash;"),
      Tag::EnDash(_) => self.out("&ndash;"),
    }
  }

  fn render_children(&mut self, children: &[Tag]) {
    for child in children {
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

  fn resolve_reference(
    &self,
    destination: Option<&str>,
    reference: Option<&str>,
  ) -> Option<String> {
    if let Some(destination) = destination {
      return Some(destination.to_string());
    }
    if let Some(reference) = reference {
      if let Some(reference_definition) = self.refs.get(reference) {
        return Some(reference_definition.destination.clone());
      }
    }
    None
  }

  fn out(&mut self, s: &str) {
    self.res.push_str(s)
  }
  fn out_escape_html(&mut self, s: &str) {
    self.res.push_str(s)
  }
}
