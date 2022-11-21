use std::collections::BTreeMap;

use crate::{
  annot::{Annot, Atom, Comp},
  ast::{
    Attrs, CodeBlock, Delete, DoubleQuoted, Emoji, Emph, Image, Insert, Link, Mark, Para,
    ReferenceDefinition, SoftBreak, Span, Str, Strong, Subscript, Superscript, Tag, Url, Verbatim,
  },
  block,
  patterns::find,
  Document, Match,
};

pub(crate) fn build(p: block::Tokenizer) -> Document {
  let mut ctx = Ctx { subject: p.subject, matches: p.matches, idx: 0, references: BTreeMap::new() };
  let mut doc = ctx.get_doc();
  doc.debug = p.debug;
  doc.references = ctx.references;
  doc
}

struct Ctx {
  subject: String,
  matches: Vec<Match>,
  references: BTreeMap<String, ReferenceDefinition>,
  idx: usize,
}

impl Ctx {
  fn get_doc(&mut self) -> Document {
    let mut res = Document::default();
    while self.idx < self.matches.len() {
      self.get_tag(&mut res.children)
    }
    res
  }

  fn get_tag(&mut self, acc: &mut Vec<Tag>) {
    self.skip_trivia();
    let m = self.matches[self.idx].clone();
    self.idx += 1;
    let res = match m.a {
      Annot::Add(comp) => match comp {
        Comp::CodeBlock => Tag::CodeBlock(self.get_code_block()),
        Comp::Para => Tag::Para(self.get_para()),
        Comp::Verbatim => Tag::Verbatim(self.get_verbatim()),
        Comp::Strong => Tag::Strong(self.get_strong()),
        Comp::Emph => Tag::Emph(self.get_emph()),
        Comp::Insert => Tag::Insert(self.get_insert()),
        Comp::Delete => Tag::Delete(self.get_delete()),
        Comp::Mark => Tag::Mark(self.get_mark()),
        Comp::Subscript => Tag::Subscript(self.get_subscript()),
        Comp::Superscript => Tag::Superscript(self.get_superscript()),
        Comp::DoubleQuoted => Tag::DoubleQuoted(self.get_double_quoted()),
        Comp::Linktext => Tag::Link(self.get_link()),
        Comp::Imagetext => Tag::Image(self.get_image()),
        Comp::Url => Tag::Url(self.get_url()),
        Comp::Attributes => todo!(),
        Comp::Span => Tag::Span(self.get_span()),
        Comp::ReferenceDefinition => {
          self.get_reference_definition();
          return;
        }
        _ => todo!("{comp:?}"),
      },
      Annot::Sub(sub) => unreachable!("-{sub}"),
      Annot::Atom(atom) => match atom {
        Atom::Str => {
          let mut text = self.subject[m.range].to_string();
          let attrs = self.get_attrs();
          if !attrs.is_empty() {
            if let Some(idx) = text.rfind(|it: char| it.is_ascii_whitespace()) {
              acc.push(Tag::Str(Str { attrs: Attrs::new(), text: text[..idx + 1].to_string() }));
              text.drain(..idx + 1);
            }
          }
          Tag::Str(Str { attrs, text })
        }
        Atom::Emoji => {
          let mut res = Emoji::default();
          res.alias = self.subject[m.range.start + 1..m.range.end - 1].to_string();
          Tag::Emoji(res)
        }
        Atom::Softbreak => Tag::SoftBreak(SoftBreak::default()),
        Atom::Class | Atom::Id => return,
        _ => todo!("{atom:?}"),
      },
    };
    acc.push(res)
  }

  fn get_code_block(&mut self) -> CodeBlock {
    let mut res = CodeBlock::default();
    let m = self.matches[self.idx].clone();
    if m.is(Atom::CodeLanguage) {
      res.lang = Some(self.subject[m.range].to_string());
      self.idx += 1;
    }
    res.text = self.get_text_until(Comp::CodeBlock);
    res
  }

  fn get_para(&mut self) -> Para {
    let mut res = Para::default();
    res.children = self.get_tags_until(Comp::Para);
    res
  }

  fn get_verbatim(&mut self) -> Verbatim {
    let mut res = Verbatim::default();
    res.text = self.get_text_until(Comp::Verbatim);
    if find(res.text.as_str(), "^ +`").is_match {
      res.text.remove(0);
    }
    if find(res.text.as_str(), "` +$").is_match {
      res.text.pop();
    }
    res
  }

  fn get_strong(&mut self) -> Strong {
    let mut res = Strong::default();
    res.children = self.get_tags_until(Comp::Strong);
    res
  }

  fn get_emph(&mut self) -> Emph {
    let mut res = Emph::default();
    res.children = self.get_tags_until(Comp::Emph);
    res
  }

  fn get_insert(&mut self) -> Insert {
    let mut res = Insert::default();
    res.children = self.get_tags_until(Comp::Insert);
    res
  }

  fn get_delete(&mut self) -> Delete {
    let mut res = Delete::default();
    res.children = self.get_tags_until(Comp::Delete);
    res
  }

  fn get_mark(&mut self) -> Mark {
    let mut res = Mark::default();
    res.children = self.get_tags_until(Comp::Mark);
    res
  }

  fn get_subscript(&mut self) -> Subscript {
    let mut res = Subscript::default();
    res.children = self.get_tags_until(Comp::Subscript);
    res
  }

  fn get_superscript(&mut self) -> Superscript {
    let mut res = Superscript::default();
    res.children = self.get_tags_until(Comp::Superscript);
    res
  }

  fn get_double_quoted(&mut self) -> DoubleQuoted {
    let mut res = DoubleQuoted::default();
    res.children = self.get_tags_until(Comp::DoubleQuoted);
    res
  }

  fn get_link(&mut self) -> Link {
    let mut res = Link::default();
    res.children = self.get_tags_until(Comp::Linktext);
    match self.get_dest() {
      LinkDest::Dest(dest) => res.destination = Some(dest),
      LinkDest::Ref(r) => res.reference = Some(r),
      LinkDest::AutoRef => res.reference = Some(get_string_content(&res.children)),
    }
    res
  }

  fn get_image(&mut self) -> Image {
    let mut res = Image::default();
    res.children = self.get_tags_until(Comp::Imagetext);
    match self.get_dest() {
      LinkDest::Dest(dest) => res.destination = Some(dest),
      LinkDest::Ref(r) => res.reference = Some(r),
      LinkDest::AutoRef => res.reference = Some(get_string_content(&res.children)),
    }
    res
  }

  fn get_dest(&mut self) -> LinkDest {
    let m = self.matches[self.idx].clone();
    self.idx += 1;
    if m.is(Comp::Destination.add()) {
      let dest = self.get_text_until(Comp::Destination);
      LinkDest::Dest(dest.replace('\n', ""))
    } else {
      let r = self.get_text_until(Comp::Reference);
      if r.is_empty() {
        LinkDest::AutoRef
      } else {
        LinkDest::Ref(r.replace('\n', " "))
      }
    }
  }

  fn get_url(&mut self) -> Url {
    let mut res = Url::default();
    res.destination = self.get_text_until(Comp::Url);
    res
  }

  fn get_span(&mut self) -> Span {
    let mut res = Span::default();
    res.children = self.get_tags_until(Comp::Span);
    res.attrs = self.get_attrs();
    res
  }

  fn get_attrs(&mut self) -> Attrs {
    if !self.matches[self.idx].is(Comp::Attributes.add()) {
      return Attrs::new();
    }
    self.idx += 1;
    let mut res = Attrs::new();
    loop {
      let m = self.matches[self.idx].clone();
      self.idx += 1;
      if m.is(Comp::Attributes.sub()) {
        break;
      }
      if m.is(Atom::Class) {
        match res.entry("class".to_string()) {
          indexmap::map::Entry::Occupied(mut it) => {
            it.insert(format!("{} {}", it.get(), &self.subject[m.range.clone()]));
          }
          indexmap::map::Entry::Vacant(it) => {
            it.insert(self.subject[m.range.clone()].to_string());
          }
        }
      }
      if m.is(Atom::Id) {
        res.insert("id".to_string(), self.subject[m.range].to_string());
      }
    }
    res
  }

  fn get_reference_definition(&mut self) {
    let mut res = ReferenceDefinition::default();
    let key = self.matches[self.idx].clone();
    self.idx += 1;
    loop {
      let m = self.matches[self.idx].clone();
      if !m.is(Atom::ReferenceValue) {
        break;
      }
      self.idx += 1;
      res.destination.push_str(&self.subject[m.range]);
    }
    assert!(self.matches[self.idx].is(Comp::ReferenceDefinition.sub()));
    self.idx += 1;
    self.references.insert(self.subject[key.range.start + 1..key.range.end - 1].to_string(), res);
  }

  fn get_tags_until(&mut self, comp: Comp) -> Vec<Tag> {
    let mut res = vec![];
    while !self.matches[self.idx].is(comp.sub()) {
      self.get_tag(&mut res)
    }
    self.idx += 1;
    res
  }

  fn get_text_until(&mut self, comp: Comp) -> String {
    let mut res = String::new();
    loop {
      let m = self.matches[self.idx].clone();
      self.idx += 1;
      if m.is(comp.sub()) {
        break;
      }
      res.push_str(&self.subject[m.range]);
    }
    res
  }

  fn skip_trivia(&mut self) {
    while self.idx < self.matches.len() {
      let m = self.matches[self.idx].clone();
      if !(m.is(Atom::Blankline) || m.is(Atom::ImageMarker) || m.is(Atom::Escape)) {
        break;
      }
      self.idx += 1;
      continue;
    }
  }
}

pub(crate) fn get_string_content(tags: &[Tag]) -> String {
  let mut res = String::new();
  for tag in tags {
    match tag {
      Tag::SoftBreak(_) => res.push('\n'),
      Tag::Str(str) => res.push_str(&str.text),
      Tag::Emph(emph) => res.push_str(&get_string_content(&emph.children)),
      _ => (),
    }
  }
  res
}

enum LinkDest {
  Dest(String),
  Ref(String),
  AutoRef,
}
