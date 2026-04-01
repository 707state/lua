#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

pub(crate) use core::ffi::{VaList, c_char, c_int, c_uint, c_void};
pub(crate) use core::mem::{offset_of, size_of};
pub(crate) use core::ptr;

pub(crate) type lua_Integer = i64;
pub(crate) type lua_Number = f64;
pub(crate) type lua_Unsigned = u64;
pub(crate) type l_mem = isize;
pub(crate) type TStatus = u8;
pub(crate) type lu_byte = u8;
pub(crate) type ls_byte = i8;
pub(crate) type lua_CFunction = Option<unsafe extern "C-unwind" fn(*mut lua_State) -> c_int>;
pub(crate) type lua_KContext = isize;
pub(crate) type lua_KFunction =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, c_int, lua_KContext) -> c_int>;
pub(crate) type lua_Reader =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void, *mut usize) -> *const c_char>;
pub(crate) type lua_Writer =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, *const c_void, usize, *mut c_void) -> c_int>;
pub(crate) type lua_Alloc =
    Option<unsafe extern "C-unwind" fn(*mut c_void, *mut c_void, usize, usize) -> *mut c_void>;
pub(crate) type lua_WarnFunction = Option<unsafe extern "C-unwind" fn(*mut c_void, *const c_char, c_int)>;
pub(crate) type lua_Hook = Option<unsafe extern "C-unwind" fn(*mut lua_State, *mut lua_Debug)>;
pub(crate) type Pfunc = Option<unsafe extern "C-unwind" fn(*mut lua_State, *mut c_void)>;
pub(crate) type Instruction = u32;

pub(crate) const LUA_VERSION_NUM: lua_Number = 505.0;
pub(crate) const LUA_REGISTRYINDEX: c_int = -(i32::MAX / 2 + 1000);
pub(crate) const LUA_OK: TStatus = 0;
pub(crate) const LUA_ERRMEM: TStatus = 4;
pub(crate) const LUA_ERRERR: TStatus = 5;
pub(crate) const LUA_MULTRET: c_int = -1;

pub(crate) const LUA_TNONE: c_int = -1;
pub(crate) const LUA_TNIL: u8 = 0;
pub(crate) const LUA_TBOOLEAN: u8 = 1;
pub(crate) const LUA_TLIGHTUSERDATA: u8 = 2;
pub(crate) const LUA_TNUMBER: u8 = 3;
pub(crate) const LUA_TSTRING: u8 = 4;
pub(crate) const LUA_TTABLE: u8 = 5;
pub(crate) const LUA_TFUNCTION: u8 = 6;
pub(crate) const LUA_TUSERDATA: u8 = 7;
pub(crate) const LUA_TTHREAD: u8 = 8;
pub(crate) const LUA_NUMTYPES: c_int = 9;
pub(crate) const LUA_TUPVAL: u8 = LUA_NUMTYPES as u8;
pub(crate) const LUA_TPROTO: u8 = LUA_NUMTYPES as u8 + 1;

pub(crate) const BIT_ISCOLLECTABLE: u8 = 1 << 6;

pub(crate) const LUA_VNIL: u8 = LUA_TNIL;
pub(crate) const LUA_VFALSE: u8 = LUA_TBOOLEAN;
pub(crate) const LUA_VTRUE: u8 = LUA_TBOOLEAN | (1 << 4);
pub(crate) const LUA_VLIGHTUSERDATA: u8 = LUA_TLIGHTUSERDATA;
pub(crate) const LUA_VNUMINT: u8 = LUA_TNUMBER;
pub(crate) const LUA_VNUMFLT: u8 = LUA_TNUMBER | (1 << 4);
pub(crate) const LUA_VSHRSTR: u8 = LUA_TSTRING;
pub(crate) const LUA_VLNGSTR: u8 = LUA_TSTRING | (1 << 4);
pub(crate) const LUA_VUSERDATA: u8 = LUA_TUSERDATA;
pub(crate) const LUA_VTHREAD: u8 = LUA_TTHREAD;
pub(crate) const LUA_VPROTO: u8 = LUA_TPROTO;
pub(crate) const LUA_VUPVAL: u8 = LUA_TUPVAL;
pub(crate) const LUA_VLCL: u8 = LUA_TFUNCTION;
pub(crate) const LUA_VLCF: u8 = LUA_TFUNCTION | (1 << 4);
pub(crate) const LUA_VCCL: u8 = LUA_TFUNCTION | (2 << 4);
pub(crate) const LUA_VTABLE: u8 = LUA_TTABLE;

pub(crate) const WHITE0BIT: u8 = 3;
pub(crate) const WHITE1BIT: u8 = 4;
pub(crate) const BLACKBIT: u8 = 5;
pub(crate) const WHITEBITS: u8 = (1 << WHITE0BIT) | (1 << WHITE1BIT);

pub(crate) const LUA_OPEQ: c_int = 0;
pub(crate) const LUA_OPLT: c_int = 1;
pub(crate) const LUA_OPLE: c_int = 2;
pub(crate) const LUA_OPUNM: c_int = 12;
pub(crate) const LUA_OPBNOT: c_int = 13;

pub(crate) const LUA_GCSTOP: c_int = 0;
pub(crate) const LUA_GCRESTART: c_int = 1;
pub(crate) const LUA_GCCOLLECT: c_int = 2;
pub(crate) const LUA_GCCOUNT: c_int = 3;
pub(crate) const LUA_GCCOUNTB: c_int = 4;
pub(crate) const LUA_GCSTEP: c_int = 5;
pub(crate) const LUA_GCISRUNNING: c_int = 6;
pub(crate) const LUA_GCGEN: c_int = 7;
pub(crate) const LUA_GCINC: c_int = 8;
pub(crate) const LUA_GCPARAM: c_int = 9;

pub(crate) const LUA_GCPN: usize = 6;
pub(crate) const KGC_INC: u8 = 0;
pub(crate) const KGC_GENMINOR: c_int = 1;
pub(crate) const GCSTPUSR: u8 = 1;
pub(crate) const GCSTPGC: u8 = 2;
pub(crate) const GCSTPCLS: u8 = 4;
pub(crate) const GCSpause: u8 = 8;

pub(crate) const CIST_C: u32 = 1 << 15;
pub(crate) const CIST_TBC: u32 = 1 << 18;
pub(crate) const CIST_OAH: u32 = 1 << 19;
pub(crate) const CIST_YPCALL: u32 = 1 << 21;

pub(crate) const LUA_RIDX_GLOBALS: lua_Integer = 2;

pub(crate) const HOK: c_int = 0;
pub(crate) const HNOTATABLE: c_int = 2;
pub(crate) const HFIRSTNODE: c_int = 3;
pub(crate) const TM_NEWINDEX: usize = 1;
pub(crate) const TM_EQ: usize = 5;
pub(crate) const MASKFLAGS: u8 = !(!0u8 << (TM_EQ + 1));

pub(crate) const MAXUPVAL: c_int = 255;
pub(crate) const MAXRESULTS: c_int = 250;
pub(crate) const MAX_SIZE: usize = lua_Integer::MAX as usize;
pub(crate) const LUA_N2SBUFFSZ: usize = 64;
pub(crate) const SHRT_MAX: c_int = i16::MAX as c_int;
pub(crate) const CLOSEKTOP: TStatus = LUA_ERRERR + 1;

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union Value {
    pub(crate) gc: *mut GCObject,
    pub(crate) p: *mut c_void,
    pub(crate) f: lua_CFunction,
    pub(crate) i: lua_Integer,
    pub(crate) n: lua_Number,
    pub(crate) ub: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct TValue {
    pub(crate) value_: Value,
    pub(crate) tt_: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union StackValue {
    pub(crate) val: TValue,
    pub(crate) tbclist: StackValueTbc,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct StackValueTbc {
    pub(crate) value_: Value,
    pub(crate) tt_: u8,
    pub(crate) delta: u16,
}

pub(crate) type StkId = *mut StackValue;

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union StkIdRel {
    pub(crate) p: StkId,
    pub(crate) offset: isize,
}

#[repr(C)]
pub(crate) struct GCObject {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
}

#[repr(C)]
pub(crate) union TStringUnion {
    pub(crate) lnglen: usize,
    pub(crate) hnext: *mut TString,
}

#[repr(C)]
pub(crate) struct TString {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) extra: u8,
    pub(crate) shrlen: i8,
    pub(crate) hash: u32,
    pub(crate) u: TStringUnion,
    pub(crate) contents: *mut c_char,
    pub(crate) falloc: lua_Alloc,
    pub(crate) ud: *mut c_void,
}

#[repr(C)]
pub(crate) struct UValue {
    pub(crate) uv: TValue,
}

#[repr(C)]
pub(crate) struct Udata {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) nuvalue: u16,
    pub(crate) len: usize,
    pub(crate) metatable: *mut Table,
    pub(crate) gclist: *mut GCObject,
    pub(crate) uv: [UValue; 1],
}

#[repr(C)]
pub(crate) struct Udata0 {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) nuvalue: u16,
    pub(crate) len: usize,
    pub(crate) metatable: *mut Table,
    pub(crate) bindata: usize,
}

#[repr(C)]
pub(crate) struct Upvaldesc {
    pub(crate) name: *mut TString,
    pub(crate) instack: u8,
    pub(crate) idx: u8,
    pub(crate) kind: u8,
}

#[repr(C)]
pub(crate) struct Proto {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) numparams: u8,
    pub(crate) flag: u8,
    pub(crate) maxstacksize: u8,
    pub(crate) sizeupvalues: c_int,
    pub(crate) sizek: c_int,
    pub(crate) sizecode: c_int,
    pub(crate) sizelineinfo: c_int,
    pub(crate) sizep: c_int,
    pub(crate) sizelocvars: c_int,
    pub(crate) sizeabslineinfo: c_int,
    pub(crate) linedefined: c_int,
    pub(crate) lastlinedefined: c_int,
    pub(crate) k: *mut TValue,
    pub(crate) code: *mut Instruction,
    pub(crate) p: *mut *mut Proto,
    pub(crate) upvalues: *mut Upvaldesc,
    pub(crate) lineinfo: *mut i8,
    pub(crate) abslineinfo: *mut c_void,
    pub(crate) locvars: *mut c_void,
    pub(crate) source: *mut TString,
    pub(crate) gclist: *mut GCObject,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct UpValOpen {
    pub(crate) next: *mut UpVal,
    pub(crate) previous: *mut *mut UpVal,
}

#[repr(C)]
pub(crate) union UpValU {
    pub(crate) open: UpValOpen,
    pub(crate) value: TValue,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union UpValV {
    pub(crate) p: *mut TValue,
    pub(crate) offset: isize,
}

#[repr(C)]
pub(crate) struct UpVal {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) v: UpValV,
    pub(crate) u: UpValU,
}

#[repr(C)]
pub(crate) struct CClosure {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) nupvalues: u8,
    pub(crate) gclist: *mut GCObject,
    pub(crate) f: lua_CFunction,
    pub(crate) upvalue: [TValue; 1],
}

#[repr(C)]
pub(crate) struct LClosure {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) nupvalues: u8,
    pub(crate) gclist: *mut GCObject,
    pub(crate) p: *mut Proto,
    pub(crate) upvals: [*mut UpVal; 1],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct NodeKey {
    pub(crate) value_: Value,
    pub(crate) tt_: u8,
    pub(crate) key_tt: u8,
    pub(crate) next: c_int,
    pub(crate) key_val: Value,
}

#[repr(C)]
pub(crate) union Node {
    pub(crate) u: NodeKey,
    pub(crate) i_val: TValue,
}

#[repr(C)]
pub(crate) struct Table {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) flags: u8,
    pub(crate) lsizenode: u8,
    pub(crate) asize: u32,
    pub(crate) array: *mut Value,
    pub(crate) node: *mut Node,
    pub(crate) metatable: *mut Table,
    pub(crate) gclist: *mut GCObject,
}

#[repr(C)]
pub(crate) struct stringtable {
    pub(crate) hash: *mut *mut TString,
    pub(crate) nuse: c_int,
    pub(crate) size: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct CallInfoLua {
    pub(crate) savedpc: *const Instruction,
    pub(crate) trap: c_int,
    pub(crate) nextraargs: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct CallInfoC {
    pub(crate) k: lua_KFunction,
    pub(crate) old_errfunc: isize,
    pub(crate) ctx: isize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union CallInfoU {
    pub(crate) l: CallInfoLua,
    pub(crate) c: CallInfoC,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union CallInfoU2 {
    pub(crate) funcidx: c_int,
    pub(crate) nyield: c_int,
    pub(crate) nres: c_int,
}

#[repr(C)]
pub(crate) struct CallInfo {
    pub(crate) func: StkIdRel,
    pub(crate) top: StkIdRel,
    pub(crate) previous: *mut CallInfo,
    pub(crate) next: *mut CallInfo,
    pub(crate) u: CallInfoU,
    pub(crate) u2: CallInfoU2,
    pub(crate) callstatus: u32,
}

#[repr(C)]
pub(crate) struct lua_longjmp {
    pub(crate) previous: *mut lua_longjmp,
    pub(crate) status: TStatus,
}

#[repr(C)]
pub(crate) struct lua_Debug {
    pub(crate) _private: [u8; 0],
}

#[repr(C)]
pub(crate) struct TransferInfo {
    pub(crate) ftransfer: c_int,
    pub(crate) ntransfer: c_int,
}

#[repr(C)]
pub(crate) struct LX {
    pub(crate) extra_: [u8; size_of::<*mut c_void>()],
    pub(crate) l: lua_State,
}

#[repr(C)]
pub(crate) struct global_State {
    pub(crate) frealloc: lua_Alloc,
    pub(crate) ud: *mut c_void,
    pub(crate) GCtotalbytes: l_mem,
    pub(crate) GCdebt: l_mem,
    pub(crate) GCmarked: l_mem,
    pub(crate) GCmajorminor: l_mem,
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
    pub(crate) panic: lua_CFunction,
    pub(crate) memerrmsg: *mut TString,
    pub(crate) tmname: [*mut TString; 25],
    pub(crate) mt: [*mut Table; LUA_NUMTYPES as usize],
    pub(crate) strcache: [[*mut TString; 2]; 53],
    pub(crate) warnf: lua_WarnFunction,
    pub(crate) ud_warn: *mut c_void,
    pub(crate) mainth: LX,
}

#[repr(C)]
pub(crate) struct lua_State {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) allowhook: u8,
    pub(crate) status: TStatus,
    pub(crate) top: StkIdRel,
    pub(crate) l_G: *mut global_State,
    pub(crate) ci: *mut CallInfo,
    pub(crate) stack_last: StkIdRel,
    pub(crate) stack: StkIdRel,
    pub(crate) openupval: *mut UpVal,
    pub(crate) tbclist: StkIdRel,
    pub(crate) gclist: *mut GCObject,
    pub(crate) twups: *mut lua_State,
    pub(crate) errorJmp: *mut lua_longjmp,
    pub(crate) base_ci: CallInfo,
    pub(crate) hook: lua_Hook,
    pub(crate) errfunc: isize,
    pub(crate) nCcalls: u32,
    pub(crate) oldpc: c_int,
    pub(crate) nci: c_int,
    pub(crate) basehookcount: c_int,
    pub(crate) hookcount: c_int,
    pub(crate) hookmask: c_int,
    pub(crate) transferinfo: TransferInfo,
}

#[repr(C)]
pub(crate) struct ZIO {
    pub(crate) n: usize,
    pub(crate) p: *const c_char,
    pub(crate) reader: lua_Reader,
    pub(crate) data: *mut c_void,
    pub(crate) L: *mut lua_State,
}

#[repr(C)]
pub(crate) struct CallS {
    pub(crate) func: StkId,
    pub(crate) nresults: c_int,
}

// Direct re-exports from modules that use crate::runtime::* types
pub(crate) use crate::do_rs::{
    luaD_call, luaD_callnoyield, luaD_growstack, luaD_pcall, luaD_protectedparser, luaD_throw,
};
pub(crate) use crate::vm_rs::{
    luaV_concat, luaV_equalobj, luaV_finishget, luaV_finishset, luaV_lessequal, luaV_lessthan,
    luaV_objlen, luaV_tointeger, luaV_tonumber_,
};
pub(crate) use crate::gc::{
    luaC_barrier_, luaC_barrierback_, luaC_changemode, luaC_checkfinalizer, luaC_fullgc,
    luaC_step,
};
#[inline]
pub(crate) unsafe fn luaU_dump(L: *mut lua_State, p: *mut Proto, writer: lua_Writer, data: *mut c_void, strip: c_int) -> c_int {
    unsafe { crate::dump::luaU_dump(L as _, p as _, core::mem::transmute(writer), data, strip) }
}

#[inline]
pub(crate) unsafe fn luaE_setdebt(g: *mut global_State, debt: l_mem) {
    unsafe { crate::state::luaE_setdebt(g as _, debt) }
}

// Wrapper functions for modules with self-contained type definitions.
// All structs are #[repr(C)] with identical layouts, so pointer casts are safe.

#[inline]
pub(crate) unsafe fn luaF_close(L: *mut lua_State, level: StkId, status: TStatus, yy: c_int) -> StkId {
    unsafe {
        crate::func::luaF_close(L as _, level as _, status, yy) as StkId
    }
}
#[inline]
pub(crate) unsafe fn luaF_newCclosure(L: *mut lua_State, nupvals: c_int) -> *mut CClosure {
    unsafe { crate::func::luaF_newCclosure(L as _, nupvals) as *mut CClosure }
}
#[inline]
pub(crate) unsafe fn luaF_newtbcupval(L: *mut lua_State, level: StkId) {
    unsafe { crate::func::luaF_newtbcupval(L as _, level as _) }
}

#[inline]
pub(crate) unsafe fn luaO_arith(L: *mut lua_State, op: c_int, p1: *const TValue, p2: *const TValue, res: StkId) {
    unsafe { crate::object::luaO_arith(L as _, op, p1 as _, p2 as _, res as _) }
}
#[inline]
pub(crate) unsafe fn luaO_tostringbuff(obj: *const TValue, buff: *mut c_char) -> u32 {
    unsafe { crate::object::luaO_tostringbuff(obj as _, buff) }
}
#[inline]
pub(crate) unsafe fn luaO_str2num(s: *const c_char, o: *mut TValue) -> usize {
    unsafe { crate::object::luaO_str2num(s, o as _) }
}
#[inline]
pub(crate) unsafe fn luaO_tostring(L: *mut lua_State, obj: *mut TValue) {
    unsafe { crate::object::luaO_tostring(L as _, obj as _) }
}
#[inline]
pub(crate) unsafe fn luaO_pushvfstring(L: *mut lua_State, fmt: *const c_char, argp: VaList<'_>) -> *const c_char {
    unsafe { crate::object::luaO_pushvfstring(L as _, fmt, argp) }
}
#[inline]
pub(crate) unsafe fn luaO_codeparam(p: u32) -> u8 {
    unsafe { crate::object::luaO_codeparam(p) }
}
#[inline]
pub(crate) unsafe fn luaO_applyparam(p: u8, x: isize) -> isize {
    unsafe { crate::object::luaO_applyparam(p, x) }
}

#[inline]
pub(crate) unsafe fn luaS_new(L: *mut lua_State, s: *const c_char) -> *mut TString {
    unsafe { crate::string::luaS_new(L as _, s) as *mut TString }
}
#[inline]
pub(crate) unsafe fn luaS_newlstr(L: *mut lua_State, s: *const c_char, len: usize) -> *mut TString {
    unsafe { crate::string::luaS_newlstr(L as _, s, len) as *mut TString }
}
#[inline]
pub(crate) unsafe fn luaS_newextlstr(L: *mut lua_State, s: *const c_char, len: usize, falloc: lua_Alloc, ud: *mut c_void) -> *mut TString {
    unsafe { crate::string::luaS_newextlstr(L as _, s, len, core::mem::transmute(falloc), ud) as *mut TString }
}
#[inline]
pub(crate) unsafe fn luaS_newudata(L: *mut lua_State, s: usize, nuvalue: u16) -> *mut Udata {
    unsafe { crate::string::luaS_newudata(L as _, s, nuvalue) as *mut Udata }
}

#[inline]
pub(crate) unsafe fn luaH_get(t: *mut Table, key: *const TValue, res: *mut TValue) -> lu_byte {
    unsafe { crate::table::runtime::luaH_get(t as _, key as _, res as _) }
}
#[inline]
pub(crate) unsafe fn luaH_getstr(t: *mut Table, key: *mut TString, res: *mut TValue) -> lu_byte {
    unsafe { crate::table::runtime::luaH_getstr(t as _, key as _, res as _) }
}
#[inline]
pub(crate) unsafe fn luaH_getint(t: *mut Table, key: lua_Integer, res: *mut TValue) -> lu_byte {
    unsafe { crate::table::runtime::luaH_getint(t as _, key, res as _) }
}
#[inline]
pub(crate) unsafe fn luaH_psetstr(t: *mut Table, key: *mut TString, val: *mut TValue) -> c_int {
    unsafe { crate::table::runtime::luaH_psetstr(t as _, key as _, val as _) }
}
#[inline]
pub(crate) unsafe fn luaH_pset(t: *mut Table, key: *const TValue, val: *mut TValue) -> c_int {
    unsafe { crate::table::runtime::luaH_pset(t as _, key as _, val as _) }
}
#[inline]
pub(crate) unsafe fn luaH_psetint(t: *mut Table, key: lua_Integer, val: *mut TValue) -> c_int {
    unsafe { crate::table::runtime::luaH_psetint(t as _, key, val as _) }
}
#[inline]
pub(crate) unsafe fn luaH_finishset(L: *mut lua_State, t: *mut Table, key: *const TValue, value: *mut TValue, hres: c_int) {
    unsafe { crate::table::runtime::luaH_finishset(L as _, t as _, key as _, value as _, hres) }
}
#[inline]
pub(crate) unsafe fn luaH_set(L: *mut lua_State, t: *mut Table, key: *const TValue, value: *mut TValue) {
    unsafe { crate::table::runtime::luaH_set(L as _, t as _, key as _, value as _) }
}
#[inline]
pub(crate) unsafe fn luaH_setint(L: *mut lua_State, t: *mut Table, key: lua_Integer, value: *mut TValue) {
    unsafe { crate::table::runtime::luaH_setint(L as _, t as _, key, value as _) }
}
#[inline]
pub(crate) unsafe fn luaH_new(L: *mut lua_State) -> *mut Table {
    unsafe { crate::table::runtime::luaH_new(L as _) as *mut Table }
}
#[inline]
pub(crate) unsafe fn luaH_resize(L: *mut lua_State, t: *mut Table, nasize: c_uint, nhsize: c_uint) {
    unsafe { crate::table::runtime::luaH_resize(L as _, t as _, nasize, nhsize) }
}
#[inline]
pub(crate) unsafe fn luaH_getn(L: *mut lua_State, t: *mut Table) -> lua_Unsigned {
    unsafe { crate::table::runtime::luaH_getn(L as _, t as _) }
}
#[inline]
pub(crate) unsafe fn luaH_next(L: *mut lua_State, t: *mut Table, key: StkId) -> c_int {
    unsafe { crate::table::runtime::luaH_next(L as _, t as _, key as _) }
}

#[inline]
pub(crate) unsafe fn luaZ_init(L: *mut lua_State, z: *mut ZIO, reader: lua_Reader, data: *mut c_void) {
    unsafe { crate::zio::luaZ_init(L as _, z as _, core::mem::transmute(reader), data) }
}

#[inline]
pub(crate) unsafe fn luaE_warning(L: *mut lua_State, msg: *const c_char, tocont: c_int) {
    unsafe { crate::state::luaE_warning(L as _, msg, tocont) }
}

#[inline]
pub(crate) unsafe fn luaT_objtypename(L: *mut lua_State, o: *const TValue) -> *const c_char {
    unsafe { crate::tm::luaT_objtypename(L as _, o as _) }
}

#[inline]
pub(crate) unsafe fn api_check(cond: bool, msg: &str) {
    assert!(cond, "{msg}");
}

#[inline]
pub(crate) unsafe fn rawtt(o: *const TValue) -> u8 {
    unsafe { (*o).tt_ }
}

#[inline]
pub(crate) unsafe fn novariant(t: u8) -> u8 {
    t & 0x0f
}

#[inline]
pub(crate) unsafe fn ttypetag(o: *const TValue) -> u8 {
    unsafe { rawtt(o) & 0x3f }
}

#[inline]
pub(crate) unsafe fn ttype(o: *const TValue) -> u8 {
    unsafe { novariant(rawtt(o)) }
}

#[inline]
pub(crate) unsafe fn iscollectable(o: *const TValue) -> bool {
    unsafe { rawtt(o) & BIT_ISCOLLECTABLE != 0 }
}

#[inline]
pub(crate) unsafe fn gcvalue(o: *const TValue) -> *mut GCObject {
    unsafe { (*o).value_.gc }
}

#[inline]
pub(crate) unsafe fn ivalue(o: *const TValue) -> lua_Integer {
    unsafe { (*o).value_.i }
}

#[inline]
pub(crate) unsafe fn fltvalue(o: *const TValue) -> lua_Number {
    unsafe { (*o).value_.n }
}

#[inline]
pub(crate) unsafe fn fvalue(o: *const TValue) -> lua_CFunction {
    unsafe { (*o).value_.f }
}

#[inline]
pub(crate) unsafe fn pvalue(o: *const TValue) -> *mut c_void {
    unsafe { (*o).value_.p }
}

#[inline]
pub(crate) unsafe fn ttisnil(o: *const TValue) -> bool {
    unsafe { ttype(o) == LUA_TNIL }
}

#[inline]
pub(crate) unsafe fn ttisfalse(o: *const TValue) -> bool {
    unsafe { rawtt(o) == LUA_VFALSE }
}

#[inline]
pub(crate) unsafe fn ttisinteger(o: *const TValue) -> bool {
    unsafe { rawtt(o) == LUA_VNUMINT }
}

#[inline]
pub(crate) unsafe fn ttisfloat(o: *const TValue) -> bool {
    unsafe { rawtt(o) == LUA_VNUMFLT }
}

#[inline]
pub(crate) unsafe fn ttisnumber(o: *const TValue) -> bool {
    unsafe { ttype(o) == LUA_TNUMBER }
}

#[inline]
pub(crate) unsafe fn ttisstring(o: *const TValue) -> bool {
    unsafe { ttype(o) == LUA_TSTRING }
}

#[inline]
pub(crate) unsafe fn ttisshrstring(o: *const TValue) -> bool {
    unsafe { rawtt(o) == (LUA_VSHRSTR | BIT_ISCOLLECTABLE) }
}

#[inline]
pub(crate) unsafe fn ttisfulluserdata(o: *const TValue) -> bool {
    unsafe { rawtt(o) == (LUA_VUSERDATA | BIT_ISCOLLECTABLE) }
}

#[inline]
pub(crate) unsafe fn ttislightuserdata(o: *const TValue) -> bool {
    unsafe { rawtt(o) == LUA_VLIGHTUSERDATA }
}

#[inline]
pub(crate) unsafe fn ttistable(o: *const TValue) -> bool {
    unsafe { rawtt(o) == (LUA_VTABLE | BIT_ISCOLLECTABLE) }
}

#[inline]
pub(crate) unsafe fn ttisthread(o: *const TValue) -> bool {
    unsafe { rawtt(o) == (LUA_VTHREAD | BIT_ISCOLLECTABLE) }
}

#[inline]
pub(crate) unsafe fn ttislcf(o: *const TValue) -> bool {
    unsafe { rawtt(o) == LUA_VLCF }
}

#[inline]
pub(crate) unsafe fn ttisCclosure(o: *const TValue) -> bool {
    unsafe { rawtt(o) == (LUA_VCCL | BIT_ISCOLLECTABLE) }
}

#[inline]
pub(crate) unsafe fn ttisLclosure(o: *const TValue) -> bool {
    unsafe { rawtt(o) == (LUA_VLCL | BIT_ISCOLLECTABLE) }
}

#[inline]
pub(crate) unsafe fn tsvalue(o: *const TValue) -> *mut TString {
    unsafe { gcvalue(o).cast() }
}

#[inline]
pub(crate) unsafe fn hvalue(o: *const TValue) -> *mut Table {
    unsafe { gcvalue(o).cast() }
}

#[inline]
pub(crate) unsafe fn uvalue(o: *const TValue) -> *mut Udata {
    unsafe { gcvalue(o).cast() }
}

#[inline]
pub(crate) unsafe fn thvalue(o: *const TValue) -> *mut lua_State {
    unsafe { gcvalue(o).cast() }
}

#[inline]
pub(crate) unsafe fn clCvalue(o: *const TValue) -> *mut CClosure {
    unsafe { gcvalue(o).cast() }
}

#[inline]
pub(crate) unsafe fn clLvalue(o: *const TValue) -> *mut LClosure {
    unsafe { gcvalue(o).cast() }
}

#[inline]
pub(crate) unsafe fn settt_(o: *mut TValue, t: u8) {
    unsafe { (*o).tt_ = t };
}

#[inline]
pub(crate) unsafe fn setobj(dst: *mut TValue, src: *const TValue) {
    unsafe {
        (*dst).value_ = (*src).value_;
        (*dst).tt_ = (*src).tt_;
    }
}

#[inline]
pub(crate) unsafe fn s2v(o: StkId) -> *mut TValue {
    unsafe { ptr::addr_of_mut!((*o).val) }
}

#[inline]
pub(crate) unsafe fn setobjs2s(_L: *mut lua_State, o1: StkId, o2: StkId) {
    unsafe { setobj(s2v(o1), s2v(o2)) };
}

#[inline]
pub(crate) unsafe fn setobj2s(_L: *mut lua_State, o1: StkId, o2: *const TValue) {
    unsafe { setobj(s2v(o1), o2) };
}

#[inline]
pub(crate) unsafe fn setobj2n(_L: *mut lua_State, o1: *mut TValue, o2: *const TValue) {
    unsafe { setobj(o1, o2) };
}

#[inline]
pub(crate) unsafe fn setnilvalue(obj: *mut TValue) {
    unsafe { settt_(obj, LUA_VNIL) };
}

#[inline]
pub(crate) unsafe fn setbfvalue(obj: *mut TValue) {
    unsafe { settt_(obj, LUA_VFALSE) };
}

#[inline]
pub(crate) unsafe fn setbtvalue(obj: *mut TValue) {
    unsafe { settt_(obj, LUA_VTRUE) };
}

#[inline]
pub(crate) unsafe fn setfltvalue(obj: *mut TValue, x: lua_Number) {
    unsafe {
        (*obj).value_.n = x;
        settt_(obj, LUA_VNUMFLT);
    }
}

#[inline]
pub(crate) unsafe fn setivalue(obj: *mut TValue, x: lua_Integer) {
    unsafe {
        (*obj).value_.i = x;
        settt_(obj, LUA_VNUMINT);
    }
}

#[inline]
pub(crate) unsafe fn setpvalue(obj: *mut TValue, x: *mut c_void) {
    unsafe {
        (*obj).value_.p = x;
        settt_(obj, LUA_VLIGHTUSERDATA);
    }
}

#[inline]
pub(crate) unsafe fn setfvalue(obj: *mut TValue, x: lua_CFunction) {
    unsafe {
        (*obj).value_.f = x;
        settt_(obj, LUA_VLCF);
    }
}

#[inline]
pub(crate) unsafe fn sethvalue2s(_L: *mut lua_State, o: StkId, h: *mut Table) {
    unsafe {
        (*s2v(o)).value_.gc = h.cast();
        settt_(s2v(o), LUA_VTABLE | BIT_ISCOLLECTABLE);
    }
}

#[inline]
pub(crate) unsafe fn setsvalue2s(_L: *mut lua_State, o: StkId, s: *mut TString) {
    unsafe {
        (*s2v(o)).value_.gc = s.cast();
        settt_(s2v(o), (*s).tt | BIT_ISCOLLECTABLE);
    }
}

#[inline]
pub(crate) unsafe fn setuvalue(_L: *mut lua_State, obj: *mut TValue, u: *mut Udata) {
    unsafe {
        (*obj).value_.gc = u.cast();
        settt_(obj, LUA_VUSERDATA | BIT_ISCOLLECTABLE);
    }
}

#[inline]
pub(crate) unsafe fn setclCvalue(_L: *mut lua_State, obj: *mut TValue, cl: *mut CClosure) {
    unsafe {
        (*obj).value_.gc = cl.cast();
        settt_(obj, LUA_VCCL | BIT_ISCOLLECTABLE);
    }
}

#[inline]
pub(crate) unsafe fn setthvalue(_L: *mut lua_State, obj: *mut TValue, th: *mut lua_State) {
    unsafe {
        (*obj).value_.gc = th.cast();
        settt_(obj, LUA_VTHREAD | BIT_ISCOLLECTABLE);
    }
}

#[inline]
pub(crate) unsafe fn tagisempty(tag: u8) -> bool {
    unsafe { novariant(tag) == LUA_TNIL }
}

#[inline]
pub(crate) unsafe fn l_isfalse(o: *const TValue) -> bool {
    unsafe { ttisfalse(o) || ttisnil(o) }
}

#[inline]
pub(crate) unsafe fn cvt2str(o: *const TValue) -> bool {
    unsafe { ttisnumber(o) }
}

#[inline]
pub(crate) unsafe fn tonumber(o: *const TValue, n: *mut lua_Number) -> c_int {
    if unsafe { ttisfloat(o) } {
        unsafe { *n = fltvalue(o) };
        1
    } else {
        unsafe { luaV_tonumber_(o, n) }
    }
}

#[inline]
pub(crate) unsafe fn tointeger(o: *const TValue, i: *mut lua_Integer) -> c_int {
    if unsafe { ttisinteger(o) } {
        unsafe { *i = ivalue(o) };
        1
    } else {
        unsafe { luaV_tointeger(o, i, 0) }
    }
}

#[inline]
pub(crate) unsafe fn G(L: *mut lua_State) -> *mut global_State {
    unsafe { (*L).l_G }
}

#[inline]
pub(crate) unsafe fn mainthread(g: *mut global_State) -> *mut lua_State {
    unsafe { ptr::addr_of_mut!((*g).mainth.l) }
}

#[inline]
pub(crate) unsafe fn yieldable(L: *mut lua_State) -> bool {
    unsafe { (*L).nCcalls & 0xffff0000 == 0 }
}

#[inline]
pub(crate) unsafe fn isLua(ci: *mut CallInfo) -> bool {
    unsafe { (*ci).callstatus & CIST_C == 0 }
}

#[inline]
pub(crate) unsafe fn getstr(ts: *mut TString) -> *const c_char {
    if unsafe { (*ts).shrlen >= 0 } {
        unsafe { ptr::addr_of!((*ts).contents).cast() }
    } else {
        unsafe { (*ts).contents.cast_const() }
    }
}

#[inline]
pub(crate) unsafe fn getlstr(ts: *mut TString, len: &mut usize) -> *const c_char {
    if unsafe { (*ts).shrlen >= 0 } {
        *len = unsafe { (*ts).shrlen as usize };
        unsafe { ptr::addr_of!((*ts).contents).cast() }
    } else {
        *len = unsafe { (*ts).u.lnglen };
        unsafe { (*ts).contents.cast_const() }
    }
}

#[inline]
pub(crate) unsafe fn api_incr_top(L: *mut lua_State) {
    unsafe {
        (*L).top.p = (*L).top.p.add(1);
        api_check((*L).top.p <= (*(*L).ci).top.p, "stack overflow");
    }
}

#[inline]
pub(crate) unsafe fn api_checknelems(L: *mut lua_State, n: c_int) {
    unsafe {
        api_check(
            (n as isize) < ((*L).top.p.offset_from((*(*L).ci).func.p)),
            "not enough elements in the stack",
        )
    };
}

#[inline]
pub(crate) unsafe fn api_checkpop(L: *mut lua_State, n: c_int) {
    unsafe {
        api_check(
            (n as isize) < (*L).top.p.offset_from((*(*L).ci).func.p)
                && (*L).tbclist.p < (*L).top.p.sub(n as usize),
            "not enough free elements in the stack",
        );
    }
}

#[inline]
pub(crate) unsafe fn adjustresults(L: *mut lua_State, nres: c_int) {
    unsafe {
        if nres <= LUA_MULTRET && (*(*L).ci).top.p < (*L).top.p {
            (*(*L).ci).top.p = (*L).top.p;
        }
    }
}

#[inline]
pub(crate) unsafe fn APIstatus(st: TStatus) -> c_int {
    st as c_int
}

#[inline]
pub(crate) unsafe fn savestack(L: *mut lua_State, pt: StkId) -> isize {
    unsafe { (pt.cast::<u8>()).offset_from((*L).stack.p.cast::<u8>()) }
}

#[inline]
pub(crate) unsafe fn restorestack(L: *mut lua_State, n: isize) -> StkId {
    unsafe { (*L).stack.p.cast::<u8>().offset(n).cast() }
}

#[inline]
pub(crate) unsafe fn isvalid(L: *mut lua_State, o: *const TValue) -> bool {
    unsafe { !ptr::eq(o, ptr::addr_of!((*G(L)).nilvalue)) }
}

#[inline]
pub(crate) unsafe fn ispseudo(i: c_int) -> bool {
    i <= LUA_REGISTRYINDEX
}

#[inline]
pub(crate) unsafe fn isupvalue(i: c_int) -> bool {
    i < LUA_REGISTRYINDEX
}

#[inline]
pub(crate) unsafe fn obj2gco<T>(v: *mut T) -> *mut GCObject {
    v.cast()
}

#[inline]
pub(crate) unsafe fn iswhite(o: *mut GCObject) -> bool {
    unsafe { (*o).marked & WHITEBITS != 0 }
}

#[inline]
pub(crate) unsafe fn isblack(o: *mut GCObject) -> bool {
    unsafe { (*o).marked & (1 << BLACKBIT) != 0 }
}

#[inline]
pub(crate) unsafe fn luaC_objbarrier(L: *mut lua_State, p: *mut GCObject, o: *mut GCObject) {
    if unsafe { isblack(p) && iswhite(o) } {
        unsafe { luaC_barrier_(L, p, o) };
    }
}

#[inline]
pub(crate) unsafe fn luaC_barrier(L: *mut lua_State, p: *mut GCObject, v: *const TValue) {
    if unsafe { iscollectable(v) } {
        unsafe { luaC_objbarrier(L, p, gcvalue(v)) };
    }
}

#[inline]
pub(crate) unsafe fn luaC_objbarrierback(L: *mut lua_State, p: *mut GCObject, o: *mut GCObject) {
    if unsafe { isblack(p) && iswhite(o) } {
        unsafe { luaC_barrierback_(L, p) };
    }
}

#[inline]
pub(crate) unsafe fn luaC_barrierback(L: *mut lua_State, p: *mut GCObject, v: *const TValue) {
    if unsafe { iscollectable(v) } {
        unsafe { luaC_objbarrierback(L, p, gcvalue(v)) };
    }
}

#[inline]
pub(crate) unsafe fn luaC_checkGC(L: *mut lua_State) {
    if unsafe { (*G(L)).GCdebt <= 0 } {
        unsafe { luaC_step(L) };
    }
}

#[inline]
pub(crate) unsafe fn gettotalbytes(g: *mut global_State) -> l_mem {
    unsafe { (*g).GCtotalbytes - (*g).GCdebt }
}

pub(crate) unsafe fn index2value(L: *mut lua_State, idx: c_int) -> *mut TValue {
    let ci = unsafe { (*L).ci };
    if idx > 0 {
        let o = unsafe { (*ci).func.p.add(idx as usize) };
        unsafe {
            api_check(
                idx as isize <= (*ci).top.p.offset_from((*ci).func.p.add(1)),
                "unacceptable index",
            )
        };
        if unsafe { o >= (*L).top.p } {
            unsafe { ptr::addr_of_mut!((*G(L)).nilvalue) }
        } else {
            unsafe { s2v(o) }
        }
    } else if !unsafe { ispseudo(idx) } {
        unsafe {
            api_check(
                idx != 0 && (-idx) as isize <= (*L).top.p.offset_from((*ci).func.p.add(1)),
                "invalid index",
            )
        };
        unsafe { s2v((*L).top.p.offset(idx as isize)) }
    } else if idx == LUA_REGISTRYINDEX {
        unsafe { ptr::addr_of_mut!((*G(L)).l_registry) }
    } else {
        let idx = LUA_REGISTRYINDEX - idx;
        unsafe { api_check(idx <= MAXUPVAL + 1, "upvalue index too large") };
        if unsafe { ttisCclosure(s2v((*ci).func.p)) } {
            let func = unsafe { clCvalue(s2v((*ci).func.p)) };
            if idx <= unsafe { (*func).nupvalues as c_int } {
                unsafe {
                    ptr::addr_of_mut!((*func).upvalue)
                        .cast::<TValue>()
                        .add((idx - 1) as usize)
                }
            } else {
                unsafe { ptr::addr_of_mut!((*G(L)).nilvalue) }
            }
        } else {
            unsafe { api_check(ttislcf(s2v((*ci).func.p)), "caller not a C function") };
            unsafe { ptr::addr_of_mut!((*G(L)).nilvalue) }
        }
    }
}

pub(crate) unsafe fn index2stack(L: *mut lua_State, idx: c_int) -> StkId {
    let ci = unsafe { (*L).ci };
    if idx > 0 {
        let o = unsafe { (*ci).func.p.add(idx as usize) };
        unsafe { api_check(o < (*L).top.p, "invalid index") };
        o
    } else {
        unsafe {
            api_check(
                idx != 0 && (-idx) as isize <= (*L).top.p.offset_from((*ci).func.p.add(1)),
                "invalid index",
            );
            api_check(!ispseudo(idx), "invalid index");
            (*L).top.p.offset(idx as isize)
        }
    }
}

pub(crate) unsafe fn reverse(L: *mut lua_State, mut from: StkId, mut to: StkId) {
    while from < to {
        let mut temp = TValue {
            value_: Value { ub: 0 },
            tt_: 0,
        };
        unsafe {
            setobj(ptr::addr_of_mut!(temp), s2v(from));
            setobjs2s(L, from, to);
            setobj2s(L, to, ptr::addr_of!(temp));
        }
        from = unsafe { from.add(1) };
        to = unsafe { to.sub(1) };
    }
}

pub(crate) unsafe fn auxgetstr(L: *mut lua_State, t: *const TValue, k: *const c_char) -> c_int {
    let str_ = unsafe { luaS_new(L, k) };
    let mut tag = if unsafe { ttistable(t) } {
        unsafe { luaH_getstr(hvalue(t), str_, s2v((*L).top.p)) }
    } else {
        LUA_TNIL | (3 << 4)
    };
    if !unsafe { tagisempty(tag) } {
        unsafe { api_incr_top(L) };
    } else {
        unsafe { setsvalue2s(L, (*L).top.p, str_) };
        unsafe { api_incr_top(L) };
        tag = unsafe { luaV_finishget(L, t, s2v((*L).top.p.sub(1)), (*L).top.p.sub(1), tag) };
    }
    unsafe { novariant(tag) as c_int }
}

pub(crate) unsafe fn getGlobalTable(L: *mut lua_State, gt: *mut TValue) {
    let registry = unsafe { hvalue(ptr::addr_of!((*G(L)).l_registry)) };
    let tag = unsafe { luaH_getint(registry, LUA_RIDX_GLOBALS, gt) };
    unsafe { api_check(novariant(tag) == LUA_TTABLE, "global table must exist") };
}

pub(crate) unsafe fn finishrawget(L: *mut lua_State, tag: u8) -> c_int {
    if unsafe { tagisempty(tag) } {
        unsafe { setnilvalue(s2v((*L).top.p)) };
    }
    unsafe { api_incr_top(L) };
    unsafe { novariant(tag) as c_int }
}

pub(crate) unsafe fn gettable(L: *mut lua_State, idx: c_int) -> *mut Table {
    let t = unsafe { index2value(L, idx) };
    unsafe { api_check(ttistable(t), "table expected") };
    unsafe { hvalue(t) }
}

pub(crate) unsafe fn auxsetstr(L: *mut lua_State, t: *const TValue, k: *const c_char) {
    let str_ = unsafe { luaS_new(L, k) };
    unsafe { api_checkpop(L, 1) };
    let hres = if unsafe { ttistable(t) } {
        unsafe { luaH_psetstr(hvalue(t), str_, s2v((*L).top.p.sub(1))) }
    } else {
        HNOTATABLE
    };
    if hres == HOK {
        unsafe { luaC_barrierback(L, gcvalue(t), s2v((*L).top.p.sub(1))) };
        unsafe { (*L).top.p = (*L).top.p.sub(1) };
    } else {
        unsafe { setsvalue2s(L, (*L).top.p, str_) };
        unsafe { api_incr_top(L) };
        unsafe { luaV_finishset(L, t, s2v((*L).top.p.sub(1)), s2v((*L).top.p.sub(2)), hres) };
        unsafe { (*L).top.p = (*L).top.p.sub(2) };
    }
}

pub(crate) unsafe fn aux_rawset(L: *mut lua_State, idx: c_int, key: *mut TValue, n: c_int) {
    unsafe { api_checkpop(L, n) };
    let t = unsafe { gettable(L, idx) };
    unsafe { luaH_set(L, t, key, s2v((*L).top.p.sub(1))) };
    unsafe { (*t).flags &= !MASKFLAGS };
    unsafe { luaC_barrierback(L, obj2gco(t), s2v((*L).top.p.sub(1))) };
    unsafe { (*L).top.p = (*L).top.p.sub(n as usize) };
}

#[inline]
pub(crate) unsafe fn getArrTag(t: *mut Table, k: u32) -> *mut u8 {
    unsafe { ((*t).array.cast::<u8>()).add(size_of::<u32>() + k as usize) }
}

#[inline]
pub(crate) unsafe fn getArrVal(t: *mut Table, k: u32) -> *mut Value {
    unsafe { (*t).array.sub(1 + k as usize) }
}

#[inline]
pub(crate) unsafe fn fval2arr(t: *mut Table, k: u32, tag: *mut u8, value: *const TValue) {
    unsafe {
        *tag = (*value).tt_;
        *getArrVal(t, k) = (*value).value_;
    }
}

#[inline]
pub(crate) unsafe fn checknoTM(mt: *mut Table, e: usize) -> bool {
    mt.is_null() || unsafe { (*mt).flags & (1u8 << e) != 0 }
}

pub(crate) unsafe extern "C-unwind" fn f_call(L: *mut lua_State, ud: *mut c_void) {
    let c = ud.cast::<CallS>();
    unsafe { luaD_callnoyield(L, (*c).func, (*c).nresults) };
}

pub(crate) unsafe fn checkresults(L: *mut lua_State, na: c_int, nr: c_int) {
    unsafe {
        api_check(
            nr == LUA_MULTRET || (*(*L).ci).top.p.offset_from((*L).top.p) >= (nr - na) as isize,
            "results from function overflow current stack size",
        );
        api_check(
            LUA_MULTRET <= nr && nr <= MAXRESULTS,
            "invalid number of results",
        );
    }
}

pub(crate) unsafe fn touserdata(o: *const TValue) -> *mut c_void {
    match unsafe { ttype(o) } {
        LUA_TUSERDATA => unsafe {
            (uvalue(o)
                .cast::<u8>()
                .add(udatamemoffset((*uvalue(o)).nuvalue))) as *mut c_void
        },
        LUA_TLIGHTUSERDATA => unsafe { pvalue(o) },
        _ => ptr::null_mut(),
    }
}

#[inline]
pub(crate) unsafe fn udatamemoffset(nuv: u16) -> usize {
    if nuv == 0 {
        offset_of!(Udata0, bindata)
    } else {
        offset_of!(Udata, uv) + size_of::<UValue>() * nuv as usize
    }
}
