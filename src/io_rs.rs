use crate::api::*;
use crate::aux_rs::{
    luaL_checkany, luaL_checkinteger, luaL_checklstring, luaL_checkoption, luaL_checkstack,
    luaL_checkudata, luaL_execresult, luaL_fileresult, luaL_newmetatable, luaL_optinteger,
    luaL_optlstring, luaL_setfuncs, luaL_setmetatable, luaL_testudata,
};
use crate::lua_module::{create_library, lua_pop, lua_upvalueindex, luaL_Reg, luaL_error, luaL_error_str, push_fail};
use crate::luaffi::LuaCFunction;
use crate::runtime::*;
use core::ffi::{c_char, c_int, c_long, c_void};
use core::{mem, ptr};
use std::ffi::CStr;

#[repr(C)]
struct File {
    _private: [u8; 0],
}

#[repr(C)]
struct luaL_Stream {
    f: *mut File,
    closef: LuaCFunction,
}

unsafe impl Sync for luaL_Stream {}

#[repr(C)]
struct RN {
    f: *mut File,
    c: c_int,
    n: c_int,
    buff: [u8; L_MAXLENNUM + 1],
}

static IOLIB: [luaL_Reg; 12] = [
    luaL_Reg {
        name: NAME_CLOSE.as_ptr().cast(),
        func: Some(io_close),
    },
    luaL_Reg {
        name: NAME_FLUSH.as_ptr().cast(),
        func: Some(io_flush),
    },
    luaL_Reg {
        name: NAME_INPUT.as_ptr().cast(),
        func: Some(io_input),
    },
    luaL_Reg {
        name: NAME_LINES.as_ptr().cast(),
        func: Some(io_lines),
    },
    luaL_Reg {
        name: NAME_OPEN.as_ptr().cast(),
        func: Some(io_open),
    },
    luaL_Reg {
        name: NAME_OUTPUT.as_ptr().cast(),
        func: Some(io_output),
    },
    luaL_Reg {
        name: NAME_POPEN.as_ptr().cast(),
        func: Some(io_popen),
    },
    luaL_Reg {
        name: NAME_READ.as_ptr().cast(),
        func: Some(io_read),
    },
    luaL_Reg {
        name: NAME_TMPFILE.as_ptr().cast(),
        func: Some(io_tmpfile),
    },
    luaL_Reg {
        name: NAME_TYPE.as_ptr().cast(),
        func: Some(io_type),
    },
    luaL_Reg {
        name: NAME_WRITE.as_ptr().cast(),
        func: Some(io_write),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

static METH: [luaL_Reg; 8] = [
    luaL_Reg {
        name: NAME_READ.as_ptr().cast(),
        func: Some(f_read),
    },
    luaL_Reg {
        name: NAME_WRITE.as_ptr().cast(),
        func: Some(f_write),
    },
    luaL_Reg {
        name: NAME_LINES.as_ptr().cast(),
        func: Some(f_lines),
    },
    luaL_Reg {
        name: NAME_FLUSH.as_ptr().cast(),
        func: Some(f_flush),
    },
    luaL_Reg {
        name: NAME_SEEK.as_ptr().cast(),
        func: Some(f_seek),
    },
    luaL_Reg {
        name: NAME_CLOSE.as_ptr().cast(),
        func: Some(f_close),
    },
    luaL_Reg {
        name: NAME_SETVBUF.as_ptr().cast(),
        func: Some(f_setvbuf),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

static METAMETH: [luaL_Reg; 5] = [
    luaL_Reg {
        name: META_INDEX.as_ptr().cast(),
        func: None,
    },
    luaL_Reg {
        name: META_GC.as_ptr().cast(),
        func: Some(f_gc),
    },
    luaL_Reg {
        name: META_CLOSE.as_ptr().cast(),
        func: Some(f_gc),
    },
    luaL_Reg {
        name: META_TOSTRING.as_ptr().cast(),
        func: Some(f_tostring),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

use core::ffi::c_uint;

// localeconv：使用 luaffi 的 Rust 实现
#[inline] fn localeconv() -> *mut LConv { crate::luaffi::localeconv() }

// C 文件 I/O（底层 C FILE*，无 Rust std 等价，保留 extern C）
unsafe extern "C" {
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut File;
    fn fdopen(fd: c_int, mode: *const c_char) -> *mut File;
    fn fclose(file: *mut File) -> c_int;
    fn tmpfile() -> *mut File;
    fn fflush(file: *mut File) -> c_int;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut File) -> usize;
    fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut File) -> usize;
    fn clearerr(stream: *mut File);
    fn ferror(stream: *mut File) -> c_int;
    fn getc(stream: *mut File) -> c_int;
    fn ungetc(c: c_int, stream: *mut File) -> c_int;
    fn popen(command: *const c_char, mode: *const c_char) -> *mut File;
    fn pclose(file: *mut File) -> c_int;
    fn setvbuf(stream: *mut File, buffer: *mut c_char, mode: c_int, size: usize) -> c_int;
    fn fseeko(stream: *mut File, offset: c_long, whence: c_int) -> c_int;
    fn ftello(stream: *mut File) -> c_long;
    fn flockfile(stream: *mut File);
    fn funlockfile(stream: *mut File);
    fn getc_unlocked(stream: *mut File) -> c_int;
}

/// 获取当前 errno 值（用 Rust std 方式）
#[inline]
fn get_errno() -> c_int {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

/// 重置 errno（通过 raw_os_error 无法主动清零，用空实现，调用处改为检查返回值）
#[inline]
fn reset_errno() {
    // io_rs 的调用处使用 reset_errno 后立即调用 C 函数并检查返回值，
    // strerror 调用改为用 std::io::Error::last_os_error()
    // 无法在纯 Rust 中将 errno 设置为 0，但可以在 strerror 处获取最新 errno
}

#[inline]
unsafe fn push_literal(state: *mut lua_State, s: &'static [u8]) {
    unsafe { lua_pushstring(state, s.as_ptr().cast()) };
}

#[inline]
unsafe fn is_none(state: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(state, idx) == LUA_TNONE }
}

#[inline]
unsafe fn is_nil(state: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(state, idx) == LUA_TNIL.into() }
}

#[inline]
unsafe fn is_none_or_nil(state: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(state, idx) <= LUA_TNIL.into() }
}

#[inline]
unsafe fn checkstring(state: *mut lua_State, arg: c_int) -> *const c_char {
    unsafe { luaL_checklstring(state, arg, ptr::null_mut()) }
}

#[inline]
unsafe fn optstring(state: *mut lua_State, arg: c_int, default: &'static [u8]) -> *const c_char {
    unsafe { luaL_optlstring(state, arg, default.as_ptr().cast(), ptr::null_mut()) }
}

#[inline]
unsafe fn tolstream(state: *mut lua_State) -> *mut luaL_Stream {
    unsafe { luaL_checkudata(state, 1, LUA_FILEHANDLE.as_ptr().cast()) as *mut luaL_Stream }
}

#[inline]
unsafe fn isclosed(stream: *mut luaL_Stream) -> bool {
    unsafe { (*stream).closef.is_none() }
}

#[inline]
unsafe fn get_localedecpoint() -> u8 {
    let conv = unsafe { localeconv() };
    if conv.is_null() || unsafe { (*conv).decimal_point.is_null() } {
        b'.'
    } else {
        unsafe { *(*conv).decimal_point.cast::<u8>() }
    }
}

unsafe fn push_dynamic_error(state: *mut lua_State, message: &str) -> c_int {
    let bytes = message.as_bytes();
    unsafe {
        crate::lua_module::lua_pushlstring(state, bytes.as_ptr().cast(), bytes.len());
        crate::lua_module::lua_error(state)
    }
}

#[inline]
unsafe fn lua_replace_local(state: *mut lua_State, index: c_int) {
    unsafe { lua_copy(state, -1, index) };
    unsafe { lua_pop(state, 1) };
}

unsafe  fn io_type(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    let p = unsafe { luaL_testudata(state, 1, LUA_FILEHANDLE.as_ptr().cast()) as *mut luaL_Stream };
    if p.is_null() {
        unsafe { push_fail(state) };
    } else if unsafe { isclosed(p) } {
        unsafe { push_literal(state, STR_CLOSED_FILE) };
    } else {
        unsafe { push_literal(state, STR_FILE) };
    }
    1
}

unsafe  fn f_tostring(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    if unsafe { isclosed(p) } {
        unsafe { push_literal(state, STR_FILE_CLOSED) };
    } else {
        unsafe { lua_pushfstring(state, c"file (%p)".as_ptr(), (*p).f) };
    }
    1
}

unsafe fn tofile(state: *mut lua_State) -> *mut File {
    let p = unsafe { tolstream(state) };
    if unsafe { isclosed(p) } {
        unsafe { luaL_error_str(state, ERR_ATTEMPT_CLOSED.as_ptr().cast()) };
    }
    unsafe { (*p).f }
}

unsafe fn newprefile(state: *mut lua_State) -> *mut luaL_Stream {
    let p =
        unsafe { lua_newuserdatauv(state, mem::size_of::<luaL_Stream>(), 0) as *mut luaL_Stream };
    unsafe {
        (*p).f = ptr::null_mut();
        (*p).closef = None;
        luaL_setmetatable(state, LUA_FILEHANDLE.as_ptr().cast());
    }
    p
}

unsafe fn newfile(state: *mut lua_State) -> *mut luaL_Stream {
    let p = unsafe { newprefile(state) };
    unsafe { (*p).closef = Some(io_fclose) };
    p
}

unsafe  fn aux_close(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    let cf = unsafe { (*p).closef };
    unsafe { (*p).closef = None };
    unsafe { cf.expect("open file has close callback")(state) }
}

unsafe  fn f_close(state: *mut lua_State) -> c_int {
    unsafe { tofile(state) };
    unsafe { aux_close(state) }
}

unsafe  fn io_close(state: *mut lua_State) -> c_int {
    if unsafe { is_none(state, 1) } {
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, IO_OUTPUT.as_ptr().cast()) };
    }
    unsafe { f_close(state) }
}

unsafe  fn f_gc(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    if !unsafe { isclosed(p) } && !unsafe { (*p).f.is_null() } {
        let _ = unsafe { aux_close(state) };
    }
    0
}

unsafe  fn io_fclose(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    unsafe { reset_errno() };
    unsafe { luaL_fileresult(state, (fclose((*p).f) == 0) as c_int, ptr::null()) }
}

fn check_mode(mode: &[u8]) -> bool {
    if mode.is_empty() {
        return false;
    }
    if !matches!(mode[0], b'r' | b'w' | b'a') {
        return false;
    }
    let mut i = 1usize;
    if i < mode.len() && mode[i] == b'+' {
        i += 1;
    }
    mode[i..].iter().all(|c| *c == b'b')
}

fn check_modep(mode: &[u8]) -> bool {
    matches!(mode, b"r" | b"w")
}

unsafe fn opencheck(state: *mut lua_State, fname: *const c_char, mode: *const c_char) {
    let p = unsafe { newfile(state) };
    unsafe { (*p).f = fopen(fname, mode) };
    if unsafe { (*p).f.is_null() } {
        unsafe {
            luaL_error(
                state,
                c"cannot open file '%s' (%s)".as_ptr(),
                fname,
                strerror(),
            )
        };
    }
}

/// 获取当前 errno 对应的错误描述字符串（静态缓冲区，线程不安全但 Lua 是单线程的）
fn strerror() -> *const c_char {
    // 获取最后一次 OS 错误的描述字符串
    let err = std::io::Error::last_os_error();
    let msg = err.to_string();
    // 需要返回 *const c_char，使用 thread_local 静态缓冲区
    use std::cell::UnsafeCell;
    struct ErrBuf(UnsafeCell<Vec<u8>>);
    unsafe impl Sync for ErrBuf {}
    static ERR_BUF: ErrBuf = ErrBuf(UnsafeCell::new(Vec::new()));
    let buf = unsafe { &mut *ERR_BUF.0.get() };
    buf.clear();
    buf.extend_from_slice(msg.as_bytes());
    buf.push(0);
    buf.as_ptr().cast()
}

unsafe  fn io_open(state: *mut lua_State) -> c_int {
    let filename = unsafe { checkstring(state, 1) };
    let mode = unsafe { optstring(state, 2, c"r".to_bytes_with_nul()) };
    let mode_bytes = unsafe { CStr::from_ptr(mode) }.to_bytes();
    if !check_mode(mode_bytes) {
        let _ =
            unsafe { crate::lua_module::luaL_argerror(state, 2, ERR_INVALID_MODE.as_ptr().cast()) };
    }
    let p = unsafe { newfile(state) };
    unsafe { reset_errno() };
    unsafe { (*p).f = fopen(filename, mode) };
    if unsafe { (*p).f.is_null() } {
        unsafe { luaL_fileresult(state, 0, filename) }
    } else {
        1
    }
}

unsafe  fn io_pclose(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    unsafe { reset_errno() };
    unsafe { luaL_execresult(state, pclose((*p).f)) }
}

unsafe  fn io_popen(state: *mut lua_State) -> c_int {
    let filename = unsafe { checkstring(state, 1) };
    let mode = unsafe { optstring(state, 2, c"r".to_bytes_with_nul()) };
    let mode_bytes = unsafe { CStr::from_ptr(mode) }.to_bytes();
    if !check_modep(mode_bytes) {
        let _ =
            unsafe { crate::lua_module::luaL_argerror(state, 2, ERR_INVALID_MODE.as_ptr().cast()) };
    }
    let p = unsafe { newprefile(state) };
    unsafe { reset_errno() };
    unsafe { (*p).f = popen(filename, mode) };
    unsafe { (*p).closef = Some(io_pclose) };
    if unsafe { (*p).f.is_null() } {
        unsafe { luaL_fileresult(state, 0, filename) }
    } else {
        1
    }
}

unsafe  fn io_tmpfile(state: *mut lua_State) -> c_int {
    let p = unsafe { newfile(state) };
    unsafe { reset_errno() };
    unsafe { (*p).f = tmpfile() };
    if unsafe { (*p).f.is_null() } {
        unsafe { luaL_fileresult(state, 0, ptr::null()) }
    } else {
        1
    }
}

unsafe fn getiofile(state: *mut lua_State, findex: &'static [u8]) -> *mut File {
    unsafe { lua_getfield(state, LUA_REGISTRYINDEX, findex.as_ptr().cast()) };
    let p = unsafe { lua_touserdata(state, -1) as *mut luaL_Stream };
    if unsafe { isclosed(p) } {
        let name = std::str::from_utf8(&findex[IOPREF_LEN..findex.len() - 1]).unwrap();
        let msg = format!("default {} file is closed", name);
        unsafe { push_dynamic_error(state, &msg) };
    }
    unsafe { (*p).f }
}

unsafe fn g_iofile(state: *mut lua_State, f: &'static [u8], mode: &'static [u8]) -> c_int {
    if !unsafe { is_none_or_nil(state, 1) } {
        let filename = unsafe { lua_tolstring(state, 1, ptr::null_mut()) };
        if !filename.is_null() {
            unsafe { opencheck(state, filename, mode.as_ptr().cast()) };
        } else {
            unsafe { tofile(state) };
            unsafe { lua_pushvalue(state, 1) };
        }
        unsafe { lua_setfield(state, LUA_REGISTRYINDEX, f.as_ptr().cast()) };
    }
    unsafe { lua_getfield(state, LUA_REGISTRYINDEX, f.as_ptr().cast()) };
    1
}

unsafe  fn io_input(state: *mut lua_State) -> c_int {
    unsafe { g_iofile(state, IO_INPUT, c"r".to_bytes_with_nul()) }
}

unsafe  fn io_output(state: *mut lua_State) -> c_int {
    unsafe { g_iofile(state, IO_OUTPUT, c"w".to_bytes_with_nul()) }
}

unsafe fn aux_lines(state: *mut lua_State, toclose: c_int) {
    let n = unsafe { lua_gettop(state) } - 1;
    if n > MAXARGLINE {
        let _ = unsafe {
            crate::lua_module::luaL_argerror(
                state,
                MAXARGLINE + 2,
                ERR_TOO_MANY_ARGUMENTS.as_ptr().cast(),
            )
        };
    }
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_pushinteger(state, n as lua_Integer) };
    unsafe { lua_pushboolean(state, toclose) };
    unsafe { lua_rotate(state, 2, 3) };
    unsafe { lua_pushcclosure(state, Some(io_readline), 3 + n) };
}

unsafe  fn f_lines(state: *mut lua_State) -> c_int {
    unsafe { tofile(state) };
    unsafe { aux_lines(state, 0) };
    1
}

unsafe  fn io_lines(state: *mut lua_State) -> c_int {
    let toclose;
    if unsafe { is_none(state, 1) } {
        unsafe { lua_pushnil(state) };
    }
    if unsafe { is_nil(state, 1) } {
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, IO_INPUT.as_ptr().cast()) };
        unsafe { lua_replace_local(state, 1) };
        unsafe { tofile(state) };
        toclose = 0;
    } else {
        let filename = unsafe { checkstring(state, 1) };
        unsafe { opencheck(state, filename, c"r".as_ptr()) };
        unsafe { lua_replace_local(state, 1) };
        toclose = 1;
    }
    unsafe { aux_lines(state, toclose) };
    if toclose != 0 {
        unsafe { lua_pushnil(state) };
        unsafe { lua_pushnil(state) };
        unsafe { lua_pushvalue(state, 1) };
        4
    } else {
        1
    }
}

unsafe fn nextc(rn: &mut RN) -> c_int {
    if rn.n as usize >= L_MAXLENNUM {
        rn.buff[0] = 0;
        0
    } else {
        rn.buff[rn.n as usize] = rn.c as u8;
        rn.n += 1;
        rn.c = unsafe { getc_unlocked(rn.f) };
        1
    }
}

unsafe fn test2(rn: &mut RN, set: &[u8; 2]) -> c_int {
    if rn.c == set[0] as c_int || rn.c == set[1] as c_int {
        unsafe { nextc(rn) }
    } else {
        0
    }
}

unsafe fn readdigits(rn: &mut RN, hex: bool) -> c_int {
    let mut count = 0;
    while rn.c != EOF_VALUE
        && (((rn.c as u8 as char).is_ascii_hexdigit() && hex)
            || ((rn.c as u8 as char).is_ascii_digit() && !hex))
        && unsafe { nextc(rn) } != 0
    {
        count += 1;
    }
    count
}

unsafe fn read_number(state: *mut lua_State, f: *mut File) -> c_int {
    let mut rn = RN {
        f,
        c: 0,
        n: 0,
        buff: [0; L_MAXLENNUM + 1],
    };
    let mut count = 0;
    let mut hex = false;
    let decp = [unsafe { get_localedecpoint() }, b'.'];
    unsafe { flockfile(rn.f) };
    loop {
        rn.c = unsafe { getc_unlocked(rn.f) };
        if rn.c == EOF_VALUE || !(rn.c as u8 as char).is_ascii_whitespace() {
            break;
        }
    }
    let _ = unsafe { test2(&mut rn, b"-+") };
    if unsafe { test2(&mut rn, b"00") } != 0 {
        if unsafe { test2(&mut rn, b"xX") } != 0 {
            hex = true;
        } else {
            count = 1;
        }
    }
    count += unsafe { readdigits(&mut rn, hex) };
    if rn.c == decp[0] as c_int || rn.c == decp[1] as c_int {
        let _ = unsafe { nextc(&mut rn) };
        count += unsafe { readdigits(&mut rn, hex) };
    }
    if count > 0
        && ((hex && matches!(rn.c as u8, b'p' | b'P'))
            || (!hex && matches!(rn.c as u8, b'e' | b'E')))
    {
        let _ = unsafe { nextc(&mut rn) };
        let _ = unsafe { test2(&mut rn, b"-+") };
        let _ = unsafe { readdigits(&mut rn, false) };
    }
    unsafe { ungetc(rn.c, rn.f) };
    unsafe { funlockfile(rn.f) };
    rn.buff[rn.n as usize] = 0;
    if unsafe { lua_stringtonumber(state, rn.buff.as_ptr().cast()) } != 0 {
        1
    } else {
        unsafe { lua_pushnil(state) };
        0
    }
}

unsafe fn test_eof(state: *mut lua_State, f: *mut File) -> c_int {
    let c = unsafe { getc(f) };
    let _ = unsafe { ungetc(c, f) };
    unsafe { push_literal(state, EMPTY_STRING) };
    (c != EOF_VALUE) as c_int
}

unsafe fn read_line(state: *mut lua_State, f: *mut File, chop: bool) -> c_int {
    let mut out = Vec::new();
    let mut c = EOF_VALUE;
    loop {
        let mut chunk = [0u8; LUAL_BUFFERSIZE];
        let mut i = 0usize;
        unsafe { flockfile(f) };
        loop {
            if i >= LUAL_BUFFERSIZE {
                break;
            }
            c = unsafe { getc_unlocked(f) };
            if c == EOF_VALUE || c == b'\n' as c_int {
                break;
            }
            chunk[i] = c as u8;
            i += 1;
        }
        unsafe { funlockfile(f) };
        out.extend_from_slice(&chunk[..i]);
        if c == EOF_VALUE || c == b'\n' as c_int {
            break;
        }
    }
    if !chop && c == b'\n' as c_int {
        out.push(b'\n');
    }
    unsafe { crate::lua_module::lua_pushlstring(state, out.as_ptr().cast(), out.len()) };
    (c == b'\n' as c_int || !out.is_empty()) as c_int
}

unsafe fn read_all(state: *mut lua_State, f: *mut File) {
    let mut out = Vec::new();
    loop {
        let mut chunk = [0u8; LUAL_BUFFERSIZE];
        let nr = unsafe { fread(chunk.as_mut_ptr().cast(), 1, LUAL_BUFFERSIZE, f) };
        out.extend_from_slice(&chunk[..nr]);
        if nr != LUAL_BUFFERSIZE {
            break;
        }
    }
    unsafe { crate::lua_module::lua_pushlstring(state, out.as_ptr().cast(), out.len()) };
}

unsafe fn read_chars(state: *mut lua_State, f: *mut File, n: usize) -> c_int {
    let mut out = vec![0u8; n];
    let nr = unsafe { fread(out.as_mut_ptr().cast(), 1, n, f) };
    unsafe { crate::lua_module::lua_pushlstring(state, out.as_ptr().cast(), nr) };
    (nr > 0) as c_int
}

unsafe fn g_read(state: *mut lua_State, f: *mut File, first: c_int) -> c_int {
    let nargs = unsafe { lua_gettop(state) } - 1;
    let mut n;
    let mut success;
    unsafe { clearerr(f) };
    unsafe { reset_errno() };
    if nargs == 0 {
        success = unsafe { read_line(state, f, true) };
        n = first + 1;
    } else {
        unsafe { luaL_checkstack(state, nargs + 20, ERR_TOO_MANY_READ_ARGS.as_ptr().cast()) };
        success = 1;
        n = first;
        let mut left = nargs;
        while left > 0 && success != 0 {
            if unsafe { lua_type(state, n) } == LUA_TNUMBER as c_int {
                let l = unsafe { luaL_checkinteger(state, n) as usize };
                success = if l == 0 {
                    unsafe { test_eof(state, f) }
                } else {
                    unsafe { read_chars(state, f, l) }
                };
            } else {
                let p = unsafe { checkstring(state, n) };
                let bytes = unsafe { CStr::from_ptr(p) }.to_bytes();
                let fmt = if bytes.first() == Some(&b'*') {
                    &bytes[1..]
                } else {
                    bytes
                };
                success = match fmt.first().copied() {
                    Some(b'n') => unsafe { read_number(state, f) },
                    Some(b'l') => unsafe { read_line(state, f, true) },
                    Some(b'L') => unsafe { read_line(state, f, false) },
                    Some(b'a') => {
                        unsafe { read_all(state, f) };
                        1
                    }
                    _ => {
                        return unsafe {
                            crate::lua_module::luaL_argerror(
                                state,
                                n,
                                ERR_INVALID_FORMAT.as_ptr().cast(),
                            )
                        };
                    }
                };
            }
            n += 1;
            left -= 1;
        }
    }
    if unsafe { ferror(f) } != 0 {
        return unsafe { luaL_fileresult(state, 0, ptr::null()) };
    }
    if success == 0 {
        unsafe { lua_pop(state, 1) };
        unsafe { push_fail(state) };
    }
    n - first
}

unsafe  fn io_read(state: *mut lua_State) -> c_int {
    unsafe { g_read(state, getiofile(state, IO_INPUT), 1) }
}

unsafe  fn f_read(state: *mut lua_State) -> c_int {
    unsafe { g_read(state, tofile(state), 2) }
}

unsafe  fn io_readline(state: *mut lua_State) -> c_int {
    let p = unsafe { lua_touserdata(state, lua_upvalueindex(1)) as *mut luaL_Stream };
    let mut isnum = 0;
    let mut n = unsafe { lua_tointegerx(state, lua_upvalueindex(2), &mut isnum) as c_int };
    if unsafe { isclosed(p) } {
        return unsafe { luaL_error_str(state, ERR_FILE_ALREADY_CLOSED.as_ptr().cast()) };
    }
    unsafe { lua_settop(state, 1) };
    unsafe { luaL_checkstack(state, n, ERR_TOO_MANY_READ_ARGS.as_ptr().cast()) };
    for i in 1..=n {
        unsafe { lua_pushvalue(state, lua_upvalueindex(3 + i)) };
    }
    n = unsafe { g_read(state, (*p).f, 2) };
    if unsafe { lua_toboolean(state, -n) } != 0 {
        n
    } else {
        if n > 1 {
            return unsafe {
                luaL_error(
                    state,
                    c"%s".as_ptr(),
                    lua_tolstring(state, -n + 1, ptr::null_mut()),
                )
            };
        }
        if unsafe { lua_toboolean(state, lua_upvalueindex(3)) } != 0 {
            unsafe { lua_settop(state, 0) };
            unsafe { lua_pushvalue(state, lua_upvalueindex(1)) };
            let _ = unsafe { aux_close(state) };
        }
        0
    }
}

unsafe fn g_write(state: *mut lua_State, f: *mut File, mut arg: c_int) -> c_int {
    let mut nargs = unsafe { lua_gettop(state) } - arg;
    let mut totalbytes = 0usize;
    unsafe { reset_errno() };
    loop {
        let current = nargs;
        nargs -= 1;
        if current == 0 {
            break;
        }
        let mut buff = [0 as c_char; LUA_N2SBUFFSZ];
        let len = unsafe { lua_numbertocstring(state, arg, buff.as_mut_ptr()) };
        let (s, slen) = if len > 0 {
            (buff.as_ptr().cast::<u8>(), len as usize - 1)
        } else {
            let mut sl = 0usize;
            let sp = unsafe { luaL_checklstring(state, arg, &mut sl) };
            (sp.cast::<u8>(), sl)
        };
        let numbytes = unsafe { fwrite(s.cast(), 1, slen, f) };
        totalbytes += numbytes;
        if numbytes < slen {
            let n = unsafe { luaL_fileresult(state, 0, ptr::null()) };
            unsafe { lua_pushinteger(state, totalbytes as lua_Integer) };
            return n + 1;
        }
        arg += 1;
    }
    1
}

unsafe  fn io_write(state: *mut lua_State) -> c_int {
    unsafe { g_write(state, getiofile(state, IO_OUTPUT), 1) }
}

unsafe  fn f_write(state: *mut lua_State) -> c_int {
    let f = unsafe { tofile(state) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { g_write(state, f, 2) }
}

unsafe  fn f_seek(state: *mut lua_State) -> c_int {
    let mode_names = [
        c"set".as_ptr(),
        c"cur".as_ptr(),
        c"end".as_ptr(),
        ptr::null(),
    ];
    let modes = [SEEK_SET_VALUE, SEEK_CUR_VALUE, SEEK_END_VALUE];
    let f = unsafe { tofile(state) };
    let op = unsafe { luaL_checkoption(state, 2, c"cur".as_ptr(), mode_names.as_ptr()) };
    let p3 = unsafe { luaL_optinteger(state, 3, 0) };
    let offset = p3 as c_long;
    if offset as lua_Integer != p3 {
        let _ =
            unsafe { crate::lua_module::luaL_argerror(state, 3, ERR_BAD_SEEK_INT.as_ptr().cast()) };
    }
    unsafe { reset_errno() };
    if unsafe { fseeko(f, offset, modes[op as usize]) } != 0 {
        unsafe { luaL_fileresult(state, 0, ptr::null()) }
    } else {
        unsafe { lua_pushinteger(state, ftello(f) as lua_Integer) };
        1
    }
}

unsafe  fn f_setvbuf(state: *mut lua_State) -> c_int {
    let mode_names = [
        c"no".as_ptr(),
        c"full".as_ptr(),
        c"line".as_ptr(),
        ptr::null(),
    ];
    let modes = [IONBF_VALUE, IOFBF_VALUE, IOLBF_VALUE];
    let f = unsafe { tofile(state) };
    let op = unsafe { luaL_checkoption(state, 2, ptr::null(), mode_names.as_ptr()) };
    let sz = unsafe { luaL_optinteger(state, 3, LUAL_BUFFERSIZE as lua_Integer) } as usize;
    unsafe { reset_errno() };
    unsafe {
        luaL_fileresult(
            state,
            (setvbuf(f, ptr::null_mut(), modes[op as usize], sz) == 0) as c_int,
            ptr::null(),
        )
    }
}

unsafe fn aux_flush(state: *mut lua_State, f: *mut File) -> c_int {
    unsafe { reset_errno() };
    unsafe { luaL_fileresult(state, (fflush(f) == 0) as c_int, ptr::null()) }
}

unsafe  fn f_flush(state: *mut lua_State) -> c_int {
    unsafe { aux_flush(state, tofile(state)) }
}

unsafe  fn io_flush(state: *mut lua_State) -> c_int {
    unsafe { aux_flush(state, getiofile(state, IO_OUTPUT)) }
}

unsafe fn createmeta(state: *mut lua_State) {
    unsafe { luaL_newmetatable(state, LUA_FILEHANDLE.as_ptr().cast()) };
    unsafe { luaL_setfuncs(state, METAMETH.as_ptr(), 0) };
    unsafe { lua_createtable(state, 0, (METH.len() - 1) as c_int) };
    unsafe { luaL_setfuncs(state, METH.as_ptr(), 0) };
    unsafe { lua_setfield(state, -2, META_INDEX.as_ptr().cast()) };
    unsafe { lua_pop(state, 1) };
}

unsafe  fn io_noclose(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    unsafe { (*p).closef = Some(io_noclose) };
    unsafe { push_fail(state) };
    unsafe { push_literal(state, ERR_CANNOT_CLOSE_STANDARD_FILE) };
    2
}

unsafe fn createstdfile(
    state: *mut lua_State,
    f: *mut File,
    k: Option<&'static [u8]>,
    fname: &'static [u8],
) {
    let p = unsafe { newprefile(state) };
    unsafe {
        (*p).f = f;
        (*p).closef = Some(io_noclose);
    }
    if let Some(key) = k {
        unsafe { lua_pushvalue(state, -1) };
        unsafe { lua_setfield(state, LUA_REGISTRYINDEX, key.as_ptr().cast()) };
    }
    unsafe { lua_setfield(state, -2, fname.as_ptr().cast()) };
}

pub(crate) unsafe  fn luaopen_io(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &IOLIB) };
    unsafe { createmeta(state) };
    let stdin_file = unsafe { fdopen(0, c"r".as_ptr()) };
    let stdout_file = unsafe { fdopen(1, c"w".as_ptr()) };
    let stderr_file = unsafe { fdopen(2, c"w".as_ptr()) };
    unsafe { createstdfile(state, stdin_file, Some(IO_INPUT), NAME_STDIN) };
    unsafe { createstdfile(state, stdout_file, Some(IO_OUTPUT), NAME_STDOUT) };
    unsafe { createstdfile(state, stderr_file, None, NAME_STDERR) };
    1
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn io_builtin_script() {
        run_lua_test(
            "test/io_builtin.lua",
            include_str!("../test/io_builtin.lua"),
        );
    }
}
