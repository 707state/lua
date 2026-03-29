use crate::lua_module::{
    LUA_REGISTRYINDEX, LuaCFunction, create_library, lua_Integer, lua_State, lua_createtable,
    lua_gettop, lua_pop, lua_pushboolean, lua_pushcclosure, lua_pushinteger, lua_pushnil,
    lua_pushstring, lua_pushvalue, lua_setfield, lua_settop, lua_upvalueindex, luaL_Reg, push_fail,
};
use core::ffi::{c_char, c_int, c_long, c_void};
use core::{mem, ptr};
use std::ffi::CStr;

const LUA_TNONE: c_int = -1;
const LUA_TNIL: c_int = 0;
const LUA_TNUMBER: c_int = 3;
const LUA_FILEHANDLE: &[u8] = b"FILE*\0";
const IO_INPUT: &[u8] = b"_IO_input\0";
const IO_OUTPUT: &[u8] = b"_IO_output\0";
const IOPREF_LEN: usize = 4;
const LUAL_BUFFERSIZE: usize = 8192;
const L_MAXLENNUM: usize = 200;
const LUA_N2SBUFFSZ: usize = 64;
const MAXARGLINE: c_int = 250;

const NAME_CLOSE: &[u8] = b"close\0";
const NAME_FLUSH: &[u8] = b"flush\0";
const NAME_INPUT: &[u8] = b"input\0";
const NAME_LINES: &[u8] = b"lines\0";
const NAME_OPEN: &[u8] = b"open\0";
const NAME_OUTPUT: &[u8] = b"output\0";
const NAME_POPEN: &[u8] = b"popen\0";
const NAME_READ: &[u8] = b"read\0";
const NAME_TMPFILE: &[u8] = b"tmpfile\0";
const NAME_TYPE: &[u8] = b"type\0";
const NAME_WRITE: &[u8] = b"write\0";
const NAME_SEEK: &[u8] = b"seek\0";
const NAME_SETVBUF: &[u8] = b"setvbuf\0";
const NAME_STDIN: &[u8] = b"stdin\0";
const NAME_STDOUT: &[u8] = b"stdout\0";
const NAME_STDERR: &[u8] = b"stderr\0";
const META_INDEX: &[u8] = b"__index\0";
const META_GC: &[u8] = b"__gc\0";
const META_CLOSE: &[u8] = b"__close\0";
const META_TOSTRING: &[u8] = b"__tostring\0";

const ERR_ATTEMPT_CLOSED: &[u8] = b"attempt to use a closed file\0";
const ERR_INVALID_MODE: &[u8] = b"invalid mode\0";
const ERR_TOO_MANY_ARGUMENTS: &[u8] = b"too many arguments\0";
const ERR_INVALID_FORMAT: &[u8] = b"invalid format\0";
const ERR_FILE_ALREADY_CLOSED: &[u8] = b"file is already closed\0";
const ERR_CANNOT_CLOSE_STANDARD_FILE: &[u8] = b"cannot close standard file\0";
const ERR_TOO_MANY_READ_ARGS: &[u8] = b"too many arguments\0";
const ERR_BAD_SEEK_INT: &[u8] = b"not an integer in proper range\0";
const STR_CLOSED_FILE: &[u8] = b"closed file\0";
const STR_FILE: &[u8] = b"file\0";
const STR_FILE_CLOSED: &[u8] = b"file (closed)\0";
const EMPTY_STRING: &[u8] = b"\0";

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

unsafe extern "C" {
    fn luaL_checkany(state: *mut lua_State, arg: c_int);
    fn luaL_checkstack(state: *mut lua_State, sz: c_int, msg: *const c_char);
    fn luaL_checklstring(state: *mut lua_State, arg: c_int, len: *mut usize) -> *const c_char;
    fn luaL_optlstring(
        state: *mut lua_State,
        arg: c_int,
        def: *const c_char,
        len: *mut usize,
    ) -> *const c_char;
    fn luaL_checkinteger(state: *mut lua_State, arg: c_int) -> lua_Integer;
    fn luaL_optinteger(state: *mut lua_State, arg: c_int, def: lua_Integer) -> lua_Integer;
    fn luaL_newmetatable(state: *mut lua_State, tname: *const c_char) -> c_int;
    fn luaL_setmetatable(state: *mut lua_State, tname: *const c_char);
    fn luaL_testudata(state: *mut lua_State, ud: c_int, tname: *const c_char) -> *mut c_void;
    fn luaL_checkudata(state: *mut lua_State, ud: c_int, tname: *const c_char) -> *mut c_void;
    fn luaL_error(state: *mut lua_State, fmt: *const c_char, ...) -> c_int;
    fn luaL_checkoption(
        state: *mut lua_State,
        arg: c_int,
        def: *const c_char,
        lst: *const *const c_char,
    ) -> c_int;
    fn luaL_fileresult(state: *mut lua_State, stat: c_int, fname: *const c_char) -> c_int;
    fn luaL_execresult(state: *mut lua_State, stat: c_int) -> c_int;
    fn luaL_setfuncs(state: *mut lua_State, regs: *const luaL_Reg, nup: c_int);

    fn lua_newuserdatauv(state: *mut lua_State, size: usize, nuvalue: c_int) -> *mut c_void;
    fn lua_copy(state: *mut lua_State, from: c_int, to: c_int);
    fn lua_getfield(state: *mut lua_State, index: c_int, key: *const c_char) -> c_int;
    fn lua_rotate(state: *mut lua_State, index: c_int, n: c_int);
    fn lua_type(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_touserdata(state: *mut lua_State, index: c_int) -> *mut c_void;
    fn lua_toboolean(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_tointegerx(state: *mut lua_State, index: c_int, isnum: *mut c_int) -> lua_Integer;
    fn lua_pushfstring(state: *mut lua_State, fmt: *const c_char, ...) -> *const c_char;
    fn lua_numbertocstring(state: *mut lua_State, idx: c_int, buff: *mut c_char) -> c_uint;
    fn lua_stringtonumber(state: *mut lua_State, s: *const c_char) -> usize;
}

use core::ffi::c_uint;

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
    fn localeconv() -> *mut LConv;
}

#[repr(C)]
struct LConv {
    decimal_point: *mut c_char,
}

const SEEK_SET_VALUE: c_int = 0;
const SEEK_CUR_VALUE: c_int = 1;
const SEEK_END_VALUE: c_int = 2;
const IONBF_VALUE: c_int = 2;
const IOFBF_VALUE: c_int = 0;
const IOLBF_VALUE: c_int = 1;
const EOF_VALUE: c_int = -1;

#[cfg(any(target_os = "macos", target_os = "ios"))]
unsafe extern "C" {
    fn __error() -> *mut c_int;
}
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
unsafe extern "C" {
    fn __errno_location() -> *mut c_int;
}

#[cfg(target_os = "android")]
unsafe extern "C" {
    fn __errno() -> *mut c_int;
}

#[inline]
unsafe fn errno_ptr() -> *mut c_int {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        unsafe { __error() }
    }
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        unsafe { __errno_location() }
    }
    #[cfg(target_os = "android")]
    {
        unsafe { __errno() }
    }
}

#[inline]
unsafe fn reset_errno() {
    unsafe { *errno_ptr() = 0 };
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
    unsafe { lua_type(state, idx) == LUA_TNIL }
}

#[inline]
unsafe fn is_none_or_nil(state: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(state, idx) <= LUA_TNIL }
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

unsafe extern "C" fn io_type(state: *mut lua_State) -> c_int {
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

unsafe extern "C" fn f_tostring(state: *mut lua_State) -> c_int {
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
        unsafe { luaL_error(state, ERR_ATTEMPT_CLOSED.as_ptr().cast()) };
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

unsafe extern "C" fn aux_close(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    let cf = unsafe { (*p).closef };
    unsafe { (*p).closef = None };
    unsafe { cf.expect("open file has close callback")(state) }
}

unsafe extern "C" fn f_close(state: *mut lua_State) -> c_int {
    unsafe { tofile(state) };
    unsafe { aux_close(state) }
}

unsafe extern "C" fn io_close(state: *mut lua_State) -> c_int {
    if unsafe { is_none(state, 1) } {
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, IO_OUTPUT.as_ptr().cast()) };
    }
    unsafe { f_close(state) }
}

unsafe extern "C" fn f_gc(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    if !unsafe { isclosed(p) } && !unsafe { (*p).f.is_null() } {
        let _ = unsafe { aux_close(state) };
    }
    0
}

unsafe extern "C" fn io_fclose(state: *mut lua_State) -> c_int {
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

unsafe fn strerror() -> *const c_char {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    unsafe extern "C" {
        fn strerror(errnum: c_int) -> *const c_char;
    }
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    unsafe extern "C" {
        fn strerror(errnum: c_int) -> *const c_char;
    }
    unsafe { strerror(*errno_ptr()) }
}

unsafe extern "C" fn io_open(state: *mut lua_State) -> c_int {
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

unsafe extern "C" fn io_pclose(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    unsafe { reset_errno() };
    unsafe { luaL_execresult(state, pclose((*p).f)) }
}

unsafe extern "C" fn io_popen(state: *mut lua_State) -> c_int {
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

unsafe extern "C" fn io_tmpfile(state: *mut lua_State) -> c_int {
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
        let filename = unsafe { crate::luaffi::lua_tolstring(state, 1, ptr::null_mut()) };
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

unsafe extern "C" fn io_input(state: *mut lua_State) -> c_int {
    unsafe { g_iofile(state, IO_INPUT, c"r".to_bytes_with_nul()) }
}

unsafe extern "C" fn io_output(state: *mut lua_State) -> c_int {
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

unsafe extern "C" fn f_lines(state: *mut lua_State) -> c_int {
    unsafe { tofile(state) };
    unsafe { aux_lines(state, 0) };
    1
}

unsafe extern "C" fn io_lines(state: *mut lua_State) -> c_int {
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
            if unsafe { lua_type(state, n) } == LUA_TNUMBER {
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

unsafe extern "C" fn io_read(state: *mut lua_State) -> c_int {
    unsafe { g_read(state, getiofile(state, IO_INPUT), 1) }
}

unsafe extern "C" fn f_read(state: *mut lua_State) -> c_int {
    unsafe { g_read(state, tofile(state), 2) }
}

unsafe extern "C" fn io_readline(state: *mut lua_State) -> c_int {
    let p = unsafe { lua_touserdata(state, lua_upvalueindex(1)) as *mut luaL_Stream };
    let mut isnum = 0;
    let mut n = unsafe { lua_tointegerx(state, lua_upvalueindex(2), &mut isnum) as c_int };
    if unsafe { isclosed(p) } {
        return unsafe { luaL_error(state, ERR_FILE_ALREADY_CLOSED.as_ptr().cast()) };
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
                    crate::luaffi::lua_tolstring(state, -n + 1, ptr::null_mut()),
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

unsafe extern "C" fn io_write(state: *mut lua_State) -> c_int {
    unsafe { g_write(state, getiofile(state, IO_OUTPUT), 1) }
}

unsafe extern "C" fn f_write(state: *mut lua_State) -> c_int {
    let f = unsafe { tofile(state) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { g_write(state, f, 2) }
}

unsafe extern "C" fn f_seek(state: *mut lua_State) -> c_int {
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

unsafe extern "C" fn f_setvbuf(state: *mut lua_State) -> c_int {
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

unsafe extern "C" fn f_flush(state: *mut lua_State) -> c_int {
    unsafe { aux_flush(state, tofile(state)) }
}

unsafe extern "C" fn io_flush(state: *mut lua_State) -> c_int {
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

unsafe extern "C" fn io_noclose(state: *mut lua_State) -> c_int {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_io(state: *mut lua_State) -> c_int {
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
