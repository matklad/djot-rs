// translation of Lua 5.2 string pattern code

use errors::*;
use std::ptr::null;

pub const LUA_MAXCAPTURES: usize = 32;
/* maximum recursion depth for 'match' */
const MAXCCALLS: usize = 200;

const L_ESC: u8 = b'%';

fn add(p: CPtr, count: usize) -> CPtr {
    unsafe {p.offset(count as isize)}
}

fn sub(p: CPtr, count: usize) -> CPtr {
    unsafe {p.offset(-(count as isize))}
}

fn next(p: CPtr) -> CPtr {
    add(p, 1)
}

fn at(p: CPtr) -> u8 {
   unsafe { *p }
}

fn diff(p1: CPtr, p2: CPtr) -> usize {
    let d = (p1 as isize).wrapping_sub(p2 as isize);
    d as usize
}

#[derive(Copy,Clone,Debug)]
pub struct LuaMatch {
    pub start: usize,
    pub end: usize,
}

#[derive(Copy,Clone)]
enum CapLen {
    Len(usize),
    Unfinished,
    Position,
}

impl CapLen {
    fn is_unfinished(&self) -> bool {
        match *self {
            CapLen::Unfinished => true,
            _ => false
        }
    }

    fn size(&self) -> Result<usize> {
        match *self {
            CapLen::Len(size) => Ok(size),
            _ => error("capture was unfinished or positional")
        }
    }

}

type CPtr = *const u8;

#[derive(Copy,Clone)]
struct Capture {
    init: CPtr,
    len: CapLen,
}

impl Capture {
    fn is_unfinished(&self) -> bool {
        self.len.is_unfinished()
    }
}

use std::result;

type Result<T> = result::Result<T,PatternError>;

fn error<T>(msg: &str) ->  Result<T> {
    Err(PatternError(msg.into()))
}

struct MatchState {
    matchdepth: usize, /* control for recursive depth (to avoid stack overflow) */
    src_init: CPtr, /* init of source string */
    src_end: CPtr, /* end ('\0') of source string */
    p_end: CPtr, /* end ('\0') of pattern */
    level: usize, /* total number of captures (finished or unfinished) */
    capture: [Capture; LUA_MAXCAPTURES],
}

impl MatchState {
    fn new(s: CPtr, se: CPtr, pe: CPtr) -> MatchState {
        MatchState {
            matchdepth: MAXCCALLS,
            src_init: s,
            src_end: se,
            p_end: pe,
            level: 0,
            capture: [Capture{init: null(), len: CapLen::Len(0) }; LUA_MAXCAPTURES],
        }
    }

    fn check_capture(&self, l: usize) -> Result<usize> {
        let l = l as i8 - b'1' as i8;
        if l < 0 || l as usize >= self.level || self.capture[l as usize].is_unfinished() {
            return error(&format!("invalid capture index %{}", l + 1));
        }
        Ok(l as usize)
    }

    fn capture_to_close(&self) -> Result<usize> {
        let mut level = (self.level - 1) as isize;
        while level >= 0 {
            if self.capture[level as usize].is_unfinished() {
                return Ok(level as usize);
            }
            level -= 1;
        }
        error("invalid pattern capture")
    }

    fn classend (&self, p: CPtr) -> Result<CPtr> {
        let ch = at(p);
        let mut next_p = next(p);
        Ok(match ch {
            L_ESC => {
                if next_p == self.p_end {
                    return error("malformed pattern (ends with '%')");
                }
                next(next_p)
            },
            b'[' => {
                if at(next_p) == b'^' {
                    next_p = next(next_p);
                }
                while at(next_p) != b']' {
                    if next_p == self.p_end {
                        return error("malformed pattern (missing ']')");
                    }
                    let ch = at(next_p);
                    next_p = next(next_p);
                    if ch == L_ESC && p < self.p_end {
                        next_p = next(next_p);  /* skip escapes (e.g. `%]') */
                    }
                }
                next(next_p)
            },
            _ => next_p
        })
    }

}

fn match_class (ch: u8, class: u8) -> bool {
    let res = match class.to_ascii_lowercase() {
        b'a' => ch.is_ascii_alphabetic(),
        b'c' => ch.is_ascii_control(),
        b'd' => ch.is_ascii_digit(),
        b'g' => ch.is_ascii_graphic(),
        b'l' => ch.is_ascii_lowercase(),
        b'p' => ch.is_ascii_punctuation(),
        b's' => ch.is_ascii_whitespace(),
        b'u' => ch.is_ascii_uppercase(),
        b'w' => ch.is_ascii_alphanumeric(),
        b'x' => ch.is_ascii_hexdigit(),
        lc => return lc == ch,
    };
    if class.is_ascii_lowercase() { res } else {! res}
}


fn matchbracketclass (c: u8, p: CPtr, ec: CPtr) -> bool {
    let mut p = p;
    // [^ inverts match
    let sig = if at(next(p)) == b'^' {
        p = next(p);
        false
    } else {
        true
    };
    p = next(p);
    while p < ec {
        if at(p) == L_ESC { // e.g %s
            p = next(p);
            if match_class(c, at(p)) {
                return sig;
            }
        } else
        // e.g a-z
        if at(next(p)) == b'-' && add(p,2) < ec {
            let lastc = at(p);
            p = add(p,2);
            if lastc <= c && c <= at(p) {
                return sig;
            }
        } else
        if at(p) == c {
            return sig;
        }
        p = next(p);
    }
    return ! sig;
}

impl MatchState {

    fn singlematch (&self, s: CPtr, p: CPtr, ep: CPtr) -> bool {
        if s >= self.src_end {
            return false;
        }
        let c = at(s);
        let pc = at(p);
        match pc {
            b'.' => true, /* matches any char */
            L_ESC => match_class(c, at(next(p))),
            b'[' => matchbracketclass(c, p, sub(ep,1)),
            _ => c == pc
        }
    }

    fn matchbalance (&self, s: CPtr, p: CPtr) -> Result<CPtr> {
        if p >= sub(self.p_end,1) {
            return error("malformed pattern (missing arguments to '%b')");
        }
        if at(s) != at(p) {
            return Ok(null());
        }
        // e.g. %b()
        let b = at(p);
        let e = at(next(p));
        let mut cont = 1;
        let mut s = next(s);
        while s < self.src_end {
            let ch = at(s);
            if ch == e {
                cont -= 1;
                if cont == 0 {
                    return Ok(next(s));
                }
            } else
            if ch == b {
                cont += 1;
            }
            s = next(s);
        }
        Ok(null()) /* string ends out of balance */
    }

    fn max_expand(&mut self, s: CPtr, p: CPtr, ep: CPtr) -> Result<CPtr> {
        let mut i = 0isize; /* counts maximum expand for item */
        while self.singlematch(add(s,i as usize),p,ep) {
            i += 1;
        }
        /* keeps trying to match with the maximum repetitions */
        while  i >= 0 {
            let res = self.patt_match(add(s,i as usize),next(ep))?;
            if ! res.is_null() {
                return Ok(res);
            }
            i -= 1; /* else didn't match; reduce 1 repetition to try again */
        }
        Ok(null())
    }

    fn min_expand(&mut self, s: CPtr, p: CPtr, ep: CPtr) -> Result<CPtr> {
        let mut s = s;
        loop {
            let res = self.patt_match(s,next(ep))?;
            if ! res.is_null() {
                return Ok(res);
            } else
            if self.singlematch(s, p, ep) {
                s = next(s);
            } else {
                return Ok(null());
            }
        }
    }

    fn start_capture(&mut self, s: CPtr, p: CPtr, what: CapLen) -> Result<CPtr> {
        let level = self.level;
        if level >= LUA_MAXCAPTURES {
            return error("too many captures");
        }
        self.capture[level].init = s;
        self.capture[level].len = what;
        self.level = level + 1;
        let res = self.patt_match(s, p)?;
        if res.is_null() { /* match failed? */
            self.level -= 1; /* undo capture */
        }
        Ok(res)
    }

    fn end_capture(&mut self, s: CPtr, p: CPtr) -> Result<CPtr> {
        let l = self.capture_to_close()?;
        self.capture[l].len = CapLen::Len(diff(s,self.capture[l].init));  /* close capture */
        let res = self.patt_match(s, p)?;
        if res.is_null() { /* match failed? */
            self.capture[l].len = CapLen::Unfinished;
        }
        Ok(res)
    }

    fn match_capture(&mut self, s: CPtr, l: usize) -> Result<CPtr> {
        let l = self.check_capture(l)?;
        let len = self.capture[l].len.size()?;
        if diff(self.src_end, s) >= len {
            unsafe {s.copy_to_nonoverlapping(self.capture[l].init as *mut u8, len);}
            return Ok(add(s,len));
        }
        Ok(null())
    }


    fn patt_match(&mut self, s: CPtr, p: CPtr) -> Result<CPtr> {
        let mut s = s;
        let mut p = p;
        self.matchdepth -= 1;
        if self.matchdepth == 0 {
            return error("pattern too complex");
        }

        if p == self.p_end {  /* end of pattern? */
            self.matchdepth += 1;
            return Ok(s);
        }
        match at(p) {
            b'(' => { /* start capture */
                if at(next(p)) == b')' { /* position capture? */
                    s = self.start_capture(s, add(p,2), CapLen::Position)?;
                } else {
                    s = self.start_capture(s, next(p), CapLen::Unfinished)?;
                }
            },
            b')' => { /* end capture */
                s = self.end_capture(s, next(p))?;
            },
            b'$' => {
                if next(p) != self.p_end { /* is the `$' the last char in pattern? */
                   /* no; go to default */
                   return self.patt_default_match(s, p);
                }
                s = if s == self.src_end {s} else {null()}; /* check end of string */
            }
            L_ESC => {  /* escaped sequences not in the format class[*+?-]? */
                match at(next(p)) {
                    b'b' => { /* balanced string? */
                        s = self.matchbalance(s, add(p,2))?;
                        if ! s.is_null() {
                            // e.g, after %b()
                            return self.patt_match(s, add(p,4));
                        }
                    },
                    b'f' => { /* frontier? */
                        p = add(p,2);
                        if at(p) != b'[' {
                            return error("missing '[' after '%f' in pattern");
                        }
                        let ep = self.classend(p)?; /* points to what is next */
                        let previous = if s == self.src_init {b'\0'} else {at(sub(s,1))};
                        let epl = sub(ep,1);
                        if ! matchbracketclass(previous,p,epl)
                           && matchbracketclass(at(s),p,epl) {
                            return self.patt_match(s, ep);
                        }
                        s = null(); /* match failed */
                    },
                    b'0'...b'9' => {  /* capture results (%0-%9)? */
                        s = self.match_capture(s,at(next(p)) as usize)?;
                        if ! s.is_null() {
                            return self.patt_match(s, add(p,2));
                        }
                    },
                    _ => return self.patt_default_match(s, p)
                }

            },
            _ => return self.patt_default_match(s, p)

        }
        self.matchdepth += 1;
        Ok(s)
    }

    fn patt_default_match(&mut self, s: CPtr, p: CPtr) -> Result<CPtr> {
        let mut s = s;
        /* pattern class plus optional suffix */
        let ep = self.classend(p)?; /* points to optional suffix */
        /* does not match at least once? */
        if ! self.singlematch(s, p, ep) {
            let epc = at(ep);
            if epc == b'*' || epc == b'?' || epc == b'-' { /* accept empty? */
                return self.patt_match(s, next(ep));
            } else { /* '+' or no suffix */
                s = null(); /* fail */
            }
        } else { /* matched once */
            match at(ep) { /* handle optional suffix */
                b'?' => {
                    let res = self.patt_match(next(s),next(ep))?;
                    if ! res.is_null() {
                        s = res;
                    } else {
                        return self.patt_match(s, next(ep));
                    }
                },
                b'+' => { /* 1 or more repetitions */
                    s = next(s);
                    s = self.max_expand(s, p, ep)?;
                },
                b'*' => { /* 0 or more repetitions */
                    s = self.max_expand(s, p, ep)?;
                },
                b'-' => { /* 0 or more repetitions (minimum) */
                    s = self.min_expand(s, p, ep)?  ;
                },
                _ => { /* no suffix */
                    return self.patt_match(next(s),ep);
                }
            }
        }
        self.matchdepth += 1;
        Ok(s)
    }

    fn push_onecapture(&mut self, i: usize, s: CPtr, e: CPtr, mm: &mut [LuaMatch]) -> Result<()> {
        if i >= self.level {
            if i == 0 { /* ms->level == 0, too */
                mm[0].start = 0;
                mm[0].end = diff(e,s);
                Ok(())
            } else {
                return error("invalid capture index");
            }
        } else {
            let init = self.capture[i].init;
            match self.capture[i].len {
                CapLen::Unfinished => error("unfinished capture"),
                CapLen::Position => {
                    mm[i].start = diff(init,next(self.src_init));
                    mm[i].end = mm[i].start;
                    Ok(())
                },
                CapLen::Len(l) => {
                    mm[i].start = diff(init,self.src_init);
                    mm[i].end = mm[i].start + l;
                    Ok(())
                }
            }
        }

    }

    fn push_captures(&mut self, s: CPtr, e: CPtr, mm: &mut [LuaMatch]) -> Result<usize> {
        let nlevels = if self.level == 0 && ! s.is_null() {1} else {self.level};
        for i in 0..nlevels {
            self.push_onecapture(i, s, e, mm)?;
        }
        Ok(nlevels)  /* number of strings pushed */
    }

    pub fn str_match_check(&mut self, p: CPtr) -> Result<()> {
        let mut level_stack = [0; LUA_MAXCAPTURES];
        let mut stack_idx = 0;
        let mut p = p;
        while p < self.p_end {
            let ch = at(p);
            p = next(p);
            match ch {
                L_ESC => {
                    //p = next(p);
                    let c = at(p);
                    match c {
                        b'b' => {
                            p = next(p);
                            if p >= self.p_end {
                                return error("malformed pattern (missing arguments to '%b')");
                            }
                        },
                        b'f' => {
                            p = next(p);
                            if at(p) != b'[' {
                                return error("missing '['  after '%f' in pattern");
                            }
                            p = sub(p,1); // so we see [...]
                        },
                        b'0' ... b'9' => {
                            let l = (c as i8) - (b'1' as i8);
                            println!("level {}", self.level);
                            if l < 0 || l as usize >= self.level || self.capture[l as usize].is_unfinished() {
                                return error(&format!("invalid capture index %{}", l + 1));
                            }
                            p = sub(p,1);
                        },
                        _ => {}
                    }
                },
                b'[' => {
                    while at(p) != b']' {
                        if p == self.p_end {
                            return error("malformed pattern (missing ']')");
                        }
                        if at(p) == L_ESC && p < self.p_end {
                            p = next(p);
                        }
                        p = next(p);
                    }
                },
                b'(' => {
                    if at(p) != b')' { // not a position capture
                        level_stack[stack_idx] = self.level;
                        stack_idx += 1;
                        self.capture[self.level].len = CapLen::Unfinished;
                        self.level += 1;
                        if self.level >= LUA_MAXCAPTURES {
                            return error("too many captures");
                        }
                    } else {
                        p = next(p);
                    }
                },
                b')' => {
                    if stack_idx == 0 {
                        return error("no open capture");
                    }
                    stack_idx -= 1;
                    self.capture[level_stack[stack_idx]].len = CapLen::Position;
                },
                _ => {}
            }
        }
        if stack_idx > 0 {
            return error("unfinished capture");
        }
        Ok(())
    }
}

pub fn str_match(s: &[u8], p: &[u8], mm: &mut [LuaMatch]) -> Result<usize> {
    let mut lp = p.len();
    let mut p = p.as_ptr();
    let ls = s.len();
    let s = s.as_ptr();
    let mut s1 = s;
    let anchor = at(p) == b'^';
    if anchor {
        p = next(p);
        lp -= 1;  /* skip anchor character */
    }

    let mut ms = MatchState::new(s,add(s,ls),add(p,lp));
    loop {
        let res = ms.patt_match(s1, p)?;
        if ! res.is_null() {
            mm[0].start = diff(s1,s); /* start */
            mm[0].end = diff(res,s); /* end */
            return Ok(ms.push_captures(null(),null(),&mut mm[1..])? + 1);
        }
        s1 = next(s1);
        if ! (s1 < ms.src_end && ! anchor) {
            break;
        }
    }
    Ok(0)
}

pub fn str_check(p: &[u8]) -> Result<()> {
    let mut lp = p.len();
    let mut p = p.as_ptr();
    let anchor = at(p) == b'^';
    if anchor {
        p = next(p);
        lp -= 1;  /* skip anchor character */
    }
    let mut ms = MatchState::new(null(),null(),add(p,lp));
    if at(sub(ms.p_end,1)) == b'%' {
        return error("malformed pattern (ends with '%')");
    }
    ms.str_match_check(p)?;
    Ok(())
}

/*
fn check(s: &[u8], p: &[u8]) {
    if let Err(e) = str_check(p) {
        println!("check error {}",e);
        return;
    }

    let mut matches = [LuaMatch{start: 0, end: 0}; 10];
    match str_match(s, p, &mut matches) {
        Ok(n) => {
            println!("ok {} matches", n);
            for i in 0..n {
                println!("match {:?} {:?}",
                    matches[i],
                    String::from_utf8(s[matches[i].start .. matches[i].end].to_vec())
                );
            }
        },
        Err(e) => {
            println!("error: {}", e)
        }
    }
}



fn main() {
    let mut args = std::env::args().skip(1);
    let pat = args.next().unwrap();
    let s = args.next().unwrap();
    check(s.as_bytes(), pat.as_bytes());

    //~ check(b"hello",b"%a");
    //~ check(b"0hello",b"%a+");
    //~ check(b"hello",b"%l(%a)");
    //check(b"hello",b"he(l+)");
    //check(b"k  {and {so}}",b"k%s+(%b{})");
}
 */
