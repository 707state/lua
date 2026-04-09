//! `io` 标准库 — 纯 Rust 实现
//!
//! 所有文件 I/O 均通过 `std::fs::File`、`std::io`、`std::process` 完成，
//! 不调用任何 C 符号（无 fopen/fclose/fread/fwrite 等）。

use crate::api::*;
use crate::aux_rs::{
    luaL_checkany, luaL_checkinteger, luaL_checklstring, luaL_checkoption, luaL_checkstack,
    luaL_checkudata, luaL_execresult, luaL_fileresult, luaL_newmetatable, luaL_optinteger,
    luaL_optlstring, luaL_setfuncs, luaL_setmetatable, luaL_testudata,
};
use crate::lua_module::{
    create_library, lua_pop, lua_replace_local, lua_upvalueindex, luaL_Reg, luaL_error,
    luaL_error_str, push_fail,
};
use crate::luaffi::LuaCFunction;
use crate::runtime::*;
use core::ffi::{c_char, c_int};
use core::{mem, ptr};
use std::io::{BufRead, Read, Seek, SeekFrom, Write};

// ─── Rust-native stream ────────────────────────────────────────────────────

/// Rust-native 文件流，替代 C `FILE*`
enum RustStream {
    /// 普通文件（`io.open`、`io.tmpfile`）
    File(std::fs::File),
    /// 子进程管道（`io.popen`）
    Pipe {
        child: std::process::Child,
        /// popen "r" → 读子进程 stdout；"w" → 写子进程 stdin
        read: bool,
    },
    /// stdin/stdout/stderr 占位（标准流，不能关闭）
    Stdio(StdioKind),
}

#[derive(Clone, Copy)]
enum StdioKind {
    Stdin,
    Stdout,
    Stderr,
}

impl RustStream {
    fn read_bytes(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            RustStream::File(f) => f.read(buf),
            RustStream::Pipe { child, read: true } => child.stdout.as_mut().unwrap().read(buf),
            RustStream::Stdio(StdioKind::Stdin) => std::io::stdin().read(buf),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "not readable",
            )),
        }
    }

    fn write_bytes(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            RustStream::File(f) => f.write(buf),
            RustStream::Pipe { child, read: false } => child.stdin.as_mut().unwrap().write(buf),
            RustStream::Stdio(StdioKind::Stdout) => {
                let written = std::io::stdout().write(buf)?;
                Ok(written)
            }
            RustStream::Stdio(StdioKind::Stderr) => {
                let written = std::io::stderr().write(buf)?;
                Ok(written)
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "not writable",
            )),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            RustStream::File(f) => f.flush(),
            RustStream::Pipe { child, read: false } => child.stdin.as_mut().unwrap().flush(),
            RustStream::Stdio(StdioKind::Stdout) => std::io::stdout().flush(),
            RustStream::Stdio(StdioKind::Stderr) => std::io::stderr().flush(),
            _ => Ok(()),
        }
    }

    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            RustStream::File(f) => f.seek(pos),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "not seekable",
            )),
        }
    }

    /// 读一行（逐字节读取，不使用 BufReader 避免预读破坏文件位置）
    fn read_line(&mut self, chop: bool) -> (Vec<u8>, bool) {
        let mut out = Vec::new();
        let mut had_newline = false;
        loop {
            match self.read_byte() {
                None => break,
                Some(b'\n') => {
                    had_newline = true;
                    if !chop {
                        out.push(b'\n');
                    }
                    break;
                }
                Some(b) => out.push(b),
            }
        }
        (out, had_newline)
    }

    /// 读全部内容
    fn read_all(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            RustStream::File(f) => {
                let _ = f.read_to_end(&mut out);
            }
            RustStream::Pipe { child, read: true } => {
                let _ = child.stdout.as_mut().unwrap().read_to_end(&mut out);
            }
            RustStream::Stdio(StdioKind::Stdin) => {
                let _ = std::io::stdin().read_to_end(&mut out);
            }
            _ => {}
        }
        out
    }

    /// 读 n 字节
    fn read_n(&mut self, n: usize) -> Vec<u8> {
        let mut out = vec![0u8; n];
        let nr = self.read_bytes(&mut out).unwrap_or(0);
        out.truncate(nr);
        out
    }

    /// 读数字（跳过空白，解析数字字面量）
    fn read_number(&mut self) -> Option<Vec<u8>> {
        // 读取字符逐个解析，不依赖 C locale
        let mut buf = Vec::with_capacity(L_MAXLENNUM);
        let mut c = self.read_byte();
        // skip whitespace
        while c.map_or(false, |b| (b as char).is_ascii_whitespace()) {
            c = self.read_byte();
        }
        // optional sign
        if c == Some(b'+') || c == Some(b'-') {
            buf.push(c.unwrap());
            c = self.read_byte();
        }
        // hex prefix?
        let hex = if c == Some(b'0') {
            buf.push(b'0');
            c = self.read_byte();
            if c == Some(b'x') || c == Some(b'X') {
                buf.push(c.unwrap());
                c = self.read_byte();
                true
            } else {
                false
            }
        } else {
            false
        };
        // digits before decimal
        let mut count = 0usize;
        while c.map_or(false, |b| {
            if hex {
                (b as char).is_ascii_hexdigit()
            } else {
                (b as char).is_ascii_digit()
            }
        }) {
            if buf.len() < L_MAXLENNUM {
                buf.push(c.unwrap());
                count += 1;
            }
            c = self.read_byte();
        }
        // decimal point
        if c == Some(b'.') {
            buf.push(b'.');
            c = self.read_byte();
            while c.map_or(false, |b| {
                if hex {
                    (b as char).is_ascii_hexdigit()
                } else {
                    (b as char).is_ascii_digit()
                }
            }) {
                if buf.len() < L_MAXLENNUM {
                    buf.push(c.unwrap());
                    count += 1;
                }
                c = self.read_byte();
            }
        }
        // exponent
        let exp_char = if hex { b'p' } else { b'e' };
        if count > 0 && c.map_or(false, |b| b.to_ascii_lowercase() == exp_char) {
            buf.push(c.unwrap());
            c = self.read_byte();
            if c == Some(b'+') || c == Some(b'-') {
                buf.push(c.unwrap());
                c = self.read_byte();
            }
            while c.map_or(false, |b| (b as char).is_ascii_digit()) {
                if buf.len() < L_MAXLENNUM {
                    buf.push(c.unwrap());
                }
                c = self.read_byte();
            }
        }
        // unread last byte
        if let Some(b) = c {
            self.unread_byte(b);
        }
        if count == 0 && buf.len() <= 2 {
            return None;
        }
        buf.push(0); // null terminator for lua_stringtonumber
        Some(buf)
    }

    /// 读单个字节
    fn read_byte(&mut self) -> Option<u8> {
        let mut b = [0u8];
        match self.read_bytes(&mut b) {
            Ok(1) => Some(b[0]),
            _ => None,
        }
    }

    /// 放回一个字节（仅支持 File，通过 seek -1 实现近似）
    fn unread_byte(&mut self, _b: u8) {
        // 对于 File，seek back by 1
        if let RustStream::File(f) = self {
            let _ = f.seek(SeekFrom::Current(-1));
        }
        // Stdio/Pipe 无法 unread，忽略（数字解析已完成）
    }

    /// 检测 EOF（读一字节，成功则 seek 回去）
    fn test_eof(&mut self) -> bool {
        match self {
            RustStream::File(f) => {
                let mut b = [0u8];
                match f.read(&mut b) {
                    Ok(1) => {
                        let _ = f.seek(SeekFrom::Current(-1));
                        true // not EOF
                    }
                    _ => false,
                }
            }
            _ => {
                let b = self.read_byte();
                // can't unread for pipe/stdio; accept data loss for EOF test
                b.is_some()
            }
        }
    }

    /// 关闭并获得子进程退出状态（仅 Pipe）
    fn close_pipe(&mut self) -> c_int {
        if let RustStream::Pipe { child, .. } = self {
            // 关闭 stdin/stdout
            drop(child.stdin.take());
            drop(child.stdout.take());
            match child.wait() {
                Ok(status) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;
                        if let Some(sig) = status.signal() {
                            return sig;
                        }
                        status.code().unwrap_or(0)
                    }
                    #[cfg(not(unix))]
                    {
                        status.code().unwrap_or(0)
                    }
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

// ─── luaL_Stream (保持 ABI 结构) ────────────────────────────────────────────

/// Lua 文件句柄结构（ABI 与 C lauxlib 定义兼容）
///
/// `f` 字段存储 `Box<RustStream>` 的原始指针（reinterpret-cast 为 `*mut RustFile`）
#[repr(C)]
struct RustFile {
    _private: [u8; 0],
}

#[repr(C)]
struct luaL_Stream {
    f: *mut RustFile,
    closef: LuaCFunction,
}

unsafe impl Sync for luaL_Stream {}

// ─── 函数表 ────────────────────────────────────────────────────────────────

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

// ─── 辅助函数 ────────────────────────────────────────────────────────────────

/// 获取 RustStream 指针（存在 luaL_Stream.f 里）
#[inline]
unsafe fn get_stream(p: *mut luaL_Stream) -> *mut RustStream {
    unsafe { (*p).f as *mut RustStream }
}

#[inline]
unsafe fn push_literal(state: *mut lua_State, s: &'static [u8]) {
    unsafe { lua_pushstring(state, s.as_ptr().cast()) };
}

#[inline]
unsafe fn is_none(state: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(state, idx) == LuaType::None.as_c_int() }
}

#[inline]
unsafe fn is_nil(state: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(state, idx) == LuaType::Nil.as_c_int() }
}

#[inline]
unsafe fn is_none_or_nil(state: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(state, idx) <= LuaType::Nil.as_c_int() }
}

#[inline]
unsafe fn checkstring(state: *mut lua_State, arg: c_int) -> *const c_char {
    luaL_checklstring(state, arg, ptr::null_mut())
}

#[inline]
unsafe fn optstring(state: *mut lua_State, arg: c_int, default: &'static [u8]) -> *const c_char {
    luaL_optlstring(state, arg, default.as_ptr().cast(), ptr::null_mut())
}

#[inline]
unsafe fn tolstream(state: *mut lua_State) -> *mut luaL_Stream {
    luaL_checkudata(state, 1, LUA_FILEHANDLE.as_ptr().cast()) as *mut luaL_Stream
}

#[inline]
unsafe fn isclosed(stream: *mut luaL_Stream) -> bool {
    unsafe { (*stream).closef.is_none() }
}

unsafe fn push_dynamic_error(state: *mut lua_State, message: &str) -> c_int {
    let bytes = message.as_bytes();
    unsafe {
        crate::lua_module::lua_pushlstring(state, bytes.as_ptr().cast(), bytes.len());
        crate::lua_module::lua_error(state)
    }
}

fn file_result_from_io(
    state: *mut lua_State,
    result: std::io::Result<()>,
    fname: Option<&str>,
) -> c_int {
    match result {
        Ok(()) => luaL_fileresult(state, 1, ptr::null()),
        Err(e) => {
            let code = e.raw_os_error().unwrap_or(-1);
            let msg = e.to_string();
            unsafe { push_fail(state) };
            if let Some(name) = fname {
                let full = format!("{name}: {msg}");
                unsafe {
                    crate::lua_module::lua_pushlstring(state, full.as_ptr().cast(), full.len())
                };
            } else {
                unsafe {
                    crate::lua_module::lua_pushlstring(state, msg.as_ptr().cast(), msg.len())
                };
            }
            unsafe { lua_pushinteger(state, code as lua_Integer) };
            3
        }
    }
}

// ─── Lua 函数实现 ─────────────────────────────────────────────────────────────

unsafe fn io_type(state: *mut lua_State) -> c_int {
    luaL_checkany(state, 1);
    let p = luaL_testudata(state, 1, LUA_FILEHANDLE.as_ptr().cast()) as *mut luaL_Stream;
    if p.is_null() {
        unsafe { push_fail(state) };
    } else if unsafe { isclosed(p) } {
        unsafe { push_literal(state, STR_CLOSED_FILE) };
    } else {
        unsafe { push_literal(state, STR_FILE) };
    }
    1
}

unsafe fn f_tostring(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    if unsafe { isclosed(p) } {
        unsafe { push_literal(state, STR_FILE_CLOSED) };
    } else {
        let addr = unsafe { (*p).f } as usize;
        unsafe { lua_pushfstring_rs(state, &format!("file (0x{addr:x})")) };
    }
    1
}

unsafe fn tofile(state: *mut lua_State) -> *mut RustStream {
    let p = unsafe { tolstream(state) };
    if unsafe { isclosed(p) } {
        unsafe { luaL_error_str(state, ERR_ATTEMPT_CLOSED.as_ptr().cast()) };
    }
    unsafe { get_stream(p) }
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

/// 将 Box<RustStream> 存入 luaL_Stream.f
unsafe fn set_stream(p: *mut luaL_Stream, stream: Box<RustStream>) {
    let raw = Box::into_raw(stream) as *mut RustFile;
    unsafe { (*p).f = raw };
}

/// 取回 Box<RustStream> 并 drop（关闭）
unsafe fn take_stream(p: *mut luaL_Stream) -> Box<RustStream> {
    let raw = unsafe { (*p).f as *mut RustStream };
    unsafe { (*p).f = ptr::null_mut() };
    unsafe { Box::from_raw(raw) }
}

unsafe fn newfile(state: *mut lua_State) -> *mut luaL_Stream {
    let p = unsafe { newprefile(state) };
    unsafe { (*p).closef = Some(io_fclose) };
    p
}

unsafe fn aux_close(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    let cf = unsafe { (*p).closef };
    unsafe { (*p).closef = None };
    unsafe { cf.expect("open file has close callback")(state) }
}

unsafe fn f_close(state: *mut lua_State) -> c_int {
    unsafe { tofile(state) };
    unsafe { aux_close(state) }
}

unsafe fn io_close(state: *mut lua_State) -> c_int {
    if unsafe { is_none(state, 1) } {
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, IO_OUTPUT.as_ptr().cast()) };
    }
    unsafe { f_close(state) }
}

unsafe fn f_gc(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    if !unsafe { isclosed(p) } && !unsafe { (*p).f.is_null() } {
        let _ = unsafe { aux_close(state) };
    }
    0
}

unsafe fn io_fclose(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    if unsafe { (*p).f.is_null() } {
        return luaL_fileresult(state, 0, ptr::null());
    }
    let stream = unsafe { take_stream(p) };
    // drop stream → 调用 std::fs::File::drop() 自动关闭
    drop(stream);
    luaL_fileresult(state, 1, ptr::null())
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

/// 将 Lua mode 字符串转换为 `std::fs::OpenOptions`
fn open_options_from_mode(mode: &[u8]) -> Option<(std::fs::OpenOptions, bool)> {
    if mode.is_empty() {
        return None;
    }
    let base = mode[0];
    let plus = mode.get(1).copied() == Some(b'+') || mode.get(2).copied() == Some(b'+');
    let mut opts = std::fs::OpenOptions::new();
    match base {
        b'r' => {
            opts.read(true).write(plus);
        }
        b'w' => {
            opts.write(true).create(true).truncate(!plus).read(plus);
        }
        b'a' => {
            opts.append(true).create(true).read(plus);
        }
        _ => return None,
    }
    Some((opts, /* binary */ false))
}

unsafe fn opencheck(state: *mut lua_State, fname: *const c_char, mode: *const c_char) {
    let fname_s = unsafe { std::ffi::CStr::from_ptr(fname) }
        .to_string_lossy()
        .into_owned();
    let mode_s = unsafe { std::ffi::CStr::from_ptr(mode) }.to_bytes();
    let Some((opts, _)) = open_options_from_mode(mode_s) else {
        unsafe { luaL_error(state, &format!("cannot open '{fname_s}': invalid mode")) };
        return;
    };
    match opts.open(&fname_s) {
        Ok(file) => {
            let p = unsafe { newfile(state) };
            unsafe { set_stream(p, Box::new(RustStream::File(file))) };
        }
        Err(e) => {
            unsafe { luaL_error(state, &format!("cannot open file '{fname_s}' ({e})")) };
        }
    }
}

unsafe fn io_open(state: *mut lua_State) -> c_int {
    let filename = unsafe { checkstring(state, 1) };
    let mode = unsafe { optstring(state, 2, c"r".to_bytes_with_nul()) };
    let mode_bytes = unsafe { std::ffi::CStr::from_ptr(mode) }.to_bytes();
    if !check_mode(mode_bytes) {
        let _ = crate::lua_module::luaL_argerror(state, 2, ERR_INVALID_MODE.as_ptr().cast());
    }
    let fname_s = unsafe { std::ffi::CStr::from_ptr(filename) }
        .to_string_lossy()
        .into_owned();
    let Some((opts, _)) = open_options_from_mode(mode_bytes) else {
        return luaL_fileresult(state, 0, filename);
    };
    match opts.open(&fname_s) {
        Ok(file) => {
            let p = unsafe { newfile(state) };
            unsafe { set_stream(p, Box::new(RustStream::File(file))) };
            1
        }
        Err(_) => luaL_fileresult(state, 0, filename),
    }
}

unsafe fn io_pclose(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    if unsafe { (*p).f.is_null() } {
        return luaL_fileresult(state, 0, ptr::null());
    }
    let mut stream = unsafe { take_stream(p) };
    let exit_code = stream.close_pipe();
    drop(stream);
    luaL_execresult(state, exit_code)
}

unsafe fn io_popen(state: *mut lua_State) -> c_int {
    let filename = unsafe { checkstring(state, 1) };
    let mode = unsafe { optstring(state, 2, c"r".to_bytes_with_nul()) };
    let mode_bytes = unsafe { std::ffi::CStr::from_ptr(mode) }.to_bytes();
    if !check_modep(mode_bytes) {
        let _ = crate::lua_module::luaL_argerror(state, 2, ERR_INVALID_MODE.as_ptr().cast());
    }
    let cmd = unsafe { std::ffi::CStr::from_ptr(filename) }
        .to_string_lossy()
        .into_owned();
    let read = mode_bytes == b"r";
    let mut cmd_builder = std::process::Command::new("sh");
    cmd_builder.arg("-c").arg(&cmd);
    if read {
        cmd_builder.stdout(std::process::Stdio::piped());
        cmd_builder.stdin(std::process::Stdio::null());
    } else {
        cmd_builder.stdin(std::process::Stdio::piped());
        cmd_builder.stdout(std::process::Stdio::null());
    }
    cmd_builder.stderr(std::process::Stdio::inherit());
    match cmd_builder.spawn() {
        Ok(child) => {
            let p = unsafe { newprefile(state) };
            unsafe { set_stream(p, Box::new(RustStream::Pipe { child, read })) };
            unsafe { (*p).closef = Some(io_pclose) };
            1
        }
        Err(_) => luaL_fileresult(state, 0, filename),
    }
}

unsafe fn io_tmpfile(state: *mut lua_State) -> c_int {
    match tempfile() {
        Ok(file) => {
            let p = unsafe { newfile(state) };
            unsafe { set_stream(p, Box::new(RustStream::File(file))) };
            1
        }
        Err(_) => luaL_fileresult(state, 0, ptr::null()),
    }
}

/// 创建匿名临时文件（使用 tempfile crate 兼容方式：std 实现）
#[cfg(not(target_arch = "wasm32"))]
fn tempfile() -> std::io::Result<std::fs::File> {
    // 用 std 实现：在 temp 目录创建文件，立即 unlink（Unix）或标记删除（Windows）
    let dir = std::env::temp_dir();
    // 使用系统时间 + 进程 ID 生成唯一名
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let path = dir.join(format!("lua_tmp_{pid}_{ts}"));
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    // 立即删除文件名（file 仍然有效）
    let _ = std::fs::remove_file(&path);
    Ok(file)
}

/// wasm32 下没有真实文件系统，io.tmpfile 始终返回错误。
#[cfg(target_arch = "wasm32")]
fn tempfile() -> std::io::Result<std::fs::File> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "io.tmpfile is not supported in WebAssembly",
    ))
}

unsafe fn getiofile(state: *mut lua_State, findex: &'static [u8]) -> *mut RustStream {
    unsafe { lua_getfield(state, LUA_REGISTRYINDEX, findex.as_ptr().cast()) };
    let p = unsafe { lua_touserdata(state, -1) as *mut luaL_Stream };
    if unsafe { isclosed(p) } {
        let name = std::str::from_utf8(&findex[IOPREF_LEN..findex.len() - 1]).unwrap();
        let msg = format!("default {} file is closed", name);
        unsafe { push_dynamic_error(state, &msg) };
    }
    unsafe { get_stream(p) }
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

unsafe fn io_input(state: *mut lua_State) -> c_int {
    unsafe { g_iofile(state, IO_INPUT, c"r".to_bytes_with_nul()) }
}

unsafe fn io_output(state: *mut lua_State) -> c_int {
    unsafe { g_iofile(state, IO_OUTPUT, c"w".to_bytes_with_nul()) }
}

unsafe fn aux_lines(state: *mut lua_State, toclose: c_int) {
    let n = unsafe { lua_gettop(state) } - 1;
    if n > MAXARGLINE {
        let _ = crate::lua_module::luaL_argerror(
            state,
            MAXARGLINE + 2,
            ERR_TOO_MANY_ARGUMENTS.as_ptr().cast(),
        );
    }
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_pushinteger(state, n as lua_Integer) };
    unsafe { lua_pushboolean(state, toclose) };
    unsafe { lua_rotate(state, 2, 3) };
    unsafe { lua_pushcclosure(state, Some(io_readline), 3 + n) };
}

unsafe fn f_lines(state: *mut lua_State) -> c_int {
    unsafe { tofile(state) };
    unsafe { aux_lines(state, 0) };
    1
}

unsafe fn io_lines(state: *mut lua_State) -> c_int {
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

// ─── 读操作 ──────────────────────────────────────────────────────────────────

unsafe fn g_read(state: *mut lua_State, stream: *mut RustStream, first: c_int) -> c_int {
    let nargs = unsafe { lua_gettop(state) } - 1;
    let mut n;
    let mut success;
    if nargs == 0 {
        // 默认读一行（不含换行符）
        let (line, had_nl) = unsafe { (*stream).read_line(true) };
        if !line.is_empty() || had_nl {
            unsafe { crate::lua_module::lua_pushlstring(state, line.as_ptr().cast(), line.len()) };
            success = 1;
        } else {
            unsafe { lua_pushnil(state) };
            success = 0;
        }
        n = first + 1;
    } else {
        luaL_checkstack(state, nargs + 20, ERR_TOO_MANY_READ_ARGS.as_ptr().cast());
        success = 1;
        n = first;
        let mut left = nargs;
        while left > 0 && success != 0 {
            if unsafe { lua_type(state, n) } == LuaType::Number.as_c_int() {
                let l = luaL_checkinteger(state, n) as usize;
                if l == 0 {
                    // 测试 EOF
                    let has_data = unsafe { (*stream).test_eof() };
                    unsafe { crate::lua_module::lua_pushlstring(state, c"".as_ptr(), 0) };
                    success = has_data as c_int;
                } else {
                    let data = unsafe { (*stream).read_n(l) };
                    let ok = !data.is_empty();
                    unsafe {
                        crate::lua_module::lua_pushlstring(state, data.as_ptr().cast(), data.len())
                    };
                    success = ok as c_int;
                }
            } else {
                let p = unsafe { checkstring(state, n) };
                let bytes = unsafe { std::ffi::CStr::from_ptr(p) }.to_bytes();
                let fmt = if bytes.first() == Some(&b'*') {
                    &bytes[1..]
                } else {
                    bytes
                };
                success = match fmt.first().copied() {
                    Some(b'n') => {
                        // 读数字
                        match unsafe { (*stream).read_number() } {
                            Some(buf) => {
                                if unsafe { lua_stringtonumber(state, buf.as_ptr().cast()) } != 0 {
                                    1
                                } else {
                                    unsafe { lua_pushnil(state) };
                                    0
                                }
                            }
                            None => {
                                unsafe { lua_pushnil(state) };
                                0
                            }
                        }
                    }
                    Some(b'l') => {
                        let (line, had_nl) = unsafe { (*stream).read_line(true) };
                        if !line.is_empty() || had_nl {
                            unsafe {
                                crate::lua_module::lua_pushlstring(
                                    state,
                                    line.as_ptr().cast(),
                                    line.len(),
                                )
                            };
                            1
                        } else {
                            unsafe { lua_pushnil(state) };
                            0
                        }
                    }
                    Some(b'L') => {
                        let (line, had_nl) = unsafe { (*stream).read_line(false) };
                        if !line.is_empty() || had_nl {
                            unsafe {
                                crate::lua_module::lua_pushlstring(
                                    state,
                                    line.as_ptr().cast(),
                                    line.len(),
                                )
                            };
                            1
                        } else {
                            unsafe { lua_pushnil(state) };
                            0
                        }
                    }
                    Some(b'a') => {
                        let data = unsafe { (*stream).read_all() };
                        unsafe {
                            crate::lua_module::lua_pushlstring(
                                state,
                                data.as_ptr().cast(),
                                data.len(),
                            )
                        };
                        1
                    }
                    _ => {
                        return crate::lua_module::luaL_argerror(
                            state,
                            n,
                            ERR_INVALID_FORMAT.as_ptr().cast(),
                        );
                    }
                };
            }
            n += 1;
            left -= 1;
        }
    }
    if success == 0 {
        unsafe { lua_pop(state, 1) };
        unsafe { push_fail(state) };
    }
    n - first
}

unsafe fn io_read(state: *mut lua_State) -> c_int {
    let stream = unsafe { getiofile(state, IO_INPUT) };
    unsafe { g_read(state, stream, 1) }
}

unsafe fn f_read(state: *mut lua_State) -> c_int {
    let stream = unsafe { tofile(state) };
    unsafe { g_read(state, stream, 2) }
}

unsafe fn io_readline(state: *mut lua_State) -> c_int {
    let p = unsafe { lua_touserdata(state, lua_upvalueindex(1)) as *mut luaL_Stream };
    let mut isnum = 0;
    let mut n = unsafe { lua_tointegerx(state, lua_upvalueindex(2), &mut isnum) as c_int };
    if unsafe { isclosed(p) } {
        return unsafe { luaL_error_str(state, ERR_FILE_ALREADY_CLOSED.as_ptr().cast()) };
    }
    unsafe { lua_settop(state, 1) };
    luaL_checkstack(state, n, ERR_TOO_MANY_READ_ARGS.as_ptr().cast());
    for i in 1..=n {
        unsafe { lua_pushvalue(state, lua_upvalueindex(3 + i)) };
    }
    let stream = unsafe { get_stream(p) };
    n = unsafe { g_read(state, stream, 2) };
    if unsafe { lua_toboolean(state, -n) } != 0 {
        n
    } else {
        if n > 1 {
            let msg_ptr = unsafe { lua_tolstring(state, -n + 1, ptr::null_mut()) };
            let msg_s = unsafe { std::ffi::CStr::from_ptr(msg_ptr) }.to_string_lossy();
            return unsafe { luaL_error(state, &msg_s) };
        }
        if unsafe { lua_toboolean(state, lua_upvalueindex(3)) } != 0 {
            unsafe { lua_settop(state, 0) };
            unsafe { lua_pushvalue(state, lua_upvalueindex(1)) };
            let _ = unsafe { aux_close(state) };
        }
        0
    }
}

// ─── 写操作 ──────────────────────────────────────────────────────────────────

unsafe fn g_write(state: *mut lua_State, stream: *mut RustStream, mut arg: c_int) -> c_int {
    let mut nargs = unsafe { lua_gettop(state) } - arg;
    let mut totalbytes = 0usize;
    loop {
        let current = nargs;
        nargs -= 1;
        if current == 0 {
            break;
        }
        let mut buff = [0 as c_char; LUA_N2SBUFFSZ];
        let len = unsafe { lua_numbertocstring(state, arg, buff.as_mut_ptr()) };
        let data: &[u8] = if len > 0 {
            let s = buff.as_ptr().cast::<u8>();
            // len includes null terminator, so actual string is len-1
            unsafe { std::slice::from_raw_parts(s, len as usize - 1) }
        } else {
            let mut sl = 0usize;
            let sp = luaL_checklstring(state, arg, &mut sl);
            unsafe { std::slice::from_raw_parts(sp.cast::<u8>(), sl) }
        };
        match unsafe { (*stream).write_bytes(data) } {
            Ok(n) => {
                totalbytes += n;
                if n < data.len() {
                    let result = luaL_fileresult(state, 0, ptr::null());
                    unsafe { lua_pushinteger(state, totalbytes as lua_Integer) };
                    return result + 1;
                }
            }
            Err(_) => {
                let result = luaL_fileresult(state, 0, ptr::null());
                unsafe { lua_pushinteger(state, totalbytes as lua_Integer) };
                return result + 1;
            }
        }
        arg += 1;
    }
    // 返回文件句柄本身（方便链式调用）
    1
}

unsafe fn io_write(state: *mut lua_State) -> c_int {
    let stream = unsafe { getiofile(state, IO_OUTPUT) };
    unsafe { g_write(state, stream, 1) }
}

unsafe fn f_write(state: *mut lua_State) -> c_int {
    let stream = unsafe { tofile(state) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { g_write(state, stream, 2) }
}

// ─── seek / setvbuf / flush ──────────────────────────────────────────────────

unsafe fn f_seek(state: *mut lua_State) -> c_int {
    let mode_names = [
        c"set".as_ptr(),
        c"cur".as_ptr(),
        c"end".as_ptr(),
        ptr::null(),
    ];
    let stream = unsafe { tofile(state) };
    let op = luaL_checkoption(state, 2, c"cur".as_ptr(), mode_names.as_ptr());
    let p3 = luaL_optinteger(state, 3, 0);
    let offset = p3 as i64;
    let seek_from = match op {
        0 => SeekFrom::Start(offset.max(0) as u64),
        1 => SeekFrom::Current(offset),
        2 => SeekFrom::End(offset),
        _ => {
            return luaL_fileresult(state, 0, ptr::null());
        }
    };
    match unsafe { (*stream).seek(seek_from) } {
        Ok(pos) => {
            unsafe { lua_pushinteger(state, pos as lua_Integer) };
            1
        }
        Err(_) => luaL_fileresult(state, 0, ptr::null()),
    }
}

unsafe fn f_setvbuf(state: *mut lua_State) -> c_int {
    // Rust std 不支持 setvbuf；此函数为 no-op，返回成功
    let mode_names = [
        c"no".as_ptr(),
        c"full".as_ptr(),
        c"line".as_ptr(),
        ptr::null(),
    ];
    unsafe { tofile(state) };
    let _ = luaL_checkoption(state, 2, ptr::null(), mode_names.as_ptr());
    let _ = luaL_optinteger(state, 3, LUAL_BUFFERSIZE as lua_Integer);
    luaL_fileresult(state, 1, ptr::null())
}

unsafe fn aux_flush(state: *mut lua_State, stream: *mut RustStream) -> c_int {
    match unsafe { (*stream).flush() } {
        Ok(()) => luaL_fileresult(state, 1, ptr::null()),
        Err(_) => luaL_fileresult(state, 0, ptr::null()),
    }
}

unsafe fn f_flush(state: *mut lua_State) -> c_int {
    let stream = unsafe { tofile(state) };
    unsafe { aux_flush(state, stream) }
}

unsafe fn io_flush(state: *mut lua_State) -> c_int {
    let stream = unsafe { getiofile(state, IO_OUTPUT) };
    unsafe { aux_flush(state, stream) }
}

// ─── 元表 / 标准文件 ──────────────────────────────────────────────────────────

unsafe fn createmeta(state: *mut lua_State) {
    luaL_newmetatable(state, LUA_FILEHANDLE.as_ptr().cast());
    luaL_setfuncs(state, METAMETH.as_ptr(), 0);
    unsafe { lua_createtable(state, 0, (METH.len() - 1) as c_int) };
    luaL_setfuncs(state, METH.as_ptr(), 0);
    unsafe { lua_setfield(state, -2, META_INDEX.as_ptr().cast()) };
    unsafe { lua_pop(state, 1) };
}

unsafe fn io_noclose(state: *mut lua_State) -> c_int {
    let p = unsafe { tolstream(state) };
    unsafe { (*p).closef = Some(io_noclose) };
    unsafe { push_fail(state) };
    unsafe { push_literal(state, ERR_CANNOT_CLOSE_STANDARD_FILE) };
    2
}

unsafe fn createstdfile(
    state: *mut lua_State,
    stream: Box<RustStream>,
    k: Option<&'static [u8]>,
    fname: &'static [u8],
) {
    let p = unsafe { newprefile(state) };
    unsafe { set_stream(p, stream) };
    unsafe { (*p).closef = Some(io_noclose) };
    if let Some(key) = k {
        unsafe { lua_pushvalue(state, -1) };
        unsafe { lua_setfield(state, LUA_REGISTRYINDEX, key.as_ptr().cast()) };
    }
    unsafe { lua_setfield(state, -2, fname.as_ptr().cast()) };
}

pub(crate) unsafe fn luaopen_io(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &IOLIB) };
    unsafe { createmeta(state) };
    unsafe {
        createstdfile(
            state,
            Box::new(RustStream::Stdio(StdioKind::Stdin)),
            Some(IO_INPUT),
            NAME_STDIN,
        );
        createstdfile(
            state,
            Box::new(RustStream::Stdio(StdioKind::Stdout)),
            Some(IO_OUTPUT),
            NAME_STDOUT,
        );
        createstdfile(
            state,
            Box::new(RustStream::Stdio(StdioKind::Stderr)),
            None,
            NAME_STDERR,
        );
    }
    1
}

// ─── LuaModule 实现 ────────────────────────────────────────────────────────

/// `io` 标准库的模块标记类型。
pub struct IoModule;

impl crate::module::LuaModule for IoModule {
    const NAME: &'static str = "io";
    const C_NAME: &'static core::ffi::CStr = c"io";

    unsafe fn open(state: *mut lua_State) -> c_int {
        unsafe { luaopen_io(state) }
    }

    fn functions() -> &'static [crate::lua_module::luaL_Reg] {
        &IOLIB
    }
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
