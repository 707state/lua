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
use std::ffi::{CStr, c_uchar, c_ulong};
use std::mem::MaybeUninit;

pub type lua_Integer = i64;
pub type lua_Number = f64;
pub type lua_Unsigned = u64;
pub(crate) type l_mem = isize;
pub(crate) type TStatus = u8;
pub(crate) type lu_byte = u8;
pub(crate) type ls_byte = i8;
pub type lua_CFunction = Option<unsafe fn(*mut lua_State) -> c_int>;
pub(crate) type lua_KContext = isize;
pub(crate) type lua_KFunction = Option<unsafe fn(*mut lua_State, c_int, lua_KContext) -> c_int>;
pub(crate) type lua_Reader =
    Option<unsafe fn(*mut lua_State, *mut c_void, *mut usize) -> *const c_char>;
pub(crate) type lua_Writer =
    Option<unsafe fn(*mut lua_State, *const c_void, usize, *mut c_void) -> c_int>;
pub(crate) type lua_Alloc =
    Option<unsafe fn(*mut c_void, *mut c_void, usize, usize) -> *mut c_void>;
pub(crate) type lua_WarnFunction = Option<unsafe fn(*mut c_void, *const c_char, c_int)>;
pub(crate) type lua_Hook = Option<unsafe fn(*mut lua_State, *mut lua_Debug)>;
pub(crate) type Pfunc = Option<unsafe fn(*mut lua_State, *mut c_void)>;
pub(crate) type Instruction = u32;

pub const LUA_VERSION_NUM: lua_Number = 505.0;
pub const LUA_REGISTRYINDEX: c_int = -(i32::MAX / 2 + 1000);
pub const LUA_OK: TStatus = 0;
pub(crate) const LUA_ERRMEM: TStatus = 4;
pub(crate) const LUA_ERRERR: TStatus = 5;
pub const LUA_MULTRET: c_int = -1;

pub(crate) const LUA_TNONE: c_int = -1;
pub const LUA_TNIL: u8 = 0;
pub(crate) const LUA_TBOOLEAN: u8 = 1;
pub(crate) const LUA_TLIGHTUSERDATA: u8 = 2;
pub(crate) const LUA_TNUMBER: u8 = 3;
pub const LUA_TSTRING: u8 = 4;
pub const LUA_TTABLE: u8 = 5;
pub(crate) const LUA_TFUNCTION: u8 = 6;
pub(crate) const LUA_TUSERDATA: u8 = 7;
pub(crate) const LUA_TTHREAD: u8 = 8;
pub(crate) const LUA_NUMTYPES: c_int = 9;
pub(crate) const LUA_TUPVAL: u8 = LUA_NUMTYPES as u8;
pub(crate) const LUA_TPROTO: u8 = LUA_NUMTYPES as u8 + 1;

pub(crate) const BIT_ISCOLLECTABLE: u8 = 1 << 6;

pub(crate) const LUA_IDSIZE: usize = 60;
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

pub(crate) const LUA_OPADD: c_int = 0;
pub(crate) const LUA_OPSUB: c_int = 1;
pub(crate) const LUA_OPMUL: c_int = 2;
pub(crate) const LUA_OPMOD: c_int = 3;
pub(crate) const LUA_OPPOW: c_int = 4;
pub(crate) const LUA_OPDIV: c_int = 5;
pub(crate) const LUA_OPIDIV: c_int = 6;
pub(crate) const LUA_OPBAND: c_int = 7;
pub(crate) const LUA_OPBOR: c_int = 8;
pub(crate) const LUA_OPBXOR: c_int = 9;
pub(crate) const LUA_OPSHL: c_int = 10;
pub(crate) const LUA_OPSHR: c_int = 11;
pub(crate) const LUA_OPUNM: c_int = 12;
pub(crate) const LUA_OPBNOT: c_int = 13;
pub(crate) const TM_ADD: c_int = 6;
pub(crate) const TM_BAND: c_int = 13;
pub(crate) const TM_BOR: c_int = 14;
pub(crate) const TM_BXOR: c_int = 15;
pub(crate) const TM_SHL: c_int = 16;
pub(crate) const TM_SHR: c_int = 17;
pub(crate) const TM_UNM: c_int = 18;
pub(crate) const TM_BNOT: c_int = 19;
pub(crate) const TM_LT: c_int = 20;
pub(crate) const TM_LE: c_int = 21;
pub(crate) const TM_CONCAT: c_int = 22;
pub(crate) const PF_VATAB: u8 = 2;
pub(crate) const PF_FIXED: u8 = 4;
pub(crate) const LUA_FLOORN2I: c_int = 0;
pub(crate) const LUA_N2SBUFFSZ: usize = 64;
pub(crate) const UTF8BUFFSZ: usize = 8;
pub(crate) const MAX_LMEM: isize = isize::MAX;
pub(crate) const MAX_SIZE: usize = lua_Integer::MAX as usize;
pub(crate) const LUA_INTEGER_FMT: &[u8] = b"%lld\0";
pub(crate) const LUA_NUMBER_FMT: &[u8] = b"%.15g\0";
pub(crate) const LUA_NUMBER_FMT_N: &[u8] = b"%.17g\0";
pub(crate) const POINTER_FMT: &[u8] = b"%p\0";
pub(crate) const LUA_MAXCAPTURES: usize= 32;
pub(crate) const RETS: &[u8] = b"...";
pub(crate) const PRE: &[u8] = b"[string \"";
pub(crate) const POS: &[u8] = b"\"]";
pub(crate) const NULL_STRING: &[u8] = b"(null)\0";
pub(crate) const LUA_OPEQ: c_int = 0;
pub(crate) const LUA_OPLT: c_int = 1;
pub(crate) const LUA_OPLE: c_int = 2;
pub(crate) const LUA_TOTALTYPES: usize = LUA_TPROTO as usize + 2;
pub const LUA_MINSTACK: usize = 20;
pub(crate) const LUA_RIDX_GLOBALS: lua_Integer = 2;
pub(crate) const LUA_RIDX_LAST: c_int = 3;
pub(crate) const LUA_EXTRASPACE: usize = size_of::<*mut c_void>();
pub(crate) const LUAI_MINORMAJOR: c_int = 70;
pub(crate) const LUAI_MAJORMINOR: c_int = 50;
pub(crate) const LUAI_GENMINORMUL: c_int = 20;
pub(crate) const EXTRA_STACK: usize = 5;
pub(crate) const BASIC_STACK_SIZE: usize = 2 * LUA_MINSTACK;
pub(crate) const KGC_INC: u8 = 0;
pub(crate) const GCSTPGC: u8 = 2;
pub(crate) const GCSPAUSE: u8 = 8;
pub(crate) const TM_N: usize = 25;
pub(crate) const STRCACHE_N: usize = 53;
pub(crate) const STRCACHE_M: usize = 2;
pub(crate) const CIST_C: u32 = 1 << 15;
pub const LUA_GCSTOP: c_int = 0;
pub const LUA_GCRESTART: c_int = 1;
pub(crate) const LUA_GCCOLLECT: c_int = 2;
pub(crate) const LUA_GCCOUNT: c_int = 3;
pub(crate) const LUA_GCCOUNTB: c_int = 4;
pub(crate) const LUA_GCSTEP: c_int = 5;
pub(crate) const LUA_GCISRUNNING: c_int = 6;
pub const LUA_GCGEN: c_int = 7;
pub(crate) const LUA_GCINC: c_int = 8;
pub(crate) const LUA_GCPARAM: c_int = 9;

pub(crate) const LUA_GCPN: usize = 6;
pub(crate) const KGC_GENMINOR: c_int = 1;
pub(crate) const GCSTPUSR: u8 = 1;
pub(crate) const GCSTPCLS: u8 = 4;
pub(crate) const GCSpause: u8 = 8;

pub(crate) const LUA_GCPMINORMUL: usize = 0;
pub(crate) const LUA_GCPMAJORMINOR: usize = 1;
pub(crate) const LUA_GCPMINORMAJOR: usize = 2;
pub(crate) const LUA_GCPPAUSE: usize = 3;
pub(crate) const LUA_GCPSTEPMUL: usize = 4;
pub(crate) const LUA_GCPSTEPSIZE: usize = 5;
pub(crate) const LUAI_MAXCCALLS: u32 = 200;

pub(crate) const LUA_RIDX_MAINTHREAD: lua_Integer = 3;
pub(crate) const LUA_YIELD: TStatus = 1;
pub(crate) const LUAI_GCPAUSE: c_int = 250;
pub(crate) const LUAI_GCMUL: c_int = 200;
pub(crate) const CIST_TBC: u32 = 1 << 18;
pub(crate) const CIST_OAH: u32 = 1 << 19;
pub(crate) const CIST_YPCALL: u32 = 1 << 21;


pub(crate) const LUA_TDEADKEY: u8 = 11;
pub(crate) const LUA_VEMPTY: u8 = 16;
pub(crate) const LUA_VABSTKEY: u8 = 32;
pub(crate) const F2IEQ: c_int = 0;
pub(crate) const LSTRREG: i8 = -1;
pub(crate) const HOK: c_int = 0;
pub(crate) const HNOTFOUND: c_int = 1;
pub(crate) const HNOTATABLE: c_int = 2;
pub(crate) const HFIRSTNODE: c_int = 3;
pub(crate) const TM_NEWINDEX: usize = 1;
pub(crate) const TM_EQ: usize = 5;
pub(crate) const MASKFLAGS: u8 = !(!0u8 << (TM_EQ + 1));

pub(crate) const MAXUPVAL: c_int = 255;
pub(crate) const MAXRESULTS: c_int = 250;
pub(crate) const SHRT_MAX: c_int = i16::MAX as c_int;
pub(crate) const CLOSEKTOP: TStatus = LUA_ERRERR + 1;

pub(crate) const ERR_RESULTING_STRING_TOO_LARGE: &[u8] = b"resulting string too large\0";
pub(crate) const ERR_STRING_SLICE_TOO_LONG: &[u8] = b"string slice too long\0";
pub(crate) const ERR_VALUE_OUT_OF_RANGE: &[u8] = b"value out of range\0";
pub(crate) const ERR_LUA_FUNCTION_EXPECTED: &[u8] = b"Lua function expected\0";
pub(crate) const ERR_INVALID_CAPTURE_INDEX_FMT: &[u8] = b"invalid capture index %%%d\0";
pub(crate) const ERR_INVALID_PATTERN_CAPTURE: &[u8] = b"invalid pattern capture\0";
pub(crate) const ERR_MALFORMED_PATTERN_ENDS_WITH_ESCAPE: &[u8] = b"malformed pattern (ends with '%%')\0";
pub(crate) const ERR_MALFORMED_PATTERN_MISSING_BRACKET: &[u8] = b"malformed pattern (missing ']')\0";
pub(crate) const ERR_MALFORMED_PATTERN_MISSING_BALANCE_ARGS: &[u8] =b"malformed pattern (missing arguments to '%%b')\0";
pub(crate) const ERR_PATTERN_TOO_COMPLEX: &[u8] = b"pattern too complex\0";
pub(crate) const ERR_MISSING_FRONTIER_SET: &[u8] = b"missing '[' after '%%f' in pattern\0";
pub(crate) const ERR_TOO_MANY_CAPTURES: &[u8] = b"too many captures\0";
pub(crate) const ERR_TOO_MANY_CAPTURES_RESULTS: &[u8] = b"too many captures\0";
pub(crate) const ERR_UNFINISHED_CAPTURE: &[u8] = b"unfinished capture\0";
pub(crate) const ERR_INVALID_REPLACEMENT_USE_FMT: &[u8] = b"invalid use of '%c' in replacement string\0";
pub(crate) const ERR_INVALID_REPLACEMENT_VALUE_FMT: &[u8] = b"invalid replacement value (a %s)\0";
pub(crate) const ERR_EXPECTED_REPLACEMENT: &[u8] = b"string/function/table\0";
pub(crate) const ERR_INVALID_CONVERSION_SPEC_FMT: &[u8] = b"invalid conversion specification: '%s'\0";
pub(crate) const ERR_INVALID_FORMAT_TOO_LONG: &[u8] = b"invalid format (too long)\0";
pub(crate) const ERR_NO_VALUE: &[u8] = b"no value\0";
pub(crate) const ERR_INVALID_CONVERSION_FMT: &[u8] = b"invalid conversion '%s' to 'format'\0";
pub(crate) const ERR_VALUE_HAS_NO_LITERAL_FORM: &[u8] = b"value has no literal form\0";
pub(crate) const ERR_SPECIFIER_Q_MODIFIERS: &[u8] = b"specifier '%%q' cannot have modifiers\0";
pub(crate) const ERR_STRING_CONTAINS_ZEROS: &[u8] = b"string contains zeros\0";
pub(crate) const ERR_RESULT_TOO_LONG: &[u8] = b"result too long\0";
pub(crate) const ERR_INTEGER_OVERFLOW: &[u8] = b"integer overflow\0";
pub(crate) const ERR_UNSIGNED_OVERFLOW: &[u8] = b"unsigned overflow\0";
pub(crate) const ERR_STRING_LONGER_THAN_GIVEN_SIZE: &[u8] = b"string longer than given size\0";
pub(crate) const ERR_STRING_LENGTH_DOES_NOT_FIT: &[u8] = b"string length does not fit in given size\0";
pub(crate) const ERR_VARIABLE_LENGTH_FORMAT: &[u8] = b"variable-length format\0";
pub(crate) const ERR_FORMAT_RESULT_TOO_LARGE: &[u8] = b"format result too large\0";
pub(crate) const ERR_INITIAL_POSITION_OUT_OF_STRING: &[u8] = b"initial position out of string\0";
pub(crate) const ERR_DATA_STRING_TOO_SHORT: &[u8] = b"data string too short\0";
pub(crate) const ERR_TOO_MANY_RESULTS: &[u8] = b"too many results\0";
pub(crate) const ERR_UNFINISHED_ZSTRING: &[u8] = b"unfinished string for format 'z'\0";
pub(crate) const ERR_INVALID_FORMAT_OPTION_FMT: &[u8] = b"invalid format option '%c'\0";
pub(crate) const ERR_INTEGRAL_SIZE_OUT_OF_LIMITS_FMT: &[u8] = b"integral size (%d) out of limits [1,%d]\0";
pub(crate) const ERR_MISSING_SIZE_FOR_C: &[u8] = b"missing size for format option 'c'\0";
pub(crate) const ERR_INVALID_NEXT_OPTION_FOR_X: &[u8] = b"invalid next option for option 'X'\0";
pub(crate) const ERR_ALIGNMENT_NOT_POWER_OF_2: &[u8] = b"format asks for alignment not power of 2\0";
pub(crate) const ERR_INT_DOES_NOT_FIT_FMT: &[u8] = b"%d-byte integer does not fit into Lua Integer\0";
pub(crate) const FIELD_INDEX: &[u8] = b"__index\0";
pub(crate) const NAME_BYTE: &[u8] = b"byte\0";
pub(crate) const NAME_CHAR: &[u8] = b"char\0";
pub(crate) const NAME_DUMP: &[u8] = b"dump\0";
pub(crate) const NAME_FIND: &[u8] = b"find\0";
pub(crate) const NAME_FORMAT: &[u8] = b"format\0";
pub(crate) const NAME_GMATCH: &[u8] = b"gmatch\0";
pub(crate) const NAME_GSUB: &[u8] = b"gsub\0";
pub(crate) const NAME_LEN: &[u8] = b"len\0";
pub(crate) const NAME_LOWER: &[u8] = b"lower\0";
pub(crate) const NAME_MATCH: &[u8] = b"match\0";
pub(crate) const NAME_REP: &[u8] = b"rep\0";
pub(crate) const NAME_REVERSE: &[u8] = b"reverse\0";
pub(crate) const NAME_SUB: &[u8] = b"sub\0";
pub(crate) const NAME_UPPER: &[u8] = b"upper\0";
pub(crate) const NAME_PACK: &[u8] = b"pack\0";
pub(crate) const NAME_PACKSIZE: &[u8] = b"packsize\0";
pub(crate) const NAME_UNPACK: &[u8] = b"unpack\0";
pub(crate) const MT_ADD: &[u8] = b"__add\0";
pub(crate) const MT_SUB: &[u8] = b"__sub\0";
pub(crate) const MT_MUL: &[u8] = b"__mul\0";
pub(crate) const MT_MOD: &[u8] = b"__mod\0";
pub(crate) const MT_POW: &[u8] = b"__pow\0";
pub(crate) const MT_DIV: &[u8] = b"__div\0";
pub(crate) const MT_IDIV: &[u8] = b"__idiv\0";
pub(crate) const MT_UNM: &[u8] = b"__unm\0";
pub(crate) const CAP_UNFINISHED: isize = -1;
pub(crate) const CAP_POSITION: isize = -2;
pub(crate) const MAXCCALLS: c_int = 200;
pub(crate) const L_ESC: u8 = b'%';
pub(crate) const SPECIALS: &[u8] = b"^$*+?.([%-";
pub(crate) const MAX_FORMAT: usize = 32;
pub(crate) const MAX_ITEM: usize = 120;
pub(crate) const MAX_ITEMF: usize = 110 + 308;
pub(crate) const MAXINTSIZE: usize = 16;
pub(crate) const LUAL_PACKPADBYTE: u8 = 0x00;
pub(crate) const NB: usize = 8;
pub(crate) const MC: u8 = 0xFF;
pub(crate) const SZINT: usize = core::mem::size_of::<lua_Integer>();
pub(crate) const L_FMTFLAGSF: &[u8] = b"-+#0 ";
pub(crate) const L_FMTFLAGSX: &[u8] = b"-#0";
pub(crate) const L_FMTFLAGSI: &[u8] = b"-+0 ";
pub(crate) const L_FMTFLAGSU: &[u8] = b"-0";
pub(crate) const L_FMTFLAGSC: &[u8] = b"-";
pub(crate) const LUA_INTEGER_FRMLEN: &[u8] = b"ll";
pub(crate) const LUA_NUMBER_FRMLEN: &[u8] = b"";



pub(crate) const LUA_ERRRUN: TStatus = 2;
pub const LUA_ERRSYNTAX: TStatus = 3;
pub(crate) const LUA_MASKCALL: c_int = 1;
pub(crate) const LUA_MASKRET: c_int = 2;
pub(crate) const LUA_HOOKCALL: c_int = 0;
pub(crate) const LUA_HOOKRET: c_int = 1;
pub(crate) const LUA_HOOKTAILCALL: c_int = 4;
pub(crate) const PF_VAHID: u8 = 1;
pub(crate) const CIST_NRESULTS: u32 = 0xff;
pub(crate) const CIST_CCMT: u32 = 8;
pub(crate) const MAX_CCMT: u32 = 0xfu32 << CIST_CCMT;
pub(crate) const CIST_RECST: u32 = 12;
pub(crate) const CIST_FRESH: u32 = CIST_C << 1;
pub(crate) const CIST_CLSRET: u32 = CIST_FRESH << 1;
pub(crate) const CIST_HOOKED: u32 = CIST_OAH << 1;
pub(crate) const CIST_TAIL: u32 = CIST_YPCALL << 1;
pub(crate) const CIST_HOOKYIELD: u32 = CIST_TAIL << 1;
pub(crate) const CIST_FIN: u32 = CIST_HOOKYIELD << 1;
pub(crate) const STACKERRSPACE: c_int = 200;
pub(crate) const LUAI_MAXSTACK: c_int = 1_000_000;
pub(crate) const MAX_SIZET: usize = usize::MAX;
pub(crate) const MAXSTACK_BYSIZET: usize = MAX_SIZET / size_of::<StackValue>() - STACKERRSPACE as usize;
pub(crate) const MAXSTACK: c_int = if (LUAI_MAXSTACK as usize) < MAXSTACK_BYSIZET {
    LUAI_MAXSTACK
} else {
    MAXSTACK_BYSIZET as c_int
};
pub(crate) const ERRORSTACKSIZE: c_int = MAXSTACK + STACKERRSPACE;
pub(crate) const LUA_SIGNATURE_0: c_char = 0x1b_u8 as c_char;
pub(crate) const NYCI: u32 = 0x10000 | 1;
pub(crate) const TM_CALL: c_int = 23;



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
pub(crate) union UValue {
    pub(crate) uv: TValue,
    n: f64,
    p: *mut c_void,
    i: lua_Integer,
    l: isize,
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
    pub(crate) abslineinfo: *mut AbsLineInfo,
    pub(crate) locvars: *mut LocVar,
    pub(crate) source: *mut TString,
    pub(crate) gclist: *mut GCObject,
}

#[repr(C)]
pub(crate) struct LocVar {
    pub(crate) varname: *mut TString,
    pub(crate) startpc: c_int,
    pub(crate) endpc: c_int,
}

#[repr(C)]
pub(crate) struct AbsLineInfo {
    pub(crate) pc: c_int,
    pub(crate) line: c_int,
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
pub struct CallInfo {
    pub(crate) func: StkIdRel,
    pub(crate) top: StkIdRel,
    pub(crate) previous: *mut CallInfo,
    pub(crate) next: *mut CallInfo,
    pub(crate) u: CallInfoU,
    pub(crate) u2: CallInfoU2,
    pub(crate) callstatus: u32,
}

/// 平台相关的 jmp_buf 大小（按 usize 对齐，足够大以容纳所有平台的 jmp_buf）
#[cfg(target_arch = "x86_64")]
pub(crate) const JMP_BUF_SIZE: usize = 25; // x86_64: 200 bytes / 8
#[cfg(target_arch = "aarch64")]
pub(crate) const JMP_BUF_SIZE: usize = 24; // aarch64 macOS: 192 bytes / 8
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(crate) const JMP_BUF_SIZE: usize = 64; // 其他平台：保守估计

#[repr(C)]
pub(crate) struct lua_longjmp {
    pub(crate) previous: *mut lua_longjmp,
    pub(crate) status: TStatus,
    pub(crate) buf: [usize; JMP_BUF_SIZE],
}

#[repr(C)]
pub(crate) struct lua_Debug {
    pub(crate) event: c_int,
    pub(crate) name: *const c_char,
    pub(crate) namewhat: *const c_char,
    pub(crate) what: *const c_char,
    pub(crate) source: *const c_char,
    pub(crate) srclen: usize,
    pub(crate) currentline: c_int,
    pub(crate) linedefined: c_int,
    pub(crate) lastlinedefined: c_int,
    pub(crate) nups: c_uchar,
    pub(crate) nparams: c_uchar,
    pub(crate) isvararg: c_char,
    pub(crate) extraargs: c_uchar,
    pub(crate) istailcall: c_char,
    pub(crate) ftransfer: c_int,
    pub(crate) ntransfer: c_int,
    pub(crate) short_src: [c_char; LUA_IDSIZE],
    pub(crate) i_ci: *mut CallInfo,
}
impl Default for lua_Debug {
    fn default() -> Self {
        Self {
            event: 0,
            name: std::ptr::null(),
            namewhat: std::ptr::null(),
            what: std::ptr::null(),
            source: std::ptr::null(),
            srclen: 0,
            currentline: 0,
            linedefined: 0,
            lastlinedefined: 0,
            nups: 0,
            nparams: 0,
            isvararg: 0,
            extraargs: 0,
            istailcall: 0,
            ftransfer: 0,
            ntransfer: 0,
            short_src: [0; LUA_IDSIZE],
            i_ci: std::ptr::null_mut(),
        }
    }
}


#[repr(C)]
pub(crate) struct LConv {
   pub(crate)  decimal_point: *mut c_char,
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
pub struct lua_State {
    pub(crate) next: *mut GCObject,
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) allowhook: u8,
    pub(crate) status: TStatus,
    pub(crate) top: StkIdRel,
    pub(crate) l_G: *mut GlobalState,
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

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union Vardesc {
    pub(crate) vd: VardescFields,
    pub(crate) k: TValue,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct VardescFields {
    pub(crate) value_: Value,
    pub(crate) tt_: u8,
    pub(crate) kind: lu_byte,
    pub(crate) ridx: lu_byte,
    pub(crate) pidx: i16,
    pub(crate) name: *mut TString,
}

#[repr(C)]
pub(crate) struct BuffFS {
    pub(crate) l: *mut lua_State,
    pub(crate) b: *mut c_char,
    pub(crate) buffsize: usize,
    pub(crate) blen: usize,
    pub(crate) err: c_int,
    pub(crate) space: [c_char; LUA_IDSIZE + LUA_N2SBUFFSZ + 95],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct ExpdescInd {
    pub(crate) idx: i16,
    pub(crate) t: lu_byte,
    pub(crate) ro: lu_byte,
    pub(crate) keystr: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct ExpdescVar {
pub(crate)     ridx: lu_byte,
pub(crate)     vidx: i16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union ExpdescUnion {
pub(crate)     ival: lua_Integer,
pub(crate)     nval: lua_Number,
pub(crate)     strval: *mut TString,
pub(crate)     info: c_int,
pub(crate)     ind: ExpdescInd,
pub(crate)     var: ExpdescVar,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct expdesc {
pub(crate)     k: c_int,
pub(crate)     u: ExpdescUnion,
pub(crate)     t: c_int,
pub(crate)     f: c_int,
}
#[repr(C)]
pub(crate) struct VardescList {
    pub(crate) arr: *mut Vardesc,
    pub(crate) n: c_int,
    pub(crate) size: c_int,
}

#[repr(C)]
pub(crate) struct Dyndata {
    pub(crate) actvar: VardescList,
    pub(crate) gt: Labellist,
    pub(crate) label: Labellist,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct Labeldesc {
    pub(crate) name: *mut TString,
    pub(crate) pc: c_int,
    pub(crate) line: c_int,
    pub(crate) nactvar: i16,
    pub(crate) close: lu_byte,
}

#[repr(C)]
pub(crate) struct Labellist {
    pub(crate) arr: *mut Labeldesc,
    pub(crate) n: c_int,
    pub(crate) size: c_int,
}
#[repr(C)]
pub(crate) struct CallS {
    pub(crate) func: StkId,
    pub(crate) nresults: c_int,
}


#[repr(C)]
pub(crate) struct Mbuffer {
    pub(crate) buffer: *mut c_char,
    pub(crate) n: usize,
    pub(crate) buffsize: usize,
}


#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct Token {
pub(crate)     token: c_int,
pub(crate)     seminfo: SemInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union SemInfo {
pub(crate)     r: lua_Number,
pub(crate)     i: lua_Integer,
pub(crate)     ts: *mut TString,
}
#[repr(C)]
pub(crate) struct LexState {
pub(crate)     current: c_int,
pub(crate)     linenumber: c_int,
pub(crate)     lastline: c_int,
pub(crate)     t: Token,
pub(crate)     lookahead: Token,
pub(crate)     fs: *mut FuncState,
pub(crate)     L: *mut lua_State,
pub(crate)     z: *mut ZIO,
pub(crate)     buff: *mut Mbuffer,
pub(crate)     h: *mut Table,
pub(crate)     dyd: *mut Dyndata,
pub(crate)     source: *mut TString,
pub(crate)     envn: *mut TString,
pub(crate)     brkn: *mut TString,
pub(crate)     glbn: *mut TString,
}

#[repr(C)]
pub(crate) struct BlockCnt {
   pub(crate)  previous: *mut BlockCnt,
   pub(crate)  firstlabel: c_int,
   pub(crate)  firstgoto: c_int,
   pub(crate)  nactvar: i16,
   pub(crate)  upval: lu_byte,
   pub(crate)  isloop: lu_byte,
   pub(crate)  insidetbc: lu_byte,
}
#[repr(C)]
pub(crate) struct FuncState {
    pub(crate) f: *mut Proto,
    pub(crate) prev: *mut FuncState,
    pub(crate) ls: *mut LexState,
    pub(crate) bl: *mut BlockCnt,
    pub(crate) kcache: *mut Table,
    pub(crate) pc: c_int,
    pub(crate) lasttarget: c_int,
    pub(crate) previousline: c_int,
    pub(crate) nk: c_int,
    pub(crate) np: c_int,
    pub(crate) nabslineinfo: c_int,
    pub(crate) firstlocal: c_int,
    pub(crate) firstlabel: c_int,
    pub(crate) ndebugvars: i16,
    pub(crate) nactvar: i16,
    pub(crate) nups: lu_byte,
    pub(crate) freereg: lu_byte,
    pub(crate) iwthabs: lu_byte,
    pub(crate) needclose: lu_byte,
}

// Direct re-exports from modules that use crate::runtime::* types
pub(crate) use crate::do_rs::{
    luaD_call, luaD_callnoyield, luaD_growstack, luaD_pcall, luaD_protectedparser, luaD_throw,
};
pub(crate) use crate::gc::{
    luaC_barrier_, luaC_barrierback_, luaC_changemode, luaC_checkfinalizer, luaC_fullgc, luaC_step,
};
use crate::luaffi::snprintf;
use crate::luavm::GlobalState;
use crate::object::{addnum2buff, addstr2buff, clearbuff, initbuff, luaO_utf8esc};
pub(crate) use crate::vm_rs::{
    luaV_concat, luaV_equalobj, luaV_finishget, luaV_finishset, luaV_lessequal, luaV_lessthan,
    luaV_objlen, luaV_tointeger, luaV_tonumber_,
};
#[inline]
pub(crate) unsafe fn luaU_dump(
    L: *mut lua_State,
    p: *mut Proto,
    writer: lua_Writer,
    data: *mut c_void,
    strip: c_int,
) -> c_int {
    unsafe { crate::dump::luaU_dump(L as _, p as _, core::mem::transmute(writer), data, strip) }
}

#[inline]
pub(crate) unsafe fn luaE_setdebt(g: *mut GlobalState, debt: l_mem) {
    unsafe { crate::state::luaE_setdebt(g as _, debt) }
}

// Wrapper functions for modules with self-contained type definitions.
// All structs are #[repr(C)] with identical layouts, so pointer casts are safe.

#[inline]
pub(crate) unsafe fn luaF_close(
    L: *mut lua_State,
    level: StkId,
    status: TStatus,
    yy: c_int,
) -> StkId {
    unsafe { crate::func::luaF_close(L as _, level as _, status, yy) as StkId }
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
pub(crate) unsafe fn luaO_arith(
    L: *mut lua_State,
    op: c_int,
    p1: *const TValue,
    p2: *const TValue,
    res: StkId,
) {
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

#[unsafe(no_mangle)]
pub(crate) unsafe  fn luaO_pushvfstring(
    state: *mut lua_State,
    mut fmt: *const c_char,
    mut argp: VaList<'_>,
) -> *const c_char {
    let mut buff = MaybeUninit::<BuffFS>::uninit();
    unsafe { initbuff(state, buff.as_mut_ptr()) };
    let buff = unsafe { buff.assume_init_mut() };
    while !fmt.is_null() && unsafe { *fmt } != 0 {
        let mut e = fmt;
        while unsafe { *e } != 0 && unsafe { *e } != b'%' as c_char {
            e = unsafe { e.add(1) };
        }
        unsafe { addstr2buff(buff, fmt, e.offset_from(fmt) as usize) };
        if unsafe { *e } == 0 {
            break;
        }
        let spec = unsafe { *e.add(1) as u8 };
        match spec {
            b's' => {
                let s = unsafe { argp.arg::<*const c_char>() };
                let s = if s.is_null() {
                    NULL_STRING.as_ptr().cast()
                } else {
                    s
                };
                let len = unsafe { CStr::from_ptr(s) }.to_bytes().len();
                unsafe { addstr2buff(buff, s, len) };
            }
            b'c' => {
                let c = unsafe { argp.arg::<c_int>() } as u8;
                let ch = [c as c_char];
                unsafe { addstr2buff(buff, ch.as_ptr(), 1) };
            }
            b'd' => {
                let mut num = MaybeUninit::<TValue>::uninit();
                unsafe { setivalue(num.as_mut_ptr(), argp.arg::<c_int>() as lua_Integer) };
                unsafe { addnum2buff(buff, num.as_mut_ptr()) };
            }
            b'I' => {
                let mut num = MaybeUninit::<TValue>::uninit();
                unsafe { setivalue(num.as_mut_ptr(), argp.arg::<lua_Integer>()) };
                unsafe { addnum2buff(buff, num.as_mut_ptr()) };
            }
            b'f' => {
                let mut num = MaybeUninit::<TValue>::uninit();
                unsafe { setfltvalue(num.as_mut_ptr(), argp.arg::<lua_Number>()) };
                unsafe { addnum2buff(buff, num.as_mut_ptr()) };
            }
            b'p' => {
                let p = unsafe { argp.arg::<*mut c_void>() };
                let mut tmp = [0 as c_char; LUA_N2SBUFFSZ];
                let len = unsafe  {
                    snprintf(tmp.as_mut_ptr(), tmp.len(), POINTER_FMT.as_ptr().cast(), p)
                };
                unsafe { addstr2buff(buff, tmp.as_ptr(), len as usize) };
            }
            b'U' => {
                let arg = unsafe { argp.arg::<c_ulong>() };
                let mut tmp = [0 as c_char; UTF8BUFFSZ];
                let len = unsafe { luaO_utf8esc(tmp.as_mut_ptr(), arg as u32) } as usize;
                unsafe { addstr2buff(buff, tmp.as_ptr().add(UTF8BUFFSZ - len), len) };
            }
            b'%' => {
                let percent = [b'%' as c_char];
                unsafe { addstr2buff(buff, percent.as_ptr(), 1) };
            }
            _ => unsafe { addstr2buff(buff, e, 2) },
        }
        fmt = unsafe { e.add(2) };
    }
    unsafe { clearbuff(buff) }
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
pub(crate) unsafe fn luaS_newextlstr(
    L: *mut lua_State,
    s: *const c_char,
    len: usize,
    falloc: lua_Alloc,
    ud: *mut c_void,
) -> *mut TString {
    unsafe {
        crate::string::luaS_newextlstr(L as _, s, len, core::mem::transmute(falloc), ud)
            as *mut TString
    }
}
#[inline]
pub(crate) unsafe fn luaS_newudata(L: *mut lua_State, s: usize, nuvalue: u16) -> *mut Udata {
    unsafe { crate::string::luaS_newudata(L as _, s, nuvalue) as *mut Udata }
}

#[inline]
pub(crate) unsafe fn luaH_get(t: *mut Table, key: *const TValue, res: *mut TValue) -> lu_byte {
    unsafe { crate::table::luaH_get(t, key, res) }
}
#[inline]
pub(crate) unsafe fn luaH_getstr(t: *mut Table, key: *mut TString, res: *mut TValue) -> lu_byte {
    unsafe { crate::table::luaH_getstr(t, key, res) }
}
#[inline]
pub(crate) unsafe fn luaH_getint(t: *mut Table, key: lua_Integer, res: *mut TValue) -> lu_byte {
    unsafe { crate::table::luaH_getint(t, key, res) }
}
#[inline]
pub(crate) unsafe fn luaH_psetstr(t: *mut Table, key: *mut TString, val: *mut TValue) -> c_int {
    unsafe { crate::table::luaH_psetstr(t, key, val) }
}
#[inline]
pub(crate) unsafe fn luaH_pset(t: *mut Table, key: *const TValue, val: *mut TValue) -> c_int {
    unsafe { crate::table::luaH_pset(t, key, val) }
}
#[inline]
pub(crate) unsafe fn luaH_psetint(t: *mut Table, key: lua_Integer, val: *mut TValue) -> c_int {
    unsafe { crate::table::luaH_psetint(t, key, val) }
}
#[inline]
pub(crate) unsafe fn luaH_finishset(
    L: *mut lua_State,
    t: *mut Table,
    key: *const TValue,
    value: *mut TValue,
    hres: c_int,
) {
    unsafe { crate::table::luaH_finishset(L, t, key, value, hres) }
}
#[inline]
pub(crate) unsafe fn luaH_set(
    L: *mut lua_State,
    t: *mut Table,
    key: *const TValue,
    value: *mut TValue,
) {
    unsafe { crate::table::luaH_set(L, t, key, value) }
}
#[inline]
pub(crate) unsafe fn luaH_setint(
    L: *mut lua_State,
    t: *mut Table,
    key: lua_Integer,
    value: *mut TValue,
) {
    unsafe { crate::table::luaH_setint(L, t, key, value) }
}
#[inline]
pub(crate) unsafe fn luaH_new(L: *mut lua_State) -> *mut Table {
    unsafe { crate::table::luaH_new(L) }
}
#[inline]
pub(crate) unsafe fn luaH_resize(L: *mut lua_State, t: *mut Table, nasize: c_uint, nhsize: c_uint) {
    unsafe { crate::table::luaH_resize(L, t, nasize, nhsize) }
}
#[inline]
pub(crate) unsafe fn luaH_getn(L: *mut lua_State, t: *mut Table) -> lua_Unsigned {
    unsafe { crate::table::luaH_getn(L, t) }
}
#[inline]
pub(crate) unsafe fn luaH_next(L: *mut lua_State, t: *mut Table, key: StkId) -> c_int {
    unsafe { crate::table::luaH_next(L, t, key) }
}

#[inline]
pub(crate) unsafe fn luaZ_init(
    L: *mut lua_State,
    z: *mut ZIO,
    reader: lua_Reader,
    data: *mut c_void,
) {
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
pub(crate) unsafe fn G(L: *mut lua_State) -> *mut GlobalState {
    unsafe { (*L).l_G }
}

#[inline]
pub(crate) unsafe fn mainthread(g: *mut GlobalState) -> *mut lua_State {
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

/// Check if a CallInfo is a Lua code frame (not C, not hooked).
#[inline]
pub(crate) unsafe fn isLuacode(ci: *mut CallInfo) -> bool {
    unsafe { ((*ci).callstatus & (CIST_C | CIST_HOOKED)) == 0 }
}

/// Grow the stack if needed, adjusting a saved pointer.
#[inline]
pub(crate) unsafe fn checkstackp(L: *mut lua_State, n: c_int, p: &mut StkId) {
    if unsafe { (*L).stack_last.p.offset_from((*L).top.p) as c_int <= n } {
        let t = unsafe { savestack(L, *p) };
        unsafe { luaD_growstack(L, n, 1) };
        *p = unsafe { restorestack(L, t) };
    }
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

/// Alias for `settt_` — used by many modules as `settt`.
#[inline]
pub(crate) unsafe fn settt(o: *mut TValue, t: u8) {
    unsafe { settt_(o, t) };
}

/// Set a TString value directly on a `*mut TValue` (not via StkId).
#[inline]
pub(crate) unsafe fn setsvalue(obj: *mut TValue, s: *mut TString) {
    unsafe {
        (*obj).value_.gc = s.cast();
        settt_(obj, (*s).tt | BIT_ISCOLLECTABLE);
    }
}

/// Set a Table value directly on a `*mut TValue` (not via StkId).
#[inline]
pub(crate) unsafe fn sethvalue(obj: *mut TValue, h: *mut Table) {
    unsafe {
        (*obj).value_.gc = h.cast();
        settt_(obj, LUA_VTABLE | BIT_ISCOLLECTABLE);
    }
}

/// Check if a TString is a short string.
#[inline]
pub(crate) unsafe fn strisshr(ts: *const TString) -> bool {
    unsafe { (*ts).shrlen >= 0 }
}

/// Get the raw pointer to a short string's contents.
#[inline]
pub(crate) unsafe fn rawgetshrstr(ts: *const TString) -> *const c_char {
    unsafe { ptr::addr_of!((*ts).contents).cast() }
}

/// Check if a TValue is a long string.
#[inline]
pub(crate) unsafe fn ttislngstring(o: *const TValue) -> bool {
    unsafe { rawtt(o) == (LUA_VLNGSTR | BIT_ISCOLLECTABLE) }
}

/// Try to convert a TValue to float, returning true on success.
#[inline]
pub(crate) unsafe fn number_to_float(value: *const TValue, out: &mut lua_Number) -> bool {
    if unsafe { ttisfloat(value) } {
        *out = unsafe { fltvalue(value) };
        true
    } else if unsafe { ttisinteger(value) } {
        *out = unsafe { ivalue(value) as lua_Number };
        true
    } else {
        false
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
    if unsafe { (*G(L)).gcdebt <= 0 } {
        unsafe { luaC_step(L) };
    }
}

#[inline]
pub(crate) unsafe fn gettotalbytes(g: *mut GlobalState) -> l_mem {
    unsafe { (*g).gctotalbytes - (*g).gcdebt }
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

pub(crate) unsafe fn f_call(L: *mut lua_State, ud: *mut c_void) {
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
