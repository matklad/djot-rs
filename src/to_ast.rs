use crate::{
  ast::{CodeBlock, Doc, Image, Link, Para, Str, Strong, Tag, TagKind, Emph},
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
    let mut node = Tag::new(match maintag {
      "doc" => TagKind::Doc(Doc {}),
      "para" => Para {}.into(),
      "imagetext" => Image { destination: String::new() }.into(),
      "linktext" => Link { destination: String::new() }.into(),
      "code_block" => CodeBlock { text: String::new() }.into(),
      "destination" => Doc {}.into(),
      "strong" => Strong {}.into(),
      "emph" => Emph {}.into(),
      _ => panic!("unhandled {maintag}"),
    });
    while self.idx < self.matches.len() {
      let (startpos, endpos, annot) = self.matches[self.idx];
      let (mode, tag) = capture2(annot, "^([-+]?)(.*)");

      if matches!(tag, "blankline" | "image_marker") {
        self.idx += 1;
        continue;
      }

      if mode == "-" && tag == maintag {
        self.idx += 1;
        return node;
      } else {
        if mode == "+" {
          let _startidx = self.idx;
          self.idx += 1;
          let mut result = self.get_node(tag);
          match tag {
            "imagetext" | "linktext" => {
              let destination = match tag {
                "imagetext" => &mut result.cast::<Image>().destination,
                "linktext" => &mut result.cast::<Link>().destination,
                _ => unreachable!(),
              };
              let (_, _, nextannot) = self.matches[self.idx];
              match nextannot {
                "+destination" => {
                  self.idx += 1;
                  let dest = self.get_node("destination");
                  *destination = get_string_content(&dest);
                }
                _ => (),
              }
            }
            "code_block" => {
              result.cast::<CodeBlock>().text = get_string_content(&result);
            }
            _ => (),
          }
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

pub(crate) fn get_string_content(dest: &Tag) -> String {
  let mut res = String::new();
  match &dest.kind {
    TagKind::SoftBreak(_) => res.push('\n'),
    TagKind::Str(str) => res.push_str(&str.text),
    _ => (),
  }
  for c in &dest.children {
    res.push_str(&get_string_content(c))
  }
  res
}
