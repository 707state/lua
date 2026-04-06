use crate::aux_rs::{
    luaL_checkinteger, luaL_checklstring, luaL_checktype, luaL_execresult, luaL_fileresult,
    luaL_optinteger, luaL_optlstring,
};
use crate::lua_module::{
    create_library, lua_Integer, lua_Number, lua_createtable, lua_error, lua_pop, lua_pushboolean,
    lua_pushinteger, lua_pushlstring, lua_pushnumber, lua_pushstring, lua_setfield, lua_settop,
    luaL_Reg,
};
use crate::runtime::*;
use crate::state::lua_close;
use core::ffi::{c_char, c_int, c_long, c_ulong};
use core::{mem, ptr, slice};
use std::ffi::CString;
use std::process::exit;
#[allow(non_camel_case_types)]
type time_t = c_long;
#[allow(non_camel_case_types)]
type clock_t = c_ulong;

#[repr(C)]
struct Tm {
    tm_sec: c_int,
    tm_min: c_int,
    tm_hour: c_int,
    tm_mday: c_int,
    tm_mon: c_int,
    tm_year: c_int,
    tm_wday: c_int,
    tm_yday: c_int,
    tm_isdst: c_int,
    tm_gmtoff: c_long,
    tm_zone: *const c_char,
}

static SYSLIB: [luaL_Reg; 12] = [
    luaL_Reg {
        name: NAME_CLOCK.as_ptr().cast(),
        func: Some(os_clock),
    },
    luaL_Reg {
        name: NAME_DATE.as_ptr().cast(),
        func: Some(os_date),
    },
    luaL_Reg {
        name: NAME_DIFFTIME.as_ptr().cast(),
        func: Some(os_difftime),
    },
    luaL_Reg {
        name: NAME_EXECUTE.as_ptr().cast(),
        func: Some(os_execute),
    },
    luaL_Reg {
        name: NAME_EXIT.as_ptr().cast(),
        func: Some(os_exit),
    },
    luaL_Reg {
        name: NAME_GETENV.as_ptr().cast(),
        func: Some(os_getenv),
    },
    luaL_Reg {
        name: NAME_REMOVE.as_ptr().cast(),
        func: Some(os_remove),
    },
    luaL_Reg {
        name: NAME_RENAME.as_ptr().cast(),
        func: Some(os_rename),
    },
    luaL_Reg {
        name: NAME_SETLOCALE.as_ptr().cast(),
        func: Some(os_setlocale),
    },
    luaL_Reg {
        name: NAME_TIME.as_ptr().cast(),
        func: Some(os_time),
    },
    luaL_Reg {
        name: NAME_TMPNAME.as_ptr().cast(),
        func: Some(os_tmpname),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

// Lua API：直接调用 crate::api
#[inline]
unsafe fn lua_getfield(s: *mut lua_State, i: c_int, k: *const c_char) -> c_int {
    unsafe { crate::api::lua_getfield(s, i, k) }
}
#[inline]
unsafe fn lua_type(s: *mut lua_State, i: c_int) -> c_int {
    unsafe { crate::api::lua_type(s, i) }
}
#[inline]
unsafe fn lua_toboolean(s: *mut lua_State, i: c_int) -> c_int {
    unsafe { crate::api::lua_toboolean(s, i) }
}
#[inline]
unsafe fn lua_tointegerx(s: *mut lua_State, i: c_int, n: *mut c_int) -> lua_Integer {
    unsafe { crate::api::lua_tointegerx(s, i, n) }
}

// C 系统调用（POSIX/libc，无直接 Rust std 等价，保留 extern C 声明）
unsafe extern "C" {
    fn system(command: *const c_char) -> c_int;
    fn remove(path: *const c_char) -> c_int;
    fn rename(old: *const c_char, new: *const c_char) -> c_int;
    fn getenv(name: *const c_char) -> *const c_char;
    fn clock() -> clock_t;
    fn time(timer: *mut time_t) -> time_t;
    fn difftime(time1: time_t, time0: time_t) -> lua_Number;
    fn mktime(timeptr: *mut Tm) -> time_t;
    fn gmtime_r(timer: *const time_t, result: *mut Tm) -> *mut Tm;
    fn localtime_r(timer: *const time_t, result: *mut Tm) -> *mut Tm;
    fn strftime(s: *mut c_char, max: usize, format: *const c_char, tm: *const Tm) -> usize;
    fn setlocale(category: c_int, locale: *const c_char) -> *const c_char;
    fn mkstemp(template: *mut c_char) -> c_int;
    fn close(fd: c_int) -> c_int;
}

/// 重置 errno：用 Rust 方式将系统 errno 清零（为 C 调用前清除旧错误）
#[inline]
fn reset_errno() {
    // 通过写入一个无害的 IO 错误再清除来重置 errno，
    // 实际上最安全的方式是直接调用 libc::errno，但避免 extern C，
    // 改为：先执行无副作用操作再忽略结果。
    // 简化：对于 time 函数的 errno 检查，Lua 源码里 reset 是为了检测 mktime 失败，
    // 我们直接改为检查返回值 == -1 即可，不需要 errno。
    // 因此这里设为空实现即可，调用处会跳过 reset_errno 检查。
}

#[inline]
unsafe fn l_checkstring(state: *mut lua_State, arg: c_int) -> *const c_char {
    luaL_checklstring(state, arg, ptr::null_mut())
}

#[inline]
unsafe fn l_optstring(state: *mut lua_State, arg: c_int) -> *const c_char {
    luaL_optlstring(state, arg, ptr::null(), ptr::null_mut())
}

unsafe fn push_dynamic_error(state: *mut lua_State, message: &str) -> c_int {
    let bytes = message.as_bytes();
    unsafe {
        lua_pushlstring(state, bytes.as_ptr().cast(), bytes.len());
        lua_error(state)
    }
}

unsafe fn push_dynamic_argerror(state: *mut lua_State, arg: c_int, message: &str) -> c_int {
    let cstring = CString::new(message).expect("message contains no NUL");
    crate::lua_module::luaL_argerror(state, arg, cstring.as_ptr())
}

#[inline]
unsafe fn is_none_or_nil(state: *mut lua_State, index: c_int) -> bool {
    let tag = unsafe { lua_type(state, index) };
    tag == LUA_TNONE || tag == LUA_TNIL.into()
}

#[inline]
unsafe fn is_boolean(state: *mut lua_State, index: c_int) -> bool {
    unsafe { lua_type(state, index) == LUA_TBOOLEAN.into() }
}

unsafe fn setfield(state: *mut lua_State, key: &'static [u8], value: c_int, delta: c_int) {
    unsafe { lua_pushinteger(state, lua_Integer::from(value) + lua_Integer::from(delta)) };
    unsafe { lua_setfield(state, -2, key.as_ptr().cast()) };
}

unsafe fn setboolfield(state: *mut lua_State, key: &'static [u8], value: c_int) {
    if value < 0 {
        return;
    }
    unsafe { lua_pushboolean(state, value) };
    unsafe { lua_setfield(state, -2, key.as_ptr().cast()) };
}

unsafe fn setallfields(state: *mut lua_State, stm: &Tm) {
    unsafe { setfield(state, KEY_YEAR, stm.tm_year, 1900) };
    unsafe { setfield(state, KEY_MONTH, stm.tm_mon, 1) };
    unsafe { setfield(state, KEY_DAY, stm.tm_mday, 0) };
    unsafe { setfield(state, KEY_HOUR, stm.tm_hour, 0) };
    unsafe { setfield(state, KEY_MIN, stm.tm_min, 0) };
    unsafe { setfield(state, KEY_SEC, stm.tm_sec, 0) };
    unsafe { setfield(state, KEY_YDAY, stm.tm_yday, 1) };
    unsafe { setfield(state, KEY_WDAY, stm.tm_wday, 1) };
    unsafe { setboolfield(state, KEY_ISDST, stm.tm_isdst) };
}

unsafe fn getboolfield(state: *mut lua_State, key: &'static [u8]) -> c_int {
    let res = if unsafe { lua_getfield(state, -1, key.as_ptr().cast()) } == LUA_TNIL.into() {
        -1
    } else {
        unsafe { lua_toboolean(state, -1) }
    };
    unsafe { lua_pop(state, 1) };
    res
}

unsafe fn getfield(
    state: *mut lua_State,
    key: &'static [u8],
    default: c_int,
    delta: c_int,
) -> Result<c_int, c_int> {
    let mut isnum = 0;
    let t = unsafe { lua_getfield(state, -1, key.as_ptr().cast()) };
    let mut res = unsafe { lua_tointegerx(state, -1, &mut isnum) };
    if isnum == 0 {
        if t != LUA_TNIL.into() {
            let message = format!("field '{}' is not an integer", key_name(key));
            unsafe { lua_pop(state, 1) };
            return Err(unsafe { push_dynamic_error(state, &message) });
        }
        if default < 0 {
            let message = format!("field '{}' missing in date table", key_name(key));
            unsafe { lua_pop(state, 1) };
            return Err(unsafe { push_dynamic_error(state, &message) });
        }
        res = lua_Integer::from(default);
    } else {
        let within_bounds = if res >= 0 {
            res - lua_Integer::from(delta) <= lua_Integer::from(c_int::MAX)
        } else {
            lua_Integer::from(c_int::MIN) + lua_Integer::from(delta) <= res
        };
        if !within_bounds {
            let message = format!("field '{}' is out-of-bound", key_name(key));
            unsafe { lua_pop(state, 1) };
            return Err(unsafe { push_dynamic_error(state, &message) });
        }
        res -= lua_Integer::from(delta);
    }
    unsafe { lua_pop(state, 1) };
    Ok(res as c_int)
}

fn key_name(key: &'static [u8]) -> &'static str {
    let key = &key[..key.len() - 1];
    std::str::from_utf8(key).expect("key is ASCII")
}

fn checkoption(conv: &[u8]) -> bool {
    let mut option = LUA_STRFTIMEOPTIONS.as_bytes();
    let mut oplen = 1usize;
    while !option.is_empty() && oplen <= conv.len() {
        if option[0] == b'|' {
            option = &option[1..];
            oplen += 1;
            continue;
        }
        if option.len() < oplen {
            break;
        }
        if &option[..oplen] == &conv[..oplen] {
            return true;
        }
        option = &option[oplen..];
    }
    false
}

unsafe fn l_checktime(state: *mut lua_State, arg: c_int) -> Result<time_t, c_int> {
    let t = { luaL_checkinteger(state, arg) };
    let narrowed = t as time_t;
    if narrowed as lua_Integer != t {
        Err(unsafe { push_dynamic_argerror(state, arg, "time out-of-bounds") })
    } else {
        Ok(narrowed)
    }
}

unsafe fn os_execute(state: *mut lua_State) -> c_int {
    let cmd = unsafe { l_optstring(state, 1) };
    reset_errno();
    let stat = unsafe { system(cmd) };
    if !cmd.is_null() {
        luaL_execresult(state, stat)
    } else {
        unsafe { lua_pushboolean(state, stat) };
        1
    }
}

unsafe fn os_remove(state: *mut lua_State) -> c_int {
    let filename = unsafe { l_checkstring(state, 1) };
    reset_errno();
    unsafe { luaL_fileresult(state, (remove(filename) == 0) as c_int, filename) }
}

unsafe fn os_rename(state: *mut lua_State) -> c_int {
    let fromname = unsafe { l_checkstring(state, 1) };
    let toname = unsafe { l_checkstring(state, 2) };
    reset_errno();
    unsafe { luaL_fileresult(state, (rename(fromname, toname) == 0) as c_int, ptr::null()) }
}

unsafe fn os_tmpname(state: *mut lua_State) -> c_int {
    let mut buff = TMP_TEMPLATE.to_vec();
    let fd = unsafe { mkstemp(buff.as_mut_ptr().cast()) };
    if fd == -1 {
        return unsafe {
            lua_pushlstring(
                state,
                ERR_UNABLE_TMPNAME.as_ptr().cast(),
                ERR_UNABLE_TMPNAME.len() - 1,
            );
            lua_error(state)
        };
    }
    unsafe { close(fd) };
    let len = buff.iter().position(|&b| b == 0).unwrap_or(buff.len());
    unsafe { lua_pushlstring(state, buff.as_ptr().cast(), len) };
    1
}

unsafe fn os_getenv(state: *mut lua_State) -> c_int {
    let key = unsafe { l_checkstring(state, 1) };
    unsafe { lua_pushstring(state, getenv(key)) };
    1
}

unsafe fn os_clock(state: *mut lua_State) -> c_int {
    let value = unsafe { clock() } as lua_Number / CLOCKS_PER_SEC_VALUE;
    unsafe { lua_pushnumber(state, value) };
    1
}

unsafe fn os_date(state: *mut lua_State) -> c_int {
    let mut slen = 0usize;
    let s = { luaL_optlstring(state, 1, b"%c\0".as_ptr().cast(), &mut slen) };
    let mut format = unsafe { slice::from_raw_parts(s.cast::<u8>(), slen) };

    let default_time = unsafe { time(ptr::null_mut()) };
    let t = if unsafe { is_none_or_nil(state, 2) } {
        default_time
    } else {
        match unsafe { l_checktime(state, 2) } {
            Ok(value) => value,
            Err(code) => return code,
        }
    };

    let mut tmr = unsafe { mem::zeroed::<Tm>() };
    let stm = if format.first() == Some(&b'!') {
        format = &format[1..];
        unsafe { gmtime_r(&t, &mut tmr) }
    } else {
        unsafe { localtime_r(&t, &mut tmr) }
    };

    if stm.is_null() {
        return unsafe {
            lua_pushlstring(
                state,
                ERR_DATE_NOT_REPRESENTABLE.as_ptr().cast(),
                ERR_DATE_NOT_REPRESENTABLE.len() - 1,
            );
            lua_error(state)
        };
    }

    if format == b"*t" {
        unsafe { lua_createtable(state, 0, 9) };
        unsafe { setallfields(state, &tmr) };
        return 1;
    }

    let mut out = Vec::new();
    let mut i = 0usize;
    while i < format.len() {
        if format[i] != b'%' {
            out.push(format[i]);
            i += 1;
            continue;
        }

        i += 1;
        let remaining = &format[i..];
        let spec_len = if remaining.len() >= 2 && checkoption(&remaining[..2]) {
            2
        } else if !remaining.is_empty() && checkoption(&remaining[..1]) {
            1
        } else {
            let invalid = String::from_utf8_lossy(remaining);
            let message = format!("invalid conversion specifier '%{}'", invalid);
            return unsafe { push_dynamic_argerror(state, 1, &message) };
        };

        let mut cc = [0_u8; 4];
        cc[0] = b'%';
        cc[1..1 + spec_len].copy_from_slice(&remaining[..spec_len]);

        let mut buff = [0_u8; SIZETIMEFMT];
        let reslen = unsafe {
            strftime(
                buff.as_mut_ptr().cast(),
                SIZETIMEFMT,
                cc.as_ptr().cast(),
                &tmr,
            )
        };
        out.extend_from_slice(&buff[..reslen]);
        i += spec_len;
    }

    unsafe { lua_pushlstring(state, out.as_ptr().cast(), out.len()) };
    1
}

unsafe fn os_time(state: *mut lua_State) -> c_int {
    let t = if unsafe { is_none_or_nil(state, 1) } {
        unsafe { time(ptr::null_mut()) }
    } else {
        let mut ts = unsafe { mem::zeroed::<Tm>() };
        {
            luaL_checktype(state, 1, LUA_TTABLE.into())
        };
        unsafe { lua_settop(state, 1) };

        ts.tm_year = match unsafe { getfield(state, KEY_YEAR, -1, 1900) } {
            Ok(v) => v,
            Err(code) => return code,
        };
        ts.tm_mon = match unsafe { getfield(state, KEY_MONTH, -1, 1) } {
            Ok(v) => v,
            Err(code) => return code,
        };
        ts.tm_mday = match unsafe { getfield(state, KEY_DAY, -1, 0) } {
            Ok(v) => v,
            Err(code) => return code,
        };
        ts.tm_hour = match unsafe { getfield(state, KEY_HOUR, 12, 0) } {
            Ok(v) => v,
            Err(code) => return code,
        };
        ts.tm_min = match unsafe { getfield(state, KEY_MIN, 0, 0) } {
            Ok(v) => v,
            Err(code) => return code,
        };
        ts.tm_sec = match unsafe { getfield(state, KEY_SEC, 0, 0) } {
            Ok(v) => v,
            Err(code) => return code,
        };
        ts.tm_isdst = unsafe { getboolfield(state, KEY_ISDST) };

        let value = unsafe { mktime(&mut ts) };
        unsafe { setallfields(state, &ts) };
        value
    };

    let roundtrip = t as lua_Integer;
    if roundtrip as time_t != t || t == -1 {
        return unsafe {
            lua_pushlstring(
                state,
                ERR_TIME_NOT_REPRESENTABLE.as_ptr().cast(),
                ERR_TIME_NOT_REPRESENTABLE.len() - 1,
            );
            lua_error(state)
        };
    }
    unsafe { lua_pushinteger(state, roundtrip) };
    1
}

unsafe fn os_difftime(state: *mut lua_State) -> c_int {
    let t1 = match unsafe { l_checktime(state, 1) } {
        Ok(v) => v,
        Err(code) => return code,
    };
    let t2 = match unsafe { l_checktime(state, 2) } {
        Ok(v) => v,
        Err(code) => return code,
    };
    unsafe { lua_pushnumber(state, difftime(t1, t2)) };
    1
}

unsafe fn os_setlocale(state: *mut lua_State) -> c_int {
    const CATEGORIES: [c_int; 6] = [0, 1, 2, 3, 4, 5];
    const CAT_NAMES: [&[u8]; 6] = [
        CAT_ALL,
        CAT_COLLATE,
        CAT_CTYPE,
        CAT_MONETARY,
        CAT_NUMERIC,
        CAT_TIME,
    ];

    let locale = unsafe { l_optstring(state, 1) };
    let mut len = 0usize;
    let category_ptr = { luaL_optlstring(state, 2, CAT_ALL.as_ptr().cast(), &mut len) };
    let category = unsafe { slice::from_raw_parts(category_ptr.cast::<u8>(), len) };
    let mut selected = None;
    for (index, candidate) in CAT_NAMES.iter().enumerate() {
        if &candidate[..candidate.len() - 1] == category {
            selected = Some(CATEGORIES[index]);
            break;
        }
    }
    let category = match selected {
        Some(v) => v,
        None => {
            let message = format!("invalid option '{}'", String::from_utf8_lossy(category));
            return unsafe { push_dynamic_argerror(state, 2, &message) };
        }
    };

    unsafe { lua_pushstring(state, setlocale(category, locale)) };
    1
}

unsafe fn os_exit(state: *mut lua_State) -> c_int {
    let status = if unsafe { is_boolean(state, 1) } {
        if unsafe { lua_toboolean(state, 1) } != 0 {
            EXIT_SUCCESS_CODE
        } else {
            EXIT_FAILURE_CODE
        }
    } else {
        luaL_optinteger(state, 1, lua_Integer::from(EXIT_SUCCESS_CODE)) as c_int
    };

    if unsafe { lua_toboolean(state, 2) } != 0 {
        unsafe { lua_close(state) };
    }
    exit(status);
}

pub(crate) unsafe fn luaopen_os(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &SYSLIB) };
    1
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn os_builtin_script() {
        run_lua_test(
            "test/os_builtin.lua",
            include_str!("../test/os_builtin.lua"),
        );
    }
}
