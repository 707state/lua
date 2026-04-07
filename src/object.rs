use crate::do_rs::luaD_rawrunprotected;
use crate::luaffi::{localeconv, strtod};
use crate::string::*;
use crate::mem::*;
use crate::runtime::*;
use crate::tm::*;
use crate::vm_rs::*;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::ffi::{CStr, CString};

#[inline]
fn intop_add(v1: lua_Integer, v2: lua_Integer) -> lua_Integer {
    (v1 as u64).wrapping_add(v2 as u64) as lua_Integer
}

#[inline]
fn intop_sub(v1: lua_Integer, v2: lua_Integer) -> lua_Integer {
    (v1 as u64).wrapping_sub(v2 as u64) as lua_Integer
}

#[inline]
fn intop_mul(v1: lua_Integer, v2: lua_Integer) -> lua_Integer {
    (v1 as u64).wrapping_mul(v2 as u64) as lua_Integer
}

#[inline]
fn intop_and(v1: lua_Integer, v2: lua_Integer) -> lua_Integer {
    (v1 as u64 & v2 as u64) as lua_Integer
}

#[inline]
fn intop_or(v1: lua_Integer, v2: lua_Integer) -> lua_Integer {
    (v1 as u64 | v2 as u64) as lua_Integer
}

#[inline]
fn intop_xor(v1: lua_Integer, v2: lua_Integer) -> lua_Integer {
    (v1 as u64 ^ v2 as u64) as lua_Integer
}

#[inline]
fn intop_neg(v1: lua_Integer) -> lua_Integer {
    (0u64).wrapping_sub(v1 as u64) as lua_Integer
}

#[inline]
fn intop_bnot(v1: lua_Integer) -> lua_Integer {
    (!(v1 as u64)) as lua_Integer
}

unsafe fn intarith(
    state: *mut lua_State,
    op: c_int,
    v1: lua_Integer,
    v2: lua_Integer,
) -> lua_Integer {
    match op {
        LUA_OPADD => intop_add(v1, v2),
        LUA_OPSUB => intop_sub(v1, v2),
        LUA_OPMUL => intop_mul(v1, v2),
        LUA_OPMOD => unsafe { luaV_mod(state, v1, v2) },
        LUA_OPIDIV => unsafe { luaV_idiv(state, v1, v2) },
        LUA_OPBAND => intop_and(v1, v2),
        LUA_OPBOR => intop_or(v1, v2),
        LUA_OPBXOR => intop_xor(v1, v2),
        LUA_OPSHL => unsafe { luaV_shiftl(v1, v2) },
        LUA_OPSHR => unsafe { luaV_shiftl(v1, intop_neg(v2)) },
        LUA_OPUNM => intop_neg(v1),
        LUA_OPBNOT => intop_bnot(v1),
        _ => 0,
    }
}

fn numarith(state: *mut lua_State, op: c_int, v1: lua_Number, v2: lua_Number) -> lua_Number {
    match op {
        LUA_OPADD => v1 + v2,
        LUA_OPSUB => v1 - v2,
        LUA_OPMUL => v1 * v2,
        LUA_OPDIV => v1 / v2,
        LUA_OPPOW => v1.powf(v2),
        LUA_OPIDIV => (v1 / v2).floor(),
        LUA_OPUNM => -v1,
        LUA_OPMOD => unsafe { luaV_modf(state, v1, v2) },
        _ => 0.0,
    }
}

pub unsafe fn luaO_ceillog2(mut x: u32) -> u8 {
    if x <= 1 {
        0
    } else {
        x = x.wrapping_sub(1);
        (u32::BITS - x.leading_zeros()) as u8
    }
}

pub unsafe fn luaO_codeparam(mut p: u32) -> u8 {
    if p >= ((0x1fu64) << (0xf - 7 - 1)) as u32 * 100 {
        0xff
    } else {
        p = ((p as u64 * 128 + 99) / 100) as u32;
        if p < 0x10 {
            p as u8
        } else {
            let log = unsafe { luaO_ceillog2(p + 1) } as u32 - 5;
            (((p >> log) - 0x10) | ((log + 1) << 4)) as u8
        }
    }
}

pub unsafe fn luaO_applyparam(p: u8, x: isize) -> isize {
    let mut m = (p & 0x0f) as isize;
    let mut e = (p >> 4) as isize;
    if e > 0 {
        e -= 1;
        m += 0x10;
    }
    e -= 7;
    if e >= 0 {
        if x < (MAX_LMEM / 0x1f) >> e {
            (x * m) << e
        } else {
            MAX_LMEM
        }
    } else {
        e = -e;
        if x < MAX_LMEM / 0x1f {
            (x * m) >> e
        } else if (x >> e) < MAX_LMEM / 0x1f {
            (x >> e) * m
        } else {
            MAX_LMEM
        }
    }
}

pub unsafe fn luaO_rawarith(
    state: *mut lua_State,
    op: c_int,
    p1: *const TValue,
    p2: *const TValue,
    res: *mut TValue,
) -> c_int {
    match op {
        LUA_OPBAND | LUA_OPBOR | LUA_OPBXOR | LUA_OPSHL | LUA_OPSHR | LUA_OPBNOT => {
            let mut i1 = 0;
            let mut i2 = 0;
            if unsafe { luaV_tointegerns(p1, &mut i1, 0) } != 0
                && unsafe { luaV_tointegerns(p2, &mut i2, 0) } != 0
            {
                unsafe { setivalue(res, intarith(state, op, i1, i2)) };
                1
            } else {
                0
            }
        }
        LUA_OPDIV | LUA_OPPOW => {
            let mut n1 = 0.0;
            let mut n2 = 0.0;
            if unsafe { number_to_float(p1, &mut n1) } && unsafe { number_to_float(p2, &mut n2) } {
                unsafe { setfltvalue(res, numarith(state, op, n1, n2)) };
                1
            } else {
                0
            }
        }
        _ => {
            let mut n1 = 0.0;
            let mut n2 = 0.0;
            if unsafe { ttisinteger(p1) } && unsafe { ttisinteger(p2) } {
                unsafe { setivalue(res, intarith(state, op, ivalue(p1), ivalue(p2))) };
                1
            } else if unsafe { number_to_float(p1, &mut n1) }
                && unsafe { number_to_float(p2, &mut n2) }
            {
                unsafe { setfltvalue(res, numarith(state, op, n1, n2)) };
                1
            } else {
                0
            }
        }
    }
}

pub unsafe fn luaO_arith(
    state: *mut lua_State,
    op: LuaArithOp,
    p1: *const TValue,
    p2: *const TValue,
    res: StkId,
) {
    if unsafe { luaO_rawarith(state, op.as_c_int(), p1, p2, s2v(res)) } == 0 {
        let event = TagMethod::from_c_int((op.as_c_int() - LUA_OPADD) + TM_ADD)
            .expect("arithmetic op must map to arithmetic tag method");
        unsafe { luaT_trybinTM(state, p1, p2, res, event) };
    }
}

pub unsafe fn luaO_hexavalue(c: c_int) -> u8 {
    let byte = c as u8;
    if byte.is_ascii_digit() {
        byte - b'0'
    } else {
        byte.to_ascii_lowercase() - b'a' + 10
    }
}

fn is_space(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | 0x0b | 0x0c | b'\r' | b' ')
}

fn is_neg(bytes: &[u8], index: &mut usize) -> bool {
    if bytes.get(*index) == Some(&b'-') {
        *index += 1;
        true
    } else {
        if bytes.get(*index) == Some(&b'+') {
            *index += 1;
        }
        false
    }
}

fn str2dloc(s: &CStr, result: &mut lua_Number, mode: u8) -> Option<usize> {
    let mut endptr = ptr::null_mut();
    *result = unsafe { strtod(s.as_ptr(), &mut endptr) };
    if endptr == s.as_ptr() as *mut c_char {
        return None;
    }
    let bytes = s.to_bytes();
    let mut end = unsafe { endptr.offset_from(s.as_ptr()) as usize };
    while end < bytes.len() && is_space(bytes[end]) {
        end += 1;
    }
    if mode == b'n' {
        return None;
    }
    (end == bytes.len()).then_some(end)
}

fn str2d(s: &CStr, result: &mut lua_Number) -> Option<usize> {
    let bytes = s.to_bytes();
    let mode = bytes
        .iter()
        .find(|&&b| matches!(b, b'.' | b'x' | b'X' | b'n' | b'N'))
        .map(|b| b.to_ascii_lowercase())
        .unwrap_or(0);
    if mode == b'n' {
        return None;
    }
    if let Some(end) = str2dloc(s, result, mode) {
        return Some(end);
    }
    let dot = bytes.iter().position(|&b| b == b'.')?;
    if bytes.len() > 200 {
        return None;
    }
    let mut owned = bytes.to_vec();
    owned.push(0);
    let locale = localeconv();
    if locale.is_null() || unsafe { (*locale).decimal_point }.is_null() {
        return None;
    }
    owned[dot] = unsafe { *(*locale).decimal_point.cast::<u8>() };
    let owned = CString::from_vec_with_nul(owned).ok()?;
    str2dloc(&owned, result, mode)
}

fn str2int(s: &CStr, result: &mut lua_Integer) -> Option<usize> {
    let bytes = s.to_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() && is_space(bytes[idx]) {
        idx += 1;
    }
    let neg = is_neg(bytes, &mut idx);
    let mut acc = 0u64;
    let mut empty = true;
    if idx + 1 < bytes.len() && bytes[idx] == b'0' && matches!(bytes[idx + 1], b'x' | b'X') {
        idx += 2;
        while idx < bytes.len() && bytes[idx].is_ascii_hexdigit() {
            acc = acc
                .wrapping_mul(16)
                .wrapping_add(unsafe { luaO_hexavalue(bytes[idx] as c_int) } as u64);
            idx += 1;
            empty = false;
        }
    } else {
        let max_lastd = (lua_Integer::MAX % 10) as u8;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            let d = bytes[idx] - b'0';
            if acc >= (lua_Integer::MAX as u64 / 10)
                && (acc > (lua_Integer::MAX as u64 / 10) || d > max_lastd + u8::from(neg))
            {
                return None;
            }
            acc = acc * 10 + u64::from(d);
            idx += 1;
            empty = false;
        }
    }
    while idx < bytes.len() && is_space(bytes[idx]) {
        idx += 1;
    }
    if empty || idx != bytes.len() {
        None
    } else {
        *result = if neg {
            (0u64).wrapping_sub(acc) as lua_Integer
        } else {
            acc as lua_Integer
        };
        Some(idx)
    }
}

pub unsafe fn luaO_str2num(s: *const c_char, o: *mut TValue) -> usize {
    let cstr = unsafe { CStr::from_ptr(s) };
    let mut i = 0;
    let mut n = 0.0;
    if let Some(end) = str2int(cstr, &mut i) {
        unsafe { setivalue(o, i) };
        end + 1
    } else if let Some(end) = str2d(cstr, &mut n) {
        unsafe { setfltvalue(o, n) };
        end + 1
    } else {
        0
    }
}

pub unsafe fn luaO_utf8esc(buff: *mut c_char, mut x: u32) -> c_int {
    let mut n = 1usize;
    if x < 0x80 {
        unsafe { *buff.add(UTF8BUFFSZ - 1) = x as c_char };
    } else {
        let mut mfb = 0x3fu32;
        loop {
            unsafe { *buff.add(UTF8BUFFSZ - n) = (0x80 | (x & 0x3f)) as c_char };
            n += 1;
            x >>= 6;
            mfb >>= 1;
            if x <= mfb {
                break;
            }
        }
        unsafe { *buff.add(UTF8BUFFSZ - n) = (((!mfb) << 1) | x) as c_char };
    }
    n as c_int
}

fn tostringbuff_float(n: lua_Number, buff: *mut c_char) -> c_int {
    // Try with 14 significant digits first (like C's %.14g), check round-trip
    let mut s = rust_g_format(n, 14);
    if let Ok(check) = s.parse::<f64>() {
        if check != n {
            // Use 17 significant digits for full precision
            s = rust_g_format(n, 17);
        }
    }
    // If the result looks like a pure integer (only digits and minus), append ".0"
    let looks_like_int = s.bytes().all(|b| matches!(b, b'-' | b'0'..=b'9'));
    if looks_like_int {
        let locale = localeconv();
        let point = if locale.is_null() || unsafe { (*locale).decimal_point }.is_null() {
            b'.'
        } else {
            unsafe { *(*locale).decimal_point.cast::<u8>() }
        };
        s.push(point as char);
        s.push('0');
    }
    let len = s.len().min(LUA_N2SBUFFSZ - 1);
    unsafe {
        ptr::copy_nonoverlapping(s.as_ptr().cast::<c_char>(), buff, len);
        *buff.add(len) = 0;
    }
    len as c_int
}

/// Format a float in %g style using Rust's built-in formatting for precision.
/// Uses `{:.prec$e}` for scientific notation and `{:.prec$}` for fixed,
/// then trims trailing zeros like C's %g.
pub(crate) fn rust_g_format(n: f64, sig_digits: usize) -> String {
    if n.is_nan() {
        return "-nan".to_string();
    }
    if n.is_infinite() {
        return if n.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    if n == 0.0 {
        return if n.is_sign_negative() {
            "-0".to_string()
        } else {
            "0".to_string()
        };
    }

    // Use Rust's {:e} to get the exponent reliably
    let sci = format!("{:.prec$e}", n, prec = sig_digits.saturating_sub(1));
    // Parse the exponent from Rust's scientific notation (e.g. "3.14000e2")
    let exp = if let Some(e_pos) = sci.rfind('e') {
        sci[e_pos + 1..].parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    // %g uses %e if exponent < -4 or >= sig_digits, otherwise %f
    if exp < -4 || exp >= sig_digits as i32 {
        // Scientific notation: use Rust's {:.prec$e} and reformat exponent
        let prec = sig_digits.saturating_sub(1);
        let raw = format!("{:.prec$e}", n, prec = prec);
        // Rust uses "e" not "e+", and no leading zeros on exponent
        // We need to reformat to match C: e+XX or e-XX with at least 2 digits
        if let Some(e_pos) = raw.rfind('e') {
            let mantissa_part = &raw[..e_pos];
            let exp_val: i32 = raw[e_pos + 1..].parse().unwrap_or(0);
            let exp_sign = if exp_val >= 0 { '+' } else { '-' };
            let abs_exp = exp_val.unsigned_abs();
            // Trim trailing zeros from mantissa
            let trimmed = if mantissa_part.contains('.') {
                mantissa_part.trim_end_matches('0').trim_end_matches('.')
            } else {
                mantissa_part
            };
            format!("{}e{}{:02}", trimmed, exp_sign, abs_exp)
        } else {
            raw
        }
    } else {
        // Fixed notation
        let decimal_places = if sig_digits as i32 > exp + 1 {
            (sig_digits as i32 - exp - 1) as usize
        } else {
            0
        };
        let mut result = format!("{:.prec$}", n, prec = decimal_places);
        // Trim trailing zeros like %g
        if result.contains('.') {
            let trimmed = result.trim_end_matches('0').trim_end_matches('.');
            result = trimmed.to_string();
        }
        result
    }
}

pub unsafe fn luaO_tostringbuff(obj: *const TValue, buff: *mut c_char) -> u32 {
    let len = if unsafe { ttisinteger(obj) } {
        let n = unsafe { ivalue(obj) };
        let s = format!("{}", n);
        let len = s.len().min(LUA_N2SBUFFSZ - 1);
        unsafe {
            ptr::copy_nonoverlapping(s.as_ptr().cast::<c_char>(), buff, len);
            *buff.add(len) = 0;
        }
        len as c_int
    } else {
        tostringbuff_float(unsafe { fltvalue(obj) }, buff)
    };
    len as u32
}

pub unsafe fn luaO_tostring(state: *mut lua_State, obj: *mut TValue) {
    let mut buff = [0 as c_char; LUA_N2SBUFFSZ];
    let len = unsafe { luaO_tostringbuff(obj, buff.as_mut_ptr()) } as usize;
    let string = unsafe { luaS_newlstr(state, buff.as_ptr(), len).cast::<TString>() };
    unsafe { setsvalue(obj, string) };
}

pub unsafe fn initbuff(state: *mut lua_State, buff: *mut BuffFS) {
    unsafe {
        (*buff).l = state;
        (*buff).b = (*buff).space.as_mut_ptr();
        (*buff).buffsize = (*buff).space.len();
        (*buff).blen = 0;
        (*buff).err = 0;
    }
}

unsafe fn pushbuff(state: *mut lua_State, ud: *mut c_void) {
    let buff = unsafe { &mut *ud.cast::<BuffFS>() };
    match buff.err {
        1 => unsafe { luaD_throw(state, LUA_ERRMEM) },
        2 => {
            if buff.buffsize - buff.blen < 3 {
                let dst = unsafe { buff.b.add(buff.blen - 3) };
                unsafe {
                    ptr::copy_nonoverlapping(RETS.as_ptr().cast::<c_char>(), dst, 3);
                    *dst.add(3) = 0;
                }
            } else {
                let dst = unsafe { buff.b.add(buff.blen) };
                unsafe {
                    ptr::copy_nonoverlapping(RETS.as_ptr().cast::<c_char>(), dst, 3);
                    *dst.add(3) = 0;
                }
                buff.blen += 3;
            }
            let ts = unsafe { luaS_newlstr(state, buff.b, buff.blen).cast::<TString>() };
            let top = unsafe { (*state).top.p };
            unsafe { setsvalue(s2v(top), ts) };
            unsafe { (*state).top.p = top.add(1) };
        }
        _ => {
            let ts = unsafe { luaS_newlstr(state, buff.b, buff.blen).cast::<TString>() };
            let top = unsafe { (*state).top.p };
            unsafe { setsvalue(s2v(top), ts) };
            unsafe { (*state).top.p = top.add(1) };
        }
    }
}

pub unsafe fn clearbuff(buff: *mut BuffFS) -> *const c_char {
    let state = unsafe { (*buff).l };
    let mut res = ptr::null();
    if unsafe { luaD_rawrunprotected(state, Some(pushbuff), buff.cast()) } == 0 {
        let top = unsafe { (*state).top.p.sub(1) };
        res = unsafe { getstr(tsvalue(s2v(top))) };
    }
    if unsafe { (*buff).b != (*buff).space.as_mut_ptr() } {
        unsafe { luaM_free_(state, (*buff).b.cast(), (*buff).buffsize) };
    }
    res
}

pub unsafe fn addstr2buff(buff: *mut BuffFS, str_ptr: *const c_char, slen: usize) {
    if unsafe { (*buff).err != 0 } {
        return;
    }
    let left = unsafe { (*buff).buffsize - (*buff).blen };
    if slen > left {
        if slen > (MAX_SIZE / 2).saturating_sub(unsafe { (*buff).blen }) {
            unsafe { ptr::copy_nonoverlapping(str_ptr, (*buff).b.add((*buff).blen), left) };
            unsafe { (*buff).blen = (*buff).buffsize };
            unsafe { (*buff).err = 2 };
            return;
        }
        let newsize = unsafe { (*buff).buffsize + slen };
        let newb = unsafe {
            if (*buff).b == (*buff).space.as_mut_ptr() {
                luaM_realloc_((*buff).l, ptr::null_mut(), 0, newsize).cast::<c_char>()
            } else {
                luaM_realloc_((*buff).l, (*buff).b.cast(), (*buff).buffsize, newsize)
                    .cast::<c_char>()
            }
        };
        if newb.is_null() {
            unsafe { (*buff).err = 1 };
            return;
        }
        if unsafe { (*buff).b == (*buff).space.as_mut_ptr() } {
            unsafe { ptr::copy_nonoverlapping((*buff).b, newb, (*buff).blen) };
        }
        unsafe {
            (*buff).b = newb;
            (*buff).buffsize = newsize;
        }
    }
    unsafe {
        ptr::copy_nonoverlapping(str_ptr, (*buff).b.add((*buff).blen), slen);
        (*buff).blen += slen;
    }
}

pub unsafe fn addnum2buff(buff: *mut BuffFS, num: *mut TValue) {
    let mut tmp = [0 as c_char; LUA_N2SBUFFSZ];
    let len = unsafe { luaO_tostringbuff(num, tmp.as_mut_ptr()) } as usize;
    unsafe { addstr2buff(buff, tmp.as_ptr(), len) };
}

/// 将 Rust `&str` 推入 Lua 栈并返回指向内部字符串的指针（替代 C 风格变参的 luaO_pushfstring）。
/// 调用方应先用 `format!()` 完成格式化，再调用此函数。
pub unsafe fn luaO_pushstr(state: *mut lua_State, s: &str) -> *const c_char {
    let ts = unsafe { luaS_newlstr(state, s.as_ptr().cast(), s.len()) }.cast::<TString>();
    let top = unsafe { (*state).top.p };
    unsafe { setsvalue(s2v(top), ts) };
    unsafe { (*state).top.p = top.add(1) };
    unsafe { getstr(ts) }
}

pub unsafe fn luaO_chunkid(out: *mut c_char, source: *const c_char, srclen: usize) {
    let source = unsafe { CStr::from_ptr(source) }.to_bytes();
    let mut outp = out;
    let mut bufflen = LUA_IDSIZE;
    match source.first().copied() {
        Some(b'=') => {
            if srclen <= bufflen {
                unsafe {
                    ptr::copy_nonoverlapping(source[1..].as_ptr().cast::<c_char>(), outp, srclen)
                };
            } else {
                unsafe {
                    ptr::copy_nonoverlapping(
                        source[1..bufflen].as_ptr().cast::<c_char>(),
                        outp,
                        bufflen - 1,
                    );
                    *outp.add(bufflen - 1) = 0;
                }
            }
        }
        Some(b'@') => {
            if srclen <= bufflen {
                unsafe {
                    ptr::copy_nonoverlapping(source[1..].as_ptr().cast::<c_char>(), outp, srclen)
                };
            } else {
                unsafe {
                    ptr::copy_nonoverlapping(RETS.as_ptr().cast::<c_char>(), outp, RETS.len());
                    outp = outp.add(RETS.len());
                }
                bufflen -= RETS.len();
                let start = 1 + srclen - bufflen;
                unsafe {
                    ptr::copy_nonoverlapping(
                        source[start..start + bufflen].as_ptr().cast::<c_char>(),
                        outp,
                        bufflen,
                    )
                };
            }
        }
        _ => {
            unsafe {
                ptr::copy_nonoverlapping(PRE.as_ptr().cast::<c_char>(), outp, PRE.len());
                outp = outp.add(PRE.len());
            }
            bufflen -= PRE.len() + RETS.len() + POS.len() + 1;
            let nl = source.iter().position(|&b| b == b'\n');
            let mut copy_len = srclen;
            if copy_len < bufflen && nl.is_none() {
                unsafe {
                    ptr::copy_nonoverlapping(source.as_ptr().cast::<c_char>(), outp, copy_len);
                    outp = outp.add(copy_len);
                }
            } else {
                if let Some(nl) = nl {
                    copy_len = nl;
                }
                if copy_len > bufflen {
                    copy_len = bufflen;
                }
                unsafe {
                    ptr::copy_nonoverlapping(source.as_ptr().cast::<c_char>(), outp, copy_len);
                    outp = outp.add(copy_len);
                    ptr::copy_nonoverlapping(RETS.as_ptr().cast::<c_char>(), outp, RETS.len());
                    outp = outp.add(RETS.len());
                }
            }
            unsafe {
                ptr::copy_nonoverlapping(POS.as_ptr().cast::<c_char>(), outp, POS.len());
                *outp.add(POS.len()) = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::lua_tolstring,
        aux_rs::{luaL_checkversion_, luaL_newstate},
        luaffi::LUAL_NUMSIZES,
        state::lua_close,
    };
    use core::mem::MaybeUninit;

    fn get_top_string(state: *mut lua_State) -> String {
        unsafe {
            let mut len = 0usize;
            let ptr = lua_tolstring(state, -1, &mut len);
            assert!(!ptr.is_null());
            String::from_utf8_lossy(core::slice::from_raw_parts(ptr.cast::<u8>(), len)).into()
        }
    }

    #[test]
    fn ceillog2_and_gc_params_match_expectations() {
        unsafe {
            assert_eq!(luaO_ceillog2(1), 0);
            assert_eq!(luaO_ceillog2(2), 1);
            assert_eq!(luaO_ceillog2(3), 2);
            assert_eq!(luaO_ceillog2(255), 8);
            assert_eq!(luaO_codeparam(0), 0);
            assert!(luaO_applyparam(luaO_codeparam(250), 100) >= 100);
        }
    }

    #[test]
    fn str2num_utf8_and_chunkid_work() {
        let mut value = MaybeUninit::<TValue>::uninit();
        unsafe {
            assert_eq!(luaO_str2num(c"123".as_ptr(), value.as_mut_ptr()), 4);
            assert!(ttisinteger(value.as_ptr()));
            assert_eq!(ivalue(value.as_ptr()), 123);

            assert_eq!(luaO_str2num(c"0x10".as_ptr(), value.as_mut_ptr()), 5);
            assert!(ttisinteger(value.as_ptr()));
            assert_eq!(ivalue(value.as_ptr()), 16);

            assert_eq!(luaO_str2num(c"1.25".as_ptr(), value.as_mut_ptr()), 5);
            assert!(ttisfloat(value.as_ptr()));
            assert_eq!(fltvalue(value.as_ptr()), 1.25);

            let mut utf8 = [0 as c_char; UTF8BUFFSZ];
            let len = luaO_utf8esc(utf8.as_mut_ptr(), 0x20ac) as usize;
            let bytes =
                core::slice::from_raw_parts(utf8.as_ptr().add(UTF8BUFFSZ - len).cast::<u8>(), len);
            assert_eq!(bytes, "€".as_bytes());

            let mut out = [0 as c_char; LUA_IDSIZE + 1];
            luaO_chunkid(
                out.as_mut_ptr(),
                c"@/tmp/some/very/long/path/test.lua".as_ptr(),
                33,
            );
            let s = CStr::from_ptr(out.as_ptr()).to_string_lossy().into_owned();
            assert!(s.ends_with("path/test.lua") || s.contains("..."));
        }
    }

    #[test]
    fn pushfstring_formats_expected_values() {
        let state = { luaL_newstate() };
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
            // 使用 Rust 格式化代替 C 变参格式化
            let euro = char::from_u32(0x20ac_u32).unwrap();
            let s = format!("item {} {} {} {} %", 7, 9_i64, 1.5_f64, euro);
            let _ = luaO_pushstr(state, &s);
            let rendered = get_top_string(state);
            assert!(rendered.starts_with("item 7 9 1.5 "));
            assert!(rendered.ends_with(" %"));
            assert!(rendered.contains('€'));
        })();

        unsafe { lua_close(state) };
        result
    }

    #[test]
    fn pushfstring_pointer_format() {
        let state = { luaL_newstate() };
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);

            // Format a non-null pointer — should produce "0x..." hex address
            let dummy: u64 = 0xDEAD;
            let ptr = &dummy as *const u64 as *mut core::ffi::c_void;
            let _ = luaO_pushstr(state, &format!("ptr={ptr:p}"));
            let rendered = get_top_string(state);
            assert!(
                rendered.starts_with("ptr=0x"),
                "expected 'ptr=0x...', got: {rendered}"
            );
            // The hex address should contain hex digits after "0x"
            let hex_part = &rendered["ptr=0x".len()..];
            assert!(!hex_part.is_empty(), "hex address should not be empty");
            assert!(
                hex_part.chars().all(|c| c.is_ascii_hexdigit()),
                "expected hex digits, got: {hex_part}"
            );

            // Format a null pointer
            let null_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
            let _ = luaO_pushstr(state, &format!("null={null_ptr:p}"));
            let rendered_null = get_top_string(state);
            assert!(
                rendered_null.starts_with("null=0x"),
                "expected 'null=0x...', got: {rendered_null}"
            );
        })();

        unsafe { lua_close(state) };
        result
    }

    #[test]
    fn tostringbuff_integers() {
        unsafe {
            let mut buff = [0 as c_char; LUA_N2SBUFFSZ];

            // Zero
            let mut val = MaybeUninit::<TValue>::uninit();
            setivalue(val.as_mut_ptr(), 0);
            let len = luaO_tostringbuff(val.as_ptr(), buff.as_mut_ptr()) as usize;
            let s =
                core::str::from_utf8(core::slice::from_raw_parts(buff.as_ptr().cast::<u8>(), len))
                    .unwrap();
            assert_eq!(s, "0");

            // Positive
            setivalue(val.as_mut_ptr(), 12345);
            let len = luaO_tostringbuff(val.as_ptr(), buff.as_mut_ptr()) as usize;
            let s =
                core::str::from_utf8(core::slice::from_raw_parts(buff.as_ptr().cast::<u8>(), len))
                    .unwrap();
            assert_eq!(s, "12345");

            // Negative
            setivalue(val.as_mut_ptr(), -9876);
            let len = luaO_tostringbuff(val.as_ptr(), buff.as_mut_ptr()) as usize;
            let s =
                core::str::from_utf8(core::slice::from_raw_parts(buff.as_ptr().cast::<u8>(), len))
                    .unwrap();
            assert_eq!(s, "-9876");

            // Large
            setivalue(val.as_mut_ptr(), 9007199254740992);
            let len = luaO_tostringbuff(val.as_ptr(), buff.as_mut_ptr()) as usize;
            let s =
                core::str::from_utf8(core::slice::from_raw_parts(buff.as_ptr().cast::<u8>(), len))
                    .unwrap();
            assert_eq!(s, "9007199254740992");
        }
    }

    #[test]
    fn tostringbuff_floats() {
        unsafe {
            let mut buff = [0 as c_char; LUA_N2SBUFFSZ];

            // Simple float
            let mut val = MaybeUninit::<TValue>::uninit();
            setfltvalue(val.as_mut_ptr(), 3.14);
            let len = luaO_tostringbuff(val.as_ptr(), buff.as_mut_ptr()) as usize;
            let s =
                core::str::from_utf8(core::slice::from_raw_parts(buff.as_ptr().cast::<u8>(), len))
                    .unwrap();
            assert_eq!(s, "3.14");

            // Float that looks like integer should have ".0"
            setfltvalue(val.as_mut_ptr(), 100.0);
            let len = luaO_tostringbuff(val.as_ptr(), buff.as_mut_ptr()) as usize;
            let s =
                core::str::from_utf8(core::slice::from_raw_parts(buff.as_ptr().cast::<u8>(), len))
                    .unwrap();
            assert!(
                s.contains('.'),
                "100.0 should contain decimal point, got: {s}"
            );

            // Zero float
            setfltvalue(val.as_mut_ptr(), 0.0);
            let len = luaO_tostringbuff(val.as_ptr(), buff.as_mut_ptr()) as usize;
            let s =
                core::str::from_utf8(core::slice::from_raw_parts(buff.as_ptr().cast::<u8>(), len))
                    .unwrap();
            assert!(
                s.contains('.'),
                "0.0 should contain decimal point, got: {s}"
            );

            // Round-trip fidelity: the string should parse back to the same value
            let test_values = [
                0.1,
                0.2,
                1.0 / 3.0,
                std::f64::consts::PI,
                1e-10,
                1e10,
                1e100,
            ];
            for &n in &test_values {
                setfltvalue(val.as_mut_ptr(), n);
                let len = luaO_tostringbuff(val.as_ptr(), buff.as_mut_ptr()) as usize;
                let s = core::str::from_utf8(core::slice::from_raw_parts(
                    buff.as_ptr().cast::<u8>(),
                    len,
                ))
                .unwrap();
                let back: f64 = s
                    .parse()
                    .unwrap_or_else(|e| panic!("failed to parse '{s}': {e}"));
                assert_eq!(
                    back, n,
                    "round-trip failed for {n}: tostring => '{s}', parse => {back}"
                );
            }
        }
    }
}
