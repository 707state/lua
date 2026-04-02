use crate::luaffi::LuaCFunction;
use crate::runtime::*;
use std::ffi::{c_char, c_int};
use std::fmt::Write as _;
use std::ptr;

#[repr(C)]
pub(crate) struct GlobalState {
    pub(crate) frealloc: lua_Alloc,
    pub(crate) ud: *mut core::ffi::c_void,
    pub(crate) gctotalbytes: isize,
    pub(crate) gcdebt: isize,
    pub(crate) gcmarked: isize,
    pub(crate) gcmajorminor: isize,
    pub(crate) strt: stringtable,
    pub(crate) l_registry: TValue,
    pub(crate) nilvalue: TValue,
    pub(crate) seed: u32,
    pub(crate) gcparams: [u8; LUA_GCPN],
    pub(crate) currentwhite: u8,
    pub(crate) gcstate: u8,
    pub(crate) gckind: u8,
    pub(crate) gcstopem: u8,
    pub(crate) gcstp: u8,
    pub(crate) gcemergency: u8,
    pub(crate) allgc: *mut GCObject,
    pub(crate) sweepgc: *mut *mut GCObject,
    pub(crate) finobj: *mut GCObject,
    pub(crate) gray: *mut GCObject,
    pub(crate) grayagain: *mut GCObject,
    pub(crate) weak: *mut GCObject,
    pub(crate) ephemeron: *mut GCObject,
    pub(crate) allweak: *mut GCObject,
    pub(crate) tobefnz: *mut GCObject,
    pub(crate) fixedgc: *mut GCObject,
    pub(crate) survival: *mut GCObject,
    pub(crate) old1: *mut GCObject,
    pub(crate) reallyold: *mut GCObject,
    pub(crate) firstold1: *mut GCObject,
    pub(crate) finobjsur: *mut GCObject,
    pub(crate) finobjold1: *mut GCObject,
    pub(crate) finobjrold: *mut GCObject,
    pub(crate) twups: *mut lua_State,
    pub(crate) panic: LuaCFunction,
    pub(crate) memerrmsg: *mut TString,
    pub(crate) tmname: [*mut TString; 25],
    pub(crate) mt: [*mut Table; LUA_NUMTYPES as usize],
    pub(crate) strcache: [[*mut TString; 2]; 53],
    pub(crate) warnf: lua_WarnFunction,
    pub(crate) ud_warn: *mut core::ffi::c_void,
    pub(crate) mainth: LX,
}

pub unsafe fn print_listing(state: *mut lua_State, full: bool) {
    let proto = unsafe { top_proto(state) };
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
        let line = getfuncline(proto, pc);
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
    let name = unsafe { eventname(state, index) };
    tstring_to_string(name)
}

unsafe fn cl_lvalue(value: *const TValue) -> *const LClosure {
    debug_assert_eq!(
        unsafe { (*value).tt_ & !BIT_ISCOLLECTABLE },
        LUA_VLCL,
        "expected Lua closure on stack top"
    );
    unsafe { (*value).value_.gc.cast() }
}

unsafe fn top_proto(state: *mut lua_State) -> *const Proto {
    let top = unsafe { (*state).top.p };
    unsafe { (*cl_lvalue(s2v(top.sub(1)))).p }
}

fn getbaseline(proto: *const Proto, pc: c_int, basepc: &mut c_int) -> c_int {
    let proto = unsafe { &*proto };
    if proto.sizeabslineinfo == 0 || pc < unsafe { (*proto.abslineinfo).pc } {
        *basepc = -1;
        proto.linedefined
    } else {
        let mut i = pc / MAXIWTHABS - 1;
        while i + 1 < proto.sizeabslineinfo
            && pc >= unsafe { (*proto.abslineinfo.add((i + 1) as usize)).pc }
        {
            i += 1;
        }
        *basepc = unsafe { (*proto.abslineinfo.add(i as usize)).pc };
        unsafe { (*proto.abslineinfo.add(i as usize)).line }
    }
}

fn getfuncline(proto: *const Proto, pc: c_int) -> c_int {
    if unsafe { (*proto).lineinfo.is_null() } {
        -1
    } else {
        let mut basepc = 0;
        let mut baseline = getbaseline(proto, pc, &mut basepc);
        while basepc < pc {
            basepc += 1;
            let lineinfo = unsafe { *(*proto).lineinfo.add(basepc as usize) };
            debug_assert_ne!(lineinfo, ABSLINEINFO);
            baseline += c_int::from(lineinfo);
        }
        baseline
    }
}

unsafe fn eventname(state: *mut lua_State, index: c_int) -> *const TString {
    unsafe { (*(*state).l_G).tmname[index as usize] }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getfuncline_uses_relative_and_absolute_entries() {
        let mut lineinfo = [0, 1, 2, ABSLINEINFO, 1, -2];
        let mut abslineinfo = [AbsLineInfo { pc: 3, line: 20 }];
        let proto = Proto {
            next: ptr::null_mut(),
            tt: 0,
            marked: 0,
            numparams: 0,
            flag: 0,
            maxstacksize: 0,
            sizeupvalues: 0,
            sizek: 0,
            sizecode: 0,
            sizelineinfo: lineinfo.len() as c_int,
            sizep: 0,
            sizelocvars: 0,
            sizeabslineinfo: abslineinfo.len() as c_int,
            linedefined: 10,
            lastlinedefined: 0,
            k: ptr::null_mut(),
            code: ptr::null_mut(),
            p: ptr::null_mut(),
            upvalues: ptr::null_mut(),
            lineinfo: lineinfo.as_mut_ptr(),
            abslineinfo: abslineinfo.as_mut_ptr(),
            locvars: ptr::null_mut(),
            source: ptr::null_mut(),
            gclist: ptr::null_mut(),
        };

        assert_eq!(getfuncline(&proto, 0), 10);
        assert_eq!(getfuncline(&proto, 2), 13);
        assert_eq!(getfuncline(&proto, 3), 20);
        assert_eq!(getfuncline(&proto, 5), 19);
    }

    #[test]
    fn getfuncline_returns_minus_one_without_debug_info() {
        let proto = Proto {
            next: ptr::null_mut(),
            tt: 0,
            marked: 0,
            numparams: 0,
            flag: 0,
            maxstacksize: 0,
            sizeupvalues: 0,
            sizek: 0,
            sizecode: 0,
            sizelineinfo: 0,
            sizep: 0,
            sizelocvars: 0,
            sizeabslineinfo: 0,
            linedefined: 0,
            lastlinedefined: 0,
            k: ptr::null_mut(),
            code: ptr::null_mut(),
            p: ptr::null_mut(),
            upvalues: ptr::null_mut(),
            lineinfo: ptr::null_mut(),
            abslineinfo: ptr::null_mut(),
            locvars: ptr::null_mut(),
            source: ptr::null_mut(),
            gclist: ptr::null_mut(),
        };

        assert_eq!(getfuncline(&proto, 0), -1);
    }
}
