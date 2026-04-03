use crate::api::*;
use crate::aux_rs::{luaL_checkinteger, luaL_checklstring, luaL_checkstack, luaL_optinteger};
use crate::lua_module::*;
use crate::runtime::*;
use core::ffi::c_int;
use core::ptr;
static UTF8LIB_REGS: [luaL_Reg; 6] = [
    luaL_Reg {
        name: NAME_OFFSET.as_ptr().cast(),
        func: Some(byteoffset),
    },
    luaL_Reg {
        name: NAME_CODEPOINT.as_ptr().cast(),
        func: Some(codepoint),
    },
    luaL_Reg {
        name: NAME_CHAR.as_ptr().cast(),
        func: Some(utfchar),
    },
    luaL_Reg {
        name: NAME_LEN.as_ptr().cast(),
        func: Some(utflen),
    },
    luaL_Reg {
        name: NAME_CODES.as_ptr().cast(),
        func: Some(iter_codes),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

#[inline]
fn iscont(byte: u8) -> bool {
    (byte & 0xC0) == 0x80
}

#[inline]
unsafe fn iscontp(ptr: *const u8) -> bool {
    iscont(unsafe { *ptr })
}

#[inline]
fn u_posrelat(pos: lua_Integer, len: usize) -> lua_Integer {
    if pos >= 0 {
        pos
    } else if (0usize).wrapping_sub(pos as usize) > len {
        0
    } else {
        len as lua_Integer + pos + 1
    }
}

unsafe fn utf8_decode(mut s: *const u8, val: *mut u32, strict: bool) -> *const u8 {
    const LIMITS: [u32; 6] = [u32::MAX, 0x80, 0x800, 0x10000, 0x200000, 0x4000000];

    let mut c = unsafe { *s } as u32;
    let mut res = 0_u32;
    if c < 0x80 {
        res = c;
    } else {
        let mut count = 0_usize;
        while (c & 0x40) != 0 {
            count += 1;
            let cc = unsafe { *s.add(count) };
            if !iscont(cc) {
                return ptr::null();
            }
            res = (res << 6) | u32::from(cc & 0x3F);
            c <<= 1;
        }
        res |= (c & 0x7F) << (count * 5);
        if count > 5 || res > MAXUTF || res < LIMITS[count] {
            return ptr::null();
        }
        s = unsafe { s.add(count) };
    }
    if strict && (res > MAXUNICODE || (0xD800..=0xDFFF).contains(&res)) {
        return ptr::null();
    }
    if !val.is_null() {
        unsafe { *val = res };
    }
    unsafe { s.add(1) }
}

fn encode_utf8(mut code: u32, out: &mut [u8; 6]) -> usize {
    if code <= 0x7F {
        out[0] = code as u8;
        return 1;
    }

    let (len, lead_mask) = if code <= 0x7FF {
        (2, 0xC0)
    } else if code <= 0xFFFF {
        (3, 0xE0)
    } else if code <= 0x1F_FFFF {
        (4, 0xF0)
    } else if code <= 0x3FF_FFFF {
        (5, 0xF8)
    } else {
        (6, 0xFC)
    };

    for i in (1..len).rev() {
        out[i] = 0x80 | (code as u8 & 0x3F);
        code >>= 6;
    }
    out[0] = lead_mask | code as u8;
    len
}

unsafe  fn utflen(state: *mut lua_State) -> c_int {
    let mut len = 0_usize;
    let s = luaL_checklstring(state, 1, &mut len).cast::<u8>();
    let mut posi = u_posrelat(luaL_optinteger(state, 2, 1), len);
    let mut posj = u_posrelat(luaL_optinteger(state, 3, -1), len);
    let lax = unsafe { lua_toboolean(state, 4) } != 0;

    unsafe {
        argcheck(
            state,
            posi >= 1 && {
                posi -= 1;
                posi <= len as lua_Integer
            },
            2,
            ERR_INITIAL_POSITION_OUT_OF_BOUNDS,
        );
        argcheck(
            state,
            {
                posj -= 1;
                posj < len as lua_Integer
            },
            3,
            ERR_FINAL_POSITION_OUT_OF_BOUNDS,
        );
    }

    let mut n = 0_i64;
    while posi <= posj {
        let s1 = unsafe { utf8_decode(s.add(posi as usize), ptr::null_mut(), !lax) };
        if s1.is_null() {
            unsafe { push_fail(state) };
            unsafe { lua_pushinteger(state, posi + 1) };
            return 2;
        }
        posi = unsafe { s1.offset_from(s) as lua_Integer };
        n += 1;
    }
    unsafe { lua_pushinteger(state, n) };
    1
}

unsafe  fn codepoint(state: *mut lua_State) -> c_int {
    let mut len = 0_usize;
    let s = luaL_checklstring(state, 1, &mut len).cast::<u8>();
    let posi = u_posrelat(luaL_optinteger(state, 2, 1), len);
    let pose = u_posrelat(luaL_optinteger(state, 3, posi), len);
    let lax = unsafe { lua_toboolean(state, 4) } != 0;

    unsafe {
        argcheck(state, posi >= 1, 2, ERR_OUT_OF_BOUNDS);
        argcheck(state, pose <= len as lua_Integer, 3, ERR_OUT_OF_BOUNDS);
    }
    if posi > pose {
        return 0;
    }
    if pose - posi >= i32::MAX as lua_Integer {
        return unsafe { raise_error(state, ERR_STRING_SLICE_TOO_LONG) };
    }

    let mut max_returns = (pose - posi + 1) as c_int;
        luaL_checkstack(
            state,
            max_returns,
            ERR_STRING_SLICE_TOO_LONG.as_ptr().cast(),
        )
    ;

    let mut current = unsafe { s.add((posi - 1) as usize) };
    let end = unsafe { s.add(pose as usize) };
    max_returns = 0;
    while current < end {
        let mut code = 0_u32;
        current = unsafe { utf8_decode(current, &mut code, !lax) };
        if current.is_null() {
            return unsafe { raise_error(state, MSG_INVALID) };
        }
        unsafe { lua_pushinteger(state, code as lua_Integer) };
        max_returns += 1;
    }
    max_returns
}

unsafe fn pushutfchar(state: *mut lua_State, arg: c_int, out: &mut Vec<u8>) {
    let code = luaL_checkinteger(state, arg) as lua_Unsigned;
    unsafe {
        argcheck(
            state,
            code <= MAXUTF as lua_Unsigned,
            arg,
            ERR_VALUE_OUT_OF_RANGE,
        )
    };

    let mut buf = [0_u8; 6];
    let len = encode_utf8(code as u32, &mut buf);
    out.extend_from_slice(&buf[..len]);
}

unsafe  fn utfchar(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    if n == 1 {
        let mut bytes = Vec::with_capacity(6);
        unsafe { pushutfchar(state, 1, &mut bytes) };
        unsafe { lua_pushlstring(state, bytes.as_ptr().cast(), bytes.len()) };
        return 1;
    }

    let mut bytes = Vec::new();
    for i in 1..=n {
        unsafe { pushutfchar(state, i, &mut bytes) };
    }
    unsafe { lua_pushlstring(state, bytes.as_ptr().cast(), bytes.len()) };
    1
}

unsafe  fn byteoffset(state: *mut lua_State) -> c_int {
    let mut len = 0_usize;
    let s = luaL_checklstring(state, 1, &mut len).cast::<u8>();
    let mut n = luaL_checkinteger(state, 2);
    let mut posi = if n >= 0 { 1 } else { len as lua_Integer + 1 };
    posi = u_posrelat(luaL_optinteger(state, 3, posi), len);
    unsafe {
        argcheck(
            state,
            posi >= 1 && {
                posi -= 1;
                posi <= len as lua_Integer
            },
            3,
            ERR_POSITION_OUT_OF_BOUNDS,
        );
    }

    if n == 0 {
        while posi > 0 && unsafe { iscontp(s.add(posi as usize)) } {
            posi -= 1;
        }
    } else {
        if unsafe { iscontp(s.add(posi as usize)) } {
            return unsafe { raise_error(state, ERR_INITIAL_CONTINUATION) };
        }
        if n < 0 {
            while n < 0 && posi > 0 {
                loop {
                    posi -= 1;
                    if posi == 0 || !unsafe { iscontp(s.add(posi as usize)) } {
                        break;
                    }
                }
                n += 1;
            }
        } else {
            n -= 1;
            while n > 0 && posi < len as lua_Integer {
                loop {
                    posi += 1;
                    if !unsafe { iscontp(s.add(posi as usize)) } {
                        break;
                    }
                }
                n -= 1;
            }
        }
    }

    if n != 0 {
        unsafe { push_fail(state) };
        return 1;
    }

    unsafe { lua_pushinteger(state, posi + 1) };
    if (unsafe { *s.add(posi as usize) } & 0x80) != 0 {
        if iscont(unsafe { *s.add(posi as usize) }) {
            return unsafe { raise_error(state, ERR_INITIAL_CONTINUATION) };
        }
        while unsafe { iscontp(s.add(posi as usize + 1)) } {
            posi += 1;
        }
    }
    unsafe { lua_pushinteger(state, posi + 1) };
    2
}

unsafe fn iter_aux(state: *mut lua_State, strict: bool) -> c_int {
    let mut len = 0_usize;
    let s = luaL_checklstring(state, 1, &mut len).cast::<u8>();
    let end = unsafe { s.add(len) };
    let mut n = unsafe { lua_tointegerx(state, 2, ptr::null_mut()) } as lua_Unsigned;
    if n < len as lua_Unsigned {
        while unsafe { iscontp(s.add(n as usize)) } {
            n += 1;
        }
    }
    if n >= len as lua_Unsigned {
        return 0;
    }

    let mut code = 0_u32;
    let next = unsafe { utf8_decode(s.add(n as usize), &mut code, strict) };
    if next.is_null() || (next < end && unsafe { iscontp(next) }) {
        return unsafe { raise_error(state, MSG_INVALID) };
    }
    unsafe { lua_pushinteger(state, (n + 1) as lua_Integer) };
    unsafe { lua_pushinteger(state, code as lua_Integer) };
    2
}

unsafe  fn iter_auxstrict(state: *mut lua_State) -> c_int {
    unsafe { iter_aux(state, true) }
}

unsafe  fn iter_auxlax(state: *mut lua_State) -> c_int {
    unsafe { iter_aux(state, false) }
}

unsafe  fn iter_codes(state: *mut lua_State) -> c_int {
    let lax = unsafe { lua_toboolean(state, 2) } != 0;
    let mut len = 0_usize;
    let s = luaL_checklstring(state, 1, &mut len).cast::<u8>();
    unsafe { argcheck(state, len == 0 || !iscontp(s), 1, MSG_INVALID) };
    unsafe {
        crate::api::lua_pushcclosure(
            state,
            if lax {
                Some(iter_auxlax)
            } else {
                Some(iter_auxstrict)
            },
            0,
        )
    };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_pushinteger(state, 0) };
    3
}

pub(crate) unsafe  fn luaopen_utf8(state: *mut lua_State) -> c_int {
    unsafe { create_library_with_nrec(state, &UTF8LIB_REGS, 6) };
    unsafe { lua_pushlstring(state, UTF8PATT.as_ptr().cast(), UTF8PATT.len()) };
    unsafe { lua_setfield(state, -2, FIELD_CHARPATTERN.as_ptr().cast()) };
    1
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn utf8_builtin_script() {
        run_lua_test(
            "test/utf8_builtin.lua",
            include_str!("../test/utf8_builtin.lua"),
        );
    }
}
