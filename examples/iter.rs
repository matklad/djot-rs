extern crate lua_patterns as lp;



fn main() {

    //~ let mut m = lp::LuaPattern::new("hello%");
    //~ m.matches("hello");
    //~ println!("ok");

    let mut m = lp::LuaPattern::new("(%a+)");
    let mut iter = m.gmatch("one two three");
    assert_eq!(iter.next(), Some("one"));
    assert_eq!(iter.next(), Some("two"));
    assert_eq!(iter.next(), Some("three"));
    assert_eq!(iter.next(), None);

    let mut m = lp::LuaPattern::new("%S+");
    let split: Vec<_> = m.gmatch("dog  cat leopard wolf").collect();
    assert_eq!(split,&["dog","cat","leopard","wolf"]);

    let mut m = lp::LuaPattern::new("%s*(%S+)%s*=%s*(.-);");
    let cc = m.captures(" hello= bonzo dog;");
    assert_eq!(cc[0], " hello= bonzo dog;");
    assert_eq!(cc[1],"hello");
    assert_eq!(cc[2],"bonzo dog");

    for cc in m.gmatch_captures("hello=bonzo dog; bye=cat;") {
        println!("'{}'='{}'",cc.get(1),cc.get(2));
    }

    let mut m = lp::LuaPattern::new("%$(%S+)");
    let res = m.gsub_with("hello $dolly you're so $fine",
        |cc| cc.get(1).to_uppercase()
    );
    assert_eq!(res,"hello DOLLY you're so FINE");

    let mut m = lp::LuaPattern::new("(%S+)%s*=%s*([^;]+);");
    let res = m.gsub_with("alpha=bonzo; beta=felix;",
        |cc| format!("{}:'{}',", cc.get(1), cc.get(2))
    );
    assert_eq!(res, "alpha:'bonzo', beta:'felix',");



}
