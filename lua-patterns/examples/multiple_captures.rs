extern crate lua_patterns as lp;

fn main() {
    let mut p = lp::LuaPattern::new("%s*(%d+)%s+(%S+)");
    if let Some((int,rest)) = p.match_maybe_2(" 233   hello dolly") {
        assert_eq!(int,"233");
        assert_eq!(rest,"hello");
    }
}
