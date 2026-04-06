//! `os` 标准库 — 纯 Rust 实现
//!
//! 所有操作系统交互均通过 `std::time`, `std::process`, `std::env`,
//! `std::fs` 完成，不调用任何 C 符号。
//!
//! # 与标准 Lua 的语义差异
//!
//! * `os.date` 格式化通过 Rust 手动实现，不依赖 `strftime`
//! * `os.clock` 使用进程 CPU 时间近似值（`std::time::Instant` 自进程启动）
//! * `os.tmpname` 使用 `std::env::temp_dir()` + 随机后缀

use crate::aux_rs::{luaL_checkinteger, luaL_checktype, luaL_optinteger, luaL_optlstring};
use crate::lua_module::{
    LuaFnList, lua_Integer, lua_Number, lua_State, lua_createtable, lua_error, lua_pop,
    lua_pushboolean, lua_pushinteger, lua_pushlstring, lua_pushnumber, lua_setfield, lua_settop,
    register_lib,
};
use crate::runtime::*;
use core::ffi::c_int;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ─── 函数表 ────────────────────────────────────────────────────────────────

pub(crate) static SYSLIB: LuaFnList = &[
    ("clock", os_clock as unsafe fn(*mut lua_State) -> c_int),
    ("date", os_date as unsafe fn(*mut lua_State) -> c_int),
    (
        "difftime",
        os_difftime as unsafe fn(*mut lua_State) -> c_int,
    ),
    ("execute", os_execute as unsafe fn(*mut lua_State) -> c_int),
    ("exit", os_exit as unsafe fn(*mut lua_State) -> c_int),
    ("getenv", os_getenv as unsafe fn(*mut lua_State) -> c_int),
    ("remove", os_remove as unsafe fn(*mut lua_State) -> c_int),
    ("rename", os_rename as unsafe fn(*mut lua_State) -> c_int),
    ("time", os_time as unsafe fn(*mut lua_State) -> c_int),
    ("tmpname", os_tmpname as unsafe fn(*mut lua_State) -> c_int),
];

// ─── 辅助函数 ────────────────────────────────────────────────────────────────

/// 向栈推送 Rust 字符串切片作为 Lua 字符串
#[inline]
unsafe fn push_str(state: *mut lua_State, s: &str) {
    unsafe {
        lua_pushlstring(state, s.as_ptr().cast(), s.len());
    }
}

/// 推送运行时错误（永不返回，通过 Lua long jump 跳转）
#[cold]
unsafe fn push_error(state: *mut lua_State, msg: String) -> c_int {
    unsafe { push_str(state, &msg) };
    unsafe { lua_error(state) }
}

/// 从 Lua 栈位置读取 Unix 时间戳（以秒计）
unsafe fn check_time_secs(state: *mut lua_State, arg: c_int) -> Result<i64, c_int> {
    let t = luaL_checkinteger(state, arg);
    Ok(t)
}

/// 从 Lua 时间表（year/month/day/hour/min/sec/isdst）构建 SystemTime
unsafe fn table_to_time(state: *mut lua_State) -> Result<i64, c_int> {
    use crate::api::{lua_getfield, lua_tointegerx};

    let get_int = |key: &str, default: Option<i64>, state: *mut lua_State| -> Result<i64, c_int> {
        let mut key_buf = key.to_string();
        key_buf.push('\0');
        let t = unsafe { lua_getfield(state, -1, key_buf.as_ptr().cast()) };
        let mut isnum = 0;
        let v = unsafe { lua_tointegerx(state, -1, &mut isnum) };
        unsafe { lua_pop(state, 1) };
        if isnum == 0 {
            if t != LUA_TNIL.into() {
                let msg = format!("field '{key}' is not an integer");
                return Err(unsafe { push_error(state, msg) });
            }
            match default {
                Some(d) => Ok(d),
                None => {
                    let msg = format!("field '{key}' missing in date table");
                    Err(unsafe { push_error(state, msg) })
                }
            }
        } else {
            Ok(v)
        }
    };

    let year = get_int("year", None, state)?;
    let month = get_int("month", None, state)?; // 1-based
    let day = get_int("day", None, state)?;
    let hour = get_int("hour", Some(12), state)?;
    let min = get_int("min", Some(0), state)?;
    let sec = get_int("sec", Some(0), state)?;

    // 转换为 Unix 时间戳（使用 Tomohiko Sakamoto 算法的 Zeller 变体）
    let y = year as i64;
    let m = month as i64; // 1-12
    let d = day as i64;

    // 使用 proleptic Gregorian calendar 计算 epoch days
    let epoch_days = days_from_civil(y, m, d);
    let total_secs = epoch_days * 86400 + hour * 3600 + min * 60 + sec;
    Ok(total_secs)
}

/// 计算从 1970-01-01 起的天数（proleptic Gregorian calendar）
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// 从 Unix 时间戳分解日期时间分量（UTC 或本地时间）
///
/// 返回 (year, month[1-12], day, hour, min, sec, weekday[0-6 Sun=0], yearday[1-366], isdst)
fn decompose_time_utc(secs: i64) -> (i32, i32, i32, i32, i32, i32, i32, i32, bool) {
    // 参考 Howard Hinnant 的 chrono-wiki 算法
    let days = secs.div_euclid(86400);
    let time_of_day = secs.rem_euclid(86400);

    let hour = (time_of_day / 3600) as i32;
    let min = ((time_of_day % 3600) / 60) as i32;
    let sec = (time_of_day % 60) as i32;

    // Gregorian decomposition from days since Unix epoch
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let y = y + if m <= 2 { 1 } else { 0 };

    // weekday: 0=Sun, Unix epoch (1970-01-01) was a Thursday (4)
    let wday = ((days + 4).rem_euclid(7)) as i32;

    // yearday (1-based)
    let yday = {
        let is_leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        let month_starts: [i32; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
        let leap_adjust = if is_leap && m > 2 { 1 } else { 0 };
        month_starts[m as usize - 1] + d as i32 + leap_adjust
    };

    (
        y as i32, m as i32, d as i32, hour, min, sec, wday, yday, false,
    )
}

/// 将日期时间分量推送为 Lua 表
unsafe fn push_time_table(
    state: *mut lua_State,
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    min: i32,
    sec: i32,
    wday: i32,
    yday: i32,
    isdst: bool,
) {
    unsafe { lua_createtable(state, 0, 9) };
    let push_int = |k: &str, v: i64, state: *mut lua_State| unsafe {
        lua_pushinteger(state, v);
        let mut kb = k.to_string();
        kb.push('\0');
        lua_setfield(state, -2, kb.as_ptr().cast());
    };
    push_int("year", year as i64, state);
    push_int("month", month as i64, state);
    push_int("day", day as i64, state);
    push_int("hour", hour as i64, state);
    push_int("min", min as i64, state);
    push_int("sec", sec as i64, state);
    push_int("wday", wday as i64 + 1, state); // Lua wday: 1=Sun
    push_int("yday", yday as i64, state);
    unsafe { lua_pushboolean(state, isdst as c_int) };
    unsafe { lua_setfield(state, -2, c"isdst".as_ptr()) };
}

// ─── strftime 简单实现（仅支持 %Y %m %d %H %M %S %A %B %p 等常用格式）────

fn simple_strftime(fmt: &[u8], secs: i64, utc: bool) -> Result<String, String> {
    let (year, month, day, hour, min, sec, wday, yday, _isdst) = decompose_time_utc(secs); // 我们只支持 UTC；本地时间用相同算法（忽略 DST）

    let _ = utc; // 当前实现：统一用 UTC 计算，不区分本地时间（跨平台安全）

    const ABBR_DAY: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const FULL_DAY: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    const ABBR_MON: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const FULL_MON: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let mut out = String::new();
    let mut i = 0;
    while i < fmt.len() {
        if fmt[i] != b'%' {
            out.push(fmt[i] as char);
            i += 1;
            continue;
        }
        i += 1;
        if i >= fmt.len() {
            break;
        }
        let spec = fmt[i];
        i += 1;
        match spec {
            b'Y' => out.push_str(&format!("{:04}", year)),
            b'y' => out.push_str(&format!("{:02}", year % 100)),
            b'm' => out.push_str(&format!("{:02}", month)),
            b'd' => out.push_str(&format!("{:02}", day)),
            b'H' => out.push_str(&format!("{:02}", hour)),
            b'M' => out.push_str(&format!("{:02}", min)),
            b'S' => out.push_str(&format!("{:02}", sec)),
            b'A' => out.push_str(FULL_DAY[wday as usize % 7]),
            b'a' => out.push_str(ABBR_DAY[wday as usize % 7]),
            b'B' => out.push_str(FULL_MON[(month - 1) as usize % 12]),
            b'b' | b'h' => out.push_str(ABBR_MON[(month - 1) as usize % 12]),
            b'e' => out.push_str(&format!("{:2}", day)),
            b'j' => out.push_str(&format!("{:03}", yday)),
            b'u' => out.push_str(&format!("{}", if wday == 0 { 7 } else { wday })),
            b'w' => out.push_str(&format!("{}", wday)),
            b'n' => out.push('\n'),
            b't' => out.push('\t'),
            b'%' => out.push('%'),
            b'p' => out.push_str(if hour < 12 { "AM" } else { "PM" }),
            b'P' => out.push_str(if hour < 12 { "am" } else { "pm" }),
            b'I' => out.push_str(&format!(
                "{:02}",
                if hour % 12 == 0 { 12 } else { hour % 12 }
            )),
            b'k' => out.push_str(&format!("{:2}", hour)),
            b'l' => out.push_str(&format!(
                "{:2}",
                if hour % 12 == 0 { 12 } else { hour % 12 }
            )),
            b'T' => out.push_str(&format!("{:02}:{:02}:{:02}", hour, min, sec)),
            b'D' => out.push_str(&format!("{:02}/{:02}/{:02}", month, day, year % 100)),
            b'F' => out.push_str(&format!("{:04}-{:02}-{:02}", year, month, day)),
            b'R' => out.push_str(&format!("{:02}:{:02}", hour, min)),
            b'Z' => out.push_str("UTC"),
            b'z' => out.push_str("+0000"),
            b'C' => out.push_str(&format!("{:02}", year / 100)),
            b'G' => out.push_str(&format!("{:04}", year)), // ISO week-year (simplified)
            b'V' => {
                // ISO week number (simplified)
                let week = (yday + 6) / 7;
                out.push_str(&format!("{:02}", week));
            }
            b'U' => {
                let week = (yday - 1 + (7 - wday)) / 7;
                out.push_str(&format!("{:02}", week));
            }
            b'W' => {
                let week = (yday - 1 + (7 - if wday == 0 { 6 } else { wday - 1 })) / 7;
                out.push_str(&format!("{:02}", week));
            }
            b'c' => {
                // locale date-time
                out.push_str(&format!(
                    "{} {} {:2} {:02}:{:02}:{:02} {:04}",
                    ABBR_DAY[wday as usize % 7],
                    ABBR_MON[(month - 1) as usize % 12],
                    day,
                    hour,
                    min,
                    sec,
                    year
                ));
            }
            b'x' => out.push_str(&format!("{:02}/{:02}/{:02}", month, day, year % 100)),
            b'X' => out.push_str(&format!("{:02}:{:02}:{:02}", hour, min, sec)),
            b'r' => out.push_str(&format!(
                "{:02}:{:02}:{:02} {}",
                if hour % 12 == 0 { 12 } else { hour % 12 },
                min,
                sec,
                if hour < 12 { "AM" } else { "PM" }
            )),
            other => return Err(format!("invalid conversion specifier '%{}'", other as char)),
        }
    }
    Ok(out)
}

// ─── Lua 函数实现 ───────────────────────────────────────────────────────────

unsafe fn os_clock(state: *mut lua_State) -> c_int {
    // Rust 没有进程 CPU 时间接口；用挂钟时间近似
    // 通过 process::cpu_time crate 可以更精确，但这里用 SystemTime
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = START.get_or_init(std::time::Instant::now);
    let elapsed = start.elapsed().as_secs_f64();
    unsafe { lua_pushnumber(state, elapsed) };
    1
}

unsafe fn os_date(state: *mut lua_State) -> c_int {
    let mut fmt_len = 0usize;
    let fmt_ptr = luaL_optlstring(state, 1, b"%c\0".as_ptr().cast(), &mut fmt_len);
    let fmt_bytes = unsafe { core::slice::from_raw_parts(fmt_ptr.cast::<u8>(), fmt_len) };

    // 读取时间参数（arg 2）
    let isnum = 0i32;
    let has_t2 = unsafe { crate::api::lua_type(state, 2) } > LUA_TNIL.into();
    let secs: i64 = if has_t2 {
        luaL_checkinteger(state, 2)
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs() as i64
    };
    let _ = isnum;

    // 判断是否强制 UTC
    let (utc, fmt_bytes) = if fmt_bytes.first() == Some(&b'!') {
        (true, &fmt_bytes[1..])
    } else {
        (false, fmt_bytes)
    };

    // 返回表格式
    if fmt_bytes == b"*t" {
        let (year, month, day, hour, min, sec, wday, yday, isdst) = decompose_time_utc(secs);
        unsafe { push_time_table(state, year, month, day, hour, min, sec, wday, yday, isdst) };
        return 1;
    }

    match simple_strftime(fmt_bytes, secs, utc) {
        Ok(s) => {
            unsafe { push_str(state, &s) };
            1
        }
        Err(e) => {
            // 通过 push_error 推送动态错误消息（包含 "invalid conversion specifier"）
            unsafe { push_error(state, e) }
        }
    }
}

unsafe fn os_difftime(state: *mut lua_State) -> c_int {
    let t2 = luaL_checkinteger(state, 1) as lua_Number;
    let t1 = luaL_checkinteger(state, 2) as lua_Number;
    unsafe { lua_pushnumber(state, t2 - t1) };
    1
}

unsafe fn os_execute(state: *mut lua_State) -> c_int {
    use crate::aux_rs::luaL_optlstring;
    let mut len = 0usize;
    let cmd_ptr = luaL_optlstring(state, 1, core::ptr::null(), &mut len);

    if cmd_ptr.is_null() {
        // os.execute() with no argument → true if shell available
        unsafe { lua_pushboolean(state, 1) };
        return 1;
    }

    let cmd_bytes = unsafe { core::slice::from_raw_parts(cmd_ptr.cast::<u8>(), len) };
    let cmd = String::from_utf8_lossy(cmd_bytes).into_owned();

    #[cfg(unix)]
    let result = std::process::Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .status();
    #[cfg(windows)]
    let result = std::process::Command::new("cmd")
        .arg("/C")
        .arg(&cmd)
        .status();
    #[cfg(not(any(unix, windows)))]
    let result: Result<std::process::ExitStatus, _> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "no shell available",
    ));

    match result {
        Ok(status) => {
            if status.success() {
                // 成功：true, "exit", 0
                unsafe { lua_pushboolean(state, 1) };
                unsafe { push_str(state, "exit") };
                unsafe { lua_pushinteger(state, 0) };
            } else {
                let code = status.code().unwrap_or(-1);
                // 失败：nil, "exit", code（Lua 5.5 规范要求返回 nil 而非 false）
                unsafe { crate::api::lua_pushnil(state) };
                unsafe { push_str(state, "exit") };
                unsafe { lua_pushinteger(state, code as lua_Integer) };
            }
            3
        }
        Err(e) => {
            // 系统错误：nil, 错误消息, -1
            unsafe { crate::api::lua_pushnil(state) };
            unsafe { push_str(state, &e.to_string()) };
            unsafe { lua_pushinteger(state, -1) };
            3
        }
    }
}

unsafe fn os_exit(state: *mut lua_State) -> c_int {
    use crate::api::{lua_toboolean, lua_type};
    let ty = unsafe { lua_type(state, 1) };
    let code = if ty == LUA_TBOOLEAN.into() {
        if unsafe { lua_toboolean(state, 1) } != 0 {
            0
        } else {
            1
        }
    } else {
        luaL_optinteger(state, 1, 0) as i32
    };

    if unsafe { lua_toboolean(state, 2) } != 0 {
        unsafe { crate::state::lua_close(state) };
    }
    std::process::exit(code);
}

unsafe fn os_getenv(state: *mut lua_State) -> c_int {
    use crate::aux_rs::luaL_checklstring;
    let mut len = 0usize;
    let key_ptr = luaL_checklstring(state, 1, &mut len);
    let key_bytes = unsafe { core::slice::from_raw_parts(key_ptr.cast::<u8>(), len) };
    let key = String::from_utf8_lossy(key_bytes);

    match std::env::var(key.as_ref()) {
        Ok(val) => {
            unsafe { push_str(state, &val) };
        }
        Err(_) => {
            unsafe { crate::api::lua_pushnil(state) };
        }
    }
    1
}

unsafe fn os_remove(state: *mut lua_State) -> c_int {
    use crate::aux_rs::luaL_checklstring;
    let mut len = 0usize;
    let path_ptr = luaL_checklstring(state, 1, &mut len);
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr.cast::<u8>(), len) };
    let path = std::path::Path::new(std::str::from_utf8(path_bytes).unwrap_or(""));

    let result = if path.is_dir() {
        std::fs::remove_dir(path)
    } else {
        std::fs::remove_file(path)
    };

    match result {
        Ok(()) => {
            unsafe { lua_pushboolean(state, 1) };
            1
        }
        Err(e) => {
            unsafe { crate::lua_module::push_fail(state) };
            unsafe { push_str(state, &e.to_string()) };
            unsafe { lua_pushinteger(state, e.raw_os_error().unwrap_or(-1) as lua_Integer) };
            3
        }
    }
}

unsafe fn os_rename(state: *mut lua_State) -> c_int {
    use crate::aux_rs::luaL_checklstring;
    let mut len = 0usize;
    let from_ptr = luaL_checklstring(state, 1, &mut len);
    let from =
        String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(from_ptr.cast::<u8>(), len) })
            .into_owned();

    len = 0;
    let to_ptr = luaL_checklstring(state, 2, &mut len);
    let to =
        String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(to_ptr.cast::<u8>(), len) })
            .into_owned();

    match std::fs::rename(&from, &to) {
        Ok(()) => {
            unsafe { lua_pushboolean(state, 1) };
            1
        }
        Err(e) => {
            unsafe { crate::lua_module::push_fail(state) };
            unsafe { push_str(state, &e.to_string()) };
            unsafe { lua_pushinteger(state, e.raw_os_error().unwrap_or(-1) as lua_Integer) };
            3
        }
    }
}

unsafe fn os_time(state: *mut lua_State) -> c_int {
    use crate::api::lua_type;

    let secs = if unsafe { lua_type(state, 1) } == LUA_TNONE
        || unsafe { lua_type(state, 1) } == LUA_TNIL.into()
    {
        // os.time() → 当前 Unix 时间戳
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs() as i64
    } else {
        luaL_checktype(state, 1, LUA_TTABLE.into());
        unsafe { lua_settop(state, 1) };
        match unsafe { table_to_time(state) } {
            Ok(t) => t,
            Err(code) => return code,
        }
    };

    unsafe { lua_pushinteger(state, secs) };
    1
}

unsafe fn os_tmpname(state: *mut lua_State) -> c_int {
    use std::time::SystemTime;
    let tmp = std::env::temp_dir();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .subsec_nanos();
    let name = format!("lua_{:08x}", nonce);
    let path = tmp.join(&name);
    let path_str = path.to_string_lossy().into_owned();
    unsafe { push_str(state, &path_str) };
    1
}

// ─── 模块入口 ────────────────────────────────────────────────────────────────

pub(crate) unsafe fn luaopen_os(state: *mut lua_State) -> c_int {
    unsafe { register_lib(state, SYSLIB) };
    1
}

// ─── LuaModule 实现 ─────────────────────────────────────────────────────────

pub struct OsModule;

impl crate::module::LuaModule for OsModule {
    const NAME: &'static str = "os";

    unsafe fn open(state: *mut lua_State) -> c_int {
        unsafe { luaopen_os(state) }
    }
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
