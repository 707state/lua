#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

pub(crate) use core::ffi::{c_char, c_int, c_uint, c_void};
pub(crate) use core::mem::{offset_of, size_of};
pub(crate) use core::ptr;
use std::ffi::c_uchar;

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
pub const LUA_MULTRET: c_int = -1;

// ============================================================
// LuaStatus — thread/call status codes
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LuaStatus {
    Ok = 0,
    Yield = 1,
    ErrRun = 2,
    ErrSyntax = 3,
    ErrMem = 4,
    ErrErr = 5,
}

impl LuaStatus {
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
    #[inline]
    pub const fn as_c_int(self) -> c_int {
        self as c_int
    }
    /// Convert a raw `TStatus` byte back to `LuaStatus`, or `None` if unknown.
    #[inline]
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Ok),
            1 => Some(Self::Yield),
            2 => Some(Self::ErrRun),
            3 => Some(Self::ErrSyntax),
            4 => Some(Self::ErrMem),
            5 => Some(Self::ErrErr),
            _ => None,
        }
    }
}

// Backward-compatible const aliases (type stays as TStatus = u8)
pub const LUA_OK: TStatus = LuaStatus::Ok.as_u8();
pub(crate) const LUA_YIELD: TStatus = LuaStatus::Yield.as_u8();
pub(crate) const LUA_ERRRUN: TStatus = LuaStatus::ErrRun.as_u8();
pub const LUA_ERRSYNTAX: TStatus = LuaStatus::ErrSyntax.as_u8();
pub(crate) const LUA_ERRMEM: TStatus = LuaStatus::ErrMem.as_u8();
pub(crate) const LUA_ERRERR: TStatus = LuaStatus::ErrErr.as_u8();

// ============================================================
// LuaType — base type tags (lower 4 bits of tt_)
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub(crate) enum LuaType {
    None = -1,
    Nil = 0,
    Boolean = 1,
    LightUserData = 2,
    Number = 3,
    String = 4,
    Table = 5,
    Function = 6,
    UserData = 7,
    Thread = 8,
    // Internal types (not exposed to Lua scripts)
    UpVal = 9,
    Proto = 10,
    DeadKey = 11,
}

impl LuaType {
    #[inline]
    pub(crate) const fn as_u8(self) -> u8 {
        self as i8 as u8
    }
    #[inline]
    pub(crate) const fn as_c_int(self) -> c_int {
        self as c_int
    }
    #[inline]
    pub(crate) const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Nil),
            1 => Some(Self::Boolean),
            2 => Some(Self::LightUserData),
            3 => Some(Self::Number),
            4 => Some(Self::String),
            5 => Some(Self::Table),
            6 => Some(Self::Function),
            7 => Some(Self::UserData),
            8 => Some(Self::Thread),
            9 => Some(Self::UpVal),
            10 => Some(Self::Proto),
            11 => Some(Self::DeadKey),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub(crate) const LUA_TNONE: c_int = LuaType::None.as_c_int();
pub const LUA_TNIL: u8 = LuaType::Nil.as_u8();
pub(crate) const LUA_TBOOLEAN: u8 = LuaType::Boolean.as_u8();
pub(crate) const LUA_TLIGHTUSERDATA: u8 = LuaType::LightUserData.as_u8();
pub(crate) const LUA_TNUMBER: u8 = LuaType::Number.as_u8();
pub const LUA_TSTRING: u8 = LuaType::String.as_u8();
pub const LUA_TTABLE: u8 = LuaType::Table.as_u8();
pub(crate) const LUA_TFUNCTION: u8 = LuaType::Function.as_u8();
pub(crate) const LUA_TUSERDATA: u8 = LuaType::UserData.as_u8();
pub(crate) const LUA_TTHREAD: u8 = LuaType::Thread.as_u8();
pub(crate) const LUA_NUMTYPES: c_int = 9;
pub(crate) const LUA_TUPVAL: u8 = LuaType::UpVal.as_u8();
pub(crate) const LUA_TPROTO: u8 = LuaType::Proto.as_u8();
pub(crate) const LUA_TDEADKEY: u8 = LuaType::DeadKey.as_u8();

pub(crate) const BIT_ISCOLLECTABLE: u8 = 1 << 6;

pub(crate) const LUA_IDSIZE: usize = 60;

// ============================================================
// LuaVariant — full variant tags (lower 6 bits of tt_)
//   encoding: base_type | (variant_index << 4)
// ============================================================
/// Variant tag values that fit in `u8`, representing the lower 6 bits of `TValue::tt_`.
/// The collectable bit (bit 6) is NOT included here; it is set separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum LuaVariant {
    Nil = 0x00,           // LUA_TNIL
    False = 0x01,         // LUA_TBOOLEAN
    True = 0x11,          // LUA_TBOOLEAN | (1 << 4)
    LightUserData = 0x02, // LUA_TLIGHTUSERDATA
    NumInt = 0x03,        // LUA_TNUMBER
    NumFlt = 0x13,        // LUA_TNUMBER   | (1 << 4)
    ShrStr = 0x04,        // LUA_TSTRING
    LngStr = 0x14,        // LUA_TSTRING   | (1 << 4)
    UserData = 0x07,      // LUA_TUSERDATA
    Thread = 0x08,        // LUA_TTHREAD
    Proto = 0x0A,         // LUA_TPROTO  (= LUA_NUMTYPES + 1 = 10)
    UpVal = 0x09,         // LUA_TUPVAL  (= LUA_NUMTYPES = 9)
    LuaClosure = 0x06,    // LUA_TFUNCTION
    LightCF = 0x16,       // LUA_TFUNCTION | (1 << 4)
    CClosure = 0x26,      // LUA_TFUNCTION | (2 << 4)
    Table = 0x05,         // LUA_TTABLE
    // Internal sentinel values
    Empty = 0x10,   // LUA_TNIL      | (1 << 4)  — empty array slot
    AbstKey = 0x20, // LUA_TNIL      | (2 << 4)  — absent hash key
}

impl LuaVariant {
    #[inline]
    pub(crate) const fn as_u8(self) -> u8 {
        self as u8
    }
    #[inline]
    pub(crate) const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Self::Nil),
            0x01 => Some(Self::False),
            0x11 => Some(Self::True),
            0x02 => Some(Self::LightUserData),
            0x03 => Some(Self::NumInt),
            0x13 => Some(Self::NumFlt),
            0x04 => Some(Self::ShrStr),
            0x14 => Some(Self::LngStr),
            0x07 => Some(Self::UserData),
            0x08 => Some(Self::Thread),
            0x0A => Some(Self::Proto),
            0x09 => Some(Self::UpVal),
            0x06 => Some(Self::LuaClosure),
            0x16 => Some(Self::LightCF),
            0x26 => Some(Self::CClosure),
            0x05 => Some(Self::Table),
            0x10 => Some(Self::Empty),
            0x20 => Some(Self::AbstKey),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub(crate) const LUA_VNIL: u8 = LuaVariant::Nil.as_u8();
pub(crate) const LUA_VFALSE: u8 = LuaVariant::False.as_u8();
pub(crate) const LUA_VTRUE: u8 = LuaVariant::True.as_u8();
pub(crate) const LUA_VLIGHTUSERDATA: u8 = LuaVariant::LightUserData.as_u8();
pub(crate) const LUA_VNUMINT: u8 = LuaVariant::NumInt.as_u8();
pub(crate) const LUA_VNUMFLT: u8 = LuaVariant::NumFlt.as_u8();
pub(crate) const LUA_VSHRSTR: u8 = LuaVariant::ShrStr.as_u8();
pub(crate) const LUA_VLNGSTR: u8 = LuaVariant::LngStr.as_u8();
pub(crate) const LUA_VUSERDATA: u8 = LuaVariant::UserData.as_u8();
pub(crate) const LUA_VTHREAD: u8 = LuaVariant::Thread.as_u8();
pub(crate) const LUA_VPROTO: u8 = LuaVariant::Proto.as_u8();
pub(crate) const LUA_VUPVAL: u8 = LuaVariant::UpVal.as_u8();
pub(crate) const LUA_VLCL: u8 = LuaVariant::LuaClosure.as_u8();
pub(crate) const LUA_VLCF: u8 = LuaVariant::LightCF.as_u8();
pub(crate) const LUA_VCCL: u8 = LuaVariant::CClosure.as_u8();
pub(crate) const LUA_VTABLE: u8 = LuaVariant::Table.as_u8();
pub(crate) const LUA_VEMPTY: u8 = LuaVariant::Empty.as_u8();
pub(crate) const LUA_VABSTKEY: u8 = LuaVariant::AbstKey.as_u8();

pub(crate) const WHITE0BIT: u8 = 3;
pub(crate) const WHITE1BIT: u8 = 4;
pub(crate) const BLACKBIT: u8 = 5;
pub(crate) const WHITEBITS: u8 = (1 << WHITE0BIT) | (1 << WHITE1BIT);

// ============================================================
// LuaArithOp — arithmetic operation codes (lua_arith / luaV_arith)
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum LuaArithOp {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Mod = 3,
    Pow = 4,
    Div = 5,
    IDiv = 6,
    BAnd = 7,
    BOr = 8,
    BXor = 9,
    Shl = 10,
    Shr = 11,
    Unm = 12,
    BNot = 13,
}

impl LuaArithOp {
    #[inline]
    pub(crate) const fn as_c_int(self) -> c_int {
        self as c_int
    }
    #[inline]
    pub(crate) const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Add),
            1 => Some(Self::Sub),
            2 => Some(Self::Mul),
            3 => Some(Self::Mod),
            4 => Some(Self::Pow),
            5 => Some(Self::Div),
            6 => Some(Self::IDiv),
            7 => Some(Self::BAnd),
            8 => Some(Self::BOr),
            9 => Some(Self::BXor),
            10 => Some(Self::Shl),
            11 => Some(Self::Shr),
            12 => Some(Self::Unm),
            13 => Some(Self::BNot),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub(crate) const LUA_OPADD: c_int = LuaArithOp::Add.as_c_int();
pub(crate) const LUA_OPSUB: c_int = LuaArithOp::Sub.as_c_int();
pub(crate) const LUA_OPMUL: c_int = LuaArithOp::Mul.as_c_int();
pub(crate) const LUA_OPMOD: c_int = LuaArithOp::Mod.as_c_int();
pub(crate) const LUA_OPPOW: c_int = LuaArithOp::Pow.as_c_int();
pub(crate) const LUA_OPDIV: c_int = LuaArithOp::Div.as_c_int();
pub(crate) const LUA_OPIDIV: c_int = LuaArithOp::IDiv.as_c_int();
pub(crate) const LUA_OPBAND: c_int = LuaArithOp::BAnd.as_c_int();
pub(crate) const LUA_OPBOR: c_int = LuaArithOp::BOr.as_c_int();
pub(crate) const LUA_OPBXOR: c_int = LuaArithOp::BXor.as_c_int();
pub(crate) const LUA_OPSHL: c_int = LuaArithOp::Shl.as_c_int();
pub(crate) const LUA_OPSHR: c_int = LuaArithOp::Shr.as_c_int();
pub(crate) const LUA_OPUNM: c_int = LuaArithOp::Unm.as_c_int();
pub(crate) const LUA_OPBNOT: c_int = LuaArithOp::BNot.as_c_int();

// ============================================================
// TagMethod — metamethod indices (index into lt->mt array)
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum TagMethod {
    Index = 0,
    NewIndex = 1,
    Gc = 2,
    Mode = 3,
    Len = 4,
    Eq = 5,
    Add = 6,
    Sub = 7,
    Mul = 8,
    Mod = 9,
    Pow = 10,
    Div = 11,
    IDiv = 12,
    Band = 13,
    Bor = 14,
    Bxor = 15,
    Shl = 16,
    Shr = 17,
    Unm = 18,
    Bnot = 19,
    Lt = 20,
    Le = 21,
    Concat = 22,
    Call = 23,
    Close = 24,
    N = 25, // sentinel: total number of tag methods
}

impl TagMethod {
    #[inline]
    pub(crate) const fn as_c_int(self) -> c_int {
        self as c_int
    }
    #[inline]
    pub(crate) const fn as_usize(self) -> usize {
        self as usize
    }
    #[inline]
    pub(crate) const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Index),
            1 => Some(Self::NewIndex),
            2 => Some(Self::Gc),
            3 => Some(Self::Mode),
            4 => Some(Self::Len),
            5 => Some(Self::Eq),
            6 => Some(Self::Add),
            7 => Some(Self::Sub),
            8 => Some(Self::Mul),
            9 => Some(Self::Mod),
            10 => Some(Self::Pow),
            11 => Some(Self::Div),
            12 => Some(Self::IDiv),
            13 => Some(Self::Band),
            14 => Some(Self::Bor),
            15 => Some(Self::Bxor),
            16 => Some(Self::Shl),
            17 => Some(Self::Shr),
            18 => Some(Self::Unm),
            19 => Some(Self::Bnot),
            20 => Some(Self::Lt),
            21 => Some(Self::Le),
            22 => Some(Self::Concat),
            23 => Some(Self::Call),
            24 => Some(Self::Close),
            25 => Some(Self::N),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub(crate) const TM_INDEX: c_int = TagMethod::Index.as_c_int();
pub(crate) const TM_NEWINDEX: c_int = TagMethod::NewIndex.as_c_int();
pub(crate) const TM_GC: c_int = TagMethod::Gc.as_c_int();
pub(crate) const TM_MODE: c_int = TagMethod::Mode.as_c_int();
pub(crate) const TM_LEN: c_int = TagMethod::Len.as_c_int();
pub(crate) const TM_EQ: c_int = TagMethod::Eq.as_c_int();
pub(crate) const TM_ADD: c_int = TagMethod::Add.as_c_int();
pub(crate) const TM_SUB: c_int = TagMethod::Sub.as_c_int();
pub(crate) const TM_MUL: c_int = TagMethod::Mul.as_c_int();
pub(crate) const TM_MOD: c_int = TagMethod::Mod.as_c_int();
pub(crate) const TM_POW: c_int = TagMethod::Pow.as_c_int();
pub(crate) const TM_DIV: c_int = TagMethod::Div.as_c_int();
pub(crate) const TM_IDIV: c_int = TagMethod::IDiv.as_c_int();
pub(crate) const TM_BAND: c_int = TagMethod::Band.as_c_int();
pub(crate) const TM_BOR: c_int = TagMethod::Bor.as_c_int();
pub(crate) const TM_BXOR: c_int = TagMethod::Bxor.as_c_int();
pub(crate) const TM_SHL: c_int = TagMethod::Shl.as_c_int();
pub(crate) const TM_SHR: c_int = TagMethod::Shr.as_c_int();
pub(crate) const TM_UNM: c_int = TagMethod::Unm.as_c_int();
pub(crate) const TM_BNOT: c_int = TagMethod::Bnot.as_c_int();
pub(crate) const TM_LT: c_int = TagMethod::Lt.as_c_int();
pub(crate) const TM_LE: c_int = TagMethod::Le.as_c_int();
pub(crate) const TM_CONCAT: c_int = TagMethod::Concat.as_c_int();
pub(crate) const TM_CALL: c_int = TagMethod::Call.as_c_int();
pub(crate) const TM_CLOSE: c_int = TagMethod::Close.as_c_int();
pub(crate) const TM_N: c_int = TagMethod::N.as_c_int();
pub(crate) const PF_VATAB: u8 = 2;
pub(crate) const PF_FIXED: u8 = 4;
pub(crate) const LUA_FLOORN2I: c_int = 0;
pub(crate) const LUA_N2SBUFFSZ: usize = 64;
pub(crate) const UTF8BUFFSZ: usize = 8;
pub(crate) const MAX_LMEM: isize = isize::MAX;
pub(crate) const MAX_SIZE: usize = lua_Integer::MAX as usize;
pub(crate) const LUA_MAXCAPTURES: usize = 32;
pub(crate) const RETS: &[u8] = b"...";
pub(crate) const PRE: &[u8] = b"[string \"";
pub(crate) const POS: &[u8] = b"\"]";
pub(crate) const NULL_STRING: &[u8] = b"(null)\0";
// ============================================================
// LuaCompareOp — comparison operation codes (lua_compare)
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum LuaCompareOp {
    Eq = 0,
    Lt = 1,
    Le = 2,
}

impl LuaCompareOp {
    #[inline]
    pub(crate) const fn as_c_int(self) -> c_int {
        self as c_int
    }
    #[inline]
    pub(crate) const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Eq),
            1 => Some(Self::Lt),
            2 => Some(Self::Le),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub(crate) const LUA_OPEQ: c_int = LuaCompareOp::Eq.as_c_int();
pub(crate) const LUA_OPLT: c_int = LuaCompareOp::Lt.as_c_int();
pub(crate) const LUA_OPLE: c_int = LuaCompareOp::Le.as_c_int();

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
pub(crate) const STRCACHE_N: usize = 53;
pub(crate) const STRCACHE_M: usize = 2;
pub(crate) const CIST_C: u32 = 1 << 15;
// ============================================================
// LuaGcWhat — GC control operations (argument to lua_gc)
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LuaGcWhat {
    Stop = 0,
    Restart = 1,
    Collect = 2,
    Count = 3,
    CountB = 4,
    Step = 5,
    IsRunning = 6,
    Gen = 7,
    Inc = 8,
    Param = 9,
}

impl LuaGcWhat {
    #[inline]
    pub const fn as_c_int(self) -> c_int {
        self as c_int
    }
    #[inline]
    pub const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Stop),
            1 => Some(Self::Restart),
            2 => Some(Self::Collect),
            3 => Some(Self::Count),
            4 => Some(Self::CountB),
            5 => Some(Self::Step),
            6 => Some(Self::IsRunning),
            7 => Some(Self::Gen),
            8 => Some(Self::Inc),
            9 => Some(Self::Param),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub const LUA_GCSTOP: c_int = LuaGcWhat::Stop.as_c_int();
pub const LUA_GCRESTART: c_int = LuaGcWhat::Restart.as_c_int();
pub(crate) const LUA_GCCOLLECT: c_int = LuaGcWhat::Collect.as_c_int();
pub(crate) const LUA_GCCOUNT: c_int = LuaGcWhat::Count.as_c_int();
pub(crate) const LUA_GCCOUNTB: c_int = LuaGcWhat::CountB.as_c_int();
pub(crate) const LUA_GCSTEP: c_int = LuaGcWhat::Step.as_c_int();
pub(crate) const LUA_GCISRUNNING: c_int = LuaGcWhat::IsRunning.as_c_int();
pub const LUA_GCGEN: c_int = LuaGcWhat::Gen.as_c_int();
pub(crate) const LUA_GCINC: c_int = LuaGcWhat::Inc.as_c_int();
pub(crate) const LUA_GCPARAM: c_int = LuaGcWhat::Param.as_c_int();

pub(crate) const LUA_GCPN: usize = 6;
pub(crate) const KGC_GENMINOR: c_int = 1;
pub(crate) const GCSTPUSR: u8 = 1;
pub(crate) const GCSTPCLS: u8 = 4;
pub(crate) const GCSpause: u8 = 8;

// ============================================================
// LuaGcParam — GC parameter indices (used with LUA_GCPARAM)
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum LuaGcParam {
    MinorMul = 0,
    MajorMinor = 1,
    MinorMajor = 2,
    Pause = 3,
    StepMul = 4,
    StepSize = 5,
}

impl LuaGcParam {
    #[inline]
    pub(crate) const fn as_usize(self) -> usize {
        self as usize
    }
    #[inline]
    pub(crate) const fn from_usize(v: usize) -> Option<Self> {
        match v {
            0 => Some(Self::MinorMul),
            1 => Some(Self::MajorMinor),
            2 => Some(Self::MinorMajor),
            3 => Some(Self::Pause),
            4 => Some(Self::StepMul),
            5 => Some(Self::StepSize),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub(crate) const LUA_GCPMINORMUL: usize = LuaGcParam::MinorMul.as_usize();
pub(crate) const LUA_GCPMAJORMINOR: usize = LuaGcParam::MajorMinor.as_usize();
pub(crate) const LUA_GCPMINORMAJOR: usize = LuaGcParam::MinorMajor.as_usize();
pub(crate) const LUA_GCPPAUSE: usize = LuaGcParam::Pause.as_usize();
pub(crate) const LUA_GCPSTEPMUL: usize = LuaGcParam::StepMul.as_usize();
pub(crate) const LUA_GCPSTEPSIZE: usize = LuaGcParam::StepSize.as_usize();

pub(crate) const LUAI_MAXCCALLS: u32 = 200;

pub(crate) const LUA_RIDX_MAINTHREAD: lua_Integer = 3;
pub(crate) const LUAI_GCPAUSE: c_int = 250;
pub(crate) const LUAI_GCMUL: c_int = 200;
pub(crate) const CIST_TBC: u32 = 1 << 18;
pub(crate) const CIST_OAH: u32 = 1 << 19;
pub(crate) const CIST_YPCALL: u32 = 1 << 21;

pub(crate) const F2IEQ: c_int = 0;
pub(crate) const LSTRREG: i8 = -1;

// ============================================================
// HResult — hash-lookup result codes (returned by luaH_get / luaV_gettable)
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum HResult {
    Ok = 0,
    NotFound = 1,
    NoTable = 2,
    FirstNode = 3,
}

impl HResult {
    #[inline]
    pub(crate) const fn as_c_int(self) -> c_int {
        self as c_int
    }
    #[inline]
    pub(crate) const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Ok),
            1 => Some(Self::NotFound),
            2 => Some(Self::NoTable),
            3 => Some(Self::FirstNode),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub(crate) const HOK: c_int = HResult::Ok.as_c_int();
pub(crate) const HNOTFOUND: c_int = HResult::NotFound.as_c_int();
pub(crate) const HNOTATABLE: c_int = HResult::NoTable.as_c_int();
pub(crate) const HFIRSTNODE: c_int = HResult::FirstNode.as_c_int();
pub(crate) const MASKFLAGS: u8 = !(!0u8 << (TM_EQ as u32 + 1));

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
pub(crate) const ERR_MALFORMED_PATTERN_ENDS_WITH_ESCAPE: &[u8] =
    b"malformed pattern (ends with '%%')\0";
pub(crate) const ERR_MALFORMED_PATTERN_MISSING_BRACKET: &[u8] =
    b"malformed pattern (missing ']')\0";
pub(crate) const ERR_MALFORMED_PATTERN_MISSING_BALANCE_ARGS: &[u8] =
    b"malformed pattern (missing arguments to '%%b')\0";
pub(crate) const ERR_PATTERN_TOO_COMPLEX: &[u8] = b"pattern too complex\0";
pub(crate) const ERR_MISSING_FRONTIER_SET: &[u8] = b"missing '[' after '%%f' in pattern\0";
pub(crate) const ERR_TOO_MANY_CAPTURES: &[u8] = b"too many captures\0";
pub(crate) const ERR_TOO_MANY_CAPTURES_RESULTS: &[u8] = b"too many captures\0";
pub(crate) const ERR_UNFINISHED_CAPTURE: &[u8] = b"unfinished capture\0";
pub(crate) const ERR_INVALID_REPLACEMENT_USE_FMT: &[u8] =
    b"invalid use of '%c' in replacement string\0";
pub(crate) const ERR_INVALID_REPLACEMENT_VALUE_FMT: &[u8] = b"invalid replacement value (a %s)\0";
pub(crate) const ERR_EXPECTED_REPLACEMENT: &[u8] = b"string/function/table\0";
pub(crate) const ERR_INVALID_CONVERSION_SPEC_FMT: &[u8] =
    b"invalid conversion specification: '%s'\0";
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
pub(crate) const ERR_STRING_LENGTH_DOES_NOT_FIT: &[u8] =
    b"string length does not fit in given size\0";
pub(crate) const ERR_VARIABLE_LENGTH_FORMAT: &[u8] = b"variable-length format\0";
pub(crate) const ERR_FORMAT_RESULT_TOO_LARGE: &[u8] = b"format result too large\0";
pub(crate) const ERR_INITIAL_POSITION_OUT_OF_STRING: &[u8] = b"initial position out of string\0";
pub(crate) const ERR_DATA_STRING_TOO_SHORT: &[u8] = b"data string too short\0";
pub(crate) const ERR_TOO_MANY_RESULTS: &[u8] = b"too many results\0";
pub(crate) const ERR_UNFINISHED_ZSTRING: &[u8] = b"unfinished string for format 'z'\0";
pub(crate) const ERR_INVALID_FORMAT_OPTION_FMT: &[u8] = b"invalid format option '%c'\0";
pub(crate) const ERR_INTEGRAL_SIZE_OUT_OF_LIMITS_FMT: &[u8] =
    b"integral size (%d) out of limits [1,%d]\0";
pub(crate) const ERR_MISSING_SIZE_FOR_C: &[u8] = b"missing size for format option 'c'\0";
pub(crate) const ERR_INVALID_NEXT_OPTION_FOR_X: &[u8] = b"invalid next option for option 'X'\0";
pub(crate) const ERR_ALIGNMENT_NOT_POWER_OF_2: &[u8] =
    b"format asks for alignment not power of 2\0";
pub(crate) const ERR_INT_DOES_NOT_FIT_FMT: &[u8] =
    b"%d-byte integer does not fit into Lua Integer\0";
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
pub(crate) const MAXSTACK_BYSIZET: usize =
    MAX_SIZET / size_of::<StackValue>() - STACKERRSPACE as usize;
pub(crate) const MAXSTACK: c_int = if (LUAI_MAXSTACK as usize) < MAXSTACK_BYSIZET {
    LUAI_MAXSTACK
} else {
    MAXSTACK_BYSIZET as c_int
};
pub(crate) const ERRORSTACKSIZE: c_int = MAXSTACK + STACKERRSPACE;
pub(crate) const LUA_SIGNATURE_0: c_char = 0x1b_u8 as c_char;
pub(crate) const NYCI: u32 = 0x10000 | 1;

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

// ============================================================
// Rust-style value system: LuaValue enum + TValue impl
// ============================================================

/// A safe Rust enum representation of a Lua value.
///
/// This is a view type — it borrows or copies data from an underlying `TValue`.
/// It is suitable for pattern matching and safe inspection, without replacing
/// the low-level `TValue` memory layout (which is required by the GC and table
/// array code that embeds `Value` unions directly).
#[derive(Clone, Copy, Debug)]
pub(crate) enum LuaValue {
    /// `nil`
    Nil,
    /// `false`
    False,
    /// `true`
    True,
    /// Integer number
    Integer(lua_Integer),
    /// Float number
    Float(lua_Number),
    /// Short or long string (GC-managed)
    String(*mut GCObject),
    /// Table (GC-managed)
    Table(*mut GCObject),
    /// Lua closure (GC-managed)
    LuaClosure(*mut GCObject),
    /// C closure (GC-managed)
    CClosure(*mut GCObject),
    /// Light C function (not GC-managed)
    LightCFunction(lua_CFunction),
    /// Full userdata (GC-managed)
    UserData(*mut GCObject),
    /// Light userdata (raw pointer, not GC-managed)
    LightUserData(*mut core::ffi::c_void),
    /// Coroutine/thread (GC-managed)
    Thread(*mut GCObject),
    /// Internal: absent key sentinel
    AbsentKey,
    /// Internal: empty slot in array
    Empty,
    /// Any other internal tag (e.g. dead keys, proto, upval)
    Other(u8),
}

impl TValue {
    // ---- constructors -------------------------------------------------------

    /// Create a nil `TValue`.
    #[inline]
    pub(crate) const fn new_nil() -> Self {
        TValue {
            value_: Value { ub: 0 },
            tt_: LUA_VNIL,
        }
    }

    /// Create a boolean `TValue`.
    #[inline]
    pub(crate) const fn new_bool(b: bool) -> Self {
        TValue {
            value_: Value { ub: 0 },
            tt_: if b { LUA_VTRUE } else { LUA_VFALSE },
        }
    }

    /// Create an integer `TValue`.
    #[inline]
    pub(crate) fn new_integer(i: lua_Integer) -> Self {
        TValue {
            value_: Value { i },
            tt_: LUA_VNUMINT,
        }
    }

    /// Create a float `TValue`.
    #[inline]
    pub(crate) fn new_float(n: lua_Number) -> Self {
        TValue {
            value_: Value { n },
            tt_: LUA_VNUMFLT,
        }
    }

    /// Create a light userdata `TValue`.
    #[inline]
    pub(crate) fn new_lightuserdata(p: *mut core::ffi::c_void) -> Self {
        TValue {
            value_: Value { p },
            tt_: LUA_VLIGHTUSERDATA,
        }
    }

    /// Create a light C function `TValue`.
    #[inline]
    pub(crate) fn new_lightcf(f: lua_CFunction) -> Self {
        TValue {
            value_: Value { f },
            tt_: LUA_VLCF,
        }
    }

    // ---- type tag queries ---------------------------------------------------

    /// Return the raw type tag byte.
    #[inline]
    pub(crate) fn raw_tag(self) -> u8 {
        self.tt_
    }

    /// Return the base type (variant bits masked off), equivalent to `ttype`.
    #[inline]
    pub(crate) fn base_type(self) -> u8 {
        self.tt_ & 0x0f
    }

    /// Return the type tag with variant bits but without the collectable bit.
    #[inline]
    pub(crate) fn type_tag(self) -> u8 {
        self.tt_ & 0x3f
    }

    /// Returns `true` if this value is GC-collectable.
    #[inline]
    pub(crate) fn is_collectable(self) -> bool {
        self.tt_ & BIT_ISCOLLECTABLE != 0
    }

    /// Returns `true` if this value is `nil`.
    #[inline]
    pub(crate) fn is_nil(self) -> bool {
        self.base_type() == LUA_TNIL
    }

    /// Returns `true` if this is the boolean `false`.
    #[inline]
    pub(crate) fn is_false(self) -> bool {
        self.tt_ == LUA_VFALSE
    }

    /// Returns `true` if this is the boolean `true`.
    #[inline]
    pub(crate) fn is_true(self) -> bool {
        self.tt_ == LUA_VTRUE
    }

    /// Returns `true` if this is a boolean (either true or false).
    #[inline]
    pub(crate) fn is_boolean(self) -> bool {
        self.base_type() == LUA_TBOOLEAN
    }

    /// Returns `true` if this is an integer.
    #[inline]
    pub(crate) fn is_integer(self) -> bool {
        self.tt_ == LUA_VNUMINT
    }

    /// Returns `true` if this is a float.
    #[inline]
    pub(crate) fn is_float(self) -> bool {
        self.tt_ == LUA_VNUMFLT
    }

    /// Returns `true` if this is any number (integer or float).
    #[inline]
    pub(crate) fn is_number(self) -> bool {
        self.base_type() == LUA_TNUMBER
    }

    /// Returns `true` if this is any string (short or long).
    #[inline]
    pub(crate) fn is_string(self) -> bool {
        self.base_type() == LUA_TSTRING
    }

    /// Returns `true` if this is a short string.
    #[inline]
    pub(crate) fn is_short_string(self) -> bool {
        self.tt_ == (LUA_VSHRSTR | BIT_ISCOLLECTABLE)
    }

    /// Returns `true` if this is a long string.
    #[inline]
    pub(crate) fn is_long_string(self) -> bool {
        self.tt_ == (LUA_VLNGSTR | BIT_ISCOLLECTABLE)
    }

    /// Returns `true` if this is a table.
    #[inline]
    pub(crate) fn is_table(self) -> bool {
        self.tt_ == (LUA_VTABLE | BIT_ISCOLLECTABLE)
    }

    /// Returns `true` if this is a Lua (non-C) closure.
    #[inline]
    pub(crate) fn is_lua_closure(self) -> bool {
        self.tt_ == (LUA_VLCL | BIT_ISCOLLECTABLE)
    }

    /// Returns `true` if this is a C closure.
    #[inline]
    pub(crate) fn is_c_closure(self) -> bool {
        self.tt_ == (LUA_VCCL | BIT_ISCOLLECTABLE)
    }

    /// Returns `true` if this is a light C function (not a closure).
    #[inline]
    pub(crate) fn is_light_cf(self) -> bool {
        self.tt_ == LUA_VLCF
    }

    /// Returns `true` if this is a full userdata.
    #[inline]
    pub(crate) fn is_full_userdata(self) -> bool {
        self.tt_ == (LUA_VUSERDATA | BIT_ISCOLLECTABLE)
    }

    /// Returns `true` if this is a light userdata.
    #[inline]
    pub(crate) fn is_light_userdata(self) -> bool {
        self.tt_ == LUA_VLIGHTUSERDATA
    }

    /// Returns `true` if this is a thread/coroutine.
    #[inline]
    pub(crate) fn is_thread(self) -> bool {
        self.tt_ == (LUA_VTHREAD | BIT_ISCOLLECTABLE)
    }

    /// Returns `true` if the value is falsy (nil or false).
    #[inline]
    pub(crate) fn is_falsy(self) -> bool {
        self.is_false() || self.is_nil()
    }

    /// Returns `true` if the value is a number or string (convertible).
    #[inline]
    pub(crate) fn is_coercible(self) -> bool {
        self.is_number()
    }

    // ---- value extractors ---------------------------------------------------

    /// Extract integer value.
    /// # Safety
    /// Caller must ensure `is_integer()` is true.
    #[inline]
    pub(crate) unsafe fn as_integer_unchecked(self) -> lua_Integer {
        unsafe { self.value_.i }
    }

    /// Extract integer value, returning `None` if not an integer.
    #[inline]
    pub(crate) fn as_integer(self) -> Option<lua_Integer> {
        if self.is_integer() {
            Some(unsafe { self.value_.i })
        } else {
            None
        }
    }

    /// Extract float value.
    /// # Safety
    /// Caller must ensure `is_float()` is true.
    #[inline]
    pub(crate) unsafe fn as_float_unchecked(self) -> lua_Number {
        unsafe { self.value_.n }
    }

    /// Extract float value, returning `None` if not a float.
    #[inline]
    pub(crate) fn as_float(self) -> Option<lua_Number> {
        if self.is_float() {
            Some(unsafe { self.value_.n })
        } else {
            None
        }
    }

    /// Extract GC-managed object pointer.
    /// # Safety
    /// Caller must ensure `is_collectable()` is true.
    #[inline]
    pub(crate) unsafe fn as_gc_unchecked(self) -> *mut GCObject {
        unsafe { self.value_.gc }
    }

    /// Extract pointer to a `TString`.
    /// # Safety
    /// Caller must ensure `is_string()` is true.
    #[inline]
    pub(crate) unsafe fn as_string_unchecked(self) -> *mut GCObject {
        unsafe { self.value_.gc }
    }

    /// Extract pointer to a `Table`.
    /// # Safety
    /// Caller must ensure `is_table()` is true.
    #[inline]
    pub(crate) unsafe fn as_table_gc_unchecked(self) -> *mut GCObject {
        unsafe { self.value_.gc }
    }

    /// Extract the light C function.
    /// # Safety
    /// Caller must ensure `is_light_cf()` is true.
    #[inline]
    pub(crate) unsafe fn as_light_cf_unchecked(self) -> lua_CFunction {
        unsafe { self.value_.f }
    }

    /// Extract light userdata pointer.
    /// # Safety
    /// Caller must ensure `is_light_userdata()` is true.
    #[inline]
    pub(crate) unsafe fn as_light_userdata_unchecked(self) -> *mut core::ffi::c_void {
        unsafe { self.value_.p }
    }

    // ---- conversion ---------------------------------------------------------

    /// Convert to a `LuaValue` enum for safe pattern matching.
    ///
    /// `type_tag()` returns `tt_ & 0x3f`, which strips `BIT_ISCOLLECTABLE` (bit 6).
    /// Therefore the GC-managed types are matched by their base variant tag, and
    /// we use `self.is_collectable()` to confirm they are actually GC-managed in the
    /// full raw `tt_` (this guards against any internal sentinel values that share
    /// a low-6-bit tag but are not truly GC-managed).
    pub(crate) fn to_lua_value(self) -> LuaValue {
        match self.type_tag() {
            LUA_VNIL => LuaValue::Nil,
            LUA_VFALSE => LuaValue::False,
            LUA_VTRUE => LuaValue::True,
            LUA_VNUMINT => LuaValue::Integer(unsafe { self.value_.i }),
            LUA_VNUMFLT => LuaValue::Float(unsafe { self.value_.n }),
            // Strings: LUA_VSHRSTR = LUA_TSTRING (= 4), LUA_VLNGSTR = LUA_TSTRING | (1 << 4)
            LUA_VSHRSTR => LuaValue::String(unsafe { self.value_.gc }),
            LUA_VLNGSTR => LuaValue::String(unsafe { self.value_.gc }),
            LUA_VTABLE => LuaValue::Table(unsafe { self.value_.gc }),
            LUA_VLCL => LuaValue::LuaClosure(unsafe { self.value_.gc }),
            LUA_VCCL => LuaValue::CClosure(unsafe { self.value_.gc }),
            LUA_VLCF => LuaValue::LightCFunction(unsafe { self.value_.f }),
            LUA_VUSERDATA => LuaValue::UserData(unsafe { self.value_.gc }),
            LUA_VLIGHTUSERDATA => LuaValue::LightUserData(unsafe { self.value_.p }),
            LUA_VTHREAD => LuaValue::Thread(unsafe { self.value_.gc }),
            LUA_VABSTKEY => LuaValue::AbsentKey,
            t if (t & 0x0f) == LUA_TNIL => LuaValue::Empty,
            t => LuaValue::Other(t),
        }
    }

    /// Write this value into a raw `*mut TValue`.
    /// # Safety
    /// `dst` must be a valid, aligned, writable pointer.
    #[inline]
    pub(crate) unsafe fn write_to(self, dst: *mut TValue) {
        unsafe {
            (*dst).value_ = self.value_;
            (*dst).tt_ = self.tt_;
        }
    }
}

impl Default for TValue {
    #[inline]
    fn default() -> Self {
        TValue::new_nil()
    }
}

impl From<lua_Integer> for TValue {
    #[inline]
    fn from(i: lua_Integer) -> Self {
        TValue::new_integer(i)
    }
}

impl From<lua_Number> for TValue {
    #[inline]
    fn from(n: lua_Number) -> Self {
        TValue::new_float(n)
    }
}

impl From<bool> for TValue {
    #[inline]
    fn from(b: bool) -> Self {
        TValue::new_bool(b)
    }
}

impl From<LuaValue> for TValue {
    /// Convert a `LuaValue` back to a low-level `TValue`.
    ///
    /// GC-managed variants use the raw `*mut GCObject` pointer that was
    /// stored inside the value — the caller is responsible for ensuring the
    /// pointer is still valid (i.e. the GC has not reclaimed the object).
    #[inline]
    fn from(v: LuaValue) -> Self {
        match v {
            LuaValue::Nil => TValue::new_nil(),
            LuaValue::False => TValue::new_bool(false),
            LuaValue::True => TValue::new_bool(true),
            LuaValue::Integer(i) => TValue::new_integer(i),
            LuaValue::Float(n) => TValue::new_float(n),
            LuaValue::LightCFunction(f) => TValue::new_lightcf(f),
            LuaValue::LightUserData(p) => TValue::new_lightuserdata(p),
            // GC-managed variants: set gc pointer + correct tag
            LuaValue::String(gc) => TValue {
                value_: Value { gc },
                // Conservatively use short-string tag; the high nibble is
                // encoded in the object's own `tt` field and is preserved by
                // the GC, but here we only have the GCObject pointer.  Callers
                // that need exact short/long distinction should construct the
                // TValue manually (or use `setsvalue` / `setsvalue2s`).
                tt_: LUA_VSHRSTR | BIT_ISCOLLECTABLE,
            },
            LuaValue::Table(gc) => TValue {
                value_: Value { gc },
                tt_: LUA_VTABLE | BIT_ISCOLLECTABLE,
            },
            LuaValue::LuaClosure(gc) => TValue {
                value_: Value { gc },
                tt_: LUA_VLCL | BIT_ISCOLLECTABLE,
            },
            LuaValue::CClosure(gc) => TValue {
                value_: Value { gc },
                tt_: LUA_VCCL | BIT_ISCOLLECTABLE,
            },
            LuaValue::UserData(gc) => TValue {
                value_: Value { gc },
                tt_: LUA_VUSERDATA | BIT_ISCOLLECTABLE,
            },
            LuaValue::Thread(gc) => TValue {
                value_: Value { gc },
                tt_: LUA_VTHREAD | BIT_ISCOLLECTABLE,
            },
            LuaValue::AbsentKey => TValue {
                value_: Value { ub: 0 },
                tt_: LUA_VABSTKEY,
            },
            LuaValue::Empty => TValue {
                value_: Value { ub: 0 },
                tt_: LUA_VEMPTY,
            },
            LuaValue::Other(tt) => TValue {
                value_: Value { ub: 0 },
                tt_: tt,
            },
        }
    }
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
pub struct GCObject {
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

/// Lua 错误 panic payload：携带错误码，由 luaD_throw 抛出，由 luaD_rawrunprotected 捕获。
/// 使用独立类型而不是裸 TStatus，以便 downcast 时精确匹配，避免误捕获其他 panic。
#[derive(Debug, Clone, Copy)]
pub(crate) struct LuaError(pub(crate) TStatus);

/// luaD_throwbaselevel 专用 payload：要求 catch_unwind 将错误传递给最外层保护点。
#[derive(Debug, Clone, Copy)]
pub(crate) struct LuaErrorBase(pub(crate) TStatus);

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
    pub(crate) decimal_point: *mut c_char,
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
    pub(crate) status: LuaStatus,
    pub(crate) top: StkIdRel,
    pub(crate) l_G: *mut GlobalState,
    pub(crate) ci: *mut CallInfo,
    pub(crate) stack_last: StkIdRel,
    pub(crate) stack: StkIdRel,
    pub(crate) openupval: *mut UpVal,
    pub(crate) tbclist: StkIdRel,
    pub(crate) gclist: *mut GCObject,
    pub(crate) twups: *mut lua_State,
    /// 当前 luaD_rawrunprotected 的嵌套层数（替代原 setjmp/longjmp 链表）。
    /// luaD_throwbaselevel 依赖此值判断是否已到最外层。
    pub(crate) nesting_level: u32,
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
    pub(crate) ridx: lu_byte,
    pub(crate) vidx: i16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union ExpdescUnion {
    pub(crate) ival: lua_Integer,
    pub(crate) nval: lua_Number,
    pub(crate) strval: *mut TString,
    pub(crate) info: c_int,
    pub(crate) ind: ExpdescInd,
    pub(crate) var: ExpdescVar,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct expdesc {
    pub(crate) k: c_int,
    pub(crate) u: ExpdescUnion,
    pub(crate) t: c_int,
    pub(crate) f: c_int,
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
    pub(crate) token: c_int,
    pub(crate) seminfo: SemInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) union SemInfo {
    pub(crate) r: lua_Number,
    pub(crate) i: lua_Integer,
    pub(crate) ts: *mut TString,
}
#[repr(C)]
pub(crate) struct LexState {
    pub(crate) current: c_int,
    pub(crate) linenumber: c_int,
    pub(crate) lastline: c_int,
    pub(crate) t: Token,
    pub(crate) lookahead: Token,
    pub(crate) fs: *mut FuncState,
    pub(crate) L: *mut lua_State,
    pub(crate) z: *mut ZIO,
    pub(crate) buff: *mut Mbuffer,
    pub(crate) h: *mut Table,
    pub(crate) dyd: *mut Dyndata,
    pub(crate) source: *mut TString,
    pub(crate) envn: *mut TString,
    pub(crate) brkn: *mut TString,
    pub(crate) glbn: *mut TString,
}

#[repr(C)]
pub(crate) struct BlockCnt {
    pub(crate) previous: *mut BlockCnt,
    pub(crate) firstlabel: c_int,
    pub(crate) firstgoto: c_int,
    pub(crate) nactvar: i16,
    pub(crate) upval: lu_byte,
    pub(crate) isloop: lu_byte,
    pub(crate) insidetbc: lu_byte,
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
use crate::luavm::GlobalState;
use crate::string::luaS_new;
pub(crate) use crate::vm_rs::{
    luaV_concat, luaV_equalobj, luaV_finishget, luaV_finishset, luaV_lessequal, luaV_lessthan,
    luaV_objlen, luaV_tointeger, luaV_tonumber_,
};

#[inline]
pub(crate) unsafe fn luaV_rawequalobj(t1: *const TValue, t2: *const TValue) -> c_int {
    unsafe { luaV_equalobj(ptr::null_mut(), t1, t2) }
}

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

pub(crate) use crate::object::{luaO_applyparam, luaO_codeparam};

pub(crate) use crate::table::{
    luaH_finishset, luaH_get, luaH_getint, luaH_getn, luaH_getstr, luaH_new, luaH_next, luaH_pset,
    luaH_psetint, luaH_psetstr, luaH_resize, luaH_set, luaH_setint,
};

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

// ---- Legacy pointer-based helpers (delegate to TValue methods) ----
// These are kept for backward compatibility with the rest of the codebase.
// New code should prefer calling methods directly on a `TValue` value.

/// Return the raw tag byte of a `*const TValue`.
#[inline]
pub(crate) unsafe fn rawtt(o: *const TValue) -> u8 {
    unsafe { (*o).raw_tag() }
}

/// Strip variant bits, returning just the base type (low 4 bits).
#[inline]
pub(crate) fn novariant(t: u8) -> u8 {
    t & 0x0f
}

/// Return `tt_ & 0x3f` (type tag without the collectable bit).
#[inline]
pub(crate) unsafe fn ttypetag(o: *const TValue) -> u8 {
    unsafe { (*o).type_tag() }
}

/// Return the base type of a `*const TValue` (equivalent to `novariant(rawtt(o))`).
#[inline]
pub(crate) unsafe fn ttype(o: *const TValue) -> u8 {
    unsafe { (*o).base_type() }
}

/// Return `true` if the value pointed to is GC-collectable.
#[inline]
pub(crate) unsafe fn iscollectable(o: *const TValue) -> bool {
    unsafe { (*o).is_collectable() }
}

/// Return the GC-object pointer from a collectable `TValue`.
#[inline]
pub(crate) unsafe fn gcvalue(o: *const TValue) -> *mut GCObject {
    unsafe { (*o).value_.gc }
}

/// Extract the integer from an integer `TValue`.
#[inline]
pub(crate) unsafe fn ivalue(o: *const TValue) -> lua_Integer {
    unsafe { (*o).value_.i }
}

/// Extract the float from a float `TValue`.
#[inline]
pub(crate) unsafe fn fltvalue(o: *const TValue) -> lua_Number {
    unsafe { (*o).value_.n }
}

/// Extract the C function pointer from a light-cf `TValue`.
#[inline]
pub(crate) unsafe fn fvalue(o: *const TValue) -> lua_CFunction {
    unsafe { (*o).value_.f }
}

/// Extract the raw pointer from a light-userdata `TValue`.
#[inline]
pub(crate) unsafe fn pvalue(o: *const TValue) -> *mut c_void {
    unsafe { (*o).value_.p }
}

/// Return `true` if the `TValue` is nil.
#[inline]
pub(crate) unsafe fn ttisnil(o: *const TValue) -> bool {
    unsafe { (*o).is_nil() }
}

/// Return `true` if the `TValue` is boolean false.
#[inline]
pub(crate) unsafe fn ttisfalse(o: *const TValue) -> bool {
    unsafe { (*o).is_false() }
}

/// Return `true` if the `TValue` is an integer.
#[inline]
pub(crate) unsafe fn ttisinteger(o: *const TValue) -> bool {
    unsafe { (*o).is_integer() }
}

/// Return `true` if the `TValue` is a float.
#[inline]
pub(crate) unsafe fn ttisfloat(o: *const TValue) -> bool {
    unsafe { (*o).is_float() }
}

/// Return `true` if the `TValue` is any number (integer or float).
#[inline]
pub(crate) unsafe fn ttisnumber(o: *const TValue) -> bool {
    unsafe { (*o).is_number() }
}

/// Return `true` if the `TValue` is any string (short or long).
#[inline]
pub(crate) unsafe fn ttisstring(o: *const TValue) -> bool {
    unsafe { (*o).is_string() }
}

/// Return `true` if the `TValue` is a short string.
#[inline]
pub(crate) unsafe fn ttisshrstring(o: *const TValue) -> bool {
    unsafe { (*o).is_short_string() }
}

/// Return `true` if the `TValue` is a full (heap) userdata.
#[inline]
pub(crate) unsafe fn ttisfulluserdata(o: *const TValue) -> bool {
    unsafe { (*o).is_full_userdata() }
}

/// Return `true` if the `TValue` is a light userdata.
#[inline]
pub(crate) unsafe fn ttislightuserdata(o: *const TValue) -> bool {
    unsafe { (*o).is_light_userdata() }
}

/// Return `true` if the `TValue` is a table.
#[inline]
pub(crate) unsafe fn ttistable(o: *const TValue) -> bool {
    unsafe { (*o).is_table() }
}

/// Return `true` if the `TValue` is a thread/coroutine.
#[inline]
pub(crate) unsafe fn ttisthread(o: *const TValue) -> bool {
    unsafe { (*o).is_thread() }
}

/// Return `true` if the `TValue` is a light C function.
#[inline]
pub(crate) unsafe fn ttislcf(o: *const TValue) -> bool {
    unsafe { (*o).is_light_cf() }
}

/// Return `true` if the `TValue` is a C closure.
#[inline]
pub(crate) unsafe fn ttisCclosure(o: *const TValue) -> bool {
    unsafe { (*o).is_c_closure() }
}

/// Return `true` if the `TValue` is a Lua closure.
#[inline]
pub(crate) unsafe fn ttisLclosure(o: *const TValue) -> bool {
    unsafe { (*o).is_lua_closure() }
}

/// Cast the GC-object pointer in a string `TValue` to `*mut TString`.
#[inline]
pub(crate) unsafe fn tsvalue(o: *const TValue) -> *mut TString {
    unsafe { gcvalue(o).cast() }
}

/// Cast the GC-object pointer in a table `TValue` to `*mut Table`.
#[inline]
pub(crate) unsafe fn hvalue(o: *const TValue) -> *mut Table {
    unsafe { gcvalue(o).cast() }
}

/// Cast the GC-object pointer in a userdata `TValue` to `*mut Udata`.
#[inline]
pub(crate) unsafe fn uvalue(o: *const TValue) -> *mut Udata {
    unsafe { gcvalue(o).cast() }
}

/// Cast the GC-object pointer in a thread `TValue` to `*mut lua_State`.
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
pub(crate) unsafe fn ci_func(ci: *mut CallInfo) -> *mut LClosure {
    unsafe { clLvalue(s2v((*ci).func.p)) }
}

#[inline]
pub(crate) unsafe fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) {
    unsafe { ptr::copy_nonoverlapping(src.cast::<u8>(), dst.cast::<u8>(), n) };
}

#[inline]
pub(crate) unsafe fn cast_byte(v: c_int) -> lu_byte {
    v as lu_byte
}

#[inline]
pub(crate) unsafe fn getabslineinfo(p: *mut Proto) -> *mut AbsLineInfo {
    unsafe { (*p).abslineinfo.cast() }
}

#[inline]
pub(crate) unsafe fn grow_vector<T>(
    L: *mut lua_State,
    block: *mut T,
    nelems: c_int,
    size: &mut c_int,
    limit: c_int,
    what: *const c_char,
) -> *mut T {
    unsafe {
        crate::mem::luaM_growaux_(
            L,
            block.cast(),
            nelems,
            size,
            size_of::<T>() as c_uint,
            limit,
            what,
        )
        .cast()
    }
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
    novariant(tag) == LUA_TNIL
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
        let mut temp = TValue::new_nil();
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
    novariant(tag) as c_int
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
    novariant(tag) as c_int
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

// ============================================================
// Centralized constants (migrated from individual source files)
// ============================================================

// --- numeric ---
pub(crate) const F2Iceil: c_int = 2;
pub(crate) const F2Ieq: c_int = 0;
pub(crate) const F2Ifloor: c_int = 1;
pub(crate) const F64_EXP_BIAS: i32 = 1023;
pub(crate) const F64_EXP_MASK: u64 = 0x7ff_u64 << 52;
pub(crate) const F64_FRAC_MASK: u64 = (1_u64 << 52) - 1;
pub(crate) const F64_MANTISSA_BITS: i32 = 52;
pub(crate) const F64_SIGN_MASK: u64 = 1_u64 << 63;

// ============================================================
// Opcode — VM instruction opcodes (sorted by numeric value)
// ============================================================
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub(crate) enum Opcode {
    Move = 0,
    LoadI = 1,
    LoadF = 2,
    LoadK = 3,
    LoadKx = 4,
    LoadFalse = 5,
    LFalseSkip = 6,
    LoadTrue = 7,
    LoadNil = 8,
    GetUpval = 9,
    SetUpval = 10,
    GetTabUp = 11,
    GetTable = 12,
    GetI = 13,
    GetField = 14,
    SetTabUp = 15,
    SetTable = 16,
    SetI = 17,
    SetField = 18,
    NewTable = 19,
    Self_ = 20,
    AddI = 21,
    AddK = 22,
    SubK = 23,
    MulK = 24,
    ModK = 25,
    PowK = 26,
    DivK = 27,
    IDivK = 28,
    BAndK = 29,
    BOrK = 30,
    BXorK = 31,
    ShlI = 32,
    ShrI = 33,
    Add = 34,
    Sub = 35,
    Mul = 36,
    Mod = 37,
    Pow = 38,
    Div = 39,
    IDiv = 40,
    BAnd = 41,
    BOr = 42,
    BXor = 43,
    Shl = 44,
    Shr = 45,
    MMBin = 46,
    MMBinI = 47,
    MMBinK = 48,
    Unm = 49,
    BNot = 50,
    Not = 51,
    Len = 52,
    Concat = 53,
    Close = 54,
    Tbc = 55,
    Jmp = 56,
    Eq = 57,
    Lt = 58,
    Le = 59,
    EqK = 60,
    EqI = 61,
    LtI = 62,
    LeI = 63,
    GtI = 64,
    GeI = 65,
    Test = 66,
    TestSet = 67,
    Call = 68,
    TailCall = 69,
    Return = 70,
    Return0 = 71,
    Return1 = 72,
    ForLoop = 73,
    ForPrep = 74,
    TForPrep = 75,
    TForCall = 76,
    TForLoop = 77,
    SetList = 78,
    Closure = 79,
    VarArg = 80,
    GetVarg = 81,
    ErrNNil = 82,
    VarArgPrep = 83,
    ExtraArg = 84,
}

impl Opcode {
    #[inline]
    pub(crate) fn as_c_int(self) -> c_int {
        self as c_int
    }
    #[inline]
    pub(crate) fn as_usize(self) -> usize {
        self as usize
    }
    #[inline]
    pub(crate) const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Move),
            1 => Some(Self::LoadI),
            2 => Some(Self::LoadF),
            3 => Some(Self::LoadK),
            4 => Some(Self::LoadKx),
            5 => Some(Self::LoadFalse),
            6 => Some(Self::LFalseSkip),
            7 => Some(Self::LoadTrue),
            8 => Some(Self::LoadNil),
            9 => Some(Self::GetUpval),
            10 => Some(Self::SetUpval),
            11 => Some(Self::GetTabUp),
            12 => Some(Self::GetTable),
            13 => Some(Self::GetI),
            14 => Some(Self::GetField),
            15 => Some(Self::SetTabUp),
            16 => Some(Self::SetTable),
            17 => Some(Self::SetI),
            18 => Some(Self::SetField),
            19 => Some(Self::NewTable),
            20 => Some(Self::Self_),
            21 => Some(Self::AddI),
            22 => Some(Self::AddK),
            23 => Some(Self::SubK),
            24 => Some(Self::MulK),
            25 => Some(Self::ModK),
            26 => Some(Self::PowK),
            27 => Some(Self::DivK),
            28 => Some(Self::IDivK),
            29 => Some(Self::BAndK),
            30 => Some(Self::BOrK),
            31 => Some(Self::BXorK),
            32 => Some(Self::ShlI),
            33 => Some(Self::ShrI),
            34 => Some(Self::Add),
            35 => Some(Self::Sub),
            36 => Some(Self::Mul),
            37 => Some(Self::Mod),
            38 => Some(Self::Pow),
            39 => Some(Self::Div),
            40 => Some(Self::IDiv),
            41 => Some(Self::BAnd),
            42 => Some(Self::BOr),
            43 => Some(Self::BXor),
            44 => Some(Self::Shl),
            45 => Some(Self::Shr),
            46 => Some(Self::MMBin),
            47 => Some(Self::MMBinI),
            48 => Some(Self::MMBinK),
            49 => Some(Self::Unm),
            50 => Some(Self::BNot),
            51 => Some(Self::Not),
            52 => Some(Self::Len),
            53 => Some(Self::Concat),
            54 => Some(Self::Close),
            55 => Some(Self::Tbc),
            56 => Some(Self::Jmp),
            57 => Some(Self::Eq),
            58 => Some(Self::Lt),
            59 => Some(Self::Le),
            60 => Some(Self::EqK),
            61 => Some(Self::EqI),
            62 => Some(Self::LtI),
            63 => Some(Self::LeI),
            64 => Some(Self::GtI),
            65 => Some(Self::GeI),
            66 => Some(Self::Test),
            67 => Some(Self::TestSet),
            68 => Some(Self::Call),
            69 => Some(Self::TailCall),
            70 => Some(Self::Return),
            71 => Some(Self::Return0),
            72 => Some(Self::Return1),
            73 => Some(Self::ForLoop),
            74 => Some(Self::ForPrep),
            75 => Some(Self::TForPrep),
            76 => Some(Self::TForCall),
            77 => Some(Self::TForLoop),
            78 => Some(Self::SetList),
            79 => Some(Self::Closure),
            80 => Some(Self::VarArg),
            81 => Some(Self::GetVarg),
            82 => Some(Self::ErrNNil),
            83 => Some(Self::VarArgPrep),
            84 => Some(Self::ExtraArg),
            _ => None,
        }
    }
}

// Backward-compatible const aliases
pub(crate) const OP_MOVE: c_int = Opcode::Move as c_int;
pub(crate) const OP_LOADI: c_int = Opcode::LoadI as c_int;
pub(crate) const OP_LOADF: c_int = Opcode::LoadF as c_int;
pub(crate) const OP_LOADK: c_int = Opcode::LoadK as c_int;
pub(crate) const OP_LOADKX: c_int = Opcode::LoadKx as c_int;
pub(crate) const OP_LOADFALSE: c_int = Opcode::LoadFalse as c_int;
pub(crate) const OP_LFALSESKIP: c_int = Opcode::LFalseSkip as c_int;
pub(crate) const OP_LOADTRUE: c_int = Opcode::LoadTrue as c_int;
pub(crate) const OP_LOADNIL: c_int = Opcode::LoadNil as c_int;
pub(crate) const OP_GETUPVAL: c_int = Opcode::GetUpval as c_int;
pub(crate) const OP_SETUPVAL: c_int = Opcode::SetUpval as c_int;
pub(crate) const OP_GETTABUP: c_int = Opcode::GetTabUp as c_int;
pub(crate) const OP_GETTABLE: c_int = Opcode::GetTable as c_int;
pub(crate) const OP_GETI: c_int = Opcode::GetI as c_int;
pub(crate) const OP_GETFIELD: c_int = Opcode::GetField as c_int;
pub(crate) const OP_SETTABUP: c_int = Opcode::SetTabUp as c_int;
pub(crate) const OP_SETTABLE: c_int = Opcode::SetTable as c_int;
pub(crate) const OP_SETI: c_int = Opcode::SetI as c_int;
pub(crate) const OP_SETFIELD: c_int = Opcode::SetField as c_int;
pub(crate) const OP_NEWTABLE: c_int = Opcode::NewTable as c_int;
pub(crate) const OP_SELF: c_int = Opcode::Self_ as c_int;
pub(crate) const OP_ADDI: c_int = Opcode::AddI as c_int;
pub(crate) const OP_ADDK: c_int = Opcode::AddK as c_int;
pub(crate) const OP_SUBK: c_int = Opcode::SubK as c_int;
pub(crate) const OP_MULK: c_int = Opcode::MulK as c_int;
pub(crate) const OP_MODK: c_int = Opcode::ModK as c_int;
pub(crate) const OP_POWK: c_int = Opcode::PowK as c_int;
pub(crate) const OP_DIVK: c_int = Opcode::DivK as c_int;
pub(crate) const OP_IDIVK: c_int = Opcode::IDivK as c_int;
pub(crate) const OP_BANDK: c_int = Opcode::BAndK as c_int;
pub(crate) const OP_BORK: c_int = Opcode::BOrK as c_int;
pub(crate) const OP_BXORK: c_int = Opcode::BXorK as c_int;
pub(crate) const OP_SHLI: c_int = Opcode::ShlI as c_int;
pub(crate) const OP_SHRI: c_int = Opcode::ShrI as c_int;
pub(crate) const OP_ADD: c_int = Opcode::Add as c_int;
pub(crate) const OP_SUB: c_int = Opcode::Sub as c_int;
pub(crate) const OP_MUL: c_int = Opcode::Mul as c_int;
pub(crate) const OP_MOD: c_int = Opcode::Mod as c_int;
pub(crate) const OP_POW: c_int = Opcode::Pow as c_int;
pub(crate) const OP_DIV: c_int = Opcode::Div as c_int;
pub(crate) const OP_IDIV: c_int = Opcode::IDiv as c_int;
pub(crate) const OP_BAND: c_int = Opcode::BAnd as c_int;
pub(crate) const OP_BOR: c_int = Opcode::BOr as c_int;
pub(crate) const OP_BXOR: c_int = Opcode::BXor as c_int;
pub(crate) const OP_SHL: c_int = Opcode::Shl as c_int;
pub(crate) const OP_SHR: c_int = Opcode::Shr as c_int;
pub(crate) const OP_MMBIN: c_int = Opcode::MMBin as c_int;
pub(crate) const OP_MMBINI: c_int = Opcode::MMBinI as c_int;
pub(crate) const OP_MMBINK: c_int = Opcode::MMBinK as c_int;
pub(crate) const OP_UNM: c_int = Opcode::Unm as c_int;
pub(crate) const OP_BNOT: c_int = Opcode::BNot as c_int;
pub(crate) const OP_NOT: c_int = Opcode::Not as c_int;
pub(crate) const OP_LEN: c_int = Opcode::Len as c_int;
pub(crate) const OP_CONCAT: c_int = Opcode::Concat as c_int;
pub(crate) const OP_CLOSE: c_int = Opcode::Close as c_int;
pub(crate) const OP_TBC: c_int = Opcode::Tbc as c_int;
pub(crate) const OP_JMP: c_int = Opcode::Jmp as c_int;
pub(crate) const OP_EQ: c_int = Opcode::Eq as c_int;
pub(crate) const OP_LT: c_int = Opcode::Lt as c_int;
pub(crate) const OP_LE: c_int = Opcode::Le as c_int;
pub(crate) const OP_EQK: c_int = Opcode::EqK as c_int;
pub(crate) const OP_EQI: c_int = Opcode::EqI as c_int;
pub(crate) const OP_LTI: c_int = Opcode::LtI as c_int;
pub(crate) const OP_LEI: c_int = Opcode::LeI as c_int;
pub(crate) const OP_GTI: c_int = Opcode::GtI as c_int;
pub(crate) const OP_GEI: c_int = Opcode::GeI as c_int;
pub(crate) const OP_TEST: c_int = Opcode::Test as c_int;
pub(crate) const OP_TESTSET: c_int = Opcode::TestSet as c_int;
pub(crate) const OP_CALL: c_int = Opcode::Call as c_int;
pub(crate) const OP_TAILCALL: c_int = Opcode::TailCall as c_int;
pub(crate) const OP_RETURN: c_int = Opcode::Return as c_int;
pub(crate) const OP_RETURN0: c_int = Opcode::Return0 as c_int;
pub(crate) const OP_RETURN1: c_int = Opcode::Return1 as c_int;
pub(crate) const OP_FORLOOP: c_int = Opcode::ForLoop as c_int;
pub(crate) const OP_FORPREP: c_int = Opcode::ForPrep as c_int;
pub(crate) const OP_TFORPREP: c_int = Opcode::TForPrep as c_int;
pub(crate) const OP_TFORCALL: c_int = Opcode::TForCall as c_int;
pub(crate) const OP_TFORLOOP: c_int = Opcode::TForLoop as c_int;
pub(crate) const OP_SETLIST: c_int = Opcode::SetList as c_int;
pub(crate) const OP_CLOSURE: c_int = Opcode::Closure as c_int;
pub(crate) const OP_VARARG: c_int = Opcode::VarArg as c_int;
pub(crate) const OP_GETVARG: c_int = Opcode::GetVarg as c_int;
pub(crate) const OP_ERRNNIL: c_int = Opcode::ErrNNil as c_int;
pub(crate) const OP_VARARGPREP: c_int = Opcode::VarArgPrep as c_int;
pub(crate) const OP_EXTRAARG: c_int = Opcode::ExtraArg as c_int;

// --- instruction_format ---
pub(crate) const MAXARG_A: c_int = 255;
pub(crate) const MAXARG_Ax: c_int = 33554431;
pub(crate) const MAXARG_B: c_int = 255;
pub(crate) const MAXARG_Bx: c_int = 131071;
pub(crate) const MAXARG_C: c_int = 255;
pub(crate) const MAXARG_sJ: c_int = 33554431;
pub(crate) const MAXARG_vB: c_int = 63;
pub(crate) const MAXARG_vC: c_int = 1023;
pub(crate) const MAXINDEXRK: c_int = MAXARG_B;
pub(crate) const MAX_FSTACK: c_int = MAXARG_A;
pub(crate) const NO_JUMP: c_int = -1;
pub(crate) const NO_REG: c_int = MAX_FSTACK;
pub(crate) const OFFSET_SC: c_int = MAXARG_C >> 1;
pub(crate) const OFFSET_sBx: c_int = MAXARG_Bx >> 1;
pub(crate) const OFFSET_sC: c_int = MAXARG_C >> 1;
pub(crate) const OFFSET_sJ: c_int = MAXARG_sJ >> 1;
pub(crate) const POS_OP: u32 = 0;
pub(crate) const SIZE_A: u32 = 8;
pub(crate) const SIZE_B: u32 = 8;
pub(crate) const SIZE_C: u32 = 8;
pub(crate) const SIZE_OP: u32 = 7;
pub(crate) const SIZE_VB: u32 = 6;
pub(crate) const SIZE_VC: u32 = 10;
pub(crate) const SIZE_vB: u32 = 6;
pub(crate) const SIZE_vC: u32 = 10;
pub(crate) const iABC: u8 = 0;
pub(crate) const iABx: u8 = 2;
pub(crate) const iAsBx: u8 = 3;
pub(crate) const iAx: u8 = 4;
pub(crate) const isJ: u8 = 5;
pub(crate) const ivABC: u8 = 1;
pub(crate) const MAXARG_VC: c_int = ((1u32 << SIZE_VC) - 1) as c_int;
pub(crate) const POS_A: u32 = POS_OP + SIZE_OP;
pub(crate) const POS_AX: u32 = POS_A;
pub(crate) const POS_Ax: u32 = POS_A;
pub(crate) const POS_K: u32 = POS_A + SIZE_A;
pub(crate) const POS_SJ: u32 = POS_A;
pub(crate) const POS_VB: u32 = POS_K + 1;
pub(crate) const POS_VC: u32 = POS_VB + SIZE_VB;
pub(crate) const POS_k: u32 = POS_A + SIZE_A;
pub(crate) const POS_sJ: u32 = POS_A;
pub(crate) const POS_vB: u32 = POS_k + 1;
pub(crate) const POS_vC: u32 = POS_vB + SIZE_vB;
pub(crate) const SIZE_BX: u32 = SIZE_C + SIZE_B + 1;
pub(crate) const SIZE_Bx: u32 = SIZE_C + SIZE_B + 1;
pub(crate) const SIZE_SJ: u32 = SIZE_BX + SIZE_A;
pub(crate) const SIZE_sJ: u32 = SIZE_Bx + SIZE_A;
pub(crate) const MAXARG_BX: c_int = ((1u32 << SIZE_BX) - 1) as c_int;
pub(crate) const MAXARG_SJ: c_int = ((1u32 << SIZE_SJ) - 1) as c_int;
pub(crate) const OFFSET_SBX: c_int = MAXARG_BX >> 1;
pub(crate) const OFFSET_SJ: c_int = MAXARG_SJ >> 1;
pub(crate) const POS_B: u32 = POS_k + 1;
pub(crate) const POS_BX: u32 = POS_K;
pub(crate) const POS_Bx: u32 = POS_k;
pub(crate) const POS_C: u32 = POS_B + SIZE_B;
pub(crate) const SIZE_AX: u32 = SIZE_BX + SIZE_A;
pub(crate) const SIZE_Ax: u32 = SIZE_Bx + SIZE_A;

// --- tokens ---
pub(crate) const FIRST_RESERVED: c_int = u8::MAX as c_int + 1;
pub(crate) const TK_AND: c_int = FIRST_RESERVED;
pub(crate) const TK_BREAK: c_int = TK_AND + 1;
pub(crate) const TK_DO: c_int = TK_BREAK + 1;
pub(crate) const TK_ELSE: c_int = TK_DO + 1;
pub(crate) const TK_ELSEIF: c_int = TK_ELSE + 1;
pub(crate) const TK_END: c_int = TK_ELSEIF + 1;
pub(crate) const TK_FALSE: c_int = TK_END + 1;
pub(crate) const TK_FOR: c_int = TK_FALSE + 1;
pub(crate) const TK_FUNCTION: c_int = TK_FOR + 1;
pub(crate) const TK_GLOBAL: c_int = TK_FUNCTION + 1;
pub(crate) const TK_GOTO: c_int = TK_GLOBAL + 1;
pub(crate) const TK_IF: c_int = TK_GOTO + 1;
pub(crate) const TK_IN: c_int = TK_IF + 1;
pub(crate) const TK_LOCAL: c_int = TK_IN + 1;
pub(crate) const TK_NIL: c_int = TK_LOCAL + 1;
pub(crate) const TK_NOT: c_int = TK_NIL + 1;
pub(crate) const TK_OR: c_int = TK_NOT + 1;
pub(crate) const TK_REPEAT: c_int = TK_OR + 1;
pub(crate) const TK_RETURN: c_int = TK_REPEAT + 1;
pub(crate) const TK_THEN: c_int = TK_RETURN + 1;
pub(crate) const TK_TRUE: c_int = TK_THEN + 1;
pub(crate) const TK_UNTIL: c_int = TK_TRUE + 1;
pub(crate) const TK_WHILE: c_int = TK_UNTIL + 1;
pub(crate) const NUM_RESERVED: usize = (TK_WHILE - FIRST_RESERVED + 1) as usize;
pub(crate) const TK_IDIV: c_int = TK_WHILE + 1;
pub(crate) const TK_CONCAT: c_int = TK_IDIV + 1;
pub(crate) const TK_DOTS: c_int = TK_CONCAT + 1;
pub(crate) const TK_EQ: c_int = TK_DOTS + 1;
pub(crate) const TK_GE: c_int = TK_EQ + 1;
pub(crate) const TK_LE: c_int = TK_GE + 1;
pub(crate) const TK_NE: c_int = TK_LE + 1;
pub(crate) const TK_SHL: c_int = TK_NE + 1;
pub(crate) const TK_SHR: c_int = TK_SHL + 1;
pub(crate) const TK_DBCOLON: c_int = TK_SHR + 1;
pub(crate) const TK_EOS: c_int = TK_DBCOLON + 1;
pub(crate) const TK_FLT: c_int = TK_EOS + 1;
pub(crate) const TK_INT: c_int = TK_FLT + 1;
pub(crate) const TK_NAME: c_int = TK_INT + 1;
pub(crate) const TK_STRING: c_int = TK_NAME + 1;

// --- gc ---
pub(crate) const AGEBITS: u8 = 7;
pub(crate) const CWUFIN: l_mem = 10;
pub(crate) const FINALIZEDBIT: u8 = 6;
pub(crate) const GCSWEEPMAX: l_mem = 20;
pub(crate) const GCSatomic: u8 = 2;
pub(crate) const GCScallfin: u8 = 7;
pub(crate) const GCSenteratomic: u8 = 1;
pub(crate) const GCSpropagate: u8 = 0;
pub(crate) const GCSswpallgc: u8 = 3;
pub(crate) const GCSswpend: u8 = 6;
pub(crate) const GCSswpfinobj: u8 = 4;
pub(crate) const GCSswptobefnz: u8 = 5;
pub(crate) const G_NEW: u8 = 0;
pub(crate) const G_OLD: u8 = 4;
pub(crate) const G_OLD0: u8 = 2;
pub(crate) const G_OLD1: u8 = 3;
pub(crate) const G_SURVIVAL: u8 = 1;
pub(crate) const G_TOUCHED1: u8 = 5;
pub(crate) const G_TOUCHED2: u8 = 6;
pub(crate) const TESTBIT: u8 = 7;

// --- lua_constants ---
pub(crate) const LUA_CMOD_SUFFIX: &str = ".dll";
pub(crate) const LUA_COPYRIGHT: &str = "Lua 5.5.0  Copyright (C) 1994-2025 Lua.org, PUC-Rio";
pub(crate) const LUA_CPATH_DEFAULT: &str =
    "/usr/local/lib/lua/5.5/?;/usr/local/lib/lua/5.5/loadall;./?";
pub(crate) const LUA_CPATH_VAR: &str = "LUA_CPATH";
pub(crate) const LUA_DIRSEP: &str = "/";
pub(crate) const LUA_ENV: &[u8] = b"_ENV\0";
pub(crate) const LUA_EXEC_DIR: &str = "!";
pub(crate) const LUA_FILEHANDLE: &[u8] = b"FILE*\0";
pub(crate) const LUA_FLOORN2I_FLOOR: c_int = 1; // F2Ifloor, distinct from runtime's LUA_FLOORN2I (F2Ieq=0)
pub(crate) const LUA_GLIBK: c_int = 1;
pub(crate) const LUA_GNAME: &[u8] = b"_G\0";
pub(crate) const LUA_HOOKCOUNT: c_int = 3;
pub(crate) const LUA_HOOKLINE: c_int = 2;
pub(crate) const LUA_IGMARK: &str = "-";
pub(crate) const LUA_INIT_VAR: &str = "LUA_INIT";
pub(crate) const LUA_INIT_VAR_VERSION: &str = "LUA_INIT_5_5";
pub(crate) const LUA_LOADED_TABLE: &[u8] = b"_LOADED\0";
pub(crate) const LUA_LOADLIBK: c_int = LUA_GLIBK << 1;
pub(crate) const LUA_LSUBSEP: &str = LUA_DIRSEP;
pub(crate) const LUA_MASKCOUNT: c_int = 8;
pub(crate) const LUA_MASKLINE: c_int = 4;
pub(crate) const LUA_MAXINTEGER: lua_Integer = i64::MAX;
pub(crate) const LUA_MINBUFFER: usize = 32;
pub(crate) const LUA_MININTEGER: lua_Integer = i64::MIN;
pub(crate) const LUA_OFSEP: &str = "_";
pub(crate) const LUA_PATH_DEFAULT: &str = "/usr/local/share/lua/5.5/?.lua;/usr/local/share/lua/5.5/?/init.lua;/usr/local/lib/lua/5.5/?.lua;/usr/local/lib/lua/5.5/?/init.lua;./?.lua;./?/init.lua";
pub(crate) const LUA_PATH_MARK: &str = "?";
pub(crate) const LUA_PATH_SEP: &str = ";";
pub(crate) const LUA_PATH_VAR: &str = "LUA_PATH";
pub(crate) const LUA_POF: &str = "luaopen_";
pub(crate) const LUA_PRELOAD_TABLE: &[u8] = b"_PRELOAD\0";
pub(crate) const LUA_PROMPT: &str = "> ";
pub(crate) const LUA_PROMPT2: &str = ">> ";
pub(crate) const LUA_REFNIL: c_int = -1;
pub(crate) const LUA_SIGNATURE: &[u8] = b"\x1bLua";
pub(crate) const LUA_STRFTIMEOPTIONS: &str =
    "aAbBcCdDeFgGhHIjmMnprRStTuUVwWxXyYzZ%||EcECExEXEyEYOdOeOHOIOmOMOSOuOUOVOwOWOy";
pub(crate) const LUA_VERSION: &[u8] = b"Lua 5.5\0";
pub(crate) const LUA_VERSUFFIX: &str = "_5_5";
pub(crate) const LUA_VNOTABLE: u8 = LUA_TNIL | (3 << 4);
pub(crate) const LUA_COLIBK: c_int = LUA_LOADLIBK << 1;
pub(crate) const LUA_CSUBSEP: &str = LUA_DIRSEP;
pub(crate) const LUA_DBLIBK: c_int = LUA_COLIBK << 1;
pub(crate) const LUA_IOLIBK: c_int = LUA_DBLIBK << 1;
pub(crate) const LUA_MATHLIBK: c_int = LUA_IOLIBK << 1;
pub(crate) const LUA_OSLIBK: c_int = LUA_MATHLIBK << 1;
pub(crate) const LUA_STRLIBK: c_int = LUA_OSLIBK << 1;
pub(crate) const LUA_TABLIBK: c_int = LUA_STRLIBK << 1;
pub(crate) const LUA_UTF8LIBK: c_int = LUA_TABLIBK << 1;

// --- lua_internal ---
pub(crate) const LUAI_MAXSHORTLEN: usize = 40;

// --- errors ---
pub(crate) const ERR_ARRAY_TOO_BIG: &[u8] = b"array too big\0";
pub(crate) const ERR_ASSERTION_FAILED: &[u8] = b"assertion failed!\0";
pub(crate) const ERR_ATTEMPT_CLOSED: &[u8] = b"attempt to use a closed file\0";
pub(crate) const ERR_BAD_SEEK_INT: &[u8] = b"not an integer in proper range\0";
pub(crate) const ERR_BASE_OUT_OF_RANGE: &[u8] = b"base out of range\0";
pub(crate) const ERR_CANNOT_CHANGE_PROTECTED_METATABLE: &[u8] =
    b"cannot change a protected metatable\0";
pub(crate) const ERR_CANNOT_CLOSE_MAIN_THREAD: &[u8] = b"cannot close main thread\0";
pub(crate) const ERR_CANNOT_CLOSE_NORMAL_COROUTINE: &[u8] = b"cannot close a normal coroutine\0";
pub(crate) const ERR_CANNOT_CLOSE_STANDARD_FILE: &[u8] = b"cannot close standard file\0";
pub(crate) const ERR_DATE_NOT_REPRESENTABLE: &[u8] =
    b"date result cannot be represented in this installation\0";
pub(crate) const ERR_DEST_WRAP_AROUND: &[u8] = b"destination wrap around\0";
pub(crate) const ERR_EXPECTED_4_LANES: &[u8] = b"expected exactly 4 lanes\0";
pub(crate) const ERR_EXPECTED_VECTOR_TABLE: &[u8] = b"expected a Lua array table\0";
pub(crate) const ERR_FILE_ALREADY_CLOSED: &[u8] = b"file is already closed\0";
pub(crate) const ERR_FINAL_POSITION_OUT_OF_BOUNDS: &[u8] = b"final position out of bounds\0";
pub(crate) const ERR_I32_RANGE: &[u8] = b"lane value is out of i32 range\0";
pub(crate) const ERR_INDEX_OUT_OF_RANGE: &[u8] = b"index out of range\0";
pub(crate) const ERR_INITIAL_CONTINUATION: &[u8] = b"initial position is a continuation byte\0";
pub(crate) const ERR_INITIAL_POSITION_OUT_OF_BOUNDS: &[u8] = b"initial position out of bounds\0";
pub(crate) const ERR_INTERVAL_EMPTY: &[u8] = b"interval is empty\0";
pub(crate) const ERR_INVALID_CONCAT_VALUE: &[u8] = b"invalid value in table for 'concat'\0";
pub(crate) const ERR_INVALID_FORMAT: &[u8] = b"invalid format\0";
pub(crate) const ERR_INVALID_MODE: &[u8] = b"invalid mode\0";
pub(crate) const ERR_INVALID_ORDER_FUNCTION: &[u8] = b"invalid order function for sorting\0";
pub(crate) const ERR_OUT_OF_BOUNDS: &[u8] = b"out of bounds\0";
pub(crate) const ERR_OUT_OF_RANGE: &[u8] = b"out of range\0";
pub(crate) const ERR_POSITION_OUT_OF_BOUNDS: &[u8] = b"position out of bounds\0";
pub(crate) const ERR_READER_MUST_RETURN_STRING: &[u8] = b"reader function must return a string\0";
pub(crate) const ERR_TIME_NOT_REPRESENTABLE: &[u8] =
    b"time result cannot be represented in this installation\0";
pub(crate) const ERR_TOO_MANY_ARGUMENTS: &[u8] = b"too many arguments\0";
pub(crate) const ERR_TOO_MANY_ARGUMENTS_TO_RESUME: &[u8] = b"too many arguments to resume\0";
pub(crate) const ERR_TOO_MANY_ELEMENTS_TO_MOVE: &[u8] = b"too many elements to move\0";
pub(crate) const ERR_TOO_MANY_NESTED_FUNCTIONS: &[u8] = b"too many nested functions\0";
pub(crate) const ERR_TOO_MANY_READ_ARGS: &[u8] = b"too many arguments\0";
pub(crate) const ERR_TOO_MANY_RESULTS_TO_RESUME: &[u8] = b"too many results to resume\0";
pub(crate) const ERR_TOO_MANY_RESULTS_TO_UNPACK: &[u8] = b"too many results to unpack\0";
pub(crate) const ERR_UNABLE_TMPNAME: &[u8] = b"unable to generate a unique filename\0";
pub(crate) const ERR_VALUE_EXPECTED: &[u8] = b"value expected\0";
pub(crate) const ERR_WRONG_INSERT_ARGS: &[u8] = b"wrong number of arguments to 'insert'\0";
pub(crate) const ERR_WRONG_NUMBER_OF_ARGUMENTS: &[u8] = b"wrong number of arguments\0";
pub(crate) const ERR_ZERO: &[u8] = b"zero\0";

// --- misc ---
pub(crate) const ABSLINEINFO: i8 = -0x80;
pub(crate) const ALPHA: u8 = 1 << 0;
pub(crate) const BITDUMMY: u8 = 1 << 6;
pub(crate) const CAT_ALL: &[u8] = b"all\0";
pub(crate) const CAT_COLLATE: &[u8] = b"collate\0";
pub(crate) const CAT_CTYPE: &[u8] = b"ctype\0";
pub(crate) const CAT_MONETARY: &[u8] = b"monetary\0";
pub(crate) const CAT_NUMERIC: &[u8] = b"numeric\0";
pub(crate) const CAT_TIME: &[u8] = b"time\0";
pub(crate) const CLIBS: &[u8] = b"_CLIBS\0";
pub(crate) const CLOCKS_PER_SEC_VALUE: lua_Number = 1_000_000.0;
pub(crate) const COMMENT: &str = "\t; ";
pub(crate) const COS_DEAD: c_int = 1;
pub(crate) const COS_NORM: c_int = 3;
pub(crate) const COS_RUN: c_int = 0;
pub(crate) const COS_YIELD: c_int = 2;
pub(crate) const DIGIT: u8 = 1 << 1;
pub(crate) const DLMSG: &[u8] = b"dynamic libraries not enabled; check your Lua installation\0";
pub(crate) const EMPTY_STRING: &[u8] = b"\0";
pub(crate) const EOFMARK: &str = "<eof>";
pub(crate) const EOF_VALUE: c_int = -1;
pub(crate) const ERRFUNC: c_int = 2;
pub(crate) const ERRLIB: c_int = 1;
pub(crate) const EXIT_FAILURE_CODE: c_int = 1;
pub(crate) const EXIT_SUCCESS_CODE: c_int = 0;
pub(crate) const FIELD_CHARPATTERN: &[u8] = b"charpattern\0";
pub(crate) const FIELD_CONFIG: &[u8] = b"config\0";
pub(crate) const FIELD_CPATH: &[u8] = b"cpath\0";
pub(crate) const FIELD_F64X4: &[u8] = b"f64x4\0";
pub(crate) const FIELD_HUGE: &[u8] = b"huge\0";
pub(crate) const FIELD_I32X4: &[u8] = b"i32x4\0";
pub(crate) const FIELD_LANES: &[u8] = b"lanes\0";
pub(crate) const FIELD_LEN: &[u8] = b"__len\0";
pub(crate) const FIELD_LOADED: &[u8] = b"loaded\0";
pub(crate) const FIELD_LUA_NOENV: &[u8] = b"LUA_NOENV\0";
pub(crate) const FIELD_MAXINTEGER: &[u8] = b"maxinteger\0";
pub(crate) const FIELD_MININTEGER: &[u8] = b"mininteger\0";
pub(crate) const FIELD_NEWINDEX: &[u8] = b"__newindex\0";
pub(crate) const FIELD_PATH: &[u8] = b"path\0";
pub(crate) const FIELD_PI: &[u8] = b"pi\0";
pub(crate) const FIELD_PRELOAD: &[u8] = b"preload\0";
pub(crate) const FIELD_SEARCHERS: &[u8] = b"searchers\0";
pub(crate) const FIELD_SIMD: &[u8] = b"simd\0";
pub(crate) const FIGS: u32 = 53;
pub(crate) const GDKCONST: u8 = 6;
pub(crate) const GDKREG: u8 = 5;
pub(crate) const HAS_E: i32 = 8;
pub(crate) const HAS_ERROR: i32 = 1;
pub(crate) const HAS_E_CAP: i32 = 16;
pub(crate) const HAS_I: i32 = 2;
pub(crate) const HAS_V: i32 = 4;
pub(crate) const HOOKKEY: &[u8] = b"_HOOKKEY\0";
pub(crate) const I2D_SCALE: lua_Number = 1.0 / ((1_u64 << FIGS) as lua_Number);
pub(crate) const I2D_SHIFT: u32 = 64 - FIGS;
pub(crate) const IOFBF_VALUE: c_int = 0;
pub(crate) const IOLBF_VALUE: c_int = 1;
pub(crate) const IONBF_VALUE: c_int = 2;
pub(crate) const IOPREF_LEN: usize = 4;
pub(crate) const IO_INPUT: &[u8] = b"_IO_input\0";
pub(crate) const IO_OUTPUT: &[u8] = b"_IO_output\0";
pub(crate) const KEY_DAY: &[u8] = b"day\0";
pub(crate) const KEY_HOUR: &[u8] = b"hour\0";
pub(crate) const KEY_ISDST: &[u8] = b"isdst\0";
pub(crate) const KEY_MIN: &[u8] = b"min\0";
pub(crate) const KEY_MONTH: &[u8] = b"month\0";
pub(crate) const KEY_SEC: &[u8] = b"sec\0";
pub(crate) const KEY_WDAY: &[u8] = b"wday\0";
pub(crate) const KEY_YDAY: &[u8] = b"yday\0";
pub(crate) const KEY_YEAR: &[u8] = b"year\0";
pub(crate) const KGC_GENMAJOR: u8 = 2;
pub(crate) const LEVELS1: i32 = 10;
pub(crate) const LEVELS2: i32 = 11;
pub(crate) const LIB_FAIL_ABSENT: &[u8] = b"absent\0";
pub(crate) const LIB_FAIL_OPEN: &[u8] = b"open\0";
pub(crate) const LIMLINEDIFF: c_int = 0x80;
pub(crate) const LSTRFIX: i8 = -2;
pub(crate) const LSTRMEM: i8 = -3;
pub(crate) const LUAC_DATA: &[u8] = b"\x19\x93\r\n\x1a\n";
pub(crate) const LUAC_FORMAT: u8 = 0;
pub(crate) const LUAC_INST: Instruction = 0x1234_5678;
pub(crate) const LUAC_INT: c_int = -0x5678;
pub(crate) const LUAC_NUM: lua_Number = -370.5;
pub(crate) const LUAC_VERSION: u8 = 0x55;
pub(crate) const LUAL_BUFFERSIZE: usize = 8192;
pub(crate) const L_MAXLENNUM: usize = 200;
pub(crate) const MAXABITS: u32 = u32::BITS - 1;
pub(crate) const MAXARGLINE: c_int = 250;
pub(crate) const MAXASIZEB: usize = usize::MAX / (size_of::<Value>() + 1);
pub(crate) const MAXDELTA: usize = u16::MAX as usize;
pub(crate) const MAXHBITS: u32 = MAXABITS - 1;
pub(crate) const MAXHSIZE: u32 = {
    let by_bits = 1usize << MAXHBITS;
    let by_mem = usize::MAX / size_of::<Node>();
    if by_bits < by_mem {
        by_bits as u32
    } else {
        by_mem as u32
    }
};
pub(crate) const MAXIWTHABS: c_int = 128;
pub(crate) const MAXSTRTB: c_int = (c_int::MAX as usize / size_of::<*mut TString>()) as c_int;
pub(crate) const MAXTAGLOOP: c_int = 2000;
pub(crate) const MAXUNICODE: u32 = 0x10FFFF;
pub(crate) const MAXUTF: u32 = 0x7FFF_FFFF;
pub(crate) const MAXVARS: c_int = 200;
pub(crate) const MEMERRMSG: &[u8] = b"not enough memory\0";
pub(crate) const META_CLOSE: &[u8] = b"__close\0";
pub(crate) const META_GC: &[u8] = b"__gc\0";
pub(crate) const META_INDEX: &[u8] = b"__index\0";
pub(crate) const META_METATABLE: &[u8] = b"__metatable\0";
pub(crate) const META_PAIRS: &[u8] = b"__pairs\0";
pub(crate) const META_TOSTRING: &[u8] = b"__tostring\0";
pub(crate) const MINSIZEARRAY: c_int = 4;
pub(crate) const MINSTRTABSIZE: c_int = 128;
pub(crate) const MSG_INVALID: &[u8] = b"invalid UTF-8 code\0";
pub(crate) const NAME_ABS: &[u8] = b"abs\0";
pub(crate) const NAME_ABS_VEC: &[u8] = b"abs\0";
pub(crate) const NAME_ACOS: &[u8] = b"acos\0";
pub(crate) const NAME_ADD: &[u8] = b"add\0";
pub(crate) const NAME_ASIN: &[u8] = b"asin\0";
pub(crate) const NAME_ATAN: &[u8] = b"atan\0";
pub(crate) const NAME_BITAND: &[u8] = b"bitand\0";
pub(crate) const NAME_BITOR: &[u8] = b"bitor\0";
pub(crate) const NAME_BITXOR: &[u8] = b"bitxor\0";
pub(crate) const NAME_CEIL: &[u8] = b"ceil\0";
pub(crate) const NAME_CEIL_VEC: &[u8] = b"ceil\0";
pub(crate) const NAME_CLOCK: &[u8] = b"clock\0";
pub(crate) const NAME_CLOSE: &[u8] = b"close\0";
pub(crate) const NAME_CODEPOINT: &[u8] = b"codepoint\0";
pub(crate) const NAME_CODES: &[u8] = b"codes\0";
pub(crate) const NAME_CONCAT: &[u8] = b"concat\0";
pub(crate) const NAME_COS: &[u8] = b"cos\0";
pub(crate) const NAME_CREATE: &[u8] = b"create\0";
pub(crate) const NAME_DATE: &[u8] = b"date\0";
pub(crate) const NAME_DEG: &[u8] = b"deg\0";
pub(crate) const NAME_DIFFTIME: &[u8] = b"difftime\0";
pub(crate) const NAME_DIV: &[u8] = b"div\0";
pub(crate) const NAME_DOT: &[u8] = b"dot\0";
pub(crate) const NAME_EQ: &[u8] = b"eq\0";
pub(crate) const NAME_EXECUTE: &[u8] = b"execute\0";
pub(crate) const NAME_EXIT: &[u8] = b"exit\0";
pub(crate) const NAME_EXP: &[u8] = b"exp\0";
pub(crate) const NAME_FLOOR: &[u8] = b"floor\0";
pub(crate) const NAME_FLOOR_VEC: &[u8] = b"floor\0";
pub(crate) const NAME_FLUSH: &[u8] = b"flush\0";
pub(crate) const NAME_FMOD: &[u8] = b"fmod\0";
pub(crate) const NAME_FREXP: &[u8] = b"frexp\0";
pub(crate) const NAME_GE: &[u8] = b"ge\0";
pub(crate) const NAME_GETENV: &[u8] = b"getenv\0";
pub(crate) const NAME_GETN: &[u8] = b"getn\0";
pub(crate) const NAME_GT: &[u8] = b"gt\0";
pub(crate) const NAME_INPUT: &[u8] = b"input\0";
pub(crate) const NAME_INSERT: &[u8] = b"insert\0";
pub(crate) const NAME_ISYIELDABLE: &[u8] = b"isyieldable\0";
pub(crate) const NAME_LDEXP: &[u8] = b"ldexp\0";
pub(crate) const NAME_LE: &[u8] = b"le\0";
pub(crate) const NAME_LINES: &[u8] = b"lines\0";
pub(crate) const NAME_LOG: &[u8] = b"log\0";
pub(crate) const NAME_LT: &[u8] = b"lt\0";
pub(crate) const NAME_MAX: &[u8] = b"max\0";
pub(crate) const NAME_MIN: &[u8] = b"min\0";
pub(crate) const NAME_MODF: &[u8] = b"modf\0";
pub(crate) const NAME_MOVE: &[u8] = b"move\0";
pub(crate) const NAME_MUL: &[u8] = b"mul\0";
pub(crate) const NAME_N: &[u8] = b"n\0";
pub(crate) const NAME_NE: &[u8] = b"ne\0";
pub(crate) const NAME_NEG: &[u8] = b"neg\0";
pub(crate) const NAME_OFFSET: &[u8] = b"offset\0";
pub(crate) const NAME_OPEN: &[u8] = b"open\0";
pub(crate) const NAME_OUTPUT: &[u8] = b"output\0";
pub(crate) const NAME_POPEN: &[u8] = b"popen\0";
pub(crate) const NAME_PRODUCT: &[u8] = b"product\0";
pub(crate) const NAME_RAD: &[u8] = b"rad\0";
pub(crate) const NAME_RANDOM: &[u8] = b"random\0";
pub(crate) const NAME_RANDOMSEED: &[u8] = b"randomseed\0";
pub(crate) const NAME_READ: &[u8] = b"read\0";
pub(crate) const NAME_RECIP: &[u8] = b"recip\0";
pub(crate) const NAME_REMOVE: &[u8] = b"remove\0";
pub(crate) const NAME_RENAME: &[u8] = b"rename\0";
pub(crate) const NAME_RESUME: &[u8] = b"resume\0";
pub(crate) const NAME_ROUND_VEC: &[u8] = b"round\0";
pub(crate) const NAME_RUNNING: &[u8] = b"running\0";
pub(crate) const NAME_SEEK: &[u8] = b"seek\0";
pub(crate) const NAME_SETLOCALE: &[u8] = b"setlocale\0";
pub(crate) const NAME_SETVBUF: &[u8] = b"setvbuf\0";
pub(crate) const NAME_SHL: &[u8] = b"shl\0";
pub(crate) const NAME_SHR: &[u8] = b"shr\0";
pub(crate) const NAME_SIMD_MAX: &[u8] = b"max\0";
pub(crate) const NAME_SIMD_MIN: &[u8] = b"min\0";
pub(crate) const NAME_SIMD_SQRT: &[u8] = b"sqrt\0";
pub(crate) const NAME_SIN: &[u8] = b"sin\0";
pub(crate) const NAME_SORT: &[u8] = b"sort\0";
pub(crate) const NAME_SPLAT: &[u8] = b"splat\0";
pub(crate) const NAME_SQRT: &[u8] = b"sqrt\0";
pub(crate) const NAME_STATUS: &[u8] = b"status\0";
pub(crate) const NAME_STDERR: &[u8] = b"stderr\0";
pub(crate) const NAME_STDIN: &[u8] = b"stdin\0";
pub(crate) const NAME_STDOUT: &[u8] = b"stdout\0";
pub(crate) const NAME_SUM: &[u8] = b"sum\0";
pub(crate) const NAME_TAN: &[u8] = b"tan\0";
pub(crate) const NAME_TIME: &[u8] = b"time\0";
pub(crate) const NAME_TMPFILE: &[u8] = b"tmpfile\0";
pub(crate) const NAME_TMPNAME: &[u8] = b"tmpname\0";
pub(crate) const NAME_TOINTEGER: &[u8] = b"tointeger\0";
pub(crate) const NAME_TRUNC_VEC: &[u8] = b"trunc\0";
pub(crate) const NAME_TYPE: &[u8] = b"type\0";
pub(crate) const NAME_ULT: &[u8] = b"ult\0";
pub(crate) const NAME_WRAP: &[u8] = b"wrap\0";
pub(crate) const NAME_WRITE: &[u8] = b"write\0";
pub(crate) const NAME_YIELD: &[u8] = b"yield\0";
pub(crate) const NON_STRING_ERROR: &[u8] = b"(error object is not a string value)\0";
pub(crate) const NOTBITDUMMY: u8 = !BITDUMMY;
pub(crate) const NO_ERROR_OBJECT: &[u8] = b"<no error object>\0";
pub(crate) const NUM_OPCODES: usize = 85;
pub(crate) const OPNAMES: &[&str] = &[
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
pub(crate) const OPR_ADD: c_int = 0;
pub(crate) const OPR_AND: c_int = 19;
pub(crate) const OPR_BAND: c_int = 7;
pub(crate) const OPR_BNOT: c_int = 1;
pub(crate) const OPR_BOR: c_int = 8;
pub(crate) const OPR_BXOR: c_int = 9;
pub(crate) const OPR_CONCAT: c_int = 12;
pub(crate) const OPR_DIV: c_int = 5;
pub(crate) const OPR_EQ: c_int = 13;
pub(crate) const OPR_GE: c_int = 18;
pub(crate) const OPR_GT: c_int = 17;
pub(crate) const OPR_IDIV: c_int = 6;
pub(crate) const OPR_LE: c_int = 15;
pub(crate) const OPR_LEN: c_int = 3;
pub(crate) const OPR_LT: c_int = 14;
pub(crate) const OPR_MINUS: c_int = 0;
pub(crate) const OPR_MOD: c_int = 3;
pub(crate) const OPR_MUL: c_int = 2;
pub(crate) const OPR_NE: c_int = 16;
pub(crate) const OPR_NOBINOPR: c_int = 21;
pub(crate) const OPR_NOT: c_int = 2;
pub(crate) const OPR_NOUNOPR: c_int = 4;
pub(crate) const OPR_OR: c_int = 20;
pub(crate) const OPR_POW: c_int = 4;
pub(crate) const OPR_SHL: c_int = 10;
pub(crate) const OPR_SHR: c_int = 11;
pub(crate) const OPR_SUB: c_int = 1;
pub(crate) const OUTPUT: &str = "luac.out";
pub(crate) const PI: lua_Number = core::f64::consts::PI;
pub(crate) const PRINT: u8 = 1 << 2;
pub(crate) const PROGNAME: &str = "luac";
pub(crate) const RANLIMIT: u32 = 100;
pub(crate) const RDKCONST: u8 = 1;
pub(crate) const RDKCTC: u8 = 4;
pub(crate) const RDKTOCLOSE: u8 = 3;
pub(crate) const RDKVAVAR: u8 = 2;
pub(crate) const RESERVEDSLOT: c_int = 5;
pub(crate) const RTLD_GLOBAL: c_int = 8;
pub(crate) const RTLD_LOCAL: c_int = 4;
pub(crate) const RTLD_NOW: c_int = 2;
pub(crate) const SEEK_CUR_VALUE: c_int = 1;
pub(crate) const SEEK_END_VALUE: c_int = 2;
pub(crate) const SEEK_SET_VALUE: c_int = 0;
pub(crate) const SIZETIMEFMT: usize = 250;
pub(crate) const SPACE: u8 = 1 << 3;
pub(crate) const STR_CLOSED_FILE: &[u8] = b"closed file\0";
pub(crate) const STR_CONSTANT: &[u8] = b"constant\0";
pub(crate) const STR_CTEMP: &[u8] = b"(C temporary)\0";
pub(crate) const STR_C_SOURCE: &[u8] = b"=[C]\0";
pub(crate) const STR_C_WHAT: &[u8] = b"C\0";
pub(crate) const STR_DEAD: &[u8] = b"dead\0";
pub(crate) const STR_EMPTY: &[u8] = b"\0";
pub(crate) const STR_FIELD: &[u8] = b"field\0";
pub(crate) const STR_FILE: &[u8] = b"file\0";
pub(crate) const STR_FILE_CLOSED: &[u8] = b"file (closed)\0";
pub(crate) const STR_FLOAT: &[u8] = b"float\0";
pub(crate) const STR_FOR_ITER: &[u8] = b"for iterator\0";
pub(crate) const STR_GC: &[u8] = b"__gc\0";
pub(crate) const STR_GLOBAL: &[u8] = b"global\0";
pub(crate) const STR_HOOK: &[u8] = b"hook\0";
pub(crate) const STR_INTEGER: &[u8] = b"integer\0";
pub(crate) const STR_INTEGER_INDEX: &[u8] = b"integer index\0";
pub(crate) const STR_LOCAL: &[u8] = b"local\0";
pub(crate) const STR_LUA: &[u8] = b"Lua\0";
pub(crate) const STR_MAIN: &[u8] = b"main\0";
pub(crate) const STR_META: &[u8] = b"metamethod\0";
pub(crate) const STR_METHOD: &[u8] = b"method\0";
pub(crate) const STR_NORMAL: &[u8] = b"normal\0";
pub(crate) const STR_QUESTION: &[u8] = b"?\0";
pub(crate) const STR_RUNNING: &[u8] = b"running\0";
pub(crate) const STR_SUSPENDED: &[u8] = b"suspended\0";
pub(crate) const STR_TEMP: &[u8] = b"(temporary)\0";
pub(crate) const STR_THREAD: &[u8] = b"thread\0";
pub(crate) const STR_UNKNOWN_SOURCE: &[u8] = b"=?\0";
pub(crate) const STR_UPVALUE: &[u8] = b"upvalue\0";
pub(crate) const STR_VARARG: &[u8] = b"(vararg)\0";
pub(crate) const TAB_L: c_int = 4;
pub(crate) const TAB_R: c_int = 1;
pub(crate) const TAB_W: c_int = 2;
pub(crate) const TMP_TEMPLATE: &[u8] = b"/tmp/lua_XXXXXX\0";
pub(crate) const UNARY_PRIORITY: c_int = 12;
pub(crate) const UTF8PATT: &[u8] = b"[\0-\x7F\xC2-\xFD][\x80-\xBF]*";
pub(crate) const VCALL: c_int = 21;
pub(crate) const VCONST: c_int = 13;
pub(crate) const VDKREG: u8 = 0;
pub(crate) const VFALSE: c_int = 3;
pub(crate) const VGLOBAL: c_int = 11;
pub(crate) const VINDEXED: c_int = 14;
pub(crate) const VINDEXI: c_int = 17;
pub(crate) const VINDEXSTR: c_int = 18;
pub(crate) const VINDEXUP: c_int = 16;
pub(crate) const VJMP: c_int = 19;
pub(crate) const VK: c_int = 4;
pub(crate) const VKFLT: c_int = 5;
pub(crate) const VKINT: c_int = 6;
pub(crate) const VKSTR: c_int = 7;
pub(crate) const VLOCAL: c_int = 9;
pub(crate) const VNIL: c_int = 1;
pub(crate) const VNONRELOC: c_int = 8;
pub(crate) const VRELOC: c_int = 20;
pub(crate) const VTRUE: c_int = 2;
pub(crate) const VUPVAL: c_int = 12;
pub(crate) const VVARARG: c_int = 22;
pub(crate) const VVARGIND: c_int = 15;
pub(crate) const VVARGVAR: c_int = 10;
pub(crate) const VVOID: c_int = 0;
pub(crate) const XDIGIT: u8 = 1 << 4;
pub(crate) const atomicstep: l_mem = -2;
pub(crate) const step2minor: l_mem = -1;
pub(crate) const step2pause: l_mem = -3;
pub(crate) const MAXASIZE: u32 = if (1usize << MAXABITS) < MAXASIZEB {
    (1usize << MAXABITS) as u32
} else {
    MAXASIZEB as u32
};
pub(crate) const TAB_RW: c_int = TAB_R | TAB_W;
