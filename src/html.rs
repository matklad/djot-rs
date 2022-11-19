use std::collections::BTreeMap;

use crate::{
  ast::{self, Attrs, Tag, TagKind},
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
  refs: &'a BTreeMap<String, ast::Tag>,
  res: String,
}
impl<'a> Ctx<'a> {
  fn render_doc(&mut self, doc: &Document) {
    for child in &doc.children {
      self.render(child)
    }
  }
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
      TagKind::Link(link) => {
        let mut attrs = Attrs::new();
        let dest = self.resolve_reference(link.destination.as_deref(), link.reference.as_deref());
        if let Some(dest) = dest {
          attrs.insert("href".to_string(), dest);
        }
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
        let dest = self.resolve_reference(image.destination.as_deref(), image.reference.as_deref());
        if let Some(dest) = dest {
          attrs.insert("src".to_string(), dest);
        }
        self.render_tag("img", &attrs)
      }
      TagKind::CodeBlock(code_block) => {
        self.render_tag("pre", &tag.attrs);
        let mut attrs = Attrs::default();
        if let Some(lang) = &code_block.lang {
          attrs.insert("class".to_string(), format!("language-{lang}"));
        }
        self.render_tag("code", &attrs);
        self.out_escape_html(&code_block.text);
        self.out("</code></pre>\n");
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
      TagKind::DoubleQuoted(_) => {
        self.out("&ldquo;");
        self.render_children(tag);
        self.out("&rdquo;");
      }
      TagKind::Softbreak(_) => self.out("\n"),
      TagKind::Url(url) => {
        let mut attrs = Attrs::new();
        attrs.insert("href".to_string(), url.destination.clone());
        self.render_tag("a", &attrs);
        self.render_children(tag);
        self.out("</a>");
      }
      TagKind::Str(str) => self.out_escape_html(&str.text),
      TagKind::Emoji(emoji) => {
        if let Some(emoji) = crate::emoji::find_emoji(&emoji.text) {
          self.out(emoji);
        } else {
          self.out(&format!(":{}:", emoji.text));
        }
      }
      TagKind::Verbatim(verbatim) => {
        self.render_tag("code", &tag.attrs);
        self.out_escape_html(&verbatim.text);
        self.out("</code>");
      }
      TagKind::Span(_) => {
        self.render_tag("span", &tag.attrs);
        self.render_children(tag);
        self.out("</span>");
      }
      TagKind::Insert(_) => {
        self.render_tag("ins", &tag.attrs);
        self.render_children(tag);
        self.out("</ins>");
      }
      TagKind::Delete(_) => {
        self.render_tag("del", &tag.attrs);
        self.render_children(tag);
        self.out("</del>");
      }
      TagKind::Mark(_) => {
        self.render_tag("mark", &tag.attrs);
        self.render_children(tag);
        self.out("</mark>");
      }
      TagKind::Superscript(_) => {
        self.render_tag("sup", &tag.attrs);
        self.render_children(tag);
        self.out("</sup>");
      }
      TagKind::Subscript(_) => {
        self.render_tag("sub", &tag.attrs);
        self.render_children(tag);
        self.out("</sub>");
      }
      TagKind::EmDash(_) => self.out("&mdash;"),
      TagKind::EnDash(_) => self.out("&ndash;"),
      TagKind::ReferenceDefinition(_) | TagKind::ReferenceKey(_) | TagKind::ReferenceValue(_) => (),
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
        if let ast::TagKind::ReferenceDefinition(reference_definition) = &reference_definition.kind
        {
          return Some(reference_definition.destination.clone());
        }
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
