use crate::{
  ast::{Doc, Para, Str, Tag, TagKind},
  patterns::capture2,
  Match,
};

impl crate::block::Parser {
  pub fn to_ast(self) -> Tag {
    Ctx { subject: self.subject, matches: self.matches, idx: 0 }.get_node("doc")
  }
}

struct Ctx {
  subject: String,
  matches: Vec<Match>,
  idx: usize,
}

impl Ctx {
  fn get_node(&mut self, maintag: &str) -> Tag {
    eprintln!("maintagc = {:?}", maintag);
    let mut node = Tag::new(match maintag {
      "doc" => TagKind::Doc(Doc {}),
      "para" => Para {}.into(),
      _ => panic!("unhandled {maintag}"),
    });
    while self.idx < self.matches.len() {
      let (startpos, endpos, annot) = self.matches[self.idx];
      let (mode, tag) = capture2(annot, "^([-+]?)(.*)");
      if mode == "-" && tag == maintag {
        self.idx += 1;
        return node;
      } else {
        if mode == "+" {
          let startidx = self.idx;
          self.idx += 1;
          let mut result = self.get_node(tag);
          node.children.push(result)
        } else if mode == "-" {
          panic!("unhandled {annot}")
        } else {
          node.children.push(Tag::new(Str::new(&self.subject[startpos..endpos])));
          self.idx += 1;
        }
      }
    }
    node
  }
}
