use lua_rs::aux_rs::{
    luaL_callmeta, luaL_checkstack, luaL_checkversion_, luaL_len, luaL_loadbufferx, luaL_loadfilex,
    luaL_newstate, luaL_tolstring, luaL_traceback,
};
use lua_rs::init::luaL_openselectedlibs;
use lua_rs::luaffi::*;
use lua_rs::{link_anchor, lua_module::lua_State};
use std::env;
use std::ffi::{CStr, CString};
use std::io::{self, IsTerminal, Write};
use std::ptr;

const LUA_COPYRIGHT: &str = "Lua 5.5.0  Copyright (C) 1994-2025 Lua.org, PUC-Rio";
const LUA_INIT_VAR: &str = "LUA_INIT";
const LUA_INIT_VAR_VERSION: &str = "LUA_INIT_5_5";
const LUA_PROMPT: &str = "> ";
const LUA_PROMPT2: &str = ">> ";
const EOFMARK: &str = "<eof>";
const NON_STRING_ERROR: &[u8] = b"(error object is not a string value)\0";

const HAS_ERROR: i32 = 1;
const HAS_I: i32 = 2;
const HAS_V: i32 = 4;
const HAS_E: i32 = 8;
const HAS_E_CAP: i32 = 16;

fn main() -> std::process::ExitCode {
    let _ = link_anchor as fn();
    let args = env::args().collect::<Vec<_>>();
    let state = luaL_newstate();
    if state.is_null() {
        eprintln!(
            "{}: cannot create state: not enough memory",
            args.first().map(String::as_str).unwrap_or("lua")
        );
        return std::process::ExitCode::FAILURE;
    }

    let mut runtime = LuaRuntime::new(state, args);
    let success = runtime.run();
    unsafe { lua_close(state) };
    if success {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

struct LuaRuntime {
    state: *mut lua_State,
    args: Vec<String>,
    progname: String,
}

impl LuaRuntime {
    fn new(state: *mut lua_State, args: Vec<String>) -> Self {
        let progname = args
            .first()
            .cloned()
            .filter(|it| !it.is_empty())
            .unwrap_or_else(|| "lua".to_string());
        Self {
            state,
            args,
            progname,
        }
    }

    fn run(&mut self) -> bool {
        let (arg_mask, script) = self.collect_args();
        luaL_checkversion_(self.state, LUA_VERSION_NUM, LUAL_NUMSIZES);
        if arg_mask == HAS_ERROR {
            let bad = self
                .args
                .get(script.max(0) as usize)
                .map(String::as_str)
                .unwrap_or("");
            self.print_usage(bad);
            return false;
        }
        if arg_mask & HAS_V != 0 {
            println!("{LUA_COPYRIGHT}");
        }
        if arg_mask & HAS_E_CAP != 0 {
            unsafe {
                lua_pushboolean(self.state, 1);
                lua_setfield(self.state, LUA_REGISTRYINDEX, cstr("LUA_NOENV").as_ptr());
            }
        }
        unsafe {
            lua_gc(self.state, LUA_GCSTOP);
            lua_gc(self.state, LUA_GCRESTART);
            lua_gc(self.state, LUA_GCGEN);
        }
        self.open_libs();
        self.create_arg_table(script);
        if arg_mask & HAS_E_CAP == 0 && self.handle_lua_init() != LUA_OK {
            return false;
        }
        let optlim = if script > 0 {
            script as usize
        } else {
            self.args.len()
        };
        if !self.run_args(optlim) {
            return false;
        }
        if script > 0 && self.handle_script(script as usize) != LUA_OK {
            return false;
        }
        if arg_mask & HAS_I != 0 {
            self.repl();
        } else if script < 1 && (arg_mask & (HAS_E | HAS_V)) == 0 {
            if io::stdin().is_terminal() {
                println!("{LUA_COPYRIGHT}");
                self.repl();
            } else if self.do_file(None) != LUA_OK {
                return false;
            }
        }
        true
    }

    fn open_libs(&mut self) {
        luaL_openselectedlibs(self.state, !0, 0);
    }

    fn collect_args(&mut self) -> (i32, i32) {
        let mut args = 0;
        if self.args.is_empty() {
            return (0, -1);
        }
        if !self.args[0].is_empty() {
            self.progname = self.args[0].clone();
        }
        let mut i = 1;
        while i < self.args.len() {
            let arg = self.args[i].as_bytes();
            if arg.first() != Some(&b'-') {
                return (args, i as i32);
            }
            match arg.get(1).copied().unwrap_or_default() {
                b'-' => {
                    if arg.len() != 2 {
                        return (HAS_ERROR, i as i32);
                    }
                    return (
                        args,
                        if i + 1 < self.args.len() {
                            (i + 1) as i32
                        } else {
                            0
                        },
                    );
                }
                0 => return (args, i as i32),
                b'E' => {
                    if arg.len() != 2 {
                        return (HAS_ERROR, i as i32);
                    }
                    args |= HAS_E_CAP;
                }
                b'W' => {
                    if arg.len() != 2 {
                        return (HAS_ERROR, i as i32);
                    }
                }
                b'i' => {
                    if arg.len() != 2 {
                        return (HAS_ERROR, i as i32);
                    }
                    args |= HAS_I | HAS_V;
                }
                b'v' => {
                    if arg.len() != 2 {
                        return (HAS_ERROR, i as i32);
                    }
                    args |= HAS_V;
                }
                b'e' | b'l' => {
                    args |= if arg[1] == b'e' { HAS_E } else { 0 };
                    if arg.len() == 2 {
                        if i + 1 >= self.args.len() || self.args[i + 1].starts_with('-') {
                            return (HAS_ERROR, i as i32);
                        }
                        i += 1;
                    }
                }
                _ => return (HAS_ERROR, i as i32),
            }
            i += 1;
        }
        (args, 0)
    }

    fn create_arg_table(&mut self, script: i32) {
        let narg = self.args.len() as i32 - (script + 1);
        unsafe {
            lua_createtable(self.state, narg, script + 1);
            for (i, arg) in self.args.iter().enumerate() {
                let value = cstr(arg);
                lua_pushstring(self.state, value.as_ptr());
                lua_rawseti(self.state, -2, i as i64 - script as i64);
            }
            lua_setglobal(self.state, cstr("arg").as_ptr());
        }
    }

    fn report(&mut self, status: i32) -> i32 {
        if status != LUA_OK {
            let msg = unsafe { lua_to_string(self.state, -1) }
                .unwrap_or_else(|| "(error message not a string)".to_string());
            self.message(Some(&self.progname), &msg);
            unsafe { lua_pop(self.state, 1) };
        }
        status
    }

    fn message(&self, pname: Option<&str>, msg: &str) {
        if let Some(pname) = pname {
            eprint!("{pname}: ");
        }
        eprintln!("{msg}");
    }

    fn print_usage(&self, badoption: &str) {
        eprint!("{}: ", self.progname);
        let bytes = badoption.as_bytes();
        if bytes.get(1) == Some(&b'e') || bytes.get(1) == Some(&b'l') {
            eprintln!("'{}' needs argument", badoption);
        } else {
            eprintln!("unrecognized option '{}'", badoption);
        }
        eprintln!("usage: {} [options] [script [args]]", self.progname);
        eprintln!("Available options are:");
        eprintln!("  -e stat   execute string 'stat'");
        eprintln!("  -i        enter interactive mode after executing 'script'");
        eprintln!("  -l mod    require library 'mod' into global 'mod'");
        eprintln!("  -l g=mod  require library 'mod' into global 'g'");
        eprintln!("  -v        show version information");
        eprintln!("  -E        ignore environment variables");
        eprintln!("  -W        turn warnings on");
        eprintln!("  --        stop handling options");
        eprintln!("  -         stop handling options and execute stdin");
    }

    fn do_call(&mut self, nargs: i32, nresults: i32) -> i32 {
        unsafe {
            let base = lua_gettop(self.state) - nargs;
            lua_pushcfunction(self.state, Some(msghandler));
            lua_insert(self.state, base);
            let status = lua_pcall(self.state, nargs, nresults, base);
            lua_remove(self.state, base);
            status
        }
    }

    fn do_chunk(&mut self, status: i32) -> i32 {
        let status = if status == LUA_OK {
            self.do_call(0, 0)
        } else {
            status
        };
        self.report(status)
    }

    fn do_file(&mut self, name: Option<&str>) -> i32 {
        let filename = name.map(cstr);
        let ptr = filename.as_ref().map_or(ptr::null(), |it| it.as_ptr());
        self.do_chunk(luaL_loadfilex(self.state, ptr, ptr::null()))
    }

    fn do_string(&mut self, source: &str, name: &str) -> i32 {
        let source = source.as_bytes();
        let name = cstr(name);
        self.do_chunk(luaL_loadbufferx(
            self.state,
            source.as_ptr().cast(),
            source.len(),
            name.as_ptr(),
            ptr::null(),
        ))
    }

    fn do_library(&mut self, spec: &str) -> i32 {
        let (global_name, module_name) = if let Some((global, module)) = spec.split_once('=') {
            (global.to_string(), module.to_string())
        } else if let Some((module, _)) = spec.split_once('-') {
            (module.to_string(), spec.to_string())
        } else {
            (spec.to_string(), spec.to_string())
        };
        unsafe {
            lua_getglobal(self.state, cstr("require").as_ptr());
            let module_name = cstr(&module_name);
            lua_pushstring(self.state, module_name.as_ptr());
        }
        let status = self.do_call(1, 1);
        if status == LUA_OK {
            unsafe { lua_setglobal(self.state, cstr(&global_name).as_ptr()) };
        }
        self.report(status)
    }

    fn run_args(&mut self, n: usize) -> bool {
        unsafe { lua_warning(self.state, cstr("@off").as_ptr(), 0) };
        let mut i = 1;
        while i < n {
            let arg = self.args[i].clone();
            if !arg.starts_with('-') {
                i += 1;
                continue;
            }
            match arg.as_bytes()[1] {
                b'e' | b'l' => {
                    let extra = if arg.len() > 2 {
                        arg[2..].to_string()
                    } else {
                        i += 1;
                        self.args[i].clone()
                    };
                    let status = if arg.as_bytes()[1] == b'e' {
                        self.do_string(&extra, "=(command line)")
                    } else {
                        self.do_library(&extra)
                    };
                    if status != LUA_OK {
                        return false;
                    }
                }
                b'W' => unsafe { lua_warning(self.state, cstr("@on").as_ptr(), 0) },
                _ => {}
            }
            i += 1;
        }
        true
    }

    fn handle_lua_init(&mut self) -> i32 {
        if let Some(init) = env::var_os(LUA_INIT_VAR_VERSION).or_else(|| env::var_os(LUA_INIT_VAR))
        {
            let init = init.to_string_lossy().into_owned();
            if let Some(rest) = init.strip_prefix('@') {
                self.do_file(Some(rest))
            } else if env::var_os(LUA_INIT_VAR_VERSION).is_some() {
                self.do_string(&init, &format!("={LUA_INIT_VAR_VERSION}"))
            } else {
                self.do_string(&init, &format!("={LUA_INIT_VAR}"))
            }
        } else {
            LUA_OK
        }
    }

    fn push_args(&mut self) -> i32 {
        unsafe {
            if lua_getglobal(self.state, cstr("arg").as_ptr()) != LUA_TTABLE {
                self.message(Some(&self.progname), "'arg' is not a table");
                return 0;
            }
            let n = luaL_len(self.state, -1) as i32;
            luaL_checkstack(
                self.state,
                n + 3,
                cstr("too many arguments to script").as_ptr(),
            );
            for i in 1..=n {
                lua_rawgeti(self.state, -i, i as i64);
            }
            lua_remove(self.state, -n - 1);
            n
        }
    }

    fn handle_script(&mut self, script: usize) -> i32 {
        let mut fname = Some(self.args[script].as_str());
        if self.args[script] == "-" && script > 0 && self.args[script - 1] != "--" {
            fname = None;
        }
        let filename = fname.map(cstr);
        let status = luaL_loadfilex(
            self.state,
            filename.as_ref().map_or(ptr::null(), |it| it.as_ptr()),
            ptr::null(),
        );
        let status = if status == LUA_OK {
            let nargs = self.push_args();
            self.do_call(nargs, LUA_MULTRET)
        } else {
            status
        };
        self.report(status)
    }

    fn get_prompt(&mut self, first_line: bool) -> String {
        let key = if first_line { "_PROMPT" } else { "_PROMPT2" };
        let prompt = unsafe {
            if lua_getglobal(self.state, cstr(key).as_ptr()) == LUA_TNIL {
                if first_line { LUA_PROMPT } else { LUA_PROMPT2 }.to_string()
            } else {
                let value = CStr::from_ptr(luaL_tolstring(self.state, -1, ptr::null_mut()))
                    .to_string_lossy()
                    .into_owned();
                lua_remove(self.state, -2);
                value
            }
        };
        unsafe { lua_pop(self.state, 1) };
        prompt
    }

    fn push_line(&mut self, first_line: bool) -> bool {
        let prompt = self.get_prompt(first_line);
        print!("{prompt}");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if io::stdin()
            .read_line(&mut line)
            .ok()
            .filter(|it| *it > 0)
            .is_none()
        {
            return false;
        }
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        unsafe {
            lua_pushlstring(self.state, line.as_ptr().cast(), line.len());
        }
        true
    }

    fn incomplete(&mut self, status: i32) -> bool {
        if status != LUA_ERRSYNTAX {
            return false;
        }
        let mut len = 0;
        let msg = unsafe { lua_tolstring(self.state, -1, &mut len) };
        if msg.is_null() {
            return false;
        }
        let bytes = unsafe { std::slice::from_raw_parts(msg.cast::<u8>(), len) };
        bytes.ends_with(EOFMARK.as_bytes())
    }

    fn add_return(&mut self) -> i32 {
        let line = unsafe { lua_to_string(self.state, -1) }.unwrap_or_default();
        let source = format!("return {line};");
        let status = luaL_loadbufferx(
            self.state,
            source.as_ptr().cast(),
            source.len(),
            cstr("=stdin").as_ptr(),
            ptr::null(),
        );
        unsafe {
            if status == LUA_OK {
                lua_remove(self.state, -2);
            } else {
                lua_pop(self.state, 2);
            }
        }
        status
    }

    fn multiline(&mut self) -> i32 {
        loop {
            let mut len = 0;
            let line_ptr = unsafe { lua_tolstring(self.state, 1, &mut len) };
            let status = luaL_loadbufferx(
                self.state,
                line_ptr,
                len,
                cstr("=stdin").as_ptr(),
                ptr::null(),
            );
            if !self.incomplete(status) || !self.push_line(false) {
                return status;
            }
            unsafe {
                lua_remove(self.state, -2);
                lua_pushlstring(self.state, b"\n".as_ptr().cast(), 1);
                lua_insert(self.state, -2);
                lua_concat(self.state, 3);
            }
        }
    }

    fn load_line(&mut self) -> i32 {
        unsafe { lua_settop(self.state, 0) };
        if !self.push_line(true) {
            return -1;
        }
        let status = if self.add_return() != LUA_OK {
            self.multiline()
        } else {
            LUA_OK
        };
        let line = unsafe { lua_to_string(self.state, 1) }.unwrap_or_default();
        unsafe { lua_remove(self.state, 1) };
        if !line.is_empty() {
            let _ = line;
        }
        status
    }

    fn l_print(&mut self) {
        let n = unsafe { lua_gettop(self.state) };
        if n <= 0 {
            return;
        }
        unsafe {
            luaL_checkstack(
                self.state,
                LUA_MINSTACK,
                cstr("too many results to print").as_ptr(),
            );
            lua_getglobal(self.state, cstr("print").as_ptr());
            lua_insert(self.state, 1);
            if lua_pcall(self.state, n, 0, 0) != LUA_OK {
                let msg = lua_to_string(self.state, -1)
                    .unwrap_or_else(|| "error calling 'print'".to_string());
                self.message(Some(&self.progname), &msg);
                lua_pop(self.state, 1);
            }
        }
    }

    fn repl(&mut self) {
        let old_progname = self.progname.clone();
        self.progname.clear();
        loop {
            let status = self.load_line();
            if status == -1 {
                break;
            }
            let status = if status == LUA_OK {
                self.do_call(0, LUA_MULTRET)
            } else {
                status
            };
            if status == LUA_OK {
                self.l_print();
            } else {
                self.report(status);
            }
        }
        unsafe { lua_settop(self.state, 0) };
        println!();
        self.progname = old_progname;
    }
}

unsafe extern "C-unwind" fn msghandler(state: *mut lua_State) -> i32 {
    let mut msg = unsafe { lua_tolstring(state, 1, ptr::null_mut()) };
    if msg.is_null() {
        let event = cstr("__tostring");
        if luaL_callmeta(state, 1, event.as_ptr()) != 0
            && unsafe { lua_type(state, -1) } == LUA_TSTRING
        {
            return 1;
        }
        msg = NON_STRING_ERROR.as_ptr().cast();
    }
    luaL_traceback(state, state, msg, 1);
    1
}

unsafe fn lua_to_string(state: *mut lua_State, index: i32) -> Option<String> {
    let ptr = unsafe { lua_tolstring(state, index, ptr::null_mut()) };
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

fn cstr(value: &str) -> CString {
    CString::new(value).expect("string contains interior NUL")
}
