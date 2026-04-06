#![allow(non_snake_case, dead_code)]

use crate::runtime::*;
use crate::string::{raw_luaS_new, raw_luaS_newlstr};
use crate::table::{raw_luaH_getstr, raw_luaH_set};
use crate::zio::{EOZ, luaZ_fill, luaZ_resizebuffer};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;
static LUA_ENV: &[u8] = b"_ENV\0";
static BREAK_NAME: &[u8] = b"break\0";
static GLOBAL_NAME: &[u8] = b"global\0";
static EXPO_EE: &[u8] = b"Ee\0";
static EXPO_PP: &[u8] = b"Pp\0";
static SIGNS: &[u8] = b"-+\0";
static XX: &[u8] = b"xX\0";
static EQ_FMT: &[u8] = b"'%s'\0";
static CHAR_FMT: &[u8] = b"'%c'\0";
static CONTROL_FMT: &[u8] = b"'<\\\\%d>'\0";
static NEAR_FMT: &[u8] = b"%s near %s\0";
static UNFINISHED_LONG_FMT: &[u8] = b"unfinished long %s (starting at line %d)\0";

static LUA_X_TOKENS: [&[u8]; (TK_STRING - FIRST_RESERVED + 1) as usize] = [
    b"and\0",
    b"break\0",
    b"do\0",
    b"else\0",
    b"elseif\0",
    b"end\0",
    b"false\0",
    b"for\0",
    b"function\0",
    b"global\0",
    b"goto\0",
    b"if\0",
    b"in\0",
    b"local\0",
    b"nil\0",
    b"not\0",
    b"or\0",
    b"repeat\0",
    b"return\0",
    b"then\0",
    b"true\0",
    b"until\0",
    b"while\0",
    b"//\0",
    b"..\0",
    b"...\0",
    b"==\0",
    b">=\0",
    b"<=\0",
    b"~=\0",
    b"<<\0",
    b">>\0",
    b"::\0",
    b"<eof>\0",
    b"<number>\0",
    b"<integer>\0",
    b"<name>\0",
    b"<string>\0",
];

#[inline]
unsafe fn luaC_fix(s: *mut lua_State, o: *mut GCObject) {
    unsafe { crate::gc::luaC_fix(s, o) }
}
#[inline]
unsafe fn luaC_step(s: *mut lua_State) {
    unsafe { crate::gc::luaC_step(s) }
}
#[inline]
unsafe fn luaD_throw(s: *mut lua_State, e: u8) -> ! {
    unsafe { crate::do_rs::luaD_throw(s, e) }
}
#[inline]
unsafe fn luaG_addinfo(
    s: *mut lua_State,
    m: *const c_char,
    src: *mut TString,
    l: c_int,
) -> *const c_char {
    unsafe { crate::debug::luaG_addinfo(s, m, src, l) }
}
#[inline]
unsafe fn luaO_hexavalue(c: c_int) -> u8 {
    unsafe { crate::object::luaO_hexavalue(c) }
}
// luaO_pushfstring 直接使用 crate::object::luaO_pushfstring，不封装（变参函数无法在非 extern 函数中转发）
#[inline]
unsafe fn luaO_str2num(s: *const c_char, o: *mut TValue) -> usize {
    unsafe { crate::object::luaO_str2num(s, o) }
}
#[inline]
unsafe fn luaO_utf8esc(b: *mut c_char, x: u32) -> c_int {
    unsafe { crate::object::luaO_utf8esc(b, x) }
}

#[inline]
fn lisprint(c: c_int) -> bool {
    (0x20..=0x7e).contains(&c)
}

#[inline]
fn lisdigit(c: c_int) -> bool {
    (b'0' as c_int..=b'9' as c_int).contains(&c)
}

#[inline]
fn lisxdigit(c: c_int) -> bool {
    lisdigit(c)
        || (b'a' as c_int..=b'f' as c_int).contains(&c)
        || (b'A' as c_int..=b'F' as c_int).contains(&c)
}

#[inline]
fn lislalpha(c: c_int) -> bool {
    (b'a' as c_int..=b'z' as c_int).contains(&c)
        || (b'A' as c_int..=b'Z' as c_int).contains(&c)
        || c == b'_' as c_int
}

#[inline]
fn lislalnum(c: c_int) -> bool {
    lislalpha(c) || lisdigit(c)
}

#[inline]
fn lisspace(c: c_int) -> bool {
    matches!(c, 9 | 10 | 11 | 12 | 13 | 32)
}

#[inline]
fn currIsNewline(ls: &LexState) -> bool {
    ls.current == '\n' as c_int || ls.current == '\r' as c_int
}

#[inline]
unsafe fn zgetc(z: *mut ZIO) -> c_int {
    if unsafe { (*z).n } > 0 {
        let byte = unsafe { *(*z).p.cast::<u8>() };
        unsafe {
            (*z).n -= 1;
            (*z).p = (*z).p.add(1);
        }
        byte as c_int
    } else {
        unsafe { luaZ_fill(z) }
    }
}

#[inline]
unsafe fn next(ls: *mut LexState) {
    unsafe { (*ls).current = zgetc((*ls).z) };
}

#[inline]
fn buffer_ptr(buffer: *mut Mbuffer) -> *mut c_char {
    unsafe { (*buffer).buffer }
}

#[inline]
fn buffer_len(buffer: *mut Mbuffer) -> usize {
    unsafe { (*buffer).n }
}

#[inline]
fn buffer_size(buffer: *mut Mbuffer) -> usize {
    unsafe { (*buffer).buffsize }
}

#[inline]
unsafe fn buffer_remove(buffer: *mut Mbuffer, n: usize) {
    unsafe { (*buffer).n -= n };
}

#[inline]
unsafe fn reset_buffer(buffer: *mut Mbuffer) {
    unsafe { (*buffer).n = 0 };
}

unsafe fn lexerror(ls: *mut LexState, msg: *const c_char, token: c_int) -> ! {
    let msg = unsafe { luaG_addinfo((*ls).L, msg, (*ls).source, (*ls).linenumber) };
    if token != 0 {
        let near = unsafe { txtToken(ls, token) };
        let msg_s = unsafe { std::ffi::CStr::from_ptr(msg) }.to_string_lossy();
        let near_s = unsafe { std::ffi::CStr::from_ptr(near) }.to_string_lossy();
        unsafe { crate::object::luaO_pushstr((*ls).L, &format!("{msg_s} near {near_s}")) };
    }
    unsafe { luaD_throw((*ls).L, LUA_ERRSYNTAX) }
}

unsafe fn save(ls: *mut LexState, c: c_int) {
    let buffer = unsafe { (*ls).buff };
    if buffer_len(buffer) + 1 > buffer_size(buffer) {
        let mut newsize = buffer_size(buffer);
        if newsize >= (MAX_SIZE / 3 * 2) {
            unsafe { lexerror(ls, c"lexical element too long".as_ptr(), 0) };
        }
        if newsize == 0 {
            newsize = LUA_MINBUFFER;
        } else {
            newsize += newsize >> 1;
        }
        unsafe { luaZ_resizebuffer((*ls).L, buffer, newsize) };
    }
    unsafe {
        *buffer_ptr(buffer).add(buffer_len(buffer)) = c as u8 as c_char;
        (*buffer).n += 1;
    }
}

#[inline]
unsafe fn save_and_next(ls: *mut LexState) {
    unsafe {
        save(ls, (*ls).current);
        next(ls);
    }
}

pub(crate) unsafe fn luaX_init(state: *mut lua_State) {
    let env = unsafe { raw_luaS_new(state.cast(), LUA_ENV.as_ptr().cast()).cast::<TString>() };
    unsafe { luaC_fix(state, env.cast()) };
    for (i, token) in LUA_X_TOKENS.iter().take(NUM_RESERVED).enumerate() {
        let ts = unsafe { raw_luaS_new(state.cast(), token.as_ptr().cast()).cast::<TString>() };
        unsafe {
            luaC_fix(state, ts.cast());
            (*ts).extra = (i + 1) as u8;
        }
    }
}

pub unsafe fn luaX_token2str(ls: *mut LexState, token: c_int) -> *const c_char {
    if token < FIRST_RESERVED {
        let c = (token as u8) as char;
        if lisprint(token) {
            unsafe { crate::object::luaO_pushstr((*ls).L, &format!("'{c}'")) }
        } else {
            unsafe { crate::object::luaO_pushstr((*ls).L, &format!("'<\\{token}>'")) }
        }
    } else {
        let s = LUA_X_TOKENS[(token - FIRST_RESERVED) as usize];
        if token < TK_EOS {
            let s_str = core::str::from_utf8(&s[..s.len() - 1]).unwrap_or("?");
            unsafe { crate::object::luaO_pushstr((*ls).L, &format!("'{s_str}'")) }
        } else {
            s.as_ptr().cast()
        }
    }
}

unsafe fn txtToken(ls: *mut LexState, token: c_int) -> *const c_char {
    match token {
        TK_NAME | TK_STRING | TK_FLT | TK_INT => {
            unsafe { save(ls, 0) };
            let buf_s =
                unsafe { std::ffi::CStr::from_ptr(buffer_ptr((*ls).buff)) }.to_string_lossy();
            unsafe { crate::object::luaO_pushstr((*ls).L, &format!("'{buf_s}'")) }
        }
        _ => unsafe { luaX_token2str(ls, token) },
    }
}

pub unsafe fn luaX_syntaxerror(ls: *mut LexState, msg: *const c_char) -> ! {
    unsafe { lexerror(ls, msg, (*ls).t.token) }
}

unsafe fn anchorstr(ls: *mut LexState, ts: *mut TString) -> *mut TString {
    unsafe {
        let mut oldts = TValue {
            value_: Value {
                gc: ptr::null_mut(),
            },
            tt_: LUA_VNIL,
        };
        let tag = raw_luaH_getstr((*ls).h.cast(), ts.cast(), ptr::addr_of_mut!(oldts).cast());
        if !tagisempty(tag) {
            tsvalue(ptr::addr_of!(oldts))
        } else {
            let top = (*(*ls).L).top.p;
            let stv = s2v(top);
            (*(*ls).L).top.p = top.add(1);
            setsvalue(stv, ts);
            raw_luaH_set((*ls).L.cast(), (*ls).h.cast(), stv.cast(), stv.cast());
            luaC_checkGC((*ls).L);
            (*(*ls).L).top.p = top;
            ts
        }
    }
}

pub unsafe fn luaX_newstring(ls: *mut LexState, str_: *const c_char, len: usize) -> *mut TString {
    let ts = unsafe { raw_luaS_newlstr((*ls).L.cast(), str_, len).cast::<TString>() };
    unsafe { anchorstr(ls, ts) }
}

unsafe fn inclinenumber(ls: *mut LexState) {
    let old = unsafe { (*ls).current };
    debug_assert!(currIsNewline(unsafe { &*ls }));
    unsafe { next(ls) };
    if currIsNewline(unsafe { &*ls }) && unsafe { (*ls).current } != old {
        unsafe { next(ls) };
    }
    unsafe {
        (*ls).linenumber += 1;
        if (*ls).linenumber >= c_int::MAX {
            lexerror(ls, c"chunk has too many lines".as_ptr(), 0);
        }
    }
}

pub unsafe fn luaX_setinput(
    state: *mut lua_State,
    ls: *mut LexState,
    z: *mut ZIO,
    source: *mut TString,
    firstchar: c_int,
) {
    unsafe {
        (*ls).t.token = 0;
        (*ls).L = state;
        (*ls).current = firstchar;
        (*ls).lookahead.token = TK_EOS;
        (*ls).z = z;
        (*ls).fs = ptr::null_mut();
        (*ls).linenumber = 1;
        (*ls).lastline = 1;
        (*ls).source = source;
        (*ls).envn = raw_luaS_new(state.cast(), LUA_ENV.as_ptr().cast()).cast();
        (*ls).brkn = raw_luaS_new(state.cast(), BREAK_NAME.as_ptr().cast()).cast();
        (*ls).glbn = raw_luaS_new(state.cast(), GLOBAL_NAME.as_ptr().cast()).cast();
        (*(*ls).glbn).extra = 0;
        luaZ_resizebuffer(state, (*ls).buff, LUA_MINBUFFER);
    }
}

unsafe fn check_next1(ls: *mut LexState, c: c_int) -> bool {
    if unsafe { (*ls).current } == c {
        unsafe { next(ls) };
        true
    } else {
        false
    }
}

unsafe fn check_next2(ls: *mut LexState, set: &[u8]) -> bool {
    debug_assert_eq!(set[2], 0);
    let current = unsafe { (*ls).current };
    if current == set[0] as c_int || current == set[1] as c_int {
        unsafe { save_and_next(ls) };
        true
    } else {
        false
    }
}

unsafe fn read_numeral(ls: *mut LexState, seminfo: *mut SemInfo) -> c_int {
    let mut obj = TValue {
        value_: Value {
            gc: ptr::null_mut(),
        },
        tt_: LUA_VNIL,
    };
    let mut expo = EXPO_EE;
    let first = unsafe { (*ls).current };
    debug_assert!(lisdigit(first));
    unsafe { save_and_next(ls) };
    if first == '0' as c_int && unsafe { check_next2(ls, XX) } {
        expo = EXPO_PP;
    }
    loop {
        if unsafe { check_next2(ls, expo) } {
            unsafe { check_next2(ls, SIGNS) };
        } else if lisxdigit(unsafe { (*ls).current }) || unsafe { (*ls).current } == '.' as c_int {
            unsafe { save_and_next(ls) };
        } else {
            break;
        }
    }
    if lislalpha(unsafe { (*ls).current }) {
        unsafe { save_and_next(ls) };
    }
    unsafe { save(ls, 0) };
    if unsafe { luaO_str2num(buffer_ptr((*ls).buff), ptr::addr_of_mut!(obj)) } == 0 {
        unsafe { lexerror(ls, c"malformed number".as_ptr(), TK_FLT) };
    }
    if unsafe { ttisinteger(ptr::addr_of!(obj)) } {
        unsafe { (*seminfo).i = ivalue(ptr::addr_of!(obj)) };
        TK_INT
    } else {
        debug_assert!(unsafe { ttisfloat(ptr::addr_of!(obj)) });
        unsafe { (*seminfo).r = fltvalue(ptr::addr_of!(obj)) };
        TK_FLT
    }
}

unsafe fn skip_sep(ls: *mut LexState) -> usize {
    let mut count = 0usize;
    let s = unsafe { (*ls).current };
    debug_assert!(s == '[' as c_int || s == ']' as c_int);
    unsafe { save_and_next(ls) };
    while unsafe { (*ls).current } == '=' as c_int {
        unsafe {
            save_and_next(ls);
        }
        count += 1;
    }
    if unsafe { (*ls).current } == s {
        count + 2
    } else if count == 0 {
        1
    } else {
        0
    }
}

unsafe fn read_long_string(ls: *mut LexState, seminfo: *mut SemInfo, sep: usize) {
    let line = unsafe { (*ls).linenumber };
    unsafe { save_and_next(ls) };
    if currIsNewline(unsafe { &*ls }) {
        unsafe { inclinenumber(ls) };
    }
    loop {
        match unsafe { (*ls).current } {
            EOZ => {
                let what = if seminfo.is_null() {
                    c"comment".as_ptr()
                } else {
                    c"string".as_ptr()
                };
                let what_s = unsafe { std::ffi::CStr::from_ptr(what) }.to_string_lossy();
                let msg_str = format!("unfinished long {what_s} (starting at line {line})");
                let msg = unsafe { crate::object::luaO_pushstr((*ls).L, &msg_str) };
                unsafe { lexerror(ls, msg, TK_EOS) };
            }
            x if x == ']' as c_int => {
                if unsafe { skip_sep(ls) } == sep {
                    unsafe { save_and_next(ls) };
                    break;
                }
            }
            x if x == '\n' as c_int || x == '\r' as c_int => unsafe {
                save(ls, '\n' as c_int);
                inclinenumber(ls);
                if seminfo.is_null() {
                    reset_buffer((*ls).buff);
                }
            },
            _ => {
                if !seminfo.is_null() {
                    unsafe { save_and_next(ls) };
                } else {
                    unsafe { next(ls) };
                }
            }
        }
    }
    if !seminfo.is_null() {
        unsafe {
            (*seminfo).ts = luaX_newstring(
                ls,
                buffer_ptr((*ls).buff).add(sep),
                buffer_len((*ls).buff) - 2 * sep,
            );
        }
    }
}

unsafe fn esccheck(ls: *mut LexState, c: bool, msg: *const c_char) {
    if !c {
        if unsafe { (*ls).current } != EOZ {
            unsafe { save_and_next(ls) };
        }
        unsafe { lexerror(ls, msg, TK_STRING) };
    }
}

unsafe fn gethexa(ls: *mut LexState) -> c_int {
    unsafe { save_and_next(ls) };
    unsafe {
        esccheck(
            ls,
            lisxdigit((*ls).current),
            c"hexadecimal digit expected".as_ptr(),
        )
    };
    unsafe { luaO_hexavalue((*ls).current) as c_int }
}

unsafe fn readhexaesc(ls: *mut LexState) -> c_int {
    let mut r = unsafe { gethexa(ls) };
    r = (r << 4) + unsafe { gethexa(ls) };
    unsafe { buffer_remove((*ls).buff, 2) };
    r
}

unsafe fn readutf8esc(ls: *mut LexState) -> u32 {
    let mut i = 4usize;
    unsafe { save_and_next(ls) };
    unsafe { esccheck(ls, (*ls).current == '{' as c_int, c"missing '{'".as_ptr()) };
    let mut r = unsafe { gethexa(ls) as u32 };
    loop {
        unsafe { save_and_next(ls) };
        if !lisxdigit(unsafe { (*ls).current }) {
            break;
        }
        i += 1;
        unsafe {
            esccheck(
                ls,
                r <= (0x7fffffff_u32 >> 4),
                c"UTF-8 value too large".as_ptr(),
            )
        };
        r = (r << 4) + unsafe { luaO_hexavalue((*ls).current) as u32 };
    }
    unsafe { esccheck(ls, (*ls).current == '}' as c_int, c"missing '}'".as_ptr()) };
    unsafe { next(ls) };
    unsafe { buffer_remove((*ls).buff, i) };
    r
}

unsafe fn utf8esc(ls: *mut LexState) {
    let mut buff = [0 as c_char; UTF8BUFFSZ];
    let n = unsafe { luaO_utf8esc(buff.as_mut_ptr(), readutf8esc(ls)) };
    let mut i = (UTF8BUFFSZ as c_int - n) as usize;
    while i < UTF8BUFFSZ {
        unsafe { save(ls, buff[i] as u8 as c_int) };
        i += 1;
    }
}

unsafe fn readdecesc(ls: *mut LexState) -> c_int {
    let mut i = 0usize;
    let mut r: c_int = 0;
    while i < 3 && lisdigit(unsafe { (*ls).current }) {
        r = 10 * r + (unsafe { (*ls).current } - '0' as c_int);
        unsafe { save_and_next(ls) };
        i += 1;
    }
    unsafe {
        esccheck(
            ls,
            r <= u8::MAX as c_int,
            c"decimal escape too large".as_ptr(),
        )
    };
    unsafe { buffer_remove((*ls).buff, i) };
    r
}

unsafe fn read_string(ls: *mut LexState, del: c_int, seminfo: *mut SemInfo) {
    unsafe { save_and_next(ls) };
    while unsafe { (*ls).current } != del {
        match unsafe { (*ls).current } {
            EOZ => unsafe { lexerror(ls, c"unfinished string".as_ptr(), TK_EOS) },
            x if x == '\n' as c_int || x == '\r' as c_int => unsafe {
                lexerror(ls, c"unfinished string".as_ptr(), TK_STRING)
            },
            x if x == '\\' as c_int => {
                unsafe { save_and_next(ls) };
                let current = unsafe { (*ls).current };
                match current {
                    x if x == 'a' as c_int => {
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, '\u{7}' as c_int) };
                    }
                    x if x == 'b' as c_int => {
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, '\u{8}' as c_int) };
                    }
                    x if x == 'f' as c_int => {
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, '\u{c}' as c_int) };
                    }
                    x if x == 'n' as c_int => {
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, '\n' as c_int) };
                    }
                    x if x == 'r' as c_int => {
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, '\r' as c_int) };
                    }
                    x if x == 't' as c_int => {
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, '\t' as c_int) };
                    }
                    x if x == 'v' as c_int => {
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, '\u{b}' as c_int) };
                    }
                    x if x == 'x' as c_int => {
                        let c = unsafe { readhexaesc(ls) };
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, c) };
                    }
                    x if x == 'u' as c_int => unsafe { utf8esc(ls) },
                    x if x == '\n' as c_int || x == '\r' as c_int => {
                        unsafe { inclinenumber(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, '\n' as c_int) };
                    }
                    x if x == '\\' as c_int || x == '"' as c_int || x == '\'' as c_int => {
                        unsafe { next(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, current) };
                    }
                    EOZ => {}
                    x if x == 'z' as c_int => {
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { next(ls) };
                        while lisspace(unsafe { (*ls).current }) {
                            if currIsNewline(unsafe { &*ls }) {
                                unsafe { inclinenumber(ls) };
                            } else {
                                unsafe { next(ls) };
                            }
                        }
                    }
                    _ => {
                        unsafe {
                            esccheck(
                                ls,
                                lisdigit((*ls).current),
                                c"invalid escape sequence".as_ptr(),
                            )
                        };
                        let c = unsafe { readdecesc(ls) };
                        unsafe { buffer_remove((*ls).buff, 1) };
                        unsafe { save(ls, c) };
                    }
                }
            }
            _ => unsafe { save_and_next(ls) },
        }
    }
    unsafe { save_and_next(ls) };
    unsafe {
        (*seminfo).ts = luaX_newstring(
            ls,
            buffer_ptr((*ls).buff).add(1),
            buffer_len((*ls).buff) - 2,
        )
    };
}

unsafe fn llex(ls: *mut LexState, seminfo: *mut SemInfo) -> c_int {
    unsafe { reset_buffer((*ls).buff) };
    loop {
        match unsafe { (*ls).current } {
            x if x == '\n' as c_int || x == '\r' as c_int => unsafe { inclinenumber(ls) },
            x if matches!(x, 32 | 12 | 9 | 11) => unsafe { next(ls) },
            x if x == '-' as c_int => {
                unsafe { next(ls) };
                if unsafe { (*ls).current } != '-' as c_int {
                    return '-' as c_int;
                }
                unsafe { next(ls) };
                if unsafe { (*ls).current } == '[' as c_int {
                    let sep = unsafe { skip_sep(ls) };
                    unsafe { reset_buffer((*ls).buff) };
                    if sep >= 2 {
                        unsafe { read_long_string(ls, ptr::null_mut(), sep) };
                        unsafe { reset_buffer((*ls).buff) };
                        continue;
                    }
                }
                while !currIsNewline(unsafe { &*ls }) && unsafe { (*ls).current } != EOZ {
                    unsafe { next(ls) };
                }
            }
            x if x == '[' as c_int => {
                let sep = unsafe { skip_sep(ls) };
                if sep >= 2 {
                    unsafe { read_long_string(ls, seminfo, sep) };
                    return TK_STRING;
                } else if sep == 0 {
                    unsafe { lexerror(ls, c"invalid long string delimiter".as_ptr(), TK_STRING) };
                }
                return '[' as c_int;
            }
            x if x == '=' as c_int => {
                unsafe { next(ls) };
                return if unsafe { check_next1(ls, '=' as c_int) } {
                    TK_EQ
                } else {
                    '=' as c_int
                };
            }
            x if x == '<' as c_int => {
                unsafe { next(ls) };
                return if unsafe { check_next1(ls, '=' as c_int) } {
                    TK_LE
                } else if unsafe { check_next1(ls, '<' as c_int) } {
                    TK_SHL
                } else {
                    '<' as c_int
                };
            }
            x if x == '>' as c_int => {
                unsafe { next(ls) };
                return if unsafe { check_next1(ls, '=' as c_int) } {
                    TK_GE
                } else if unsafe { check_next1(ls, '>' as c_int) } {
                    TK_SHR
                } else {
                    '>' as c_int
                };
            }
            x if x == '/' as c_int => {
                unsafe { next(ls) };
                return if unsafe { check_next1(ls, '/' as c_int) } {
                    TK_IDIV
                } else {
                    '/' as c_int
                };
            }
            x if x == '~' as c_int => {
                unsafe { next(ls) };
                return if unsafe { check_next1(ls, '=' as c_int) } {
                    TK_NE
                } else {
                    '~' as c_int
                };
            }
            x if x == ':' as c_int => {
                unsafe { next(ls) };
                return if unsafe { check_next1(ls, ':' as c_int) } {
                    TK_DBCOLON
                } else {
                    ':' as c_int
                };
            }
            x if x == '"' as c_int || x == '\'' as c_int => {
                unsafe { read_string(ls, x, seminfo) };
                return TK_STRING;
            }
            x if x == '.' as c_int => {
                unsafe { save_and_next(ls) };
                if unsafe { check_next1(ls, '.' as c_int) } {
                    return if unsafe { check_next1(ls, '.' as c_int) } {
                        TK_DOTS
                    } else {
                        TK_CONCAT
                    };
                } else if !lisdigit(unsafe { (*ls).current }) {
                    return '.' as c_int;
                } else {
                    return unsafe { read_numeral(ls, seminfo) };
                }
            }
            x if lisdigit(x) => return unsafe { read_numeral(ls, seminfo) },
            EOZ => return TK_EOS,
            _ => {
                if lislalpha(unsafe { (*ls).current }) {
                    loop {
                        unsafe { save_and_next(ls) };
                        if !lislalnum(unsafe { (*ls).current }) {
                            break;
                        }
                    }
                    let ts = unsafe {
                        raw_luaS_newlstr(
                            (*ls).L.cast(),
                            buffer_ptr((*ls).buff),
                            buffer_len((*ls).buff),
                        )
                        .cast::<TString>()
                    };
                    if unsafe { (*ts).shrlen >= 0 && (*ts).extra > 0 } {
                        return unsafe { (*ts).extra as c_int - 1 + FIRST_RESERVED };
                    }
                    unsafe { (*seminfo).ts = anchorstr(ls, ts) };
                    return TK_NAME;
                } else {
                    let c = unsafe { (*ls).current };
                    unsafe { next(ls) };
                    return c;
                }
            }
        }
    }
}

pub unsafe fn luaX_next(ls: *mut LexState) {
    unsafe {
        (*ls).lastline = (*ls).linenumber;
        if (*ls).lookahead.token != TK_EOS {
            (*ls).t = (*ls).lookahead;
            (*ls).lookahead.token = TK_EOS;
        } else {
            (*ls).t.token = llex(ls, ptr::addr_of_mut!((*ls).t.seminfo));
        }
    }
}

pub unsafe fn luaX_lookahead(ls: *mut LexState) -> c_int {
    debug_assert_eq!(unsafe { (*ls).lookahead.token }, TK_EOS);
    unsafe {
        (*ls).lookahead.token = llex(ls, ptr::addr_of_mut!((*ls).lookahead.seminfo));
        (*ls).lookahead.token
    }
}

pub(crate) unsafe fn raw_luaX_init(state: *mut c_void) {
    unsafe { luaX_init(state.cast()) };
}

#[cfg(test)]
mod tests {
    use crate::api::lua_tolstring;
    use crate::aux_rs::{luaL_checkversion_, luaL_loadbufferx, luaL_newstate};
    use crate::init::luaL_openselectedlibs;
    use crate::luaffi::LUAL_NUMSIZES;
    use crate::runtime::{LUA_OK, LUA_VERSION_NUM};
    use crate::state::lua_close;
    use crate::test_support::run_lua_test;
    use std::ptr;

    fn load_error(source: &str) -> String {
        let state = unsafe { luaL_newstate() };
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
            luaL_openselectedlibs(state, !0, 0);
            let name = c"@lex_error.lua";
            let status = luaL_loadbufferx(
                state,
                source.as_ptr().cast(),
                source.len(),
                name.as_ptr(),
                ptr::null(),
            );
            assert_ne!(status, LUA_OK.into(), "chunk should fail to load");
            let mut len = 0usize;
            let err = lua_tolstring(state, -1, &mut len);
            String::from_utf8_lossy(core::slice::from_raw_parts(err.cast::<u8>(), len)).into_owned()
        })();

        unsafe { lua_close(state) };
        result
    }

    #[test]
    fn lexer_handles_long_strings_comments_and_escapes() {
        run_lua_test(
            "test/lex_roundtrip.lua",
            r#"
                local x = 0x1.fp3
                local y = "a\n\x41\u{42}\z
                      C"
                local z = [==[
hello
]==]
                global sample = 7
                local _ = x
                local _ = y
                local _ = z
                local _ = sample
            "#,
        );
    }

    #[test]
    fn lexer_reports_invalid_long_string_delimiter() {
        let err = load_error("local x = [=oops]");
        assert!(err.contains("invalid long string delimiter"));
    }
}
