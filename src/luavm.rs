use crate::lua_module::lua_State;
use std::ffi::{c_char, c_int};
use std::fmt::Write as _;

pub type Instruction = u32;

#[repr(C)]
struct GCObject {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
}

#[repr(C)]
union Value {
    gc: *mut GCObject,
    p: *mut core::ffi::c_void,
    f: *mut core::ffi::c_void,
    i: i64,
    n: f64,
    ub: u8,
}

#[repr(C)]
struct TValue {
    value_: Value,
    tt_: u8,
}

#[repr(C)]
union TStringUnion {
    lnglen: usize,
    hnext: *mut TString,
}

#[repr(C)]
pub struct TString {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    extra: u8,
    shrlen: i8,
    hash: u32,
    u: TStringUnion,
    contents: *mut c_char,
    falloc: *mut core::ffi::c_void,
    ud: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct Upvaldesc {
    name: *const TString,
    instack: u8,
    idx: u8,
    kind: u8,
}

#[repr(C)]
pub struct LocVar {
    varname: *const TString,
    startpc: c_int,
    endpc: c_int,
}

#[repr(C)]
pub struct AbsLineInfo {
    pc: c_int,
    line: c_int,
}

#[repr(C)]
pub struct Proto {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    numparams: u8,
    flag: u8,
    maxstacksize: u8,
    sizeupvalues: c_int,
    sizek: c_int,
    sizecode: c_int,
    sizelineinfo: c_int,
    sizep: c_int,
    sizelocvars: c_int,
    sizeabslineinfo: c_int,
    linedefined: c_int,
    lastlinedefined: c_int,
    k: *const TValue,
    code: *const Instruction,
    p: *const *const Proto,
    upvalues: *const Upvaldesc,
    lineinfo: *const i8,
    abslineinfo: *const AbsLineInfo,
    locvars: *const LocVar,
    source: *const TString,
    gclist: *mut GCObject,
}

const LUA_VNIL: u8 = 0;
const LUA_VFALSE: u8 = 1;
const LUA_VTRUE: u8 = 17;
const LUA_VNUMINT: u8 = 3;
const LUA_VNUMFLT: u8 = 19;
const LUA_VSHRSTR: u8 = 68;
const LUA_VLNGSTR: u8 = 84;
const PF_VAHID: u8 = 1;
const PF_VATAB: u8 = 2;

const SIZE_C: u32 = 8;
const SIZE_VC: u32 = 10;
const SIZE_B: u32 = 8;
const SIZE_VB: u32 = 6;
const SIZE_BX: u32 = SIZE_C + SIZE_B + 1;
const SIZE_A: u32 = 8;
const SIZE_AX: u32 = SIZE_BX + SIZE_A;
const SIZE_SJ: u32 = SIZE_BX + SIZE_A;

const POS_OP: u32 = 0;
const POS_A: u32 = POS_OP + 7;
const POS_K: u32 = POS_A + SIZE_A;
const POS_B: u32 = POS_K + 1;
const POS_VB: u32 = POS_K + 1;
const POS_C: u32 = POS_B + SIZE_B;
const POS_VC: u32 = POS_VB + SIZE_VB;
const POS_BX: u32 = POS_K;
const POS_AX: u32 = POS_A;
const POS_SJ: u32 = POS_A;

const MAXARG_BX: i32 = (1 << SIZE_BX) - 1;
const OFFSET_SBX: i32 = MAXARG_BX >> 1;
const MAXARG_SJ: i32 = (1 << SIZE_SJ) - 1;
const OFFSET_SJ: i32 = MAXARG_SJ >> 1;
const MAXARG_C: i32 = (1 << SIZE_C) - 1;
const OFFSET_SC: i32 = MAXARG_C >> 1;
const COMMENT: &str = "\t; ";

const OPNAMES: &[&str] = &[
    "MOVE",
    "LOADI",
    "LOADF",
    "LOADK",
    "LOADKX",
    "LOADFALSE",
    "LFALSESKIP",
    "LOADTRUE",
    "LOADNIL",
    "GETUPVAL",
    "SETUPVAL",
    "GETTABUP",
    "GETTABLE",
    "GETI",
    "GETFIELD",
    "SETTABUP",
    "SETTABLE",
    "SETI",
    "SETFIELD",
    "NEWTABLE",
    "SELF",
    "ADDI",
    "ADDK",
    "SUBK",
    "MULK",
    "MODK",
    "POWK",
    "DIVK",
    "IDIVK",
    "BANDK",
    "BORK",
    "BXORK",
    "SHLI",
    "SHRI",
    "ADD",
    "SUB",
    "MUL",
    "MOD",
    "POW",
    "DIV",
    "IDIV",
    "BAND",
    "BOR",
    "BXOR",
    "SHL",
    "SHR",
    "MMBIN",
    "MMBINI",
    "MMBINK",
    "UNM",
    "BNOT",
    "NOT",
    "LEN",
    "CONCAT",
    "CLOSE",
    "TBC",
    "JMP",
    "EQ",
    "LT",
    "LE",
    "EQK",
    "EQI",
    "LTI",
    "LEI",
    "GTI",
    "GEI",
    "TEST",
    "TESTSET",
    "CALL",
    "TAILCALL",
    "RETURN",
    "RETURN0",
    "RETURN1",
    "FORLOOP",
    "FORPREP",
    "TFORPREP",
    "TFORCALL",
    "TFORLOOP",
    "SETLIST",
    "CLOSURE",
    "VARARG",
    "GETVARG",
    "ERRNNIL",
    "VARARGPREP",
    "EXTRAARG",
];

pub unsafe fn print_listing(state: *mut lua_State, full: bool) {
    let proto = unsafe { rust_luavm_top_proto(state) };
    unsafe { print_function(state, proto, full) };
}

unsafe fn print_function(state: *mut lua_State, proto: *const Proto, full: bool) {
    unsafe { print_header(proto) };
    unsafe { print_code(state, proto) };
    if full {
        unsafe { print_debug(proto) };
    }
    let nested = unsafe { (*proto).sizep };
    for i in 0..nested {
        let child = unsafe { *(*proto).p.add(i as usize) };
        unsafe { print_function(state, child, full) };
    }
}

unsafe fn print_header(proto: *const Proto) {
    let source = proto_source(proto);
    let mut source_text = source.as_deref().unwrap_or("=?");
    if let Some(stripped) = source_text
        .strip_prefix('@')
        .or_else(|| source_text.strip_prefix('='))
    {
        source_text = stripped;
    } else if source_text.as_bytes().first() == Some(&0x1b) {
        source_text = "(bstring)";
    } else {
        source_text = "(string)";
    }
    let kind = if unsafe { (*proto).linedefined } == 0 {
        "main"
    } else {
        "function"
    };
    println!(
        "\n{} <{}:{},{}> ({} instruction{} at {:p})",
        kind,
        source_text,
        unsafe { (*proto).linedefined },
        unsafe { (*proto).lastlinedefined },
        unsafe { (*proto).sizecode },
        plural(unsafe { (*proto).sizecode }),
        proto
    );
    let vararg = if is_vararg(unsafe { (*proto).flag }) {
        "+"
    } else {
        ""
    };
    println!(
        "{}{} param{}, {} slot{}, {} upvalue{}, {} local{}, {} constant{}, {} function{}",
        unsafe { (*proto).numparams },
        vararg,
        plural(unsafe { (*proto).numparams.into() }),
        unsafe { (*proto).maxstacksize },
        plural(unsafe { (*proto).maxstacksize.into() }),
        unsafe { (*proto).sizeupvalues },
        plural(unsafe { (*proto).sizeupvalues }),
        unsafe { (*proto).sizelocvars },
        plural(unsafe { (*proto).sizelocvars }),
        unsafe { (*proto).sizek },
        plural(unsafe { (*proto).sizek }),
        unsafe { (*proto).sizep },
        plural(unsafe { (*proto).sizep }),
    );
}

unsafe fn print_debug(proto: *const Proto) {
    let constants = unsafe { (*proto).sizek };
    println!("constants ({}) for {:p}:", constants, proto);
    for i in 0..constants {
        print!("\t{}\t", i);
        print_type(proto, i);
        print_constant(proto, i);
        println!();
    }
    let locals = unsafe { (*proto).sizelocvars };
    println!("locals ({}) for {:p}:", locals, proto);
    for i in 0..locals {
        let local = unsafe { &*(*proto).locvars.add(i as usize) };
        println!(
            "\t{}\t{}\t{}\t{}",
            i,
            tstring_to_string(local.varname),
            local.startpc + 1,
            local.endpc + 1
        );
    }
    let upvalues = unsafe { (*proto).sizeupvalues };
    println!("upvalues ({}) for {:p}:", upvalues, proto);
    for i in 0..upvalues {
        let upvalue = unsafe { &*(*proto).upvalues.add(i as usize) };
        println!(
            "\t{}\t{}\t{}\t{}",
            i,
            upval_name(proto, i),
            upvalue.instack,
            upvalue.idx
        );
    }
}

unsafe fn print_code(state: *mut lua_State, proto: *const Proto) {
    let sizecode = unsafe { (*proto).sizecode };
    for pc in 0..sizecode {
        let instruction = unsafe { *(*proto).code.add(pc as usize) };
        let opcode = get_opcode(instruction);
        let a = getarg_a(instruction);
        let b = getarg_b(instruction);
        let c = getarg_c(instruction);
        let ax = getarg_ax(instruction);
        let bx = getarg_bx(instruction);
        let sb = getarg_sb(instruction);
        let sc = getarg_sc(instruction);
        let vb = getarg_vb(instruction);
        let vc = getarg_vc(instruction);
        let sbx = getarg_sbx(instruction);
        let isk = getarg_k(instruction);
        let line = unsafe { rust_luavm_getfuncline(proto, pc) };
        print!("\t{}\t", pc + 1);
        if line > 0 {
            print!("[{}]\t", line);
        } else {
            print!("[-]\t");
        }
        print!("{:<9}\t", OPNAMES[opcode as usize]);
        match opcode {
            0 => print!("{a} {b}"),
            1 | 2 => print!("{a} {sbx}"),
            3 => {
                print!("{a} {bx}");
                print!("{COMMENT}");
                print_constant(proto, bx);
            }
            4 => {
                print!("{a}");
                print!("{COMMENT}");
                print_constant(proto, extraarg(proto, pc));
            }
            5..=7 => print!("{a}"),
            8 => {
                print!("{a} {b}");
                print!("{COMMENT}{} out", b + 1);
            }
            9 | 10 => {
                print!("{a} {b}");
                print!("{COMMENT}{}", upval_name(proto, b));
            }
            11 => {
                print!("{a} {b} {c}");
                print!("{COMMENT}{}", upval_name(proto, b));
                print!(" ");
                print_constant(proto, c);
            }
            12 | 13 => print!("{a} {b} {c}"),
            14 => {
                print!("{a} {b} {c}");
                print!("{COMMENT}");
                print_constant(proto, c);
            }
            15 => {
                print!("{a} {b} {c}{}", if isk != 0 { "k" } else { "" });
                print!("{COMMENT}{}", upval_name(proto, a));
                print!(" ");
                print_constant(proto, b);
                if isk != 0 {
                    print!(" ");
                    print_constant(proto, c);
                }
            }
            16 | 17 => {
                print!("{a} {b} {c}{}", if isk != 0 { "k" } else { "" });
                if isk != 0 {
                    print!("{COMMENT}");
                    print_constant(proto, c);
                }
            }
            18 => {
                print!("{a} {b} {c}{}", if isk != 0 { "k" } else { "" });
                print!("{COMMENT}");
                print_constant(proto, b);
                if isk != 0 {
                    print!(" ");
                    print_constant(proto, c);
                }
            }
            19 => {
                print!("{a} {vb} {vc}{}", if isk != 0 { "k" } else { "" });
                print!("{COMMENT}{}", vc + extraargc(proto, pc));
            }
            20 => {
                print!("{a} {b} {c}{}", if isk != 0 { "k" } else { "" });
                if isk != 0 {
                    print!("{COMMENT}");
                    print_constant(proto, c);
                }
            }
            21 | 32 | 33 => print!("{a} {b} {sc}"),
            22..=31 => {
                print!("{a} {b} {c}");
                print!("{COMMENT}");
                print_constant(proto, c);
            }
            34..=45 => print!("{a} {b} {c}"),
            46 => {
                print!("{a} {b} {c}");
                print!("{COMMENT}{}", event_name(state, c));
            }
            47 => {
                print!("{a} {sb} {c} {isk}");
                print!("{COMMENT}{}", event_name(state, c));
                if isk != 0 {
                    print!(" flip");
                }
            }
            48 => {
                print!("{a} {b} {c} {isk}");
                print!("{COMMENT}{} ", event_name(state, c));
                print_constant(proto, b);
                if isk != 0 {
                    print!(" flip");
                }
            }
            49..=53 => print!("{a} {b}"),
            54 | 55 => print!("{a}"),
            56 => {
                let sj = getarg_sj(instruction);
                print!("{sj}");
                print!("{COMMENT}to {}", sj + pc + 2);
            }
            57..=59 => print!("{a} {b} {isk}"),
            60 => {
                print!("{a} {b} {isk}");
                print!("{COMMENT}");
                print_constant(proto, b);
            }
            61..=65 => print!("{a} {sb} {isk}"),
            66 => print!("{a} {isk}"),
            67 => print!("{a} {b} {isk}"),
            68 => {
                print!("{a} {b} {c}");
                print!("{COMMENT}");
                if b == 0 {
                    print!("all in ");
                } else {
                    print!("{} in ", b - 1);
                }
                if c == 0 {
                    print!("all out");
                } else {
                    print!("{} out", c - 1);
                }
            }
            69 => {
                print!("{a} {b} {c}{}", if isk != 0 { "k" } else { "" });
                print!("{COMMENT}{} in", b - 1);
            }
            70 => {
                print!("{a} {b} {c}{}", if isk != 0 { "k" } else { "" });
                print!("{COMMENT}");
                if b == 0 {
                    print!("all out");
                } else {
                    print!("{} out", b - 1);
                }
            }
            71 => {}
            72 => print!("{a}"),
            73 | 77 => {
                print!("{a} {bx}");
                print!("{COMMENT}to {}", pc - bx + 2);
            }
            74 => {
                print!("{a} {bx}");
                print!("{COMMENT}exit to {}", pc + bx + 3);
            }
            75 => {
                print!("{a} {bx}");
                print!("{COMMENT}to {}", pc + bx + 2);
            }
            76 => print!("{a} {c}"),
            78 => {
                print!("{a} {vb} {vc}{}", if isk != 0 { "k" } else { "" });
                if isk != 0 {
                    print!("{COMMENT}{}", c + extraargc(proto, pc));
                }
            }
            79 => {
                print!("{a} {bx}");
                print!("{COMMENT}{:p}", unsafe { *(*proto).p.add(bx as usize) });
            }
            80 => {
                print!("{a} {b} {c}{}", if isk != 0 { "k" } else { "" });
                print!("{COMMENT}");
                if c == 0 {
                    print!("all out");
                } else {
                    print!("{} out", c - 1);
                }
            }
            81 => print!("{a} {b} {c}"),
            82 => {
                print!("{a} {bx}");
                print!("{COMMENT}");
                if bx == 0 {
                    print!("?");
                } else {
                    print_constant(proto, bx - 1);
                }
            }
            83 => print!("{a}"),
            84 => print!("{ax}"),
            _ => {}
        }
        println!();
    }
}

fn print_type(proto: *const Proto, index: i32) {
    let value = unsafe { &*(*proto).k.add(index as usize) };
    let marker = match value.tt_ {
        LUA_VNIL => "N",
        LUA_VFALSE | LUA_VTRUE => "B",
        LUA_VNUMFLT => "F",
        LUA_VNUMINT => "I",
        LUA_VSHRSTR | LUA_VLNGSTR => "S",
        other => {
            print!("?{other}\t");
            return;
        }
    };
    print!("{marker}\t");
}

fn print_constant(proto: *const Proto, index: i32) {
    let value = unsafe { &*(*proto).k.add(index as usize) };
    match value.tt_ {
        LUA_VNIL => print!("nil"),
        LUA_VFALSE => print!("false"),
        LUA_VTRUE => print!("true"),
        LUA_VNUMFLT => {
            let number = unsafe { value.value_.n };
            let mut text = format!("{number}");
            if text
                .bytes()
                .all(|byte| byte == b'-' || byte.is_ascii_digit())
            {
                text.push_str(".0");
            }
            print!("{text}");
        }
        LUA_VNUMINT => print!("{}", unsafe { value.value_.i }),
        LUA_VSHRSTR | LUA_VLNGSTR => {
            let ts = unsafe { value.value_.gc as *const TString };
            print!("{}", quote_lua_string(&tstring_to_string(ts)));
        }
        other => print!("?{other}"),
    }
}

fn proto_source(proto: *const Proto) -> Option<String> {
    let source = unsafe { (*proto).source };
    if source.is_null() {
        None
    } else {
        Some(tstring_to_string(source))
    }
}

fn plural(value: i32) -> &'static str {
    if value == 1 { "" } else { "s" }
}

fn is_vararg(flag: u8) -> bool {
    flag & (PF_VAHID | PF_VATAB) != 0
}

fn quote_lua_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{b}' => out.push_str("\\v"),
            c if c.is_ascii_graphic() || c == ' ' => out.push(c),
            c => {
                let _ = write!(out, "\\{:03}", c as u32);
            }
        }
    }
    out.push('"');
    out
}

pub fn tstring_to_string(ts: *const TString) -> String {
    if ts.is_null() {
        return "-".to_string();
    }
    let (ptr, len) = unsafe {
        // rust-analyzer off
        if (*ts).shrlen >= 0 {
            (
                std::ptr::addr_of!((*ts).contents).cast::<u8>(),
                (*ts).shrlen as usize,
            )
        } else {
            ((*ts).contents.cast_const().cast::<u8>(), (*ts).u.lnglen)
        }
        // rust-analyzer on
    };
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(bytes).into_owned()
}

fn upval_name(proto: *const Proto, index: i32) -> String {
    let upval = unsafe { &*(*proto).upvalues.add(index as usize) };
    tstring_to_string(upval.name)
}

fn event_name(state: *mut lua_State, index: i32) -> String {
    let name = unsafe { rust_luavm_eventname(state, index) };
    tstring_to_string(name)
}

fn getarg(i: Instruction, pos: u32, size: u32) -> i32 {
    ((i >> pos) & ((1u32 << size) - 1)) as i32
}

fn get_opcode(i: Instruction) -> i32 {
    getarg(i, POS_OP, 7)
}
fn getarg_a(i: Instruction) -> i32 {
    getarg(i, POS_A, SIZE_A)
}
fn getarg_b(i: Instruction) -> i32 {
    getarg(i, POS_B, SIZE_B)
}
fn getarg_vb(i: Instruction) -> i32 {
    getarg(i, POS_VB, SIZE_VB)
}
fn getarg_sb(i: Instruction) -> i32 {
    getarg_b(i) - OFFSET_SC
}
fn getarg_c(i: Instruction) -> i32 {
    getarg(i, POS_C, SIZE_C)
}
fn getarg_vc(i: Instruction) -> i32 {
    getarg(i, POS_VC, SIZE_VC)
}
fn getarg_sc(i: Instruction) -> i32 {
    getarg_c(i) - OFFSET_SC
}
fn getarg_k(i: Instruction) -> i32 {
    getarg(i, POS_K, 1)
}
fn getarg_bx(i: Instruction) -> i32 {
    getarg(i, POS_BX, SIZE_BX)
}
fn getarg_ax(i: Instruction) -> i32 {
    getarg(i, POS_AX, SIZE_AX)
}
fn getarg_sbx(i: Instruction) -> i32 {
    getarg_bx(i) - OFFSET_SBX
}
fn getarg_sj(i: Instruction) -> i32 {
    getarg(i, POS_SJ, SIZE_SJ) - OFFSET_SJ
}

fn extraarg(proto: *const Proto, pc: i32) -> i32 {
    unsafe { getarg_ax(*(*proto).code.add((pc + 1) as usize)) }
}

fn extraargc(proto: *const Proto, pc: i32) -> i32 {
    extraarg(proto, pc) * (MAXARG_C + 1)
}

unsafe extern "C" {
    fn rust_luavm_top_proto(state: *mut lua_State) -> *const Proto;
    fn rust_luavm_getfuncline(proto: *const Proto, pc: c_int) -> c_int;
    fn rust_luavm_eventname(state: *mut lua_State, idx: c_int) -> *const TString;
}
