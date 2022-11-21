//! This is a Rust binding to [Lua string patterns](https://www.lua.org/pil/20.2.html),
//! using the original code from Lua 5.2.
//!
//! Although not regular expressions (they lack alternation) they are a powerful
//! and lightweight way to process text. Please note that they are not
//! UTF-8-aware, and in fact can process arbitrary binary data.
//!
//! `LuaPattern` can be created from a string _or_ a byte slice, and has
//! methods which are similar to the original Lua API. Please see
//! [the README](https://github.com/stevedonovan/lua-patterns/blob/master/readme.md)
//! for more discussion.
//!
//! [LuaPattern](struct.LuaPattern.html) implements the public API.
//!
//! ## Examples
//!
//! ```rust
//! extern crate lua_patterns;
//! let mut m = lua_patterns::LuaPattern::new("one");
//! let text = "hello one two";
//! assert!(m.matches(text));
//! let r = m.range();
//! assert_eq!(r.start, 6);
//! assert_eq!(r.end, 9);
//! ```
//!
//! Collecting captures from a match:
//!
//! ```rust
//! extern crate lua_patterns;
//! let text = "  hello one";
//! let mut m = lua_patterns::LuaPattern::new("(%S+) one");
//!
//! // allocates a vector of captures
//! let v = m.captures(text);
//! assert_eq!(v, &["hello one","hello"]);
//! let mut v = Vec::new();
//! // writes captures into preallocated vector
//! if m.capture_into(text,&mut v) {
//!     assert_eq!(v, &["hello one","hello"]);
//! }
//! ```

use std::ops;

pub mod errors;
use errors::*;
mod luapat;
use luapat::*;


/// Represents a Lua string pattern and the results of a match
pub struct LuaPattern<'a> {
    patt: &'a [u8],
    matches: [LuaMatch; LUA_MAXCAPTURES],
    n_match: usize
}

impl <'a> LuaPattern<'a> {
    /// Maybe create a new Lua pattern from a slice of bytes
    pub fn from_bytes_try (bytes: &'a [u8]) -> Result<LuaPattern<'a>,PatternError> {
        str_check(bytes)?;
        let matches = [LuaMatch{start: 0, end: 0}; LUA_MAXCAPTURES];
        Ok(LuaPattern{patt: bytes, matches: matches, n_match: 0})
    }

    /// Maybe create a new Lua pattern from a string
    pub fn new_try(patt: &'a str) -> Result<LuaPattern<'a>,PatternError> {
        LuaPattern::from_bytes_try(patt.as_bytes())
    }

    /// Create a new Lua pattern from a string, panicking if bad
    pub fn new(patt: &'a str) -> LuaPattern<'a> {
        LuaPattern::new_try(patt).expect("bad pattern")
    }

    /// Create a new Lua pattern from a slice of bytes, panicking if bad
    pub fn from_bytes (bytes: &'a [u8]) -> LuaPattern<'a> {
        LuaPattern::from_bytes_try(bytes).expect("bad pattern")
    }

    /// Match a slice of bytes with a pattern
    ///
    /// ```
    /// let patt = &[0xFE,0xEE,b'+',0xED];
    /// let mut m = lua_patterns::LuaPattern::from_bytes(patt);
    /// let bytes = &[0x00,0x01,0xFE,0xEE,0xEE,0xED,0xEF];
    /// assert!(m.matches_bytes(bytes));
    /// assert_eq!(&bytes[m.range()], &[0xFE,0xEE,0xEE,0xED]);
    /// ```
    pub fn matches_bytes(&mut self, s: &[u8]) -> bool {
        self.n_match = str_match(s,self.patt,&mut self.matches).expect("Should not fail - report as bug");
        self.n_match > 0
    }

    /// Match a string with a pattern
    ///
    /// ```
    /// let mut m = lua_patterns::LuaPattern::new("(%a+) one");
    /// let text = " hello one two";
    /// assert!(m.matches(text));
    /// ```
    pub fn matches(&mut self, text: &str) -> bool {
        self.matches_bytes(text.as_bytes())
    }

    /// Match a string, returning first capture if successful
    ///
    /// ```
    /// let mut m = lua_patterns::LuaPattern::new("OK%s+(%d+)");
    /// let res = m.match_maybe("and that's OK 400 to you");
    /// assert_eq!(res, Some("400"));
    /// ```
    pub fn match_maybe<'t>(&mut self, text: &'t str) -> Option<&'t str> {
        if self.matches(text) {
            Some(&text[self.first_capture()])
        } else {
            None
        }
    }

    /// Match a string, returning first two explicit captures if successful
    ///
    /// ```
    /// let mut p = lua_patterns::LuaPattern::new("%s*(%d+)%s+(%S+)");
    /// let (int,rest) = p.match_maybe_2(" 233   hello dolly").unwrap();
    /// assert_eq!(int,"233");
    /// assert_eq!(rest,"hello");
    /// ```
    pub fn match_maybe_2<'t>(&mut self, text: &'t str) -> Option<(&'t str,&'t str)> {
        if self.matches(text) {
            let cc = self.match_captures(text);
            if cc.num_matches() != 3 { return None; }
            Some((cc.get(1),cc.get(2)))
        } else {
            None
        }
    }

    /// Match a string, returning first three explicit captures if successful
    ///
    /// ```
    /// let mut p = lua_patterns::LuaPattern::new("(%d+)/(%d+)/(%d+)");
    /// let (y,m,d) = p.match_maybe_3("2017/11/10").unwrap();
    /// assert_eq!(y,"2017");
    /// assert_eq!(m,"11");
    /// assert_eq!(d,"10");
    /// ```
    pub fn match_maybe_3<'t>(&mut self, text: &'t str) -> Option<(&'t str,&'t str,&'t str)> {
        if self.matches(text) {
            let cc = self.match_captures(text);
            if cc.num_matches() != 4 { return None; }
            Some((cc.get(1),cc.get(2),cc.get(3)))
        } else {
            None
        }
    }

    /// Match and collect all captures as a vector of string slices
    ///
    /// ```
    /// let mut m = lua_patterns::LuaPattern::new("(one).+");
    /// assert_eq!(m.captures(" one two"), &["one two","one"]);
    /// ```
    pub fn captures<'b>(&mut self, text: &'b str) -> Vec<&'b str> {
        let mut res = Vec::new();
        self.capture_into(text, &mut res);
        res
    }

    /// A convenient way to access the captures with no allocation
    ///
    /// ```rust
    /// let text = "  hello one";
    /// let mut m = lua_patterns::LuaPattern::new("(%S+) one");
    /// if m.matches(text) {
    ///     let cc = m.match_captures(text);
    ///     assert_eq!(cc.get(0), "hello one");
    ///     assert_eq!(cc.get(1), "hello");
    /// }
    /// ```
    pub fn match_captures<'b,'c>(&'c self, text: &'b str) -> Captures<'a,'b,'c> {
        Captures {m: self, text: text}
    }

    /// Match and collect all captures into the provided vector.
    ///
    /// ```rust
    /// let text = "  hello one";
    /// let mut m = lua_patterns::LuaPattern::new("(%S+) one");
    /// let mut v = Vec::new();
    /// if m.capture_into(text,&mut v) {
    ///     assert_eq!(v, &["hello one","hello"]);
    /// }
    /// ```
    pub fn capture_into<'b>(&mut self, text: &'b str, vec: &mut Vec<&'b str>) -> bool {
        self.matches(text);
        vec.clear();
        for i in 0..self.n_match {
            vec.push(&text[self.capture(i)]);
        }
        self.n_match > 0
    }

    /// The full match (same as `capture(0)`)
    pub fn range(&self) -> ops::Range<usize> {
        self.capture(0)
    }

    /// Get the nth capture of the match.
    ///
    /// ```
    /// let mut m = lua_patterns::LuaPattern::new("(%a+) one");
    /// let text = " hello one two";
    /// assert!(m.matches(text));
    /// assert_eq!(m.capture(0),1..10);
    /// assert_eq!(m.capture(1),1..6);
    /// ```
    pub fn capture(&self, i: usize) -> ops::Range<usize> {
        ops::Range{
            start: self.matches[i].start as usize,
            end: self.matches[i].end as usize
        }
    }

    /// Get the 'first' capture of the match
    ///
    /// If there are no matches, this is the same as `range`,
    /// otherwise it's `capture(1)`
    pub fn first_capture(&self) -> ops::Range<usize> {
        let idx = if self.n_match > 1 {1} else {0};
        self.capture(idx)
    }

    /// An iterator over all matches in a string.
    ///
    /// The matches are returned as string slices; if there are no
    /// captures the full match is used, otherwise the first capture.
    /// That is, this example will also work with the pattern "(%S+)".
    ///
    /// ```
    /// let mut m = lua_patterns::LuaPattern::new("%S+");
    /// let split: Vec<_> = m.gmatch("dog  cat leopard wolf").collect();
    /// assert_eq!(split,&["dog","cat","leopard","wolf"]);
    /// ```
    pub fn gmatch<'b,'c>(&'c mut self, text: &'b str) -> GMatch<'a,'b,'c> {
        GMatch{m: self, text: text}
    }

    /// An iterator over all captures in a string.
    ///
    /// The matches are returned as captures; this is a _streaming_
    /// iterator, so don't try to collect the captures directly; extract
    /// the string slices using `get`.
    ///
    /// ```
    /// let mut m = lua_patterns::LuaPattern::new("(%S)%S+");
    /// let split: Vec<_> = m.gmatch_captures("dog  cat leopard wolf")
    ///       .map(|cc| cc.get(1)).collect();
    /// assert_eq!(split,&["d","c","l","w"]);
    /// ```
    pub fn gmatch_captures<'b,'c>(&'c mut self, text: &'b str) -> GMatchCaptures<'a,'b,'c> {
        GMatchCaptures{m: self, text: text}
    }

    /// An iterator over all matches in a slice of bytes.
    ///
    /// ```
    /// let bytes = &[0xAA,0x01,0x01,0x03,0xBB,0x01,0x01,0x01];
    /// let patt = &[0x01,b'+'];
    /// let mut m = lua_patterns::LuaPattern::from_bytes(patt);
    /// let mut iter = m.gmatch_bytes(bytes);
    /// assert_eq!(iter.next().unwrap(), &[0x01,0x01]);
    /// assert_eq!(iter.next().unwrap(), &[0x01,0x01,0x01]);
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn gmatch_bytes<'b>(&'a mut self, bytes: &'b [u8]) -> GMatchBytes<'a,'b> {
        GMatchBytes{m: self, bytes: bytes}
    }

    /// Globally substitute all matches with a replacement
    /// provided by a function of the captures.
    ///
    /// ```
    /// let mut m = lua_patterns::LuaPattern::new("%$(%S+)");
    /// let res = m.gsub_with("hello $dolly you're so $fine!",
    ///     |cc| cc.get(1).to_uppercase()
    /// );
    /// assert_eq!(res, "hello DOLLY you're so FINE!");
    /// ```
    pub fn gsub_with <F> (&mut self, text: &str, lookup: F) -> String
    where F: Fn(Captures)-> String {
        let mut slice = text;
        let mut res = String::new();
        while self.matches(slice) {
            // full range of match
            let all = self.range();
            // append everything up to match
            res.push_str(&slice[0..all.start]);
            let captures = Captures{m: self, text: slice};
            let repl = lookup(captures);
            res.push_str(&repl);
            slice = &slice[all.end..];
        }
        res.push_str(slice);
        res
    }

    /// Globally substitute all matches with a replacement string
    ///
    /// This string _may_ have capture references ("%0",..). Use "%%"
    /// to represent "%". Plain strings like "" work just fine ;)
    ///
    /// ```
    /// let mut m = lua_patterns::LuaPattern::new("(%S+)%s*=%s*(%S+);%s*");
    /// let res = m.gsub("a=2; b=3; c = 4;", "'%2':%1 ");
    /// assert_eq!(res,"'2':a '3':b '4':c ");
    /// ```
    pub fn gsub (&mut self, text: &str, repl: &str) -> String {
        let repl = generate_gsub_patterns(repl);
        let mut slice = text;
        let mut res = String::new();
        while self.matches(slice) {
            let all = self.range();
            res.push_str(&slice[0..all.start]);
            let captures = Captures{m: self, text: slice};
            for r in &repl {
                match *r {
                    Subst::Text(ref s) => res.push_str(&s),
                    Subst::Capture(i) => res.push_str(captures.get(i))
                }
            }
            slice = &slice[all.end..];
        }
        res.push_str(slice);
        res
    }

    /// Globally substitute all _byte_ matches with a replacement
    /// provided by a function of the captures.
    ///
    /// ```
    /// let bytes = &[0xAA,0x01,0x02,0x03,0xBB];
    /// let patt = &[0x01,0x02];
    /// let mut m = lua_patterns::LuaPattern::from_bytes(patt);
    /// let res = m.gsub_bytes_with(bytes,|cc| vec![0xFF]);
    /// assert_eq!(res, &[0xAA,0xFF,0x03,0xBB]);
    /// ```
    pub fn gsub_bytes_with <F> (&mut self, bytes: &[u8], lookup: F) -> Vec<u8>
    where F: Fn(ByteCaptures)-> Vec<u8> {
        let mut slice = bytes;
        let mut res = Vec::new();
        while self.matches_bytes(slice) {
            let all = self.range();
            let capture = &slice[0..all.start];
            res.extend_from_slice(capture);
            let captures = ByteCaptures{m: self, bytes: slice};
            let repl = lookup(captures);
            res.extend(repl);
            slice = &slice[all.end..];
        }
        res.extend_from_slice(slice);
        res
    }

}

#[derive(Debug)]
pub enum Subst {
    Text(String),
    Capture(usize)
}

impl Subst {
    fn new_text(text: &str) -> Subst {
        Subst::Text(text.to_string())
    }
}

pub fn generate_gsub_patterns(repl: &str) -> Vec<Subst> {
    let mut m = LuaPattern::new("%%([%%%d])");
    let mut res = Vec::new();
    let mut slice = repl;
    while m.matches(slice) {
        let all = m.range();
        let before = &slice[0..all.start];
        if before != "" {
            res.push(Subst::new_text(before));
        }
        let capture = &slice[m.capture(1)];
        if capture == "%" { // escaped literal '%'
            res.push(Subst::new_text("%"));
        } else { // has to be a digit
            let index: usize = capture.parse().unwrap();
            res.push(Subst::Capture(index));
        }
        slice = &slice[all.end..];
    }
    res.push(Subst::new_text(slice));
    res
}

pub struct Substitute {
    repl: Vec<Subst>
}

impl Substitute {
    pub fn new(repl: &str) -> Substitute {
        Substitute{
            repl: generate_gsub_patterns(repl)
        }
    }

    pub fn subst(&self, patt: &LuaPattern, text: &str) -> String {
        let mut res = String::new();
        let captures = patt.match_captures(text);
        for r in &self.repl {
            match *r {
                Subst::Text(ref s) => res.push_str(&s),
                Subst::Capture(i) => res.push_str(captures.get(i))
            }
        }
        res
    }

}



/// Low-overhead convenient access to string match captures
// note: there are three borrows going on here.
// The lifetime 'a is for the _pattern_, the lifetime 'b is
// for the _source string_, and 'c is for the reference to LuaPattern
// And the LuaPattern reference cannot live longer than the pattern reference
pub struct Captures<'a,'b,'c> where 'a: 'c {
    m: &'c LuaPattern<'a>,
    text: &'b str
}

impl <'a,'b,'c> Captures<'a,'b,'c> {
    /// get the capture as a string slice
    pub fn get(&self, i: usize) -> &'b str {
        &self.text[self.m.capture(i)]
    }

    /// number of matches
    pub fn num_matches(&self) -> usize {
        self.m.n_match
    }
}


/// Low-overhead convenient access to byte match captures
pub struct ByteCaptures<'a,'b> {
    m: &'a LuaPattern<'a>,
    bytes: &'b [u8]
}

impl <'a,'b> ByteCaptures<'a,'b> {
    /// get the capture as a byte slice
    pub fn get(&self, i: usize) -> &'b [u8] {
        &self.bytes[self.m.capture(i)]
    }

    /// number of matches
    pub fn num_matches(&self) -> usize {
        self.m.n_match
    }
}

/// Iterator for all string slices from `gmatch`
// note lifetimes as for Captures above!
pub struct GMatch<'a,'b,'c> where 'a: 'c {
    m: &'c mut LuaPattern<'a>,
    text: &'b str
}

impl <'a,'b,'c>Iterator for GMatch<'a,'b,'c> {
    type Item = &'b str;

    fn next(&mut self) -> Option<Self::Item> {
        if ! self.m.matches(self.text) {
            None
        } else {
            let slice = &self.text[self.m.first_capture()];
            self.text = &self.text[self.m.range().end..];
            Some(slice)
        }
    }

}

/// Unsafe version of Captures, needed for gmatch_captures
// It's unsafe because the lifetime only depends on the original
// text, not the borrowed matches.
pub struct CapturesUnsafe<'b>{
    matches: *const LuaMatch,
    text: &'b str
}

impl <'b> CapturesUnsafe<'b> {
    /// get the capture as a string slice
    pub fn get(&self, i: usize) -> &'b str {
        unsafe {
            let p = self.matches.offset(i as isize);
            let range =
                ops::Range{
                    start: (*p).start as usize,
                    end: (*p).end as usize
                };
            &self.text[range]
        }
    }
}

/// Streaming iterator for all captures from `gmatch_captures`
// lifetimes as for Captures above!
// 'a is pattern, 'b is text, 'c is ref to LuaPattern
pub struct GMatchCaptures<'a,'b,'c> where 'a: 'c {
    m: &'c mut LuaPattern<'a>,
    text: &'b str
}

impl <'a,'b,'c> Iterator for GMatchCaptures<'a,'b,'c>  where 'a: 'c {
    type Item = CapturesUnsafe<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        if ! self.m.matches(self.text) {
            None
        } else {
            let split = self.text.split_at(self.m.range().end);
            self.text = split.1;
            let match_ptr: *const LuaMatch = self.m.matches.as_ptr();
            Some(CapturesUnsafe{matches: match_ptr, text: split.0})
        }
    }

}

/// Iterator for all byte slices from `gmatch_bytes`
pub struct GMatchBytes<'a,'b> {
    m: &'a mut LuaPattern<'a>,
    bytes: &'b [u8]
}

impl <'a,'b>Iterator for GMatchBytes<'a,'b> {
    type Item = &'b [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if ! self.m.matches_bytes(self.bytes) {
            None
        } else {
            let slice = &self.bytes[self.m.first_capture()];
            self.bytes = &self.bytes[self.m.range().end..];
            Some(slice)
        }
    }

}

/// Build a byte Lua pattern, optionally escaping 'magic' characters
pub struct LuaPatternBuilder {
    bytes: Vec<u8>
}

impl LuaPatternBuilder {
    /// Create a new Lua pattern builder
    pub fn new() -> LuaPatternBuilder {
        LuaPatternBuilder{bytes: Vec::new()}
    }

    /// Add unescaped characters from a string
    ///
    /// ```
    /// let patt = lua_patterns::LuaPatternBuilder::new()
    ///     .text("(boo)")
    ///     .build();
    /// assert_eq!(std::str::from_utf8(&patt).unwrap(), "(boo)");
    /// ```
    pub fn text(&mut self, s: &str) -> &mut Self {
        self.bytes.extend_from_slice(s.as_bytes());
        self
    }

    /// Add unescaped characters from lines
    ///
    /// This looks for first non-whitespace run in each line,
    /// useful for spreading patterns out and commmenting them.
    /// Works with patterns that use '%s' religiously!
    ///
    /// ```
    /// let patt = lua_patterns::LuaPatternBuilder::new()
    ///     .text_lines("
    ///       hello-dolly
    ///       you-are-fine  # comment
    ///       cool
    ///      ")
    ///     .build();
    /// assert_eq!(std::str::from_utf8(&patt).unwrap(),
    ///   "hello-dollyyou-are-finecool");
    /// ```
    pub fn text_lines(&mut self, lines: &str) -> &mut Self {
        let mut text = String::new();
        for line in lines.lines() {
            if let Some(first) = line.split_whitespace().next() {
                text.push_str(first);
            }
        }
        self.text(&text)
    }

    /// Add escaped bytes from a slice
    ///
    /// ```
    /// let patt = lua_patterns::LuaPatternBuilder::new()
    ///     .text("^")
    ///     .bytes(b"^") // magic character!
    ///     .build();
    /// assert_eq!(std::str::from_utf8(&patt).unwrap(), "^%^");
    /// ```
    pub fn bytes(&mut self, b: &[u8]) -> &mut Self {
        let mut m = LuaPattern::new("[%-%.%+%[%]%(%)%$%^%%%?%*]");
        let bb = m.gsub_bytes_with(b,|cc| {
            let mut res = Vec::new();
            res.push(b'%');
            res.push(cc.get(0)[0]);
            res
        });
        self.bytes.extend(bb);
        self
    }

    /// Add escaped bytes from hex string
    ///
    /// This consists of adjacent pairs of hex digits.
    ///
    /// ```
    /// let patt = lua_patterns::LuaPatternBuilder::new()
    ///     .text("^")
    ///     .bytes_as_hex("5E") // which is ASCII '^'
    ///     .build();
    /// assert_eq!(std::str::from_utf8(&patt).unwrap(), "^%^");
    /// ```
    pub fn bytes_as_hex(&mut self, bs: &str) -> &mut Self {
        let bb = LuaPatternBuilder::hex_to_bytes(bs);
        self.bytes(&bb)
    }

    /// Create the pattern
    pub fn build(&mut self) -> Vec<u8> {
        let mut v = Vec::new();
        std::mem::swap(&mut self.bytes, &mut v);
        v
    }

    /// Utility to create a vector of bytes from a hex string
    ///
    /// ```
    /// let bb = lua_patterns::LuaPatternBuilder::hex_to_bytes("AEFE00FE");
    /// assert_eq!(bb, &[0xAE,0xFE,0x00,0xFE]);
    /// ```
    pub fn hex_to_bytes(s: &str) -> Vec<u8> {
        let mut m = LuaPattern::new("%x%x");
        m.gmatch(s).map(|pair| u8::from_str_radix(pair,16).unwrap()).collect()
    }

    /// Utility to create a hex string from a slice of bytes
    ///
    /// ```
    /// let hex = lua_patterns::LuaPatternBuilder::bytes_to_hex(&[0xAE,0xFE,0x00,0xFE]);
    /// assert_eq!(hex,"AEFE00FE");
    ///
    /// ```
    pub fn bytes_to_hex(s: &[u8]) -> String {
        s.iter().map(|b| format!("{:02X}",b)).collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_and_matching() {
        let mut m = LuaPattern::new("(one).+");
        assert_eq!(m.captures(" one two"), &["one two","one"]);
        let empty: &[&str] = &[];
        assert_eq!(m.captures("four"), empty);

        assert_eq!(m.matches("one dog"), true);
        assert_eq!(m.matches("dog one "), true);
        assert_eq!(m.matches("dog one"), false);

        let text = "one dog";
        let mut m = LuaPattern::new("^(%a+)");
        assert_eq!(m.matches(text), true);
        assert_eq!(&text[m.capture(1)], "one");
        assert_eq!(m.matches(" one dog"), false);

        // captures without allocation
        m.matches(text);
        let captures = m.match_captures(text);
        assert_eq!(captures.get(0), "one");
        assert_eq!(captures.get(1), "one");

        let mut m = LuaPattern::new("(%S+)%s*=%s*(.+)");

        //  captures as Vec
        let cc = m.captures(" hello= bonzo dog");
        assert_eq!(cc[0], "hello= bonzo dog");
        assert_eq!(cc[1], "hello");
        assert_eq!(cc[2], "bonzo dog");

    }

    #[test]
    fn multiple_captures() {
        let mut p = LuaPattern::new("%s*(%d+)%s+(%S+)");
        let (int,rest) = p.match_maybe_2(" 233   hello dolly").unwrap();
        assert_eq!(int,"233");
        assert_eq!(rest,"hello");
    }

    #[test]
    fn gmatch() {
        let mut m = LuaPattern::new("%a+");
        let mut iter = m.gmatch("one two three");
        assert_eq!(iter.next(), Some("one"));
        assert_eq!(iter.next(), Some("two"));
        assert_eq!(iter.next(), Some("three"));
        assert_eq!(iter.next(), None);

        let mut m = LuaPattern::new("(%a+)");
        let mut iter = m.gmatch("one two three");
        assert_eq!(iter.next(), Some("one"));
        assert_eq!(iter.next(), Some("two"));
        assert_eq!(iter.next(), Some("three"));
        assert_eq!(iter.next(), None);

        let mut m = LuaPattern::new("(%a+)");
        let mut iter = m.gmatch_captures("one two three");
        assert_eq!(iter.next().unwrap().get(1), "one");
        assert_eq!(iter.next().unwrap().get(1), "two");
        assert_eq!(iter.next().unwrap().get(1), "three");
    }

    #[test]
    fn gsub() {
        use std::collections::HashMap;

        let mut m = LuaPattern::new("%$(%S+)");
        let res = m.gsub_with("hello $dolly you're so $fine!",
            |cc| cc.get(1).to_uppercase()
        );
        assert_eq!(res, "hello DOLLY you're so FINE!");

        let mut map = HashMap::new();
        map.insert("dolly", "baby");
        map.insert("fine", "cool");
        map.insert("good-looking", "pretty");

        let mut m = LuaPattern::new("%$%((.-)%)");
        let res = m.gsub_with("hello $(dolly) you're so $(fine) and $(good-looking)",
            |cc| map.get(cc.get(1)).unwrap_or(&"?").to_string()
        );
        assert_eq!(res, "hello baby you're so cool and pretty");

        let mut m = LuaPattern::new("%s+");
        let res = m.gsub("hello dolly you're so fine","");
        assert_eq!(res, "hellodollyyou'resofine");

        let mut m = LuaPattern::new("(%S+)%s*=%s*(%S+);%s*");
        let res = m.gsub("a=2; b=3; c = 4;", "'%2':%1 ");
        assert_eq!(res,"'2':a '3':b '4':c ");
    }

    #[test]
    fn bad_patterns() {
       let bad = [
        ("bonzo %","malformed pattern (ends with '%')"),
        ("bonzo (dog%(","unfinished capture"),
        ("alles [%a%[","malformed pattern (missing ']')"),
        ("bonzo (dog (cat)","unfinished capture"),
        ("frodo %f[%A","malformed pattern (missing ']')"),
        ("frodo (1) (2(3)%2)%1","invalid capture index %2"),
        ];
        for p in bad.iter() {
            let res = LuaPattern::new_try(p.0);
            if let Err(e) = res {
                assert_eq!(e, PatternError(p.1.into()));
            } else {
                panic!("false positive");
            }
        }
    }
}
