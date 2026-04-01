#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::lua_module::lua_Integer;
use crate::string::raw_luaS_new;
use crate::table::{
    raw_luaH_Hgetshortstr, raw_luaH_getint, raw_luaH_getshortstr, raw_luaH_new, raw_luaH_resize,
    raw_luaH_set, raw_luaH_setint,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

const LUA_TNIL: u8 = 0;
const LUA_TBOOLEAN: u8 = 1;
const LUA_TNUMBER: u8 = 3;
const LUA_TSTRING: u8 = 4;
const LUA_TTABLE: u8 = 5;
const LUA_TFUNCTION: u8 = 6;
const LUA_TUSERDATA: u8 = 7;
const LUA_TTHREAD: u8 = 8;
const LUA_TUPVAL: u8 = 9;
const LUA_TPROTO: u8 = 10;
const LUA_TOTALTYPES: usize = 12;

const LUA_VNIL: u8 = 0;
const LUA_VFALSE: u8 = 1;
const LUA_VNUMINT: u8 = 3;
const LUA_VNUMFLT: u8 = 19;
const LUA_VSHRSTR: u8 = 4;
const LUA_VLNGSTR: u8 = 20;
const LUA_VTABLE: u8 = 5;
const LUA_VUSERDATA: u8 = 7;

const BIT_ISCOLLECTABLE: u8 = 1 << 6;
const TM_EQ: c_int = 5;
const TM_ADD: c_int = 6;
const TM_BAND: c_int = 13;
const TM_BOR: c_int = 14;
const TM_BXOR: c_int = 15;
const TM_SHL: c_int = 16;
const TM_SHR: c_int = 17;
const TM_UNM: c_int = 18;
const TM_BNOT: c_int = 19;
const TM_LT: c_int = 20;
const TM_LE: c_int = 21;
const TM_CONCAT: c_int = 22;
const TM_N: usize = 25;
const STRCACHE_N: usize = 53;
const STRCACHE_M: usize = 2;
const LUA_NUMTYPES: usize = 9;
const LUA_GCPN: usize = 6;

const PF_VAHID: u8 = 1;
const PF_VATAB: u8 = 2;

const CIST_C: u32 = 1 << 15;
const CIST_HOOKED: u32 = 1 << 20;
const LUA_FLOORN2I: c_int = 0;

#[derive(Copy, Clone)]
#[repr(C)]
union Value {
    gc: *mut GCObject,
    p: *mut c_void,
    f: Option<unsafe extern "C-unwind" fn(*mut lua_State) -> c_int>,
    i: lua_Integer,
    n: f64,
    ub: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TValue {
    value_: Value,
    tt_: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
union StackValue {
    val: TValue,
    tbclist: StackValueTbc,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct StackValueTbc {
    value_: Value,
    tt_: u8,
    delta: u16,
}

type StkId = *mut StackValue;

#[derive(Copy, Clone)]
#[repr(C)]
union StkIdRel {
    p: StkId,
    offset: isize,
}

#[repr(C)]
struct GCObject {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
}

#[repr(C)]
union TStringUnion {
    lnglen: usize,
    hnext: *mut TString,
}

#[repr(C)]
struct TString {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    extra: u8,
    shrlen: i8,
    hash: u32,
    u: TStringUnion,
    contents: *mut c_char,
    falloc: Option<unsafe extern "C-unwind" fn(*mut c_void, *mut c_void, usize, usize) -> *mut c_void>,
    ud: *mut c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct NodeKey {
    value_: Value,
    tt_: u8,
    key_tt: u8,
    next: c_int,
    key_val: Value,
}

#[repr(C)]
union Node {
    u: NodeKey,
    i_val: TValue,
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct SyncCharPtr(*const c_char);

unsafe impl Sync for SyncCharPtr {}

#[repr(C)]
struct Table {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    flags: u8,
    lsizenode: u8,
    asize: u32,
    array: *mut Value,
    node: *mut Node,
    metatable: *mut Table,
    gclist: *mut GCObject,
}

#[repr(C)]
union UValue {
    uv: TValue,
    n: f64,
    p: *mut c_void,
    i: lua_Integer,
    l: isize,
}

#[repr(C)]
struct Udata {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    nuvalue: u16,
    len: usize,
    metatable: *mut Table,
    gclist: *mut GCObject,
    uv: [UValue; 1],
}

#[repr(C)]
struct Proto {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    numparams: u8,
    flag: u8,
    maxstacksize: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct CallInfoLua {
    savedpc: *const u32,
    trap: c_int,
    nextraargs: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct CallInfoC {
    k: Option<unsafe extern "C-unwind" fn(*mut lua_State, c_int, isize) -> c_int>,
    old_errfunc: isize,
    ctx: isize,
}

#[derive(Copy, Clone)]
#[repr(C)]
union CallInfoU {
    l: CallInfoLua,
    c: CallInfoC,
}

#[derive(Copy, Clone)]
#[repr(C)]
union CallInfoU2 {
    funcidx: c_int,
    nyield: c_int,
    nres: c_int,
}

#[repr(C)]
pub struct CallInfo {
    func: StkIdRel,
    top: StkIdRel,
    previous: *mut CallInfo,
    next: *mut CallInfo,
    u: CallInfoU,
    u2: CallInfoU2,
    callstatus: u32,
}

#[repr(C)]
struct stringtable {
    hash: *mut *mut TString,
    nuse: c_int,
    size: c_int,
}

#[repr(C)]
struct TransferInfo {
    ftransfer: c_int,
    ntransfer: c_int,
}

#[repr(C)]
struct lua_longjmp {
    _private: [u8; 0],
}

#[repr(C)]
struct global_State {
    frealloc: *mut c_void,
    ud: *mut c_void,
    GCtotalbytes: isize,
    GCdebt: isize,
    GCmarked: isize,
    GCmajorminor: isize,
    strt: stringtable,
    l_registry: TValue,
    nilvalue: TValue,
    seed: u32,
    gcparams: [u8; LUA_GCPN],
    currentwhite: u8,
    gcstate: u8,
    gckind: u8,
    gcstopem: u8,
    gcstp: u8,
    gcemergency: u8,
    allgc: *mut GCObject,
    sweepgc: *mut *mut GCObject,
    finobj: *mut GCObject,
    gray: *mut GCObject,
    grayagain: *mut GCObject,
    weak: *mut GCObject,
    ephemeron: *mut GCObject,
    allweak: *mut GCObject,
    tobefnz: *mut GCObject,
    fixedgc: *mut GCObject,
    survival: *mut GCObject,
    old1: *mut GCObject,
    reallyold: *mut GCObject,
    firstold1: *mut GCObject,
    finobjsur: *mut GCObject,
    finobjold1: *mut GCObject,
    finobjrold: *mut GCObject,
    twups: *mut lua_State,
    panic: *mut c_void,
    memerrmsg: *mut TString,
    tmname: [*mut TString; TM_N],
    mt: [*mut Table; LUA_NUMTYPES],
    strcache: [[*mut TString; STRCACHE_M]; STRCACHE_N],
    warnf: *mut c_void,
    ud_warn: *mut c_void,
}

#[repr(C)]
pub struct lua_State {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    allowhook: u8,
    status: u8,
    top: StkIdRel,
    l_g: *mut global_State,
    ci: *mut CallInfo,
    stack_last: StkIdRel,
    stack: StkIdRel,
    openupval: *mut c_void,
    tbclist: StkIdRel,
    gclist: *mut GCObject,
    twups: *mut lua_State,
    errorJmp: *mut lua_longjmp,
    base_ci: CallInfo,
    hook: *mut c_void,
    errfunc: isize,
    nCcalls: u32,
    oldpc: c_int,
    nci: c_int,
    basehookcount: c_int,
    hookcount: c_int,
    hookmask: c_int,
    transferinfo: TransferInfo,
}

unsafe extern "C-unwind" {
    fn luaC_fix(state: *mut lua_State, o: *mut GCObject);
    fn luaC_step(state: *mut lua_State);
    fn luaD_call(state: *mut lua_State, func: StkId, nresults: c_int);
    fn luaD_callnoyield(state: *mut lua_State, func: StkId, nresults: c_int);
    fn luaD_growstack(state: *mut lua_State, n: c_int, raiseerror: c_int) -> c_int;
    fn luaG_tointerror(state: *mut lua_State, p1: *const TValue, p2: *const TValue) -> !;
    fn luaG_opinterror(
        state: *mut lua_State,
        p1: *const TValue,
        p2: *const TValue,
        msg: *const c_char,
    ) -> !;
    fn luaG_concaterror(state: *mut lua_State, p1: *const TValue, p2: *const TValue) -> !;
    fn luaG_ordererror(state: *mut lua_State, p1: *const TValue, p2: *const TValue) -> !;
    fn luaG_runerror(state: *mut lua_State, fmt: *const c_char, ...) -> !;
    fn luaV_tointegerns(obj: *const TValue, p: *mut lua_Integer, mode: c_int) -> c_int;
}

static UDATATYPE_NAME: &[u8] = b"userdata\0";
static NO_VALUE: &[u8] = b"no value\0";
static NIL_NAME: &[u8] = b"nil\0";
static BOOLEAN_NAME: &[u8] = b"boolean\0";
static NUMBER_NAME: &[u8] = b"number\0";
static STRING_NAME: &[u8] = b"string\0";
static TABLE_NAME: &[u8] = b"table\0";
static FUNCTION_NAME: &[u8] = b"function\0";
static THREAD_NAME: &[u8] = b"thread\0";
static UPVALUE_NAME: &[u8] = b"upvalue\0";
static PROTO_NAME: &[u8] = b"proto\0";

pub(crate) static luaT_typenames_: [SyncCharPtr; LUA_TOTALTYPES] = [
    SyncCharPtr(NO_VALUE.as_ptr().cast()),
    SyncCharPtr(NIL_NAME.as_ptr().cast()),
    SyncCharPtr(BOOLEAN_NAME.as_ptr().cast()),
    SyncCharPtr(UDATATYPE_NAME.as_ptr().cast()),
    SyncCharPtr(NUMBER_NAME.as_ptr().cast()),
    SyncCharPtr(STRING_NAME.as_ptr().cast()),
    SyncCharPtr(TABLE_NAME.as_ptr().cast()),
    SyncCharPtr(FUNCTION_NAME.as_ptr().cast()),
    SyncCharPtr(UDATATYPE_NAME.as_ptr().cast()),
    SyncCharPtr(THREAD_NAME.as_ptr().cast()),
    SyncCharPtr(UPVALUE_NAME.as_ptr().cast()),
    SyncCharPtr(PROTO_NAME.as_ptr().cast()),
];

#[inline]
unsafe fn G(state: *mut lua_State) -> *mut global_State {
    unsafe { (*state).l_g }
}

#[inline]
unsafe fn s2v(slot: StkId) -> *mut TValue {
    unsafe { ptr::addr_of_mut!((*slot).val) }
}

#[inline]
unsafe fn settt(obj: *mut TValue, tt: u8) {
    unsafe { (*obj).tt_ = tt };
}

#[inline]
unsafe fn setobj(dst: *mut TValue, src: *const TValue) {
    unsafe {
        (*dst).value_ = (*src).value_;
        (*dst).tt_ = (*src).tt_;
    }
}

#[inline]
unsafe fn setobj2s(_state: *mut lua_State, dst: StkId, src: *const TValue) {
    unsafe { setobj(s2v(dst), src) };
}

#[inline]
unsafe fn setobjs2s(state: *mut lua_State, dst: StkId, src: StkId) {
    unsafe { setobj2s(state, dst, s2v(src)) };
}

#[inline]
unsafe fn setnilvalue(obj: *mut TValue) {
    unsafe { settt(obj, LUA_VNIL) };
}

#[inline]
unsafe fn setivalue(obj: *mut TValue, x: lua_Integer) {
    unsafe {
        (*obj).value_.i = x;
        settt(obj, LUA_VNUMINT);
    }
}

#[inline]
unsafe fn setfltvalue(obj: *mut TValue, x: f64) {
    unsafe {
        (*obj).value_.n = x;
        settt(obj, LUA_VNUMFLT);
    }
}

#[inline]
unsafe fn setsvalue(state: *mut lua_State, obj: *mut TValue, x: *mut TString) {
    let _ = state;
    unsafe {
        (*obj).value_.gc = x.cast();
        settt(obj, (*x).tt | BIT_ISCOLLECTABLE);
    }
}

#[inline]
unsafe fn sethvalue(state: *mut lua_State, obj: *mut TValue, x: *mut Table) {
    let _ = state;
    unsafe {
        (*obj).value_.gc = x.cast();
        settt(obj, (*x).tt | BIT_ISCOLLECTABLE);
    }
}

#[inline]
unsafe fn rawtt(obj: *const TValue) -> u8 {
    unsafe { (*obj).tt_ }
}

#[inline]
unsafe fn novariant(tt: u8) -> u8 {
    tt & 0x0f
}

#[inline]
unsafe fn ttype(obj: *const TValue) -> u8 {
    unsafe { novariant(rawtt(obj)) }
}

#[inline]
unsafe fn ttypetag(obj: *const TValue) -> u8 {
    unsafe { rawtt(obj) & 0x3f }
}

#[inline]
unsafe fn ttisnil(obj: *const TValue) -> bool {
    unsafe { ttype(obj) == LUA_TNIL }
}

#[inline]
unsafe fn ttisstring(obj: *const TValue) -> bool {
    unsafe { ttype(obj) == LUA_TSTRING }
}

#[inline]
unsafe fn ttistable(obj: *const TValue) -> bool {
    unsafe { rawtt(obj) == (LUA_VTABLE | BIT_ISCOLLECTABLE) }
}

#[inline]
unsafe fn ttisfulluserdata(obj: *const TValue) -> bool {
    unsafe { rawtt(obj) == (LUA_VUSERDATA | BIT_ISCOLLECTABLE) }
}

#[inline]
unsafe fn ttisinteger(obj: *const TValue) -> bool {
    unsafe { rawtt(obj) == LUA_VNUMINT }
}

#[inline]
unsafe fn ttisnumber(obj: *const TValue) -> bool {
    unsafe { ttype(obj) == LUA_TNUMBER }
}

#[inline]
unsafe fn tsvalue(obj: *const TValue) -> *mut TString {
    unsafe { (*obj).value_.gc.cast() }
}

#[inline]
unsafe fn hvalue(obj: *const TValue) -> *mut Table {
    unsafe { (*obj).value_.gc.cast() }
}

#[inline]
unsafe fn uvalue(obj: *const TValue) -> *mut Udata {
    unsafe { (*obj).value_.gc.cast() }
}

#[inline]
unsafe fn ivalue(obj: *const TValue) -> lua_Integer {
    unsafe { (*obj).value_.i }
}

#[inline]
unsafe fn strisshr(ts: *const TString) -> bool {
    unsafe { (*ts).shrlen >= 0 }
}

#[inline]
unsafe fn rawgetshrstr(ts: *const TString) -> *const c_char {
    unsafe { ptr::addr_of!((*ts).contents).cast() }
}

#[inline]
unsafe fn getstr(ts: *const TString) -> *const c_char {
    if unsafe { strisshr(ts) } {
        unsafe { rawgetshrstr(ts) }
    } else {
        unsafe { (*ts).contents.cast_const() }
    }
}

#[inline]
unsafe fn isLuacode(ci: *mut CallInfo) -> bool {
    unsafe { ((*ci).callstatus & (CIST_C | CIST_HOOKED)) == 0 }
}

#[inline]
unsafe fn savestack(state: *mut lua_State, pt: StkId) -> isize {
    unsafe { pt.cast::<u8>().offset_from((*state).stack.p.cast::<u8>()) }
}

#[inline]
unsafe fn restorestack(state: *mut lua_State, n: isize) -> StkId {
    unsafe { (*state).stack.p.cast::<u8>().offset(n).cast() }
}

#[inline]
fn tagisempty(tag: u8) -> bool {
    (tag & 0x0f) == LUA_TNIL
}

#[inline]
fn tagisfalse(tag: u8) -> bool {
    tag == LUA_VFALSE || (tag & 0x0f) == LUA_TNIL
}

#[inline]
unsafe fn checkstackp(state: *mut lua_State, n: c_int, where_: &mut StkId) {
    if unsafe { (*state).stack_last.p.offset_from((*state).top.p) as c_int <= n } {
        let t = unsafe { savestack(state, *where_) };
        unsafe { luaD_growstack(state, n, 1) };
        *where_ = unsafe { restorestack(state, t) };
    }
}

#[inline]
unsafe fn luaC_checkGC(state: *mut lua_State) {
    if unsafe { (*G(state)).GCdebt <= 0 } {
        unsafe { luaC_step(state) };
    }
}

pub(crate) unsafe fn luaT_init(state: *mut lua_State) {
    static EVENT_NAMES: [&[u8]; TM_N] = [
        b"__index\0",
        b"__newindex\0",
        b"__gc\0",
        b"__mode\0",
        b"__len\0",
        b"__eq\0",
        b"__add\0",
        b"__sub\0",
        b"__mul\0",
        b"__mod\0",
        b"__pow\0",
        b"__div\0",
        b"__idiv\0",
        b"__band\0",
        b"__bor\0",
        b"__bxor\0",
        b"__shl\0",
        b"__shr\0",
        b"__unm\0",
        b"__bnot\0",
        b"__lt\0",
        b"__le\0",
        b"__concat\0",
        b"__call\0",
        b"__close\0",
    ];
    let g = unsafe { G(state) };
    for (i, name) in EVENT_NAMES.iter().enumerate() {
        let ts = unsafe { raw_luaS_new(state.cast(), name.as_ptr().cast()).cast::<TString>() };
        unsafe {
            (&mut (*g).tmname)[i] = ts;
            luaC_fix(state, ts.cast());
        }
    }
}

pub(crate) unsafe fn raw_luaT_init(state: *mut c_void) {
    unsafe { luaT_init(state.cast()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_gettm(
    events: *mut Table,
    event: c_int,
    ename: *mut TString,
) -> *const TValue {
    let tm = unsafe { raw_luaH_Hgetshortstr(events.cast(), ename.cast()).cast::<TValue>() };
    if unsafe { ttisnil(tm) } {
        unsafe { (*events).flags |= (1u8) << event };
        ptr::null()
    } else {
        tm
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_gettmbyobj(
    state: *mut lua_State,
    o: *const TValue,
    event: c_int,
) -> *const TValue {
    let mt = match unsafe { ttype(o) } {
        LUA_TTABLE => unsafe { (*hvalue(o)).metatable },
        LUA_TUSERDATA => unsafe { (*uvalue(o)).metatable },
        other => unsafe { (&(*G(state)).mt)[other as usize] },
    };
    if mt.is_null() {
        unsafe { ptr::addr_of!((*G(state)).nilvalue) }
    } else {
        unsafe {
            raw_luaH_Hgetshortstr(mt.cast(), (&(*G(state)).tmname)[event as usize].cast())
                .cast::<TValue>()
        }
    }
}

pub(crate) unsafe fn luaT_objtypename(
    state: *mut lua_State,
    o: *const TValue,
) -> *const c_char {
    let mut mt = ptr::null_mut();
    if unsafe { ttistable(o) } {
        mt = unsafe { (*hvalue(o)).metatable };
    } else if unsafe { ttisfulluserdata(o) } {
        mt = unsafe { (*uvalue(o)).metatable };
    }
    if !mt.is_null() {
        let name = unsafe {
            raw_luaH_Hgetshortstr(
                mt.cast(),
                raw_luaS_new(state.cast(), c"__name".as_ptr()).cast(),
            )
            .cast::<TValue>()
        };
        if unsafe { ttisstring(name) } {
            return unsafe { getstr(tsvalue(name)) };
        }
    }
    luaT_typenames_[unsafe { ttype(o) as usize + 1 }].0
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_callTM(
    state: *mut lua_State,
    f: *const TValue,
    p1: *const TValue,
    p2: *const TValue,
    p3: *const TValue,
) {
    let func = unsafe { (*state).top.p };
    unsafe {
        setobj2s(state, func, f);
        setobj2s(state, func.add(1), p1);
        setobj2s(state, func.add(2), p2);
        setobj2s(state, func.add(3), p3);
        (*state).top.p = func.add(4);
    }
    if unsafe { isLuacode((*state).ci) } {
        unsafe { luaD_call(state, func, 0) };
    } else {
        unsafe { luaD_callnoyield(state, func, 0) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_callTMres(
    state: *mut lua_State,
    f: *const TValue,
    p1: *const TValue,
    p2: *const TValue,
    res: StkId,
) -> u8 {
    let result = unsafe { savestack(state, res) };
    let func = unsafe { (*state).top.p };
    unsafe {
        setobj2s(state, func, f);
        setobj2s(state, func.add(1), p1);
        setobj2s(state, func.add(2), p2);
        (*state).top.p = (*state).top.p.add(3);
    }
    if unsafe { isLuacode((*state).ci) } {
        unsafe { luaD_call(state, func, 1) };
    } else {
        unsafe { luaD_callnoyield(state, func, 1) };
    }
    let res = unsafe { restorestack(state, result) };
    unsafe {
        (*state).top.p = (*state).top.p.sub(1);
        setobjs2s(state, res, (*state).top.p);
    }
    unsafe { ttypetag(s2v(res)) }
}

unsafe fn callbinTM(
    state: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
    res: StkId,
    event: c_int,
) -> c_int {
    let mut tm = unsafe { luaT_gettmbyobj(state, p1, event) };
    if unsafe { ttisnil(tm) } {
        tm = unsafe { luaT_gettmbyobj(state, p2, event) };
    }
    if unsafe { ttisnil(tm) } {
        -1
    } else {
        unsafe { luaT_callTMres(state, tm, p1, p2, res) as c_int }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_trybinTM(
    state: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
    res: StkId,
    event: c_int,
) {
    if unsafe { callbinTM(state, p1, p2, res, event) } < 0 {
        match event {
            TM_BAND | TM_BOR | TM_BXOR | TM_SHL | TM_SHR | TM_BNOT => {
                if unsafe { ttisnumber(p1) && ttisnumber(p2) } {
                    unsafe { luaG_tointerror(state, p1, p2) };
                } else {
                    unsafe {
                        luaG_opinterror(state, p1, p2, c"perform bitwise operation on".as_ptr())
                    };
                }
            }
            _ => unsafe { luaG_opinterror(state, p1, p2, c"perform arithmetic on".as_ptr()) },
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_tryconcatTM(state: *mut lua_State) {
    let p1 = unsafe { (*state).top.p.sub(2) };
    if unsafe { callbinTM(state, s2v(p1), s2v(p1.add(1)), p1, TM_CONCAT) } < 0 {
        unsafe { luaG_concaterror(state, s2v(p1), s2v(p1.add(1))) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_trybinassocTM(
    state: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
    flip: c_int,
    res: StkId,
    event: c_int,
) {
    if flip != 0 {
        unsafe { luaT_trybinTM(state, p2, p1, res, event) };
    } else {
        unsafe { luaT_trybinTM(state, p1, p2, res, event) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_trybiniTM(
    state: *mut lua_State,
    p1: *const TValue,
    i2: lua_Integer,
    flip: c_int,
    res: StkId,
    event: c_int,
) {
    let mut aux = TValue {
        value_: Value { i: 0 },
        tt_: LUA_VNIL,
    };
    unsafe { setivalue(ptr::addr_of_mut!(aux), i2) };
    unsafe { luaT_trybinassocTM(state, p1, ptr::addr_of!(aux), flip, res, event) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_callorderTM(
    state: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
    event: c_int,
) -> c_int {
    let tag = unsafe { callbinTM(state, p1, p2, (*state).top.p, event) };
    if tag >= 0 {
        (!tagisfalse(tag as u8)) as c_int
    } else {
        unsafe { luaG_ordererror(state, p1, p2) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_callorderiTM(
    state: *mut lua_State,
    mut p1: *const TValue,
    v2: c_int,
    flip: c_int,
    isfloat: c_int,
    event: c_int,
) -> c_int {
    let mut aux = TValue {
        value_: Value { i: 0 },
        tt_: LUA_VNIL,
    };
    if isfloat != 0 {
        unsafe { setfltvalue(ptr::addr_of_mut!(aux), v2 as f64) };
    } else {
        unsafe { setivalue(ptr::addr_of_mut!(aux), v2 as lua_Integer) };
    }
    let p2 = if flip != 0 {
        let old = p1;
        p1 = ptr::addr_of!(aux);
        old
    } else {
        ptr::addr_of!(aux)
    };
    unsafe { luaT_callorderTM(state, p1, p2, event) }
}

unsafe fn createvarargtab(state: *mut lua_State, f: StkId, n: c_int) {
    let mut key = TValue {
        value_: Value { i: 0 },
        tt_: LUA_VNIL,
    };
    let mut value = TValue {
        value_: Value { i: 0 },
        tt_: LUA_VNIL,
    };
    let t = unsafe { raw_luaH_new(state.cast()).cast::<Table>() };
    unsafe {
        sethvalue(state, s2v((*state).top.p), t);
        (*state).top.p = (*state).top.p.add(1);
        raw_luaH_resize(state.cast(), t.cast(), n as u32, 1);
        setsvalue(
            state,
            ptr::addr_of_mut!(key),
            raw_luaS_new(state.cast(), c"n".as_ptr()).cast(),
        );
        setivalue(ptr::addr_of_mut!(value), n as lua_Integer);
        raw_luaH_set(
            state.cast(),
            t.cast(),
            ptr::addr_of!(key).cast(),
            ptr::addr_of_mut!(value).cast(),
        );
    }
    for i in 0..n {
        unsafe {
            raw_luaH_setint(
                state.cast(),
                t.cast(),
                (i + 1) as lua_Integer,
                s2v(f.add(i as usize)).cast(),
            )
        };
    }
    unsafe { luaC_checkGC(state) };
}

unsafe fn buildhiddenargs(
    state: *mut lua_State,
    ci: *mut CallInfo,
    p: *const Proto,
    totalargs: c_int,
    nfixparams: c_int,
    nextra: c_int,
) {
    unsafe {
        (*ci).u.l.nextraargs = nextra;
        luaD_growstack(state, (*p).maxstacksize as c_int + 1, 1);
        setobjs2s(state, (*state).top.p, (*ci).func.p);
        (*state).top.p = (*state).top.p.add(1);
    }
    for i in 1..=nfixparams {
        unsafe {
            setobjs2s(state, (*state).top.p, (*ci).func.p.add(i as usize));
            (*state).top.p = (*state).top.p.add(1);
            setnilvalue(s2v((*ci).func.p.add(i as usize)));
        }
    }
    unsafe {
        (*ci).func.p = (*ci).func.p.add((totalargs + 1) as usize);
        (*ci).top.p = (*ci).top.p.add((totalargs + 1) as usize);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_adjustvarargs(
    state: *mut lua_State,
    ci: *mut CallInfo,
    p: *const Proto,
) {
    let totalargs = unsafe { (*state).top.p.offset_from((*ci).func.p) as c_int - 1 };
    let nfixparams = unsafe { (*p).numparams as c_int };
    let nextra = totalargs - nfixparams;
    if unsafe { (*p).flag & PF_VATAB } != 0 {
        unsafe { createvarargtab(state, (*ci).func.p.add((nfixparams + 1) as usize), nextra) };
        unsafe {
            setobjs2s(
                state,
                (*ci).func.p.add((nfixparams + 1) as usize),
                (*state).top.p.sub(1),
            )
        };
    } else {
        unsafe { buildhiddenargs(state, ci, p, totalargs, nfixparams, nextra) };
        unsafe { setnilvalue(s2v((*ci).func.p.add((nfixparams + 1) as usize))) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_getvararg(ci: *mut CallInfo, ra: StkId, rc: *mut TValue) {
    let nextra = unsafe { (*ci).u.l.nextraargs };
    let mut n = 0;
    if unsafe { luaV_tointegerns(rc, ptr::addr_of_mut!(n), LUA_FLOORN2I) } != 0 {
        if (n as u64).wrapping_sub(1) < nextra as u64 {
            let slot = unsafe { (*ci).func.p.sub(nextra as usize).add((n - 1) as usize) };
            unsafe { setobjs2s(ptr::null_mut(), ra, slot) };
            return;
        }
    } else if unsafe { ttisstring(rc) } {
        let s = unsafe { getstr(tsvalue(rc)) };
        if unsafe { *s == b'n' as c_char && *s.add(1) == 0 } {
            unsafe { setivalue(s2v(ra), nextra as lua_Integer) };
            return;
        }
    }
    unsafe { setnilvalue(s2v(ra)) };
}

unsafe fn getnumargs(state: *mut lua_State, ci: *mut CallInfo, h: *mut Table) -> c_int {
    if h.is_null() {
        unsafe { (*ci).u.l.nextraargs }
    } else {
        let mut res = TValue {
            value_: Value { i: 0 },
            tt_: LUA_VNIL,
        };
        if unsafe {
            raw_luaH_getshortstr(
                h.cast(),
                raw_luaS_new(state.cast(), c"n".as_ptr()).cast(),
                ptr::addr_of_mut!(res).cast(),
            )
        } != LUA_VNUMINT
            || unsafe { ivalue(ptr::addr_of!(res)) as u64 > (c_int::MAX as u64 / 2) }
        {
            unsafe { luaG_runerror(state, c"vararg table has no proper 'n'".as_ptr()) };
        }
        unsafe { ivalue(ptr::addr_of!(res)) as c_int }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaT_getvarargs(
    state: *mut lua_State,
    ci: *mut CallInfo,
    mut where_: StkId,
    mut wanted: c_int,
    vatab: c_int,
) {
    let h = if vatab < 0 {
        ptr::null_mut()
    } else {
        unsafe { hvalue(s2v((*ci).func.p.add((vatab + 1) as usize))) }
    };
    let nargs = unsafe { getnumargs(state, ci, h) };
    let touse;
    if wanted < 0 {
        touse = nargs;
        wanted = nargs;
        unsafe { checkstackp(state, nargs, &mut where_) };
        unsafe { (*state).top.p = where_.add(nargs as usize) };
    } else {
        touse = if nargs > wanted { wanted } else { nargs };
    }
    let mut i = 0;
    if h.is_null() {
        while i < touse {
            unsafe {
                setobjs2s(
                    state,
                    where_.add(i as usize),
                    (*ci).func.p.sub(nargs as usize).add(i as usize),
                )
            };
            i += 1;
        }
    } else {
        while i < touse {
            let tag = unsafe {
                raw_luaH_getint(
                    h.cast(),
                    (i + 1) as lua_Integer,
                    s2v(where_.add(i as usize)).cast(),
                )
            };
            if tagisempty(tag) {
                unsafe { setnilvalue(s2v(where_.add(i as usize))) };
            }
            i += 1;
        }
    }
    while i < wanted {
        unsafe { setnilvalue(s2v(where_.add(i as usize))) };
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aux_rs::{luaL_checkversion_, luaL_newstate};
    use crate::luaffi::{LUA_VERSION_NUM, LUAL_NUMSIZES, lua_close};
    use crate::test_support::run_lua_test;

    #[test]
    fn tm_names_are_initialized_and_fixed() {
        let state = unsafe { luaL_newstate() }.cast::<lua_State>();
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state.cast(), LUA_VERSION_NUM, LUAL_NUMSIZES);
            let g = G(state);
            let add = (&(*g).tmname)[TM_ADD as usize];
            assert!(!add.is_null());
            let s = std::ffi::CStr::from_ptr(getstr(add)).to_str().unwrap();
            assert_eq!(s, "__add");
        })();

        unsafe { lua_close(state.cast()) };
        result
    }

    #[test]
    fn metamethods_and_varargs_work_via_vm_paths() {
        run_lua_test(
            "test/tm_runtime.lua",
            r##"
local mt = {
  __add = function(a, b) return a.v + b.v end,
  __concat = function(a, b) return a.s .. ":" .. b.s end,
  __name = "VecLike",
}

local a = setmetatable({ v = 3, s = "a" }, mt)
local b = setmetatable({ v = 9, s = "b" }, mt)
assert(a + b == 12)
assert(a .. b == "a:b")

local function pack(...)
  return select("#", ...), select(1, ...), select(2, ...), select(3, ...)
end

local n, x, y, z = pack(10, 20, 30)
assert(n == 3 and x == 10 and y == 20 and z == 30)

local function hidden(...)
  local function inner(...)
    return select("#", ...), select(2, ...)
  end
  return inner(...)
end

local hn, hv = hidden("k", "m", "n")
assert(hn == 3 and hv == "m")
"##,
        );
    }
}
