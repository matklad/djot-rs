use crate::{
  annot::{Annot, Atom, Comp},
  ast::{
    CodeBlock, Delete, Doc, DoubleQuoted, EmDash, Emoji, Emph, EnDash, Image, Insert, Link, Mark,
    Para, ReferenceDefinition, ReferenceKey, ReferenceValue, Softbreak, Span, Str, Strong,
    Subscript, Superscript, Tag, TagKind, Url, Verbatim,
  },
  block,
  patterns::find,
  Document, Match,
};

pub(crate) fn build(p: block::Parser) -> Document {
  let tag = Ctx { subject: p.subject, matches: p.matches, idx: 0 }.get_node(Comp::Doc);
  Document { children: tag.children, debug: p.debug }
}

struct Ctx {
  subject: String,
  matches: Vec<Match>,
  idx: usize,
}

impl Ctx {
  fn get_node(&mut self, maintag: Comp) -> Tag {
    let mut node = Tag::new(match maintag {
      Comp::Doc => TagKind::Doc(Doc {}),
      Comp::Para => Para {}.into(),
      Comp::Imagetext => Image { destination: None, reference: None }.into(),
      Comp::Linktext => Link { destination: None, reference: None }.into(),
      Comp::CodeBlock => CodeBlock { text: String::new(), lang: None }.into(),
      Comp::Destination => Doc {}.into(),
      Comp::Strong => Strong {}.into(),
      Comp::Emph => Emph {}.into(),
      Comp::Insert => Insert {}.into(),
      Comp::Delete => Delete {}.into(),
      Comp::Mark => Mark {}.into(),
      Comp::Superscript => Superscript {}.into(),
      Comp::Subscript => Subscript {}.into(),
      Comp::DoubleQuoted => DoubleQuoted {}.into(),
      Comp::Verbatim => Verbatim { text: String::new() }.into(),
      Comp::Reference => Span {}.into(),
      Comp::ReferenceDefinition => ReferenceDefinition {}.into(),
      Comp::Url => Url { destination: String::new() }.into(),
      _ => panic!("unhandled {maintag}"),
    });
    while self.idx < self.matches.len() {
      let m = self.matches[self.idx];

      if m.is(Atom::Blankline) || m.is(Atom::ImageMarker) || m.is(Atom::Escape) {
        self.idx += 1;
        continue;
      }

      if m.is(maintag.sub()) {
        self.idx += 1;
        return node;
      } else {
        match m.a {
          Annot::Add(tag) => {
            let _startidx = self.idx;
            self.idx += 1;
            let mut result = self.get_node(tag);
            match tag {
              Comp::Imagetext | Comp::Linktext => {
                if self.matches[self.idx].is(Comp::Destination.add()) {
                  self.idx += 1;
                  let dest = self.get_node(Comp::Destination);

                  let destination = match tag {
                    Comp::Imagetext => &mut result.cast::<Image>().destination,
                    Comp::Linktext => &mut result.cast::<Link>().destination,
                    _ => unreachable!(),
                  };
                  *destination = Some(get_string_content(&dest));
                } else if self.matches[self.idx].is(Comp::Reference.add()) {
                  self.idx += 1;
                  let span = self.get_node(Comp::Reference);

                  let reference = match tag {
                    Comp::Imagetext => &mut result.cast::<Image>().reference,
                    Comp::Linktext => &mut result.cast::<Link>().reference,
                    _ => unreachable!(),
                  };
                  *reference = Some(get_string_content(&span));
                }
              }
              Comp::CodeBlock => result.cast::<CodeBlock>().text = get_string_content(&result),
              Comp::Verbatim => {
                let mut text = get_string_content(&result);
                if find(text.as_str(), "^ +`").is_match {
                  text.remove(0);
                }
                if find(text.as_str(), "` +$").is_match {
                  text.pop();
                }
                result.cast::<Verbatim>().text = text;
              }
              Comp::Url => result.cast::<Url>().destination = get_string_content(&result),
              _ => (),
            }
            node.children.push(result)
          }
          Annot::Sub(_) => panic!("unexpected {}", m.a),
          Annot::Atom(atom) => {
            let tag = match atom {
              Atom::Str => Tag::new(Str::new(&self.subject[m.s..m.e])),
              Atom::Emoji => Tag::new(Emoji::new(&self.subject[m.s + 1..m.e - 1])),
              Atom::ReferenceKey => {
                Tag::new(ReferenceKey { text: self.subject[m.s..m.e].to_string() })
              }
              Atom::ReferenceValue => {
                Tag::new(ReferenceValue { text: self.subject[m.s..m.e].to_string() })
              }
              Atom::Softbreak => Tag::new(Softbreak {}),
              Atom::CodeLanguage => {
                node.cast::<CodeBlock>().lang = Some(self.subject[m.s..m.e].to_string());
                self.idx += 1;
                continue;
              }
              Atom::EmDash => Tag::new(EmDash {}),
              Atom::EnDash => Tag::new(EnDash {}),
              _ => todo!("todo atom: {atom}"),
            };
            node.children.push(tag);
            self.idx += 1;
          }
        }
      }
    }
    node
  }
}

pub(crate) fn get_string_content(dest: &Tag) -> String {
  let mut res = String::new();
  match &dest.kind {
    TagKind::Softbreak(_) => res.push('\n'),
    TagKind::Str(str) => res.push_str(&str.text),
    _ => (),
  }
  for c in &dest.children {
    res.push_str(&get_string_content(c))
  }
  res
}
