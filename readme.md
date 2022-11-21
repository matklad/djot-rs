## Lua string patterns in Rust

[Lua string patterns](https://www.lua.org/pil/20.2.html) are a powerful
yet lightweight alternative to full regular expressions. They are not
regexps, since there is no alternation (the `|` operator), but this
is not usually a problem. In fact, full regexps become _too powerful_ and
power can be dangerous or just plain confusing.
This is why OpenBSD's httpd has [Lua patterns](http://man.openbsd.org/patterns.7).
The decision to use `%` as the escape rather than the traditional `\` is refreshing.
In the Rust context, `lua-patterns` is a very lightweight dependency, if you
don't need the full power of the `regex` crate.

This library reuses the original source from Lua 5.2 - only
400 lines of battle-tested C. I originally did this for a similar project to bring
[these patterns to C++](https::/github.com/stevedonovan/rx-cpp).

More information can be found on [the Lua wiki](http://lua-users.org/wiki/PatternsTutorial).
The cool thing is that Lua is a 300KB download, if you want to test patterns out
without going through Rust.

I've organized the Rust interface much as the original Lua library, 'match',
'gmatch' and 'gsub', but made these methods of a `LuaPattern` struct. This is
for two main reasons:

  - although string patterns are not compiled, they can be validated upfront
  - after a match, the struct contains the results

```rust
extern crate lua_patterns;
use lua_patterns::LuaPattern;

let mut m = LuaPattern::new("one");
let text = "hello one two";
assert!(m.matches(text));
let r = m.range();
assert_eq!(r.start, 6);
assert_eq!(r.end, 9);
```
This not in itself impressive, since it can be done with the string `find`
method. (`new` will panic if you feed it a bad pattern, so use `new_try` if
you want more control.)

Once we start using patterns it gets more exciting, especially
with _captures_:

```rust
let mut m = LuaPattern::new("(%a+) one");
let text = " hello one two";
assert!(m.matches(text));
assert_eq!(m.capture(0),1..10); // "hello one"
assert_eq!(m.capture(1),1..6); // "hello"
```
Lua patterns (like regexps) are not anchored by default, so this finds
the first match and works from there. The 0 capture always exists
(the full match) and here the 1 capture just picks up the first word.

> There is an obvious limitation: "%a" refers specifically to a single byte
> representing a letter according to the C locale. Lua people will often
> look for 'sequence of non-spaces' ("%S+"), etc - that is, identify maybe-UTF-8
> sequences using surronding punctuation or spaces.

If you want your captures as strings, then there are several options. If there's
just one, then `match_maybe` is useful:

```rust
let mut m = LuaPattern::new("OK%s+(%d+)");
let res = m.match_maybe("and that's OK 400 to you");
assert_eq!(res, Some("400"));
```
You can grab them as a vector (it will be empty if the match fails.)

```rust
let mut m = LuaPattern::new("(%a+) one");
let text = " hello one two";
let v = m.captures(text);
assert_eq!(v, &["hello one","hello"]);
```
This will create a vector. You can avoid excessive allocations with `capture_into`:

```rust
let mut v = Vec::new();
if m.capture_into(text,&mut v) {
    assert_eq!(v, &["hello one","hello"]);
}
```
Imagine that this is happening in a loop - the vector is only allocated the first
time it is filled, and thereafter there are no allocations. It's a convenient
method if you are checking text against several patterns, and is actually
more ergonomic than using Lua's `string.match`.  (Personally I prefer
to use those marvelous things called "if statements" rather than elaborate
regular expressions.)

The `gmatch` method creates an interator over all matched strings.

```rust
let mut m = lp::LuaPattern::new("%S+");
let split: Vec<_> = m.gmatch("dog  cat leopard wolf  ").collect();
assert_eq!(split,&["dog","cat","leopard","wolf"]);
```
A single match is returned; if the pattern has no captures, you get the full match,
otherwise you get the first match. So "(%S+)" would give you the same result.

A more general version is `gmatch_captures` which creates a _streaming_ iterator
over captures. You have to be a little careful with this one; in particular, you
will get nonsense if you try to `collect` on the return captures: don't try to
keep these values.
It is fine to collect from an expression involving the `get` method however!

```rust
let mut m = lua_patterns::LuaPattern::new("(%S)%S+");
let split: Vec<_> = m.gmatch_captures("dog  cat leopard wolf")
       .map(|cc| cc.get(1)).collect();
assert_eq!(split,&["d","c","l","w"]);
```

Text substitution is an old favourite of mine, so here's `gsub_with`:

```rust
let mut m = lp::LuaPattern::new("%$(%S+)");
let res = m.gsub_with("hello $dolly you're so $fine",
    |cc| cc.get(1).to_uppercase()
);
assert_eq!(res,"hello DOLLY you're so FINE");
```
The closure is passed a `Closures` object and the captures are accessed
using the `get` method; it returns a `String`.

The second form of `gsub` is convenient when you have a replacement
string, which may contain closure references. (To add a literal "%" escape
it like so "%%")

```rust
let mut m = LuaPattern::new("%s+");
let res = m.gsub("hello dolly you're so fine","");
assert_eq!(res, "hellodollyyou'resofine");

let mut m = LuaPattern::new("(%S+)%s*=%s*(%S+);%s*");
let res = m.gsub("a=2; b=3; c = 4;", "'%2':%1 ");
assert_eq!(res, "'2':a '3':b '4':c ");
```
The third form of `string.gsub` in Lua does lookup with a table - that is, a map.
But for maps you really want to handle the 'not found' case in some special way:

```rust
let mut map = HashMap::new();
// updating old lines for the 21st Century
map.insert("dolly", "baby");
map.insert("fine", "cool");
map.insert("good-looking", "pretty");

let mut m = LuaPattern::new("%$%((.-)%)");
let res = m.gsub_with("hello $(dolly) you're so $(fine) and $(good-looking)",
    |cc| map.get(cc.get(1)).unwrap_or(&"?").to_string()
);
assert_eq!(res,"hello baby you're so cool and pretty");
```

(The ".-" pattern means 'match as little as possible' - often called 'lazy'
matching.)

This is equivalent to a replace string "%1:'%2'":

```rust
let mut m = lp::LuaPattern::new("(%S+)%s*=%s*([^;]+);");
let res = m.gsub_with("alpha=bonzo; beta=felix;",
    |cc| format!("{}:'{}',", cc.get(1), cc.get(2))
);
assert_eq!(res, "alpha:'bonzo', beta:'felix',");
```
Having a byte-oriented pattern matcher can be useful. For instance, this
is basically the old `strings` utility - we read all of a 'binary' file into
a vector of bytes, and then use `gmatch_bytes` to iterate over all `&[u8]`
matches corresponding to two or more adjacent ASCII letters:

```rust
let mut words = LuaPattern::new("%a%a+");
for w in words.gmatch_bytes(&buf) {
    println!("{}",std::str::from_utf8(w).unwrap());
}
```
The pattern itself may be arbitrary bytes - Lua 'string' matching does
not care about embedded nul bytes:

```rust
let patt = &[0xDE,0x00,b'+',0xBE];
let bytes = &[0xFF,0xEE,0x0,0xDE,0x0,0x0,0xBE,0x0,0x0];

let mut m = LuaPattern::from_bytes(patt);
assert!(m.matches_bytes(bytes));
assert_eq!(&bytes[m.capture(0)], &[0xDE,0x00,0x00,0xBE]);
```
The problem here is that it's not obvious when our 'arbitrary' bytes
include one of the special matching characters like `$` (which is 0x24)
and so on. Hence there is `LuaPatternBuilder`:

```rust
let bytes = &[0xFF,0xEE,0x0,0xDE,0x24,0x24,0xBE,0x0,0x0];

let patt = LuaPatternBuilder::new()
    .bytes_as_hex("DE24") // less tedious than a byte slice
    .text("+")  // unescaped
    .bytes(&[0xBE]) // byte slice
    .build();

let mut m = LuaPattern::from_bytes(&patt);
// picks up "DE2424BE"
```
> Static verification: this version attempts to verify string patterns. If you
> want errors, use `new_try` and `from_bytes_try`, otherwise the constructors panic.
> If a match panics after successful verification, it is a __BUG__ - please
> report the offending pattern.

