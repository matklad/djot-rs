use crate::{
  annot::{Annot, Atom, Comp},
  ast::{
    CodeBlock, Doc, DoubleQuoted, Emph, Image, Link, Para, Softbreak, Str, Strong, Tag, TagKind,
    Verbatim,
  },
  Match,
};

impl crate::block::Parser {
  pub fn to_ast(self) -> Tag {
    Ctx { subject: self.subject, matches: self.matches, idx: 0 }.get_node(Comp::Doc)
  }
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
      Comp::Imagetext => Image { destination: String::new() }.into(),
      Comp::Linktext => Link { destination: String::new() }.into(),
      Comp::CodeBlock => CodeBlock { text: String::new() }.into(),
      Comp::Destination => Doc {}.into(),
      Comp::Strong => Strong {}.into(),
      Comp::Emph => Emph {}.into(),
      Comp::DoubleQuoted => DoubleQuoted {}.into(),
      Comp::Verbatim => Verbatim { text: String::new() }.into(),
      _ => panic!("unhandled {maintag}"),
    });
    while self.idx < self.matches.len() {
      let m = self.matches[self.idx];

      if m.is(Atom::Blankline) || m.is(Atom::ImageMarker) {
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
                let destination = match tag {
                  Comp::Imagetext => &mut result.cast::<Image>().destination,
                  Comp::Linktext => &mut result.cast::<Link>().destination,
                  _ => unreachable!(),
                };
                if self.matches[self.idx].is(Comp::Destination.add()) {
                  self.idx += 1;
                  let dest = self.get_node(Comp::Destination);
                  *destination = get_string_content(&dest);
                }
              }
              Comp::CodeBlock => result.cast::<CodeBlock>().text = get_string_content(&result),
              Comp::Verbatim => result.cast::<Verbatim>().text = get_string_content(&result),
              _ => (),
            }
            node.children.push(result)
          }
          Annot::Sub(_) => panic!("unhandled {}", m.a),
          Annot::Atom(atom) => {
            let tag = match atom {
              Atom::Str => Tag::new(Str::new(&self.subject[m.s..m.e])),
              Atom::Softbreak => Tag::new(Softbreak {}),
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
