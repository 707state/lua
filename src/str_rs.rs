use crate::api::*;
use crate::aux_rs::{
    luaL_checkinteger, luaL_checklstring, luaL_checknumber, luaL_checkstack, luaL_getmetafield,
    luaL_optinteger, luaL_optlstring, luaL_tolstring, luaL_typeerror,
};
use crate::lua_module::{
    argcheck, create_library, lua_Integer, lua_Number, lua_Unsigned, lua_createtable, lua_error,
    lua_gettop, lua_pop, lua_pushcclosure, lua_pushinteger, lua_pushlstring, lua_pushnumber,
    lua_pushstring, lua_pushvalue, lua_setfield, lua_settop, lua_upvalueindex, luaL_Reg,
    luaL_error, luaL_error_str, luaL_setfuncs, push_fail,
};
use crate::luaffi::*;
use crate::runtime::*;
use core::ffi::c_void;
use core::ffi::{c_char, c_int, c_uchar};
use core::ptr::{self, null};

#[derive(Default)]
struct DumpWriterState {
    bytes: Vec<u8>,
}

#[derive(Copy, Clone)]
struct Capture {
    init: *const u8,
    len: isize,
}

impl Default for Capture {
    fn default() -> Self {
        Self {
            init: ptr::null(),
            len: 0,
        }
    }
}

struct MatchState {
    src_init: *const u8,
    src_end: *const u8,
    p_end: *const u8,
    state: *mut lua_State,
    matchdepth: c_int,
    level: c_int,
    capture: [Capture; LUA_MAXCAPTURES],
}

#[repr(C)]
struct GMatchState {
    src: *const u8,
    p: *const u8,
    lastmatch: *const u8,
    ms: MatchState,
}

#[derive(Copy, Clone)]
enum KOption {
    Kint,
    Kuint,
    Kfloat,
    Knumber,
    Kdouble,
    Kchar,
    Kstring,
    Kzstr,
    Kpadding,
    Kpaddalign,
    Knop,
}

struct Header {
    state: *mut lua_State,
    islittle: bool,
    maxalign: usize,
}

static STRLIB: [luaL_Reg; 18] = [
    luaL_Reg {
        name: NAME_BYTE.as_ptr().cast(),
        func: Some(str_byte),
    },
    luaL_Reg {
        name: NAME_CHAR.as_ptr().cast(),
        func: Some(str_char),
    },
    luaL_Reg {
        name: NAME_DUMP.as_ptr().cast(),
        func: Some(str_dump),
    },
    luaL_Reg {
        name: NAME_FIND.as_ptr().cast(),
        func: Some(str_find),
    },
    luaL_Reg {
        name: NAME_FORMAT.as_ptr().cast(),
        func: Some(str_format),
    },
    luaL_Reg {
        name: NAME_GMATCH.as_ptr().cast(),
        func: Some(gmatch),
    },
    luaL_Reg {
        name: NAME_GSUB.as_ptr().cast(),
        func: Some(str_gsub),
    },
    luaL_Reg {
        name: NAME_LEN.as_ptr().cast(),
        func: Some(str_len),
    },
    luaL_Reg {
        name: NAME_LOWER.as_ptr().cast(),
        func: Some(str_lower),
    },
    luaL_Reg {
        name: NAME_MATCH.as_ptr().cast(),
        func: Some(str_match),
    },
    luaL_Reg {
        name: NAME_REP.as_ptr().cast(),
        func: Some(str_rep),
    },
    luaL_Reg {
        name: NAME_REVERSE.as_ptr().cast(),
        func: Some(str_reverse),
    },
    luaL_Reg {
        name: NAME_SUB.as_ptr().cast(),
        func: Some(str_sub),
    },
    luaL_Reg {
        name: NAME_UPPER.as_ptr().cast(),
        func: Some(str_upper),
    },
    luaL_Reg {
        name: NAME_PACK.as_ptr().cast(),
        func: Some(str_pack),
    },
    luaL_Reg {
        name: NAME_PACKSIZE.as_ptr().cast(),
        func: Some(str_packsize),
    },
    luaL_Reg {
        name: NAME_UNPACK.as_ptr().cast(),
        func: Some(str_unpack),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

static STRING_METAMETHODS: [luaL_Reg; 10] = [
    luaL_Reg {
        name: MT_ADD.as_ptr().cast(),
        func: Some(arith_add),
    },
    luaL_Reg {
        name: MT_SUB.as_ptr().cast(),
        func: Some(arith_sub),
    },
    luaL_Reg {
        name: MT_MUL.as_ptr().cast(),
        func: Some(arith_mul),
    },
    luaL_Reg {
        name: MT_MOD.as_ptr().cast(),
        func: Some(arith_mod),
    },
    luaL_Reg {
        name: MT_POW.as_ptr().cast(),
        func: Some(arith_pow),
    },
    luaL_Reg {
        name: MT_DIV.as_ptr().cast(),
        func: Some(arith_div),
    },
    luaL_Reg {
        name: MT_IDIV.as_ptr().cast(),
        func: Some(arith_idiv),
    },
    luaL_Reg {
        name: MT_UNM.as_ptr().cast(),
        func: Some(arith_unm),
    },
    luaL_Reg {
        name: FIELD_INDEX.as_ptr().cast(),
        func: None,
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

#[inline]
fn posrelat_i(pos: lua_Integer, len: usize) -> usize {
    if pos > 0 {
        pos as usize
    } else if pos == 0 || pos < -(len as lua_Integer) {
        1
    } else {
        (len as lua_Integer + pos + 1) as usize
    }
}

#[inline]
unsafe fn getendpos(state: *mut lua_State, arg: c_int, def: lua_Integer, len: usize) -> usize {
    let pos = { luaL_optinteger(state, arg, def) };
    if pos > len as lua_Integer {
        len
    } else if pos >= 0 {
        pos as usize
    } else if pos < -(len as lua_Integer) {
        0
    } else {
        (len as lua_Integer + pos + 1) as usize
    }
}

#[inline]
unsafe fn runtime_error(state: *mut lua_State, message: &'static [u8]) -> c_int {
    unsafe { lua_pushstring(state, message.as_ptr().cast()) };
    unsafe { lua_error(state) }
}

unsafe fn dump_writer(
    _state: *mut lua_State,
    block: *const c_void,
    size: usize,
    data: *mut c_void,
) -> c_int {
    if !block.is_null() && size != 0 {
        let writer = unsafe { &mut *(data as *mut DumpWriterState) };
        let bytes = unsafe { core::slice::from_raw_parts(block.cast::<u8>(), size) };
        writer.bytes.extend_from_slice(bytes);
    }
    0
}

#[inline]
unsafe fn tonum(state: *mut lua_State, arg: c_int) -> bool {
    if unsafe { lua_type(state, arg) } == LuaType::Number.as_c_int() {
        unsafe { lua_pushvalue(state, arg) };
        true
    } else {
        let mut len = 0;
        let s = unsafe { lua_tolstring(state, arg, &mut len) };
        !s.is_null() && unsafe { lua_stringtonumber(state, s) } == len + 1
    }
}

unsafe fn trymt(state: *mut lua_State, mtkey: &'static [u8]) {
    unsafe { lua_settop(state, 2) };
    if unsafe { lua_type(state, 2) } == LuaType::String.as_c_int() || {
        luaL_getmetafield(state, 2, mtkey.as_ptr().cast())
    } == 0
    {
        let opname = unsafe { mtkey.as_ptr().add(2).cast::<c_char>() };
        let left = unsafe { lua_typename(state, lua_type(state, -2)) };
        let right = unsafe { lua_typename(state, lua_type(state, -1)) };
        let opname_s = unsafe { std::ffi::CStr::from_ptr(opname) }.to_string_lossy();
        let left_s = unsafe { std::ffi::CStr::from_ptr(left) }.to_string_lossy();
        let right_s = unsafe { std::ffi::CStr::from_ptr(right) }.to_string_lossy();
        let _ = unsafe {
            luaL_error(
                state,
                &format!("attempt to {opname_s} a '{left_s}' with a '{right_s}'"),
            )
        };
    }
    unsafe { lua_insert(state, -3) };
    unsafe { lua_call(state, 2, 1) };
}

unsafe fn arith(state: *mut lua_State, op: c_int, mtname: &'static [u8]) -> c_int {
    if unsafe { tonum(state, 1) } && unsafe { tonum(state, 2) } {
        unsafe { lua_arith(state, op) };
    } else {
        unsafe { trymt(state, mtname) };
    }
    1
}

#[inline]
unsafe fn error_str(state: *mut lua_State, msg: &'static [u8]) {
    let _ = unsafe { luaL_error_str(state, msg.as_ptr().cast::<c_char>()) };
}

unsafe fn check_capture(ms: &mut MatchState, l: c_int) -> c_int {
    let l = l - c_int::from(b'1');
    if l < 0 || l >= ms.level || ms.capture[l as usize].len == CAP_UNFINISHED {
        let _ = unsafe { luaL_error(ms.state, &format!("invalid capture index %{}", l + 1)) };
    }
    l
}

unsafe fn capture_to_close(ms: &mut MatchState) -> c_int {
    let mut level = ms.level - 1;
    while level >= 0 {
        if ms.capture[level as usize].len == CAP_UNFINISHED {
            return level;
        }
        level -= 1;
    }
    unsafe { error_str(ms.state, ERR_INVALID_PATTERN_CAPTURE) };
    0
}

unsafe fn classend(ms: &mut MatchState, mut p: *const u8) -> *const u8 {
    let ch = unsafe { *p };
    p = unsafe { p.add(1) };
    match ch {
        L_ESC => {
            if p == ms.p_end {
                unsafe { error_str(ms.state, ERR_MALFORMED_PATTERN_ENDS_WITH_ESCAPE) };
            }
            unsafe { p.add(1) }
        }
        b'[' => {
            if p < ms.p_end && unsafe { *p } == b'^' {
                p = unsafe { p.add(1) };
            }
            loop {
                if p == ms.p_end {
                    unsafe { error_str(ms.state, ERR_MALFORMED_PATTERN_MISSING_BRACKET) };
                }
                let ch = unsafe { *p };
                p = unsafe { p.add(1) };
                if ch == L_ESC && p < ms.p_end {
                    p = unsafe { p.add(1) };
                }
                if p < ms.p_end && unsafe { *p } == b']' {
                    return unsafe { p.add(1) };
                }
            }
        }
        _ => p,
    }
}

fn match_class(c: c_int, cl: c_int) -> bool {
    let class = cl as u8;
    let res = match class.to_ascii_lowercase() {
        b'a' => isalpha(c) != 0,
        b'c' => iscntrl(c) != 0,
        b'd' => isdigit(c) != 0,
        b'g' => isgraph(c) != 0,
        b'l' => islower(c) != 0,
        b'p' => ispunct(c) != 0,
        b's' => isspace(c) != 0,
        b'u' => isupper(c) != 0,
        b'w' => isalnum(c) != 0,
        b'x' => isxdigit(c) != 0,
        b'z' => c == 0,
        _ => cl == c,
    };
    if class.is_ascii_uppercase() {
        !res
    } else {
        res
    }
}

unsafe fn matchbracketclass(c: c_int, mut p: *const u8, ec: *const u8) -> bool {
    let mut sig = true;
    if unsafe { *p.add(1) } == b'^' {
        sig = false;
        p = unsafe { p.add(1) };
    }
    p = unsafe { p.add(1) };
    while p < ec {
        let ch = unsafe { *p };
        if ch == L_ESC {
            p = unsafe { p.add(1) };
            if match_class(c, c_int::from(unsafe { *p })) {
                return sig;
            }
        } else if unsafe { p.add(2) < ec } && unsafe { *p.add(1) } == b'-' {
            let start = ch;
            p = unsafe { p.add(2) };
            let end = unsafe { *p };
            if c_int::from(start) <= c && c <= c_int::from(end) {
                return sig;
            }
        } else if c_int::from(ch) == c {
            return sig;
        }
        p = unsafe { p.add(1) };
    }
    !sig
}

unsafe fn singlematch(ms: &mut MatchState, s: *const u8, p: *const u8, ep: *const u8) -> bool {
    if s >= ms.src_end {
        return false;
    }
    let c = c_int::from(unsafe { *s });
    match unsafe { *p } {
        b'.' => true,
        L_ESC => match_class(c, c_int::from(unsafe { *p.add(1) })),
        b'[' => unsafe { matchbracketclass(c, p, ep.sub(1)) },
        x => c_int::from(x) == c,
    }
}

unsafe fn matchbalance(ms: &mut MatchState, mut s: *const u8, p: *const u8) -> *const u8 {
    if p >= unsafe { ms.p_end.sub(1) } {
        unsafe { error_str(ms.state, ERR_MALFORMED_PATTERN_MISSING_BALANCE_ARGS) };
    }
    if s >= ms.src_end || unsafe { *s } != unsafe { *p } {
        return ptr::null();
    }
    let b = unsafe { *p };
    let e = unsafe { *p.add(1) };
    let mut cont = 1;
    s = unsafe { s.add(1) };
    while s < ms.src_end {
        let ch = unsafe { *s };
        if ch == e {
            cont -= 1;
            if cont == 0 {
                return unsafe { s.add(1) };
            }
        } else if ch == b {
            cont += 1;
        }
        s = unsafe { s.add(1) };
    }
    ptr::null()
}

unsafe fn max_expand(ms: &mut MatchState, s: *const u8, p: *const u8, ep: *const u8) -> *const u8 {
    let mut i: usize = 0;
    while unsafe { singlematch(ms, s.add(i), p, ep) } {
        i += 1;
    }
    loop {
        let res = unsafe { match_impl(ms, s.add(i), ep.add(1)) };
        if !res.is_null() {
            return res;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    ptr::null()
}

unsafe fn min_expand(
    ms: &mut MatchState,
    mut s: *const u8,
    p: *const u8,
    ep: *const u8,
) -> *const u8 {
    loop {
        let res = unsafe { match_impl(ms, s, ep.add(1)) };
        if !res.is_null() {
            return res;
        }
        if unsafe { singlematch(ms, s, p, ep) } {
            s = unsafe { s.add(1) };
        } else {
            return ptr::null();
        }
    }
}

unsafe fn start_capture(ms: &mut MatchState, s: *const u8, p: *const u8, what: isize) -> *const u8 {
    let level = ms.level as usize;
    if level >= LUA_MAXCAPTURES {
        unsafe { error_str(ms.state, ERR_TOO_MANY_CAPTURES) };
    }
    ms.capture[level].init = s;
    ms.capture[level].len = what;
    ms.level += 1;
    let res = unsafe { match_impl(ms, s, p) };
    if res.is_null() {
        ms.level -= 1;
    }
    res
}

unsafe fn end_capture(ms: &mut MatchState, s: *const u8, p: *const u8) -> *const u8 {
    let l = unsafe { capture_to_close(ms) } as usize;
    ms.capture[l].len = unsafe { s.offset_from(ms.capture[l].init) as isize };
    let res = unsafe { match_impl(ms, s, p) };
    if res.is_null() {
        ms.capture[l].len = CAP_UNFINISHED;
    }
    res
}

unsafe fn match_capture(ms: &mut MatchState, s: *const u8, l: c_int) -> *const u8 {
    let l = unsafe { check_capture(ms, l) } as usize;
    let len = ms.capture[l].len as usize;
    if unsafe { ms.src_end.offset_from(s) as usize } >= len {
        let cap = unsafe { core::slice::from_raw_parts(ms.capture[l].init, len) };
        let cur = unsafe { core::slice::from_raw_parts(s, len) };
        if cap == cur {
            return unsafe { s.add(len) };
        }
    }
    ptr::null()
}

unsafe fn match_impl(ms: &mut MatchState, mut s: *const u8, mut p: *const u8) -> *const u8 {
    if ms.matchdepth == 0 {
        unsafe { error_str(ms.state, ERR_PATTERN_TOO_COMPLEX) };
    }
    ms.matchdepth -= 1;
    while p != ms.p_end {
        match unsafe { *p } {
            b'(' => {
                s = if unsafe { *p.add(1) } == b')' {
                    unsafe { start_capture(ms, s, p.add(2), CAP_POSITION) }
                } else {
                    unsafe { start_capture(ms, s, p.add(1), CAP_UNFINISHED) }
                };
                break;
            }
            b')' => {
                s = unsafe { end_capture(ms, s, p.add(1)) };
                break;
            }
            b'$' if unsafe { p.add(1) } == ms.p_end => {
                s = if s == ms.src_end { s } else { ptr::null() };
                break;
            }
            L_ESC => match unsafe { *p.add(1) } {
                b'b' => {
                    s = unsafe { matchbalance(ms, s, p.add(2)) };
                    if !s.is_null() {
                        p = unsafe { p.add(4) };
                        continue;
                    }
                    break;
                }
                b'f' => {
                    p = unsafe { p.add(2) };
                    if p == ms.p_end || unsafe { *p } != b'[' {
                        unsafe { error_str(ms.state, ERR_MISSING_FRONTIER_SET) };
                    }
                    let ep = unsafe { classend(ms, p) };
                    let previous = if s == ms.src_init {
                        0
                    } else {
                        unsafe { *s.sub(1) }
                    };
                    let current = if s < ms.src_end { unsafe { *s } } else { 0 };
                    if !unsafe { matchbracketclass(c_int::from(previous), p, ep.sub(1)) }
                        && unsafe { matchbracketclass(c_int::from(current), p, ep.sub(1)) }
                    {
                        p = ep;
                        continue;
                    }
                    s = ptr::null();
                    break;
                }
                b'0'..=b'9' => {
                    s = unsafe { match_capture(ms, s, c_int::from(*p.add(1))) };
                    if !s.is_null() {
                        p = unsafe { p.add(2) };
                        continue;
                    }
                    break;
                }
                _ => {}
            },
            _ => {}
        }

        let ep = unsafe { classend(ms, p) };
        if !unsafe { singlematch(ms, s, p, ep) } {
            let suf = if ep < ms.p_end { unsafe { *ep } } else { 0 };
            if suf == b'*' || suf == b'?' || suf == b'-' {
                p = unsafe { ep.add(1) };
                continue;
            }
            s = ptr::null();
            break;
        } else {
            match if ep < ms.p_end { unsafe { *ep } } else { 0 } {
                b'?' => {
                    let res = unsafe { match_impl(ms, s.add(1), ep.add(1)) };
                    if !res.is_null() {
                        s = res;
                    } else {
                        p = unsafe { ep.add(1) };
                        continue;
                    }
                }
                b'+' => {
                    s = unsafe { s.add(1) };
                    s = unsafe { max_expand(ms, s, p, ep) };
                }
                b'*' => {
                    s = unsafe { max_expand(ms, s, p, ep) };
                }
                b'-' => {
                    s = unsafe { min_expand(ms, s, p, ep) };
                }
                _ => {
                    s = unsafe { s.add(1) };
                    p = ep;
                    continue;
                }
            }
            break;
        }
    }
    ms.matchdepth += 1;
    s
}

unsafe fn get_onecapture(
    ms: &mut MatchState,
    i: c_int,
    s: *const u8,
    e: *const u8,
    cap: &mut *const u8,
) -> isize {
    if i >= ms.level {
        if i != 0 {
            let _ = unsafe { luaL_error(ms.state, &format!("invalid capture index %{}", i + 1)) };
        }
        *cap = s;
        return unsafe { e.offset_from(s) as isize };
    }
    let capture = ms.capture[i as usize];
    *cap = capture.init;
    if capture.len == CAP_UNFINISHED {
        unsafe { error_str(ms.state, ERR_UNFINISHED_CAPTURE) };
    } else if capture.len == CAP_POSITION {
        unsafe {
            lua_pushinteger(
                ms.state,
                capture.init.offset_from(ms.src_init) as lua_Integer + 1,
            )
        };
    }
    capture.len
}

unsafe fn push_onecapture(ms: &mut MatchState, i: c_int, s: *const u8, e: *const u8) {
    let mut cap = ptr::null();
    let l = unsafe { get_onecapture(ms, i, s, e, &mut cap) };
    if l != CAP_POSITION {
        unsafe { lua_pushlstring(ms.state, cap.cast(), l as usize) };
    }
}

unsafe fn push_captures(ms: &mut MatchState, s: *const u8, e: *const u8) -> c_int {
    let nlevels = if ms.level == 0 && !s.is_null() {
        1
    } else {
        ms.level
    };
    {
        luaL_checkstack(
            ms.state,
            nlevels,
            ERR_TOO_MANY_CAPTURES_RESULTS.as_ptr().cast(),
        )
    };
    for i in 0..nlevels {
        unsafe { push_onecapture(ms, i, s, e) };
    }
    nlevels
}

fn nospecials(p: &[u8]) -> bool {
    let mut upto = 0;
    while upto <= p.len() {
        let rest = &p[upto..];
        let next_nul = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
        if rest[..next_nul].iter().any(|ch| SPECIALS.contains(ch)) {
            return false;
        }
        upto += next_nul + 1;
        if next_nul == rest.len() {
            break;
        }
    }
    true
}

fn prepstate(
    ms: &mut MatchState,
    state: *mut lua_State,
    s: *const u8,
    ls: usize,
    p: *const u8,
    lp: usize,
) {
    ms.state = state;
    ms.matchdepth = MAXCCALLS;
    ms.src_init = s;
    ms.src_end = unsafe { s.add(ls) };
    ms.p_end = unsafe { p.add(lp) };
}

fn reprepstate(ms: &mut MatchState) {
    ms.level = 0;
}

unsafe fn lmemfind(s1: *const u8, l1: usize, s2: *const u8, l2: usize) -> *const u8 {
    if l2 == 0 {
        return s1;
    }
    if l2 > l1 {
        return ptr::null();
    }
    let hay = unsafe { core::slice::from_raw_parts(s1, l1) };
    let needle = unsafe { core::slice::from_raw_parts(s2, l2) };
    for i in 0..=l1 - l2 {
        if &hay[i..i + l2] == needle {
            return unsafe { s1.add(i) };
        }
    }
    ptr::null()
}

unsafe fn str_find_aux(state: *mut lua_State, find: bool) -> c_int {
    let mut ls = 0;
    let mut lp = 0;
    let s = { luaL_checklstring(state, 1, &mut ls) }.cast::<u8>();
    let p = { luaL_checklstring(state, 2, &mut lp) }.cast::<u8>();
    let init = posrelat_i(luaL_optinteger(state, 3, 1), ls) - 1;
    if init > ls {
        unsafe { push_fail(state) };
        return 1;
    }
    if find
        && (unsafe { lua_toboolean(state, 4) } != 0
            || nospecials(unsafe { core::slice::from_raw_parts(p, lp) }))
    {
        let s2 = unsafe { lmemfind(s.add(init), ls - init, p, lp) };
        if !s2.is_null() {
            unsafe { lua_pushinteger(state, s2.offset_from(s) as lua_Integer + 1) };
            unsafe { lua_pushinteger(state, s2.offset_from(s) as lua_Integer + lp as lua_Integer) };
            return 2;
        }
    } else {
        let mut ms = MatchState {
            src_init: ptr::null(),
            src_end: ptr::null(),
            p_end: ptr::null(),
            state,
            matchdepth: MAXCCALLS,
            level: 0,
            capture: [Capture::default(); LUA_MAXCAPTURES],
        };
        let mut s1 = unsafe { s.add(init) };
        let anchor = lp > 0 && unsafe { *p } == b'^';
        let (pstart, lpstart) = if anchor {
            (unsafe { p.add(1) }, lp - 1)
        } else {
            (p, lp)
        };
        prepstate(&mut ms, state, s, ls, pstart, lpstart);
        loop {
            reprepstate(&mut ms);
            let res = unsafe { match_impl(&mut ms, s1, pstart) };
            if !res.is_null() {
                if find {
                    unsafe { lua_pushinteger(state, s1.offset_from(s) as lua_Integer + 1) };
                    unsafe { lua_pushinteger(state, res.offset_from(s) as lua_Integer) };
                    return unsafe { push_captures(&mut ms, ptr::null(), ptr::null()) + 2 };
                }
                return unsafe { push_captures(&mut ms, s1, res) };
            }
            if s1 >= ms.src_end || anchor {
                break;
            }
            s1 = unsafe { s1.add(1) };
        }
    }
    unsafe { push_fail(state) };
    1
}

unsafe fn add_s(ms: &mut MatchState, out: &mut Vec<u8>, s: *const u8, e: *const u8) {
    let mut news_len = 0;
    let news = unsafe { lua_tolstring(ms.state, 3, &mut news_len) }.cast::<u8>();
    let news_slice = unsafe { core::slice::from_raw_parts(news, news_len) };
    let mut i = 0;
    while i < news_slice.len() {
        if news_slice[i] != L_ESC {
            out.push(news_slice[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= news_slice.len() {
            let _ = unsafe {
                luaL_error(
                    ms.state,
                    &format!("invalid use of '{}' in replacement string", L_ESC as char),
                )
            };
        }
        let ch = news_slice[i];
        match ch {
            L_ESC => out.push(ch),
            b'0' => {
                let len = unsafe { e.offset_from(s) as usize };
                out.extend_from_slice(unsafe { core::slice::from_raw_parts(s, len) });
            }
            b'1'..=b'9' => {
                let mut cap = ptr::null();
                let resl = unsafe { get_onecapture(ms, c_int::from(ch - b'1'), s, e, &mut cap) };
                if resl == CAP_POSITION {
                    let mut len = 0;
                    let ptr = unsafe { lua_tolstring(ms.state, -1, &mut len) }.cast::<u8>();
                    out.extend_from_slice(unsafe { core::slice::from_raw_parts(ptr, len) });
                    unsafe { lua_pop(ms.state, 1) };
                } else {
                    out.extend_from_slice(unsafe {
                        core::slice::from_raw_parts(cap, resl as usize)
                    });
                }
            }
            _ => {
                let _ = unsafe {
                    luaL_error(
                        ms.state,
                        &format!("invalid use of '{}' in replacement string", L_ESC as char),
                    )
                };
            }
        }
        i += 1;
    }
}

unsafe fn add_value(
    ms: &mut MatchState,
    out: &mut Vec<u8>,
    s: *const u8,
    e: *const u8,
    tr: c_int,
) -> bool {
    match tr as u8 {
        LUA_TFUNCTION => {
            unsafe { lua_pushvalue(ms.state, 3) };
            let n = unsafe { push_captures(ms, s, e) };
            unsafe { lua_call(ms.state, n, 1) };
        }
        LUA_TTABLE => {
            unsafe { push_onecapture(ms, 0, s, e) };
            let _ = unsafe { lua_gettable(ms.state, 3) };
        }
        _ => {
            unsafe { add_s(ms, out, s, e) };
            return true;
        }
    }
    if unsafe { lua_toboolean(ms.state, -1) } == 0 {
        unsafe { lua_pop(ms.state, 1) };
        out.extend_from_slice(unsafe { core::slice::from_raw_parts(s, e.offset_from(s) as usize) });
        false
    } else if unsafe { lua_isstring(ms.state, -1) } == 0 {
        let tname =
            unsafe { std::ffi::CStr::from_ptr(lua_typename(ms.state, lua_type(ms.state, -1))) }
                .to_string_lossy();
        let _ = unsafe { luaL_error(ms.state, &format!("invalid replacement value (a {tname})")) };
        false
    } else {
        let mut len = 0;
        let ptr = unsafe { lua_tolstring(ms.state, -1, &mut len) }.cast::<u8>();
        out.extend_from_slice(unsafe { core::slice::from_raw_parts(ptr, len) });
        unsafe { lua_pop(ms.state, 1) };
        true
    }
}

#[inline]
fn native_is_little() -> bool {
    cfg!(target_endian = "little")
}

#[inline]
fn digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn getnum(fmt: &mut *const u8, default: usize) -> usize {
    if unsafe { !digit(**fmt) } {
        return default;
    }
    let mut a = 0usize;
    while unsafe { digit(**fmt) } && a <= (MAX_SIZE - 9) / 10 {
        a = a * 10 + usize::from(unsafe { **fmt - b'0' });
        *fmt = unsafe { (*fmt).add(1) };
    }
    a
}

unsafe fn getnumlimit(h: &Header, fmt: &mut *const u8, default: usize) -> usize {
    let sz = getnum(fmt, default);
    if sz == 0 || sz > MAXINTSIZE {
        let _ = unsafe {
            luaL_error(
                h.state,
                &format!("integral size ({sz}) out of limits [1,{MAXINTSIZE}]"),
            )
        };
    }
    sz
}

fn initheader(state: *mut lua_State) -> Header {
    Header {
        state,
        islittle: native_is_little(),
        maxalign: 1,
    }
}

unsafe fn getoption(h: &mut Header, fmt: &mut *const u8, size: &mut usize) -> KOption {
    let opt = unsafe { **fmt };
    *fmt = unsafe { (*fmt).add(1) };
    *size = 0;
    match opt {
        b'b' => {
            *size = core::mem::size_of::<i8>();
            KOption::Kint
        }
        b'B' => {
            *size = core::mem::size_of::<u8>();
            KOption::Kuint
        }
        b'h' => {
            *size = core::mem::size_of::<i16>();
            KOption::Kint
        }
        b'H' => {
            *size = core::mem::size_of::<u16>();
            KOption::Kuint
        }
        b'l' => {
            *size = core::mem::size_of::<i64>();
            KOption::Kint
        }
        b'L' => {
            *size = core::mem::size_of::<u64>();
            KOption::Kuint
        }
        b'j' => {
            *size = core::mem::size_of::<lua_Integer>();
            KOption::Kint
        }
        b'J' => {
            *size = core::mem::size_of::<lua_Integer>();
            KOption::Kuint
        }
        b'T' => {
            *size = core::mem::size_of::<usize>();
            KOption::Kuint
        }
        b'f' => {
            *size = core::mem::size_of::<f32>();
            KOption::Kfloat
        }
        b'n' => {
            *size = core::mem::size_of::<lua_Number>();
            KOption::Knumber
        }
        b'd' => {
            *size = core::mem::size_of::<f64>();
            KOption::Kdouble
        }
        b'i' => {
            *size = unsafe { getnumlimit(h, fmt, core::mem::size_of::<c_int>()) };
            KOption::Kint
        }
        b'I' => {
            *size = unsafe { getnumlimit(h, fmt, core::mem::size_of::<c_int>()) };
            KOption::Kuint
        }
        b's' => {
            *size = unsafe { getnumlimit(h, fmt, core::mem::size_of::<usize>()) };
            KOption::Kstring
        }
        b'c' => {
            *size = getnum(fmt, usize::MAX);
            if *size == usize::MAX {
                unsafe { error_str(h.state, ERR_MISSING_SIZE_FOR_C) };
            }
            KOption::Kchar
        }
        b'z' => KOption::Kzstr,
        b'x' => {
            *size = 1;
            KOption::Kpadding
        }
        b'X' => KOption::Kpaddalign,
        b' ' => KOption::Knop,
        b'<' => {
            h.islittle = true;
            KOption::Knop
        }
        b'>' => {
            h.islittle = false;
            KOption::Knop
        }
        b'=' => {
            h.islittle = native_is_little();
            KOption::Knop
        }
        b'!' => {
            h.maxalign = unsafe { getnumlimit(h, fmt, core::mem::align_of::<u128>()) };
            KOption::Knop
        }
        _ => {
            let _ =
                unsafe { luaL_error(h.state, &format!("invalid format option '{}'", opt as char)) };
            KOption::Knop
        }
    }
}

fn ispow2(v: usize) -> bool {
    v != 0 && (v & (v - 1)) == 0
}

unsafe fn getdetails(
    h: &mut Header,
    totalsize: usize,
    fmt: &mut *const u8,
    psize: &mut usize,
    ntoalign: &mut usize,
) -> KOption {
    let opt = unsafe { getoption(h, fmt, psize) };
    let mut align = *psize;
    if matches!(opt, KOption::Kpaddalign) {
        if unsafe { **fmt } == 0
            || matches!(unsafe { getoption(h, fmt, &mut align) }, KOption::Kchar)
            || align == 0
        {
            let _ = { luaL_typeerror(h.state, 1, ERR_INVALID_NEXT_OPTION_FOR_X.as_ptr().cast()) };
        }
    }
    if align <= 1 || matches!(opt, KOption::Kchar) {
        *ntoalign = 0;
    } else {
        if align > h.maxalign {
            align = h.maxalign;
        }
        if !ispow2(align) {
            unsafe { error_str(h.state, ERR_ALIGNMENT_NOT_POWER_OF_2) };
        }
        let szmoda = totalsize & (align - 1);
        *ntoalign = (align - szmoda) & (align - 1);
    }
    opt
}

fn packint(out: &mut Vec<u8>, mut n: lua_Unsigned, islittle: bool, size: usize, neg: bool) {
    let start = out.len();
    out.resize(start + size, 0);
    out[start + if islittle { 0 } else { size - 1 }] = (n & u64::from(MC)) as u8;
    for i in 1..size {
        n >>= NB;
        out[start + if islittle { i } else { size - 1 - i }] = (n & u64::from(MC)) as u8;
    }
    if neg && size > SZINT {
        for i in SZINT..size {
            out[start + if islittle { i } else { size - 1 - i }] = MC;
        }
    }
}

fn copywithendian(dest: &mut [u8], src: &[u8], islittle: bool) {
    if islittle == native_is_little() {
        dest.copy_from_slice(src);
    } else {
        for (d, s) in dest.iter_mut().zip(src.iter().rev()) {
            *d = *s;
        }
    }
}

fn addquoted(out: &mut Vec<u8>, s: &[u8]) {
    out.push(b'"');
    for (idx, &ch) in s.iter().enumerate() {
        if ch == b'"' || ch == b'\\' || ch == b'\n' {
            out.push(b'\\');
            out.push(ch);
        } else if ch.is_ascii_control() {
            let next_is_digit = s.get(idx + 1).is_some_and(|b| b.is_ascii_digit());
            let esc = if next_is_digit {
                format!("\\{:03}", ch)
            } else {
                format!("\\{}", ch)
            };
            out.extend_from_slice(esc.as_bytes());
        } else {
            out.push(ch);
        }
    }
    out.push(b'"');
}

unsafe fn addliteral(state: *mut lua_State, out: &mut Vec<u8>, arg: c_int) {
    match unsafe { lua_type(state, arg) as u8 } {
        LUA_TSTRING => {
            let mut len = 0;
            let s = unsafe { lua_tolstring(state, arg, &mut len) }.cast::<u8>();
            addquoted(out, unsafe { core::slice::from_raw_parts(s, len) });
        }
        LUA_TNUMBER => {
            if unsafe { lua_isinteger(state, arg) } == 0 {
                let n = unsafe { lua_tonumberx(state, arg, ptr::null_mut()) };
                if n.is_infinite() {
                    let text = if n.is_sign_positive() {
                        "1e9999"
                    } else {
                        "-1e9999"
                    };
                    out.extend_from_slice(text.as_bytes());
                } else if n.is_nan() {
                    out.extend_from_slice(b"(0/0)");
                } else {
                    format_hex_float(n, None, false, out);
                }
            } else {
                let n = unsafe { lua_tointegerx(state, arg, ptr::null_mut()) };
                out.extend_from_slice(format!("{n}").as_bytes());
            }
        }
        LUA_TNIL | LUA_TBOOLEAN => {
            let mut len = 0;
            let s = luaL_tolstring(state, arg, &mut len).cast::<u8>();
            out.extend_from_slice(unsafe { core::slice::from_raw_parts(s, len) });
            unsafe { lua_pop(state, 1) };
        }
        _ => {
            let _ = { luaL_typeerror(state, arg, ERR_VALUE_HAS_NO_LITERAL_FORM.as_ptr().cast()) };
        }
    }
}

fn get2digits(mut idx: usize, bytes: &[u8]) -> usize {
    if idx < bytes.len() && bytes[idx].is_ascii_digit() {
        idx += 1;
        if idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
    }
    idx
}

unsafe fn checkformat(state: *mut lua_State, form: &[u8], flags: &[u8], precision: bool) {
    let mut spec = 1usize;
    while spec < form.len() && flags.contains(&form[spec]) {
        spec += 1;
    }
    if spec < form.len() && form[spec] != b'0' {
        spec = get2digits(spec, form);
        if spec < form.len() && form[spec] == b'.' && precision {
            spec += 1;
            spec = get2digits(spec, form);
        }
    }
    if spec >= form.len() || !(form[spec] as char).is_ascii_alphabetic() {
        let form_s = core::str::from_utf8(form).unwrap_or("?");
        let _ = unsafe {
            luaL_error(
                state,
                &format!("invalid conversion specification: '{form_s}'"),
            )
        };
    }
}

unsafe fn getformat(state: *mut lua_State, strfrmt: &[u8]) -> (Vec<u8>, usize) {
    let mut len = 0usize;
    while len < strfrmt.len()
        && (L_FMTFLAGSF.contains(&strfrmt[len])
            || strfrmt[len].is_ascii_digit()
            || strfrmt[len] == b'.')
    {
        len += 1;
    }
    len += 1;
    if len >= MAX_FORMAT - 10 {
        unsafe { error_str(state, ERR_INVALID_FORMAT_TOO_LONG) };
    }
    let mut form = Vec::with_capacity(len + 2);
    form.push(b'%');
    form.extend_from_slice(&strfrmt[..len]);
    form.push(0);
    (form, len)
}

pub(crate) unsafe fn str_len(state: *mut lua_State) -> c_int {
    let mut len = 0;
    let _ = { luaL_checklstring(state, 1, &mut len) };
    unsafe { lua_pushinteger(state, len as lua_Integer) };
    1
}

pub(crate) unsafe fn str_sub(state: *mut lua_State) -> c_int {
    let mut len = 0;
    let s = { luaL_checklstring(state, 1, &mut len) }.cast::<u8>();
    let start = posrelat_i(luaL_checkinteger(state, 2), len);
    let end = unsafe { getendpos(state, 3, -1, len) };
    if start <= end {
        unsafe {
            lua_pushlstring(
                state,
                s.add(start - 1).cast(),
                (end - start).saturating_add(1),
            )
        };
    } else {
        unsafe { lua_pushlstring(state, c"".as_ptr(), 0) };
    }
    1
}

pub(crate) unsafe fn str_reverse(state: *mut lua_State) -> c_int {
    let mut len = 0;
    let s = { luaL_checklstring(state, 1, &mut len) }.cast::<u8>();
    let slice = unsafe { core::slice::from_raw_parts(s, len) };
    let mut out = Vec::with_capacity(len);
    out.extend(slice.iter().rev().copied());
    unsafe { lua_pushlstring(state, out.as_ptr().cast(), out.len()) };
    1
}

pub(crate) unsafe fn str_lower(state: *mut lua_State) -> c_int {
    let mut len = 0;
    let s = { luaL_checklstring(state, 1, &mut len) }.cast::<u8>();
    let slice = unsafe { core::slice::from_raw_parts(s, len) };
    let mut out = Vec::with_capacity(len);
    for &byte in slice {
        out.push(tolower(c_int::from(byte)) as c_uchar);
    }
    unsafe { lua_pushlstring(state, out.as_ptr().cast(), out.len()) };
    1
}

pub(crate) unsafe fn str_upper(state: *mut lua_State) -> c_int {
    let mut len = 0;
    let s = { luaL_checklstring(state, 1, &mut len) }.cast::<u8>();
    let slice = unsafe { core::slice::from_raw_parts(s, len) };
    let mut out = Vec::with_capacity(len);
    for &byte in slice {
        out.push(toupper(c_int::from(byte)) as c_uchar);
    }
    unsafe { lua_pushlstring(state, out.as_ptr().cast(), out.len()) };
    1
}

pub(crate) unsafe fn str_rep(state: *mut lua_State) -> c_int {
    let mut len = 0;
    let s = { luaL_checklstring(state, 1, &mut len) }.cast::<u8>();
    let n = { luaL_checkinteger(state, 2) };
    let mut sep_len = 0;
    let sep = { luaL_optlstring(state, 3, c"".as_ptr(), &mut sep_len) }.cast::<u8>();

    if n <= 0 {
        unsafe { lua_pushlstring(state, c"".as_ptr(), 0) };
        return 1;
    }

    let n_usize = n as usize;
    if len > MAX_SIZE.saturating_sub(sep_len) {
        return unsafe { runtime_error(state, ERR_RESULTING_STRING_TOO_LARGE) };
    }
    let unit = len + sep_len;
    if unit > 0 && unit > MAX_SIZE / n_usize {
        return unsafe { runtime_error(state, ERR_RESULTING_STRING_TOO_LARGE) };
    }

    let total_len = n_usize
        .checked_mul(unit)
        .and_then(|v| v.checked_sub(sep_len))
        .ok_or(())
        .map_err(|_| unsafe { runtime_error(state, ERR_RESULTING_STRING_TOO_LARGE) });
    let total_len = match total_len {
        Ok(v) => v,
        Err(err) => return err,
    };

    let slice = unsafe { core::slice::from_raw_parts(s, len) };
    let sep_slice = unsafe { core::slice::from_raw_parts(sep, sep_len) };
    let mut out = Vec::with_capacity(total_len);
    for idx in 0..n_usize {
        out.extend_from_slice(slice);
        if idx + 1 != n_usize && !sep_slice.is_empty() {
            out.extend_from_slice(sep_slice);
        }
    }
    unsafe { lua_pushlstring(state, out.as_ptr().cast(), out.len()) };
    1
}

pub(crate) unsafe fn str_byte(state: *mut lua_State) -> c_int {
    let mut len = 0;
    let s = { luaL_checklstring(state, 1, &mut len) }.cast::<u8>();
    let pi = { luaL_optinteger(state, 2, 1) };
    let posi = posrelat_i(pi, len);
    let pose = unsafe { getendpos(state, 3, pi, len) };
    if posi > pose {
        return 0;
    }
    if pose - posi >= c_int::MAX as usize {
        return unsafe { runtime_error(state, ERR_STRING_SLICE_TOO_LONG) };
    }
    let n = (pose - posi + 1) as c_int;
    {
        luaL_checkstack(state, n, ERR_STRING_SLICE_TOO_LONG.as_ptr().cast())
    };
    for i in 0..n {
        unsafe { lua_pushinteger(state, *s.add(posi + i as usize - 1) as lua_Integer) };
    }
    n
}

pub(crate) unsafe fn str_char(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    let mut out = Vec::with_capacity(n.max(0) as usize);
    for i in 1..=n {
        let c = { luaL_checkinteger(state, i) };
        unsafe {
            argcheck(
                state,
                (0..=u8::MAX as lua_Integer).contains(&c),
                i,
                ERR_VALUE_OUT_OF_RANGE,
            )
        };
        out.push(c as u8);
    }
    let ptr = if out.is_empty() {
        ptr::null()
    } else {
        out.as_ptr().cast()
    };
    unsafe { lua_pushlstring(state, ptr, out.len()) };
    1
}

pub(crate) unsafe fn str_dump(state: *mut lua_State) -> c_int {
    let strip = unsafe { lua_toboolean(state, 2) };
    unsafe {
        argcheck(
            state,
            lua_type(state, 1) == LUA_TFUNCTION.into() && lua_iscfunction(state, 1) == 0,
            1,
            ERR_LUA_FUNCTION_EXPECTED,
        )
    };
    unsafe { lua_pushvalue(state, 1) };
    let mut writer = DumpWriterState::default();
    let _ = unsafe {
        lua_dump(
            state,
            Some(dump_writer),
            (&mut writer as *mut DumpWriterState).cast(),
            strip,
        )
    };
    unsafe { lua_settop(state, 0) };
    let ptr = if writer.bytes.is_empty() {
        null()
    } else {
        writer.bytes.as_ptr().cast()
    };
    unsafe { lua_pushlstring(state, ptr, writer.bytes.len()) };
    1
}

pub(crate) unsafe fn arith_add(state: *mut lua_State) -> c_int {
    unsafe { arith(state, LUA_OPADD, MT_ADD) }
}

pub(crate) unsafe fn arith_sub(state: *mut lua_State) -> c_int {
    unsafe { arith(state, LUA_OPSUB, MT_SUB) }
}

pub(crate) unsafe fn arith_mul(state: *mut lua_State) -> c_int {
    unsafe { arith(state, LUA_OPMUL, MT_MUL) }
}

pub(crate) unsafe fn arith_mod(state: *mut lua_State) -> c_int {
    unsafe { arith(state, LUA_OPMOD, MT_MOD) }
}

pub(crate) unsafe fn arith_pow(state: *mut lua_State) -> c_int {
    unsafe { arith(state, LUA_OPPOW, MT_POW) }
}

pub(crate) unsafe fn arith_div(state: *mut lua_State) -> c_int {
    unsafe { arith(state, LUA_OPDIV, MT_DIV) }
}

pub(crate) unsafe fn arith_idiv(state: *mut lua_State) -> c_int {
    unsafe { arith(state, LUA_OPIDIV, MT_IDIV) }
}

pub(crate) unsafe fn arith_unm(state: *mut lua_State) -> c_int {
    unsafe { arith(state, LUA_OPUNM, MT_UNM) }
}

unsafe fn gmatch_aux(state: *mut lua_State) -> c_int {
    let gm = unsafe { &mut *(lua_touserdata(state, lua_upvalueindex(3)) as *mut GMatchState) };
    gm.ms.state = state;
    let mut src = gm.src;
    while src <= gm.ms.src_end {
        reprepstate(&mut gm.ms);
        let e = unsafe { match_impl(&mut gm.ms, src, gm.p) };
        if !e.is_null() && e != gm.lastmatch {
            gm.src = e;
            gm.lastmatch = e;
            return unsafe { push_captures(&mut gm.ms, src, e) };
        }
        src = unsafe { src.add(1) };
    }
    0
}

pub(crate) unsafe fn str_find(state: *mut lua_State) -> c_int {
    unsafe { str_find_aux(state, true) }
}

pub(crate) unsafe fn str_match(state: *mut lua_State) -> c_int {
    unsafe { str_find_aux(state, false) }
}

pub(crate) unsafe fn gmatch(state: *mut lua_State) -> c_int {
    let mut ls = 0;
    let mut lp = 0;
    let s = { luaL_checklstring(state, 1, &mut ls) }.cast::<u8>();
    let p = { luaL_checklstring(state, 2, &mut lp) }.cast::<u8>();
    let mut init = posrelat_i(luaL_optinteger(state, 3, 1), ls) - 1;
    unsafe { lua_settop(state, 2) };
    let gm = unsafe {
        lua_newuserdatauv(state, core::mem::size_of::<GMatchState>(), 0) as *mut GMatchState
    };
    if init > ls {
        init = ls + 1;
    }
    unsafe {
        ptr::write(
            gm,
            GMatchState {
                src: s.add(init),
                p,
                lastmatch: ptr::null(),
                ms: MatchState {
                    src_init: ptr::null(),
                    src_end: ptr::null(),
                    p_end: ptr::null(),
                    state,
                    matchdepth: MAXCCALLS,
                    level: 0,
                    capture: [Capture::default(); LUA_MAXCAPTURES],
                },
            },
        )
    };
    prepstate(unsafe { &mut (*gm).ms }, state, s, ls, p, lp);
    unsafe { lua_pushcclosure(state, Some(gmatch_aux), 3) };
    1
}

pub(crate) unsafe fn str_gsub(state: *mut lua_State) -> c_int {
    let mut srcl = 0;
    let mut lp = 0;
    let mut src = { luaL_checklstring(state, 1, &mut srcl) }.cast::<u8>();
    let p = { luaL_checklstring(state, 2, &mut lp) }.cast::<u8>();
    let mut lastmatch = ptr::null();
    let tr = unsafe { lua_type(state, 3) };
    let max_s = { luaL_optinteger(state, 4, srcl as lua_Integer + 1) };
    let anchor = lp > 0 && unsafe { *p } == b'^';
    let mut n = 0;
    let mut changed = false;
    if tr != LUA_TNUMBER.into()
        && tr != LUA_TSTRING.into()
        && tr != LUA_TFUNCTION.into()
        && tr != LUA_TTABLE.into()
    {
        let _ = { luaL_typeerror(state, 3, ERR_EXPECTED_REPLACEMENT.as_ptr().cast()) };
    }
    let (pstart, lpstart) = if anchor {
        (unsafe { p.add(1) }, lp - 1)
    } else {
        (p, lp)
    };
    let mut ms = MatchState {
        src_init: ptr::null(),
        src_end: ptr::null(),
        p_end: ptr::null(),
        state,
        matchdepth: MAXCCALLS,
        level: 0,
        capture: [Capture::default(); LUA_MAXCAPTURES],
    };
    prepstate(&mut ms, state, src, srcl, pstart, lpstart);
    let mut out = Vec::new();
    while n < max_s {
        reprepstate(&mut ms);
        let e = unsafe { match_impl(&mut ms, src, pstart) };
        if !e.is_null() && e != lastmatch {
            n += 1;
            changed = unsafe { add_value(&mut ms, &mut out, src, e, tr) } || changed;
            src = e;
            lastmatch = e;
        } else if src < ms.src_end {
            out.push(unsafe { *src });
            src = unsafe { src.add(1) };
        } else {
            break;
        }
        if anchor {
            break;
        }
    }
    if !changed {
        unsafe { lua_pushvalue(state, 1) };
    } else {
        out.extend_from_slice(unsafe {
            core::slice::from_raw_parts(src, ms.src_end.offset_from(src) as usize)
        });
        let ptr = if out.is_empty() {
            null()
        } else {
            out.as_ptr().cast()
        };
        unsafe { lua_pushlstring(state, ptr, out.len()) };
    }
    unsafe { lua_pushinteger(state, n) };
    2
}

/// Parsed representation of a C-style format specifier's flags, width, and precision.
struct FmtSpec {
    left_align: bool,
    force_sign: bool,
    space_sign: bool,
    alt_form: bool,
    zero_pad: bool,
    width: Option<usize>,
    precision: Option<usize>,
}

impl FmtSpec {
    /// Parse flags, width, and precision from the format bytes between '%' and the specifier char.
    /// `form` is the full form slice including leading '%' and trailing specifier+NUL, e.g. b"%-10.3d\0".
    fn parse(form: &[u8]) -> Self {
        let mut spec = FmtSpec {
            left_align: false,
            force_sign: false,
            space_sign: false,
            alt_form: false,
            zero_pad: false,
            width: None,
            precision: None,
        };
        // Skip leading '%'
        let mut i = 1usize;
        // The form ends with specifier + NUL, so meaningful content is form[1..form.len()-2]
        let end = if form.len() >= 2 {
            form.len() - 2
        } else {
            return spec;
        };
        // Parse flags
        while i < end {
            match form[i] {
                b'-' => spec.left_align = true,
                b'+' => spec.force_sign = true,
                b' ' => spec.space_sign = true,
                b'#' => spec.alt_form = true,
                b'0' if spec.width.is_none() => spec.zero_pad = true,
                _ => break,
            }
            i += 1;
        }
        // Parse width
        if i < end && form[i].is_ascii_digit() {
            let start = i;
            while i < end && form[i].is_ascii_digit() {
                i += 1;
            }
            spec.width = Some(
                core::str::from_utf8(&form[start..i])
                    .unwrap()
                    .parse()
                    .unwrap(),
            );
        }
        // Parse precision
        if i < end && form[i] == b'.' {
            i += 1;
            let start = i;
            while i < end && form[i].is_ascii_digit() {
                i += 1;
            }
            if start == i {
                spec.precision = Some(0);
            } else {
                spec.precision = Some(
                    core::str::from_utf8(&form[start..i])
                        .unwrap()
                        .parse()
                        .unwrap(),
                );
            }
        }
        spec
    }

    /// Apply padding to a formatted string according to width/alignment/zero-pad settings.
    fn pad(&self, out: &mut Vec<u8>, formatted: &[u8]) {
        let w = match self.width {
            Some(w) if w > formatted.len() => w,
            _ => {
                out.extend_from_slice(formatted);
                return;
            }
        };
        let pad_len = w - formatted.len();
        if self.left_align {
            out.extend_from_slice(formatted);
            out.extend(core::iter::repeat_n(b' ', pad_len));
        } else if self.zero_pad && !formatted.is_empty() {
            // For zero-padding, keep sign/prefix before zeros
            let prefix_len = if formatted[0] == b'-' || formatted[0] == b'+' || formatted[0] == b' '
            {
                1
            } else if formatted.starts_with(b"0x") || formatted.starts_with(b"0X") {
                2
            } else {
                0
            };
            out.extend_from_slice(&formatted[..prefix_len]);
            out.extend(core::iter::repeat_n(b'0', pad_len));
            out.extend_from_slice(&formatted[prefix_len..]);
        } else {
            out.extend(core::iter::repeat_n(b' ', pad_len));
            out.extend_from_slice(formatted);
        }
    }
}

/// Format a signed integer with Rust formatting.
fn rust_fmt_int(spec: &FmtSpec, n: lua_Integer, out: &mut Vec<u8>) {
    use std::fmt::Write;
    let mut tmp = String::new();
    // Apply sign
    if n < 0 {
        write!(tmp, "{}", n).unwrap();
    } else if spec.force_sign {
        write!(tmp, "+{}", n).unwrap();
    } else if spec.space_sign {
        write!(tmp, " {}", n).unwrap();
    } else {
        write!(tmp, "{}", n).unwrap();
    }
    spec.pad(out, tmp.as_bytes());
}

/// Format an unsigned integer with various bases using Rust formatting.
fn rust_fmt_uint(spec: &FmtSpec, n: lua_Unsigned, base_spec: u8, out: &mut Vec<u8>) {
    use std::fmt::Write;
    let mut tmp = String::new();
    match base_spec {
        b'u' => write!(tmp, "{}", n).unwrap(),
        b'o' => {
            if spec.alt_form && n != 0 {
                write!(tmp, "0{:o}", n).unwrap();
            } else {
                write!(tmp, "{:o}", n).unwrap();
            }
        }
        b'x' => {
            if spec.alt_form && n != 0 {
                write!(tmp, "0x{:x}", n).unwrap();
            } else {
                write!(tmp, "{:x}", n).unwrap();
            }
        }
        b'X' => {
            if spec.alt_form && n != 0 {
                write!(tmp, "0X{:X}", n).unwrap();
            } else {
                write!(tmp, "{:X}", n).unwrap();
            }
        }
        _ => write!(tmp, "{}", n).unwrap(),
    }
    spec.pad(out, tmp.as_bytes());
}

/// Format a floating-point number using Rust formatting.
/// Handles %e, %E, %f, %g, %G specifiers.
fn rust_fmt_float(spec: &FmtSpec, n: lua_Number, float_spec: u8, out: &mut Vec<u8>) {
    use std::fmt::Write;
    let mut tmp = String::new();
    let prec = spec.precision.unwrap_or(6);

    // Build sign prefix
    let sign_prefix = if n.is_sign_negative() && !n.is_nan() {
        "" // negative sign is included by the formatting
    } else if spec.force_sign {
        "+"
    } else if spec.space_sign {
        " "
    } else {
        ""
    };

    match float_spec {
        b'f' => {
            if !sign_prefix.is_empty() && !n.is_sign_negative() {
                write!(tmp, "{}{:.prec$}", sign_prefix, n, prec = prec).unwrap();
            } else {
                write!(tmp, "{:.prec$}", n, prec = prec).unwrap();
            }
        }
        b'e' => {
            let formatted = format_scientific(n, prec, false);
            if !sign_prefix.is_empty() && !n.is_sign_negative() {
                tmp.push_str(sign_prefix);
            }
            tmp.push_str(&formatted);
        }
        b'E' => {
            let formatted = format_scientific(n, prec, true);
            if !sign_prefix.is_empty() && !n.is_sign_negative() {
                tmp.push_str(sign_prefix);
            }
            tmp.push_str(&formatted);
        }
        b'g' | b'G' => {
            let p = if prec == 0 { 1 } else { prec };
            let formatted = format_general(n, p, float_spec == b'G', spec.alt_form);
            if !sign_prefix.is_empty() && !n.is_sign_negative() {
                tmp.push_str(sign_prefix);
            }
            tmp.push_str(&formatted);
        }
        _ => {
            write!(tmp, "{}", n).unwrap();
        }
    }

    if spec.alt_form && matches!(float_spec, b'f' | b'e' | b'E') && !tmp.contains('.') {
        // Ensure decimal point is present for '#' flag
        tmp.push('.');
    }

    spec.pad(out, tmp.as_bytes());
}

/// Format a number in scientific notation (like C's %e/%E).
/// Uses Rust's `{:.prec$e}` for reliable precision, then reformats the exponent.
fn format_scientific(n: f64, prec: usize, upper: bool) -> String {
    if n == 0.0 {
        let sign = if n.is_sign_negative() { "-" } else { "" };
        let e_char = if upper { 'E' } else { 'e' };
        return if prec > 0 {
            format!("{}0.{}{}+00", sign, "0".repeat(prec), e_char)
        } else {
            format!("{}0{}+00", sign, e_char)
        };
    }
    if n.is_infinite() {
        return if n.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    if n.is_nan() {
        return "nan".to_string();
    }

    // Use Rust's built-in scientific notation for reliable precision
    let raw = format!("{:.prec$e}", n, prec = prec);
    // Rust produces e.g. "3.14e2" or "-1.23e-5"; reformat exponent to C style (e+02, e-05)
    if let Some(e_pos) = raw.rfind('e') {
        let mantissa_part = &raw[..e_pos];
        let exp_val: i32 = raw[e_pos + 1..].parse().unwrap_or(0);
        let e_char = if upper { 'E' } else { 'e' };
        let exp_sign = if exp_val >= 0 { '+' } else { '-' };
        let abs_exp = exp_val.unsigned_abs();
        format!("{}{}{}{:02}", mantissa_part, e_char, exp_sign, abs_exp)
    } else {
        raw
    }
}

/// Format a number in general notation (like C's %g/%G).
/// Uses Rust's built-in formatting for reliable precision.
fn format_general(n: f64, sig_digits: usize, upper: bool, alt_form: bool) -> String {
    if n == 0.0 {
        let sign = if n.is_sign_negative() { "-" } else { "" };
        if alt_form && sig_digits > 1 {
            return format!("{}0.{}", sign, "0".repeat(sig_digits - 1));
        }
        return format!("{}0", sign);
    }
    if n.is_infinite() {
        return if n.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    if n.is_nan() {
        return if upper {
            "NAN".to_string()
        } else {
            "nan".to_string()
        };
    }

    // Use Rust's {:e} to determine the exponent reliably
    let sci = format!("{:.prec$e}", n, prec = sig_digits.saturating_sub(1));
    let exp = if let Some(e_pos) = sci.rfind('e') {
        sci[e_pos + 1..].parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    // %g uses %e if exponent < -4 or >= precision, otherwise %f
    if exp < -4 || exp >= sig_digits as i32 {
        let prec = if sig_digits > 1 { sig_digits - 1 } else { 0 };
        let mut result = format_scientific(n, prec, upper);
        if !alt_form {
            // Remove trailing zeros after decimal point in mantissa (before 'e'/'E')
            let e_marker = if upper { 'E' } else { 'e' };
            if let Some(e_pos) = result.find(e_marker) {
                let mantissa_part = &result[..e_pos];
                let exp_part = &result[e_pos..];
                if mantissa_part.contains('.') {
                    let trimmed = mantissa_part.trim_end_matches('0').trim_end_matches('.');
                    result = format!("{}{}", trimmed, exp_part);
                }
            }
        }
        result
    } else {
        // Use %f style
        let decimal_places = if sig_digits as i32 > exp + 1 {
            (sig_digits as i32 - exp - 1) as usize
        } else {
            0
        };
        let mut result = format!("{:.prec$}", n, prec = decimal_places);
        if !alt_form && result.contains('.') {
            // Remove trailing zeros
            let trimmed = result.trim_end_matches('0').trim_end_matches('.');
            result = trimmed.to_string();
        }
        result
    }
}

/// Format a hex float (%a/%A) using pure Rust.
fn format_hex_float(n: f64, prec: Option<usize>, upper: bool, out: &mut Vec<u8>) {
    if n.is_nan() {
        out.extend_from_slice(if upper { b"NAN" } else { b"nan" });
        return;
    }
    if n.is_infinite() {
        if n.is_sign_negative() {
            out.push(b'-');
        }
        out.extend_from_slice(if upper { b"INF" } else { b"inf" });
        return;
    }

    let bits = n.to_bits();
    let sign = (bits >> 63) != 0;
    let biased_exp = ((bits >> 52) & 0x7ff) as i64;
    let mantissa = bits & 0x000f_ffff_ffff_ffff;

    if sign {
        out.push(b'-');
    }
    out.extend_from_slice(if upper { b"0X" } else { b"0x" });

    if biased_exp == 0 && mantissa == 0 {
        // Zero
        out.push(b'0');
        if let Some(p) = prec {
            if p > 0 {
                out.push(b'.');
                out.extend(core::iter::repeat_n(b'0', p));
            }
        }
        out.extend_from_slice(if upper { b"P+0" } else { b"p+0" });
        return;
    }

    let (exp, full_mantissa) = if biased_exp == 0 {
        // Subnormal: exponent is -1022, no implicit leading 1
        (-1022i64, mantissa)
    } else {
        // Normal: exponent is biased_exp - 1023, implicit leading 1
        (biased_exp - 1023, mantissa | (1u64 << 52))
    };

    // Format as 1.xxxxx * 2^exp (or 0.xxxxx for subnormals)
    // The mantissa has 53 bits for normals (1 + 52 fraction bits)
    // We want to display as X.YYYYp+E where X is the leading hex digit
    let lead_digit = (full_mantissa >> 52) as u8;
    let frac_bits = full_mantissa & 0x000f_ffff_ffff_ffff;

    // The fraction part is 52 bits = 13 hex digits
    let hex_chars: &[u8] = if upper {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };

    let mut frac_hex = [0u8; 13];
    let mut tmp = frac_bits;
    for digit in frac_hex.iter_mut().rev() {
        *digit = hex_chars[(tmp & 0xf) as usize];
        tmp >>= 4;
    }

    // Determine how many hex digits to show
    let frac_digits = match prec {
        Some(p) => p,
        None => {
            // Default: show minimum needed (trim trailing zeros)
            let mut len = 13;
            while len > 0 && frac_hex[len - 1] == b'0' {
                len -= 1;
            }
            if len == 0 && frac_bits != 0 { 1 } else { len }
        }
    };

    // Write leading digit
    out.push(hex_chars[lead_digit as usize]);

    if frac_digits > 0 {
        out.push(b'.');
        let available = frac_hex.len().min(frac_digits);
        out.extend_from_slice(&frac_hex[..available]);
        // Pad with zeros if precision exceeds available digits
        if frac_digits > available {
            out.extend(core::iter::repeat_n(b'0', frac_digits - available));
        }
    }

    // Write exponent
    out.push(if upper { b'P' } else { b'p' });
    if exp >= 0 {
        out.push(b'+');
        out.extend_from_slice(format!("{}", exp).as_bytes());
    } else {
        out.extend_from_slice(format!("{}", exp).as_bytes());
    }
}

/// Format a string with width and precision (like C's %s with modifiers).
fn rust_fmt_str(spec: &FmtSpec, s: &[u8], out: &mut Vec<u8>) {
    // Precision truncates the string
    let data = match spec.precision {
        Some(p) if p < s.len() => &s[..p],
        _ => s,
    };
    spec.pad(out, data);
}

pub(crate) unsafe fn str_format(state: *mut lua_State) -> c_int {
    let top = unsafe { lua_gettop(state) };
    let mut arg = 1;
    let mut sfl = 0usize;
    let strfrmt = { luaL_checklstring(state, arg, &mut sfl) }.cast::<u8>();
    let bytes = unsafe { core::slice::from_raw_parts(strfrmt, sfl) };
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < bytes.len() {
        if bytes[i] != L_ESC {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i < bytes.len() && bytes[i] == L_ESC {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        arg += 1;
        if arg > top {
            let _ = { luaL_typeerror(state, arg, ERR_NO_VALUE.as_ptr().cast()) };
        }
        let (form, consumed) = unsafe { getformat(state, &bytes[i..]) };
        let spec_char = bytes[i + consumed - 1];
        i += consumed;
        let fmt = FmtSpec::parse(&form);
        match spec_char {
            b'c' => {
                unsafe { checkformat(state, &form[..form.len() - 1], L_FMTFLAGSC, false) };
                let n = luaL_checkinteger(state, arg) as u8;
                let ch = [n];
                fmt.pad(&mut out, &ch);
            }
            b'd' | b'i' => {
                unsafe { checkformat(state, &form[..form.len() - 1], L_FMTFLAGSI, true) };
                let n = luaL_checkinteger(state, arg);
                rust_fmt_int(&fmt, n, &mut out);
            }
            b'u' | b'o' | b'x' | b'X' => {
                let flags = match spec_char {
                    b'u' => L_FMTFLAGSU,
                    _ => L_FMTFLAGSX,
                };
                unsafe { checkformat(state, &form[..form.len() - 1], flags, true) };
                let n = luaL_checkinteger(state, arg) as lua_Unsigned;
                rust_fmt_uint(&fmt, n, spec_char, &mut out);
            }
            b'a' | b'A' => {
                unsafe { checkformat(state, &form[..form.len() - 1], L_FMTFLAGSF, true) };
                let n = luaL_checknumber(state, arg);
                format_hex_float(n, fmt.precision, spec_char == b'A', &mut out);
            }
            b'e' | b'E' | b'f' | b'g' | b'G' => {
                unsafe { checkformat(state, &form[..form.len() - 1], L_FMTFLAGSF, true) };
                let n = luaL_checknumber(state, arg);
                rust_fmt_float(&fmt, n, spec_char, &mut out);
            }
            b'p' => {
                unsafe { checkformat(state, &form[..form.len() - 1], L_FMTFLAGSC, false) };
                let p = unsafe { lua_topointer(state, arg) };
                if p.is_null() {
                    fmt.pad(&mut out, b"(null)");
                } else {
                    let formatted = format!("{:p}", p);
                    fmt.pad(&mut out, formatted.as_bytes());
                }
            }
            b'q' => {
                if form.len() != 3 {
                    unsafe { error_str(state, ERR_SPECIFIER_Q_MODIFIERS) };
                }
                unsafe { addliteral(state, &mut out, arg) };
            }
            b's' => {
                let mut l = 0usize;
                let s = { luaL_tolstring(state, arg, &mut l) }.cast::<u8>();
                if form.len() == 3 {
                    out.extend_from_slice(unsafe { core::slice::from_raw_parts(s, l) });
                    unsafe { lua_pop(state, 1) };
                } else {
                    let slice = unsafe { core::slice::from_raw_parts(s, l) };
                    unsafe { argcheck(state, !slice.contains(&0), arg, ERR_STRING_CONTAINS_ZEROS) };
                    unsafe { checkformat(state, &form[..form.len() - 1], L_FMTFLAGSC, true) };
                    if !form[..form.len() - 1].contains(&b'.') && l >= 100 {
                        out.extend_from_slice(slice);
                        unsafe { lua_pop(state, 1) };
                    } else {
                        rust_fmt_str(&fmt, slice, &mut out);
                        unsafe { lua_pop(state, 1) };
                    }
                }
            }
            _ => {
                let form_s = core::str::from_utf8(&form[..form.len() - 1]).unwrap_or("?");
                let _ = unsafe {
                    luaL_error(state, &format!("invalid conversion '{form_s}' to 'format'"))
                };
            }
        }
    }
    let ptr = if out.is_empty() {
        null()
    } else {
        out.as_ptr().cast()
    };
    unsafe { lua_pushlstring(state, ptr, out.len()) };
    1
}

fn unpackint(
    state: *mut lua_State,
    bytes: &[u8],
    islittle: bool,
    size: usize,
    issigned: bool,
) -> lua_Integer {
    let mut res: lua_Unsigned = 0;
    let limit = size.min(SZINT);
    for i in (0..limit).rev() {
        res <<= NB;
        res |= lua_Unsigned::from(bytes[if islittle { i } else { size - 1 - i }]);
    }
    if size < SZINT {
        if issigned {
            let mask = 1u64 << (size * NB - 1);
            res = (res ^ mask).wrapping_sub(mask);
        }
    } else if size > SZINT {
        let mask = if !issigned || (res as lua_Integer) >= 0 {
            0
        } else {
            MC
        };
        for i in limit..size {
            if bytes[if islittle { i } else { size - 1 - i }] != mask {
                let _ = unsafe {
                    luaL_error(
                        state,
                        &format!("{size}-byte integer does not fit into Lua Integer"),
                    )
                };
            }
        }
    }
    res as lua_Integer
}

pub(crate) unsafe fn str_pack(state: *mut lua_State) -> c_int {
    let mut fmt_len = 0usize;
    let fmt = { luaL_checklstring(state, 1, &mut fmt_len) }.cast::<u8>();
    let mut fmtp = fmt;
    let mut arg = 1;
    let mut totalsize = 0usize;
    let mut h = initheader(state);
    let mut out = Vec::new();
    while unsafe { *fmtp } != 0 {
        let mut ntoalign = 0usize;
        let mut size = 0usize;
        let opt = unsafe { getdetails(&mut h, totalsize, &mut fmtp, &mut size, &mut ntoalign) };
        unsafe {
            argcheck(
                state,
                size + ntoalign <= MAX_SIZE - totalsize,
                arg,
                ERR_RESULT_TOO_LONG,
            )
        };
        totalsize += ntoalign + size;
        out.extend(std::iter::repeat_n(LUAL_PACKPADBYTE, ntoalign));
        arg += 1;
        match opt {
            KOption::Kint => {
                let n = { luaL_checkinteger(state, arg) };
                if size < SZINT {
                    let lim = 1i64 << (size * NB - 1);
                    unsafe { argcheck(state, -lim <= n && n < lim, arg, ERR_INTEGER_OVERFLOW) };
                }
                packint(&mut out, n as lua_Unsigned, h.islittle, size, n < 0);
            }
            KOption::Kuint => {
                let n = { luaL_checkinteger(state, arg) };
                if size < SZINT {
                    unsafe {
                        argcheck(
                            state,
                            (n as lua_Unsigned) < (1u64 << (size * NB)),
                            arg,
                            ERR_UNSIGNED_OVERFLOW,
                        )
                    };
                }
                packint(&mut out, n as lua_Unsigned, h.islittle, size, false);
            }
            KOption::Kfloat => {
                let f = { luaL_checknumber(state, arg) } as f32;
                let mut bytes = [0u8; core::mem::size_of::<f32>()];
                copywithendian(&mut bytes, &f.to_ne_bytes(), h.islittle);
                out.extend_from_slice(&bytes);
            }
            KOption::Knumber => {
                let f = { luaL_checknumber(state, arg) };
                let mut bytes = [0u8; core::mem::size_of::<lua_Number>()];
                copywithendian(&mut bytes, &f.to_ne_bytes(), h.islittle);
                out.extend_from_slice(&bytes);
            }
            KOption::Kdouble => {
                let f = { luaL_checknumber(state, arg) };
                let mut bytes = [0u8; core::mem::size_of::<f64>()];
                copywithendian(&mut bytes, &f.to_ne_bytes(), h.islittle);
                out.extend_from_slice(&bytes);
            }
            KOption::Kchar => {
                let mut len = 0usize;
                let s = { luaL_checklstring(state, arg, &mut len) }.cast::<u8>();
                unsafe { argcheck(state, len <= size, arg, ERR_STRING_LONGER_THAN_GIVEN_SIZE) };
                out.extend_from_slice(unsafe { core::slice::from_raw_parts(s, len) });
                out.extend(std::iter::repeat_n(LUAL_PACKPADBYTE, size - len));
            }
            KOption::Kstring => {
                let mut len = 0usize;
                let s = { luaL_checklstring(state, arg, &mut len) }.cast::<u8>();
                unsafe {
                    argcheck(
                        state,
                        size >= core::mem::size_of::<lua_Unsigned>()
                            || len < (1usize << (size * NB)),
                        arg,
                        ERR_STRING_LENGTH_DOES_NOT_FIT,
                    )
                };
                packint(&mut out, len as lua_Unsigned, h.islittle, size, false);
                out.extend_from_slice(unsafe { core::slice::from_raw_parts(s, len) });
                totalsize += len;
            }
            KOption::Kzstr => {
                let mut len = 0usize;
                let s = { luaL_checklstring(state, arg, &mut len) }.cast::<u8>();
                let slice = unsafe { core::slice::from_raw_parts(s, len) };
                unsafe { argcheck(state, !slice.contains(&0), arg, ERR_STRING_CONTAINS_ZEROS) };
                out.extend_from_slice(slice);
                out.push(0);
                totalsize += len + 1;
            }
            KOption::Kpadding => out.push(LUAL_PACKPADBYTE),
            KOption::Kpaddalign | KOption::Knop => arg -= 1,
        }
    }
    let ptr = if out.is_empty() {
        null()
    } else {
        out.as_ptr().cast()
    };
    unsafe { lua_pushlstring(state, ptr, out.len()) };
    1
}

pub(crate) unsafe fn str_packsize(state: *mut lua_State) -> c_int {
    let mut fmt_len = 0usize;
    let fmt = { luaL_checklstring(state, 1, &mut fmt_len) }.cast::<u8>();
    let mut fmtp = fmt;
    let mut totalsize = 0usize;
    let mut h = initheader(state);
    while unsafe { *fmtp } != 0 {
        let mut ntoalign = 0usize;
        let mut size = 0usize;
        let opt = unsafe { getdetails(&mut h, totalsize, &mut fmtp, &mut size, &mut ntoalign) };
        unsafe {
            argcheck(
                state,
                !matches!(opt, KOption::Kstring | KOption::Kzstr),
                1,
                ERR_VARIABLE_LENGTH_FORMAT,
            )
        };
        let all = size + ntoalign;
        unsafe {
            argcheck(
                state,
                totalsize <= lua_Integer::MAX as usize - all,
                1,
                ERR_FORMAT_RESULT_TOO_LARGE,
            )
        };
        totalsize += all;
    }
    unsafe { lua_pushinteger(state, totalsize as lua_Integer) };
    1
}

pub(crate) unsafe fn str_unpack(state: *mut lua_State) -> c_int {
    let mut fmt_len = 0usize;
    let fmt = { luaL_checklstring(state, 1, &mut fmt_len) }.cast::<u8>();
    let mut data_len = 0usize;
    let data = { luaL_checklstring(state, 2, &mut data_len) }.cast::<u8>();
    let mut fmtp = fmt;
    let mut pos = posrelat_i(luaL_optinteger(state, 3, 1), data_len) - 1;
    let mut n = 0;
    unsafe {
        argcheck(
            state,
            pos <= data_len,
            3,
            ERR_INITIAL_POSITION_OUT_OF_STRING,
        )
    };
    let mut h = initheader(state);
    let data_slice = unsafe { core::slice::from_raw_parts(data, data_len) };
    while unsafe { *fmtp } != 0 {
        let mut ntoalign = 0usize;
        let mut size = 0usize;
        let opt = unsafe { getdetails(&mut h, pos, &mut fmtp, &mut size, &mut ntoalign) };
        unsafe {
            argcheck(
                state,
                ntoalign + size <= data_len - pos,
                2,
                ERR_DATA_STRING_TOO_SHORT,
            )
        };
        pos += ntoalign;
        {
            luaL_checkstack(state, 2, ERR_TOO_MANY_RESULTS.as_ptr().cast())
        };
        n += 1;
        match opt {
            KOption::Kint | KOption::Kuint => {
                let res = unpackint(
                    state,
                    &data_slice[pos..pos + size],
                    h.islittle,
                    size,
                    matches!(opt, KOption::Kint),
                );
                unsafe { lua_pushinteger(state, res) };
            }
            KOption::Kfloat => {
                let mut bytes = [0u8; core::mem::size_of::<f32>()];
                copywithendian(&mut bytes, &data_slice[pos..pos + size], h.islittle);
                unsafe { lua_pushnumber(state, f32::from_ne_bytes(bytes) as lua_Number) };
            }
            KOption::Knumber => {
                let mut bytes = [0u8; core::mem::size_of::<lua_Number>()];
                copywithendian(&mut bytes, &data_slice[pos..pos + size], h.islittle);
                unsafe { lua_pushnumber(state, lua_Number::from_ne_bytes(bytes)) };
            }
            KOption::Kdouble => {
                let mut bytes = [0u8; core::mem::size_of::<f64>()];
                copywithendian(&mut bytes, &data_slice[pos..pos + size], h.islittle);
                unsafe { lua_pushnumber(state, f64::from_ne_bytes(bytes) as lua_Number) };
            }
            KOption::Kchar => unsafe {
                lua_pushlstring(state, data.add(pos).cast(), size);
            },
            KOption::Kstring => {
                let len = unpackint(state, &data_slice[pos..pos + size], h.islittle, size, false)
                    as usize;
                unsafe {
                    argcheck(
                        state,
                        len <= data_len - pos - size,
                        2,
                        ERR_DATA_STRING_TOO_SHORT,
                    )
                };
                unsafe { lua_pushlstring(state, data.add(pos + size).cast(), len) };
                pos += len;
            }
            KOption::Kzstr => {
                let tail = &data_slice[pos..];
                let len = tail.iter().position(|&b| b == 0).unwrap_or(usize::MAX);
                unsafe { argcheck(state, pos + len < data_len, 2, ERR_UNFINISHED_ZSTRING) };
                unsafe { lua_pushlstring(state, data.add(pos).cast(), len) };
                pos += len + 1;
            }
            KOption::Kpaddalign | KOption::Kpadding | KOption::Knop => n -= 1,
        }
        pos += size;
    }
    unsafe { lua_pushinteger(state, pos as lua_Integer + 1) };
    n + 1
}

unsafe fn createmetatable(state: *mut lua_State) {
    unsafe { lua_createtable(state, 0, (STRING_METAMETHODS.len() - 1) as c_int) };
    {
        luaL_setfuncs(state, STRING_METAMETHODS.as_ptr(), 0)
    };
    unsafe { lua_pushlstring(state, c"".as_ptr(), 0) };
    unsafe { lua_pushvalue(state, -2) };
    unsafe { lua_setmetatable(state, -2) };
    unsafe { lua_pop(state, 1) };
    unsafe { lua_pushvalue(state, -2) };
    unsafe { lua_setfield(state, -2, FIELD_INDEX.as_ptr().cast()) };
    unsafe { lua_pop(state, 1) };
}

pub(crate) unsafe fn luaopen_string(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &STRLIB) };
    unsafe { createmetatable(state) };
    1
}

// ─── LuaModule 实现 ────────────────────────────────────────────────────────

/// `string` 标准库的模块标记类型。
pub struct StringModule;

impl crate::module::LuaModule for StringModule {
    const NAME: &'static str = "string";
    const C_NAME: &'static core::ffi::CStr = c"string";

    unsafe fn open(state: *mut lua_State) -> c_int {
        unsafe { luaopen_string(state) }
    }

    fn functions() -> &'static [crate::lua_module::luaL_Reg] {
        &STRLIB
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn string_builtin_script() {
        run_lua_test(
            "test/string_builtin.lua",
            include_str!("../test/string_builtin.lua"),
        );
    }

    #[test]
    fn string_format_comprehensive() {
        run_lua_test(
            "test/string_format.lua",
            include_str!("../test/string_format.lua"),
        );
    }
}
