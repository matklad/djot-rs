// Similar to the strings(1) utility
// We print any sequences involving four or more ASCII letters
extern crate lua_patterns;
use lua_patterns::LuaPattern;

use std::env;
use std::str;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let file = env::args().skip(1).next().expect("provide a binary file");
    let mut f = File::open(&file).expect("can't open file");
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).expect("can't read file");

    let mut words = LuaPattern::new("%a%a%a%a+");
    for w in words.gmatch_bytes(&buf) {
        println!("{}",str::from_utf8(w).unwrap());
    }

}
