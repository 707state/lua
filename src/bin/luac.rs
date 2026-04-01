use lua_rs::aux_rs::{luaL_checkversion_, luaL_loadbufferx, luaL_loadfilex, luaL_newstate};
use lua_rs::luaffi::*;
use lua_rs::{link_anchor, lua_module::lua_State, luavm};
use std::env;
use std::ffi::{CStr, CString, c_void};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::ptr;

const LUA_COPYRIGHT: &str = "Lua 5.5.0  Copyright (C) 1994-2025 Lua.org, PUC-Rio";
const PROGNAME: &str = "luac";
const OUTPUT: &str = "luac.out";

fn main() -> std::process::ExitCode {
    let _ = link_anchor as fn();
    match Luac::new(env::args().collect()).and_then(|mut luac| luac.run()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            std::process::ExitCode::FAILURE
        }
    }
}

struct Luac {
    args: Vec<String>,
    progname: String,
    listing: usize,
    dumping: bool,
    stripping: bool,
    output: Option<String>,
}

impl Luac {
    fn new(args: Vec<String>) -> Result<Self, String> {
        let progname = args
            .first()
            .cloned()
            .unwrap_or_else(|| PROGNAME.to_string());
        Ok(Self {
            args,
            progname,
            listing: 0,
            dumping: true,
            stripping: false,
            output: Some(OUTPUT.to_string()),
        })
    }

    fn run(&mut self) -> Result<(), String> {
        let first = self.parse_args()?;
        let files = self.args[first..].to_vec();
        if files.is_empty() {
            return Err(self.usage("no input files given"));
        }
        let state = luaL_newstate();
        if state.is_null() {
            return Err(format!(
                "{}: cannot create state: not enough memory",
                self.progname
            ));
        }
        let result = self.compile(state, &files);
        unsafe { lua_close(state) };
        result
    }

    fn parse_args(&mut self) -> Result<usize, String> {
        let mut i = 1;
        let mut version = 0usize;
        while i < self.args.len() {
            let arg = self.args[i].clone();
            if !arg.starts_with('-') {
                break;
            }
            match arg.as_str() {
                "--" => {
                    i += 1;
                    if version > 0 {
                        version += 1;
                    }
                    break;
                }
                "-" => break,
                "-l" => self.listing += 1,
                "-o" => {
                    i += 1;
                    let output = self
                        .args
                        .get(i)
                        .cloned()
                        .ok_or_else(|| self.usage("'-o' needs argument"))?;
                    if output.is_empty() || (output.starts_with('-') && output != "-") {
                        return Err(self.usage("'-o' needs argument"));
                    }
                    self.output = if output == "-" { None } else { Some(output) };
                }
                "-p" => self.dumping = false,
                "-s" => self.stripping = true,
                "-v" => version += 1,
                _ => return Err(self.usage(&arg)),
            }
            i += 1;
        }
        if i == self.args.len() && (self.listing > 0 || !self.dumping) {
            self.dumping = false;
            self.args.push(OUTPUT.to_string());
        }
        if version > 0 {
            println!("{LUA_COPYRIGHT}");
            if version == self.args.len().saturating_sub(1) {
                return Ok(self.args.len());
            }
        }
        Ok(i)
    }

    fn usage(&self, message: &str) -> String {
        let mut out = String::new();
        if message.starts_with('-') {
            out.push_str(&format!(
                "{}: unrecognized option '{}'\n",
                self.progname, message
            ));
        } else {
            out.push_str(&format!("{}: {}\n", self.progname, message));
        }
        out.push_str(&format!(
            "usage: {} [options] [filenames]\n\
Available options are:\n\
  -l       list (use -l -l for full listing)\n\
  -o name  output to file 'name' (default is \"{}\")\n\
  -p       parse only\n\
  -s       strip debug information\n\
  -v       show version information\n\
  --       stop handling options\n\
  -        stop handling options and process stdin",
            self.progname, OUTPUT
        ));
        out
    }

    fn compile(&self, state: *mut lua_State, files: &[String]) -> Result<(), String> {
        luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
        if files.len() == 1 {
            let filename = if files[0] == "-" {
                None
            } else {
                Some(files[0].as_str())
            };
            let name = filename.map(cstr);
            let status = luaL_loadfilex(
                state,
                name.as_ref().map_or(ptr::null(), |it| it.as_ptr()),
                ptr::null(),
            );
            if status != LUA_OK {
                return Err(format!(
                    "{}: {}",
                    self.progname,
                    unsafe { lua_to_string(state, -1) }
                        .unwrap_or_else(|| "compile failed".to_string())
                ));
            }
        } else {
            let source = self.build_combined_source(files)?;
            let chunk_name = cstr("=(luac)");
            let status = luaL_loadbufferx(
                state,
                source.as_ptr().cast(),
                source.len(),
                chunk_name.as_ptr(),
                ptr::null(),
            );
            if status != LUA_OK {
                return Err(format!(
                    "{}: {}",
                    self.progname,
                    unsafe { lua_to_string(state, -1) }
                        .unwrap_or_else(|| "compile failed".to_string())
                ));
            }
        }
        if self.listing > 0 {
            unsafe { luavm::print_listing(state, self.listing > 1) };
        }
        if self.dumping {
            self.dump_chunk(state)?;
        }
        Ok(())
    }

    fn build_combined_source(&self, files: &[String]) -> Result<Vec<u8>, String> {
        let mut source = Vec::new();
        for file in files {
            let content = self.read_source(file)?;
            source.extend_from_slice(b"(function(...)\n");
            source.extend_from_slice(&content);
            source.extend_from_slice(b"\nend)(...);\n");
        }
        Ok(source)
    }

    fn read_source(&self, file: &str) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        if file == "-" {
            io::stdin()
                .read_to_end(&mut bytes)
                .map_err(|err| format!("{}: cannot read stdin: {}", self.progname, err))?;
        } else {
            File::open(PathBuf::from(file))
                .and_then(|mut handle| handle.read_to_end(&mut bytes))
                .map_err(|err| format!("{}: cannot read {}: {}", self.progname, file, err))?;
        }
        if bytes.starts_with(b"#") {
            if let Some(pos) = bytes.iter().position(|&byte| byte == b'\n') {
                bytes.drain(..=pos);
            } else {
                bytes.clear();
            }
        }
        Ok(bytes)
    }

    fn dump_chunk(&self, state: *mut lua_State) -> Result<(), String> {
        let target = match &self.output {
            Some(path) => DumpTarget::File(
                File::create(path)
                    .map_err(|err| format!("{}: cannot open {}: {}", self.progname, path, err))?,
            ),
            None => DumpTarget::Stdout(io::stdout()),
        };
        let mut writer = DumpWriter {
            target,
            error: None,
        };
        let status = unsafe {
            lua_dump(
                state,
                Some(write_chunk),
                (&mut writer as *mut DumpWriter).cast::<c_void>(),
                if self.stripping { 1 } else { 0 },
            )
        };
        if status != LUA_OK {
            return Err(format!("{}: unable to dump bytecode", self.progname));
        }
        if let Some(error) = writer.error {
            return Err(error);
        }
        Ok(())
    }
}

enum DumpTarget {
    File(File),
    Stdout(io::Stdout),
}

struct DumpWriter {
    target: DumpTarget,
    error: Option<String>,
}

unsafe extern "C-unwind" fn write_chunk(
    _state: *mut lua_State,
    pointer: *const c_void,
    size: usize,
    data: *mut c_void,
) -> i32 {
    let writer = unsafe { &mut *(data as *mut DumpWriter) };
    let bytes = if size == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), size) }
    };
    let result = match &mut writer.target {
        DumpTarget::File(file) => file.write_all(bytes),
        DumpTarget::Stdout(stdout) => stdout.write_all(bytes),
    };
    if let Err(error) = result {
        writer.error = Some(format!("luac: write failed: {error}"));
        1
    } else {
        0
    }
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
