use crate::lua_module::{lua_Integer, lua_Number};
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr;

type TStatus = u8;
type Instruction = u32;
type LMem = isize;
type LuMem = usize;
type LuaAlloc = Option<unsafe extern "C" fn(*mut c_void, *mut c_void, usize, usize) -> *mut c_void>;
type LuaCFunction = Option<unsafe extern "C" fn(*mut lua_State) -> c_int>;
type LuaKFunction = Option<unsafe extern "C" fn(*mut lua_State, c_int, isize) -> c_int>;
type LuaWarnFunction = Option<unsafe extern "C" fn(*mut c_void, *const c_char, c_int)>;
type LuaHook = Option<unsafe extern "C" fn(*mut lua_State, *mut lua_Debug)>;
type Pfunc = Option<unsafe extern "C" fn(*mut lua_State, *mut c_void)>;

const LUA_OK: TStatus = 0;
const LUA_YIELD: TStatus = 1;
const LUA_TTHREAD: u8 = 8;
const LUA_VNIL: u8 = 0;
const LUA_VFALSE: u8 = 1;
const LUA_VTHREAD: u8 = 8;
const BIT_ISCOLLECTABLE: u8 = 1 << 6;
const LUA_MINSTACK: usize = 20;
const LUA_RIDX_GLOBALS: lua_Integer = 2;
const LUA_RIDX_MAINTHREAD: lua_Integer = 3;
const LUA_RIDX_LAST: c_int = 3;
const LUA_NUMTYPES: usize = 9;
const LUA_EXTRASPACE: usize = size_of::<*mut c_void>();
const LUA_GCPMINORMUL: usize = 0;
const LUA_GCPMAJORMINOR: usize = 1;
const LUA_GCPMINORMAJOR: usize = 2;
const LUA_GCPPAUSE: usize = 3;
const LUA_GCPSTEPMUL: usize = 4;
const LUA_GCPSTEPSIZE: usize = 5;
const LUA_GCPN: usize = 6;
const LUAI_MAXCCALLS: u32 = 200;
const LUAI_GCPAUSE: c_int = 250;
const LUAI_GCMUL: c_int = 200;
const LUAI_MINORMAJOR: c_int = 70;
const LUAI_MAJORMINOR: c_int = 50;
const LUAI_GENMINORMUL: c_int = 20;
const EXTRA_STACK: usize = 5;
const BASIC_STACK_SIZE: usize = 2 * LUA_MINSTACK;
const KGC_INC: u8 = 0;
const GCSTPGC: u8 = 2;
const GCSpause: u8 = 8;
const WHITE0BIT: u8 = 3;
const WHITE1BIT: u8 = 4;
const WHITEBITS: u8 = (1 << WHITE0BIT) | (1 << WHITE1BIT);
const TM_N: usize = 25;
const STRCACHE_N: usize = 53;
const STRCACHE_M: usize = 2;
const CIST_C: u32 = 1 << 15;
const MAX_LMEM: LMem = isize::MAX;

#[derive(Copy, Clone)]
#[repr(C)]
union Value {
    gc: *mut GCObject,
    p: *mut c_void,
    f: LuaCFunction,
    i: lua_Integer,
    n: lua_Number,
    ub: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct TValue {
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
struct TString {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    extra: u8,
    shrlen: i8,
    hash: u32,
}

#[repr(C)]
struct Table {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
}

#[repr(C)]
struct stringtable {
    hash: *mut *mut TString,
    nuse: c_int,
    size: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct CallInfoLua {
    savedpc: *const Instruction,
    trap: c_int,
    nextraargs: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct CallInfoC {
    k: LuaKFunction,
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
struct TransferInfo {
    ftransfer: c_int,
    ntransfer: c_int,
}

#[repr(C)]
struct lua_longjmp {
    _private: [u8; 0],
}

#[repr(C)]
struct lua_Debug {
    _private: [u8; 0],
}

#[repr(C)]
pub struct lua_State {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    allowhook: u8,
    status: TStatus,
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
    hook: LuaHook,
    errfunc: isize,
    nCcalls: u32,
    oldpc: c_int,
    nci: c_int,
    basehookcount: c_int,
    hookcount: c_int,
    hookmask: c_int,
    transferinfo: TransferInfo,
}

#[repr(C)]
struct LX {
    extra_: [u8; LUA_EXTRASPACE],
    l: lua_State,
}

#[repr(C)]
pub struct global_State {
    frealloc: LuaAlloc,
    ud: *mut c_void,
    GCtotalbytes: LMem,
    GCdebt: LMem,
    GCmarked: LMem,
    GCmajorminor: LMem,
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
    panic: LuaCFunction,
    memerrmsg: *mut TString,
    tmname: [*mut TString; TM_N],
    mt: [*mut Table; LUA_NUMTYPES],
    strcache: [[*mut TString; STRCACHE_M]; STRCACHE_N],
    warnf: LuaWarnFunction,
    ud_warn: *mut c_void,
    mainth: LX,
}

unsafe extern "C" {
    fn luaC_freeallobjects(state: *mut lua_State);
    fn luaC_newobjdt(state: *mut lua_State, tt: u8, sz: usize, offset: usize) -> *mut GCObject;
    fn luaC_step(state: *mut lua_State);
    fn luaD_closeprotected(state: *mut lua_State, level: isize, status: TStatus) -> TStatus;
    fn luaD_errerr(state: *mut lua_State) -> !;
    fn luaD_rawrunprotected(state: *mut lua_State, f: Pfunc, ud: *mut c_void) -> TStatus;
    fn luaD_reallocstack(state: *mut lua_State, newsize: c_int, raiseerror: c_int) -> c_int;
    fn luaD_seterrorobj(state: *mut lua_State, errcode: TStatus, oldtop: StkId);
    fn luaD_throwbaselevel(state: *mut lua_State, errcode: TStatus) -> !;
    fn luaF_closeupval(state: *mut lua_State, level: StkId);
    fn luaG_runerror(state: *mut lua_State, fmt: *const c_char, ...) -> !;
    fn luaH_new(state: *mut lua_State) -> *mut Table;
    fn luaH_resize(state: *mut lua_State, table: *mut Table, newasize: u32, nhsize: u32);
    fn luaH_setint(state: *mut lua_State, table: *mut Table, key: lua_Integer, value: *mut TValue);
    fn luaM_free_(state: *mut lua_State, block: *mut c_void, osize: usize);
    fn luaM_malloc_(state: *mut lua_State, size: usize, tag: c_int) -> *mut c_void;
    fn luaO_codeparam(value: c_int) -> u8;
    fn luaS_init(state: *mut lua_State);
    fn luaT_init(state: *mut lua_State);
    fn luaX_init(state: *mut lua_State);
}

#[inline]
unsafe fn g(state: *mut lua_State) -> *mut global_State {
    unsafe { (*state).l_g }
}

#[inline]
unsafe fn mainthread(g: *mut global_State) -> *mut lua_State {
    unsafe { ptr::addr_of_mut!((*g).mainth.l) }
}

#[inline]
unsafe fn s2v(stack: StkId) -> *mut TValue {
    unsafe { ptr::addr_of_mut!((*stack).val) }
}

#[inline]
unsafe fn settt(obj: *mut TValue, tt: u8) {
    unsafe { (*obj).tt_ = tt };
}

#[inline]
unsafe fn setnilvalue(obj: *mut TValue) {
    unsafe { settt(obj, LUA_VNIL) };
}

#[inline]
unsafe fn setivalue(obj: *mut TValue, value: lua_Integer) {
    unsafe {
        (*obj).value_.i = value;
        settt(obj, 3);
    }
}

#[inline]
unsafe fn setbfvalue(obj: *mut TValue) {
    unsafe { settt(obj, LUA_VFALSE) };
}

#[inline]
unsafe fn sethvalue(obj: *mut TValue, table: *mut Table) {
    unsafe {
        (*obj).value_.gc = table.cast();
        settt(obj, (*table).tt | BIT_ISCOLLECTABLE);
    }
}

#[inline]
unsafe fn setthvalue(obj: *mut TValue, thread: *mut lua_State) {
    unsafe {
        (*obj).value_.gc = thread.cast();
        settt(obj, (*thread).tt | BIT_ISCOLLECTABLE);
    }
}

#[inline]
unsafe fn setthvalue2s(state: *mut lua_State, stack: StkId, thread: *mut lua_State) {
    let _ = state;
    unsafe { setthvalue(s2v(stack), thread) };
}

#[inline]
unsafe fn ttisnil(value: *const TValue) -> bool {
    unsafe { ((*value).tt_ & 0x0f) == LUA_VNIL }
}

#[inline]
unsafe fn gettotalbytes(g: *mut global_State) -> LMem {
    unsafe { (*g).GCtotalbytes - (*g).GCdebt }
}

#[inline]
unsafe fn stacksize(state: *mut lua_State) -> c_int {
    unsafe { (*state).stack_last.p.offset_from((*state).stack.p) as c_int }
}

#[inline]
unsafe fn get_ccalls(state: *mut lua_State) -> u32 {
    unsafe { (*state).nCcalls & 0xffff }
}

#[inline]
unsafe fn incnny(state: *mut lua_State) {
    unsafe { (*state).nCcalls += 0x10000 };
}

#[inline]
unsafe fn resethookcount(state: *mut lua_State) {
    unsafe { (*state).hookcount = (*state).basehookcount };
}

#[inline]
unsafe fn lua_c_white(g: *mut global_State) -> u8 {
    unsafe { (*g).currentwhite & WHITEBITS }
}

#[inline]
unsafe fn completestate(g: *mut global_State) -> bool {
    unsafe { ttisnil(ptr::addr_of!((*g).nilvalue)) }
}

#[inline]
unsafe fn api_incr_top(state: *mut lua_State) {
    unsafe { (*state).top.p = (*state).top.p.add(1) };
}

#[inline]
unsafe fn lua_getextraspace(state: *mut lua_State) -> *mut u8 {
    unsafe { state.cast::<u8>().sub(LUA_EXTRASPACE) }
}

#[inline]
unsafe fn new_callinfo(state: *mut lua_State) -> *mut CallInfo {
    unsafe { luaM_malloc_(state, size_of::<CallInfo>(), 0).cast() }
}

#[inline]
unsafe fn free_callinfo(state: *mut lua_State, ci: *mut CallInfo) {
    unsafe { luaM_free_(state, ci.cast(), size_of::<CallInfo>()) };
}

#[inline]
unsafe fn new_stackvector(state: *mut lua_State, count: usize) -> *mut StackValue {
    unsafe { luaM_malloc_(state, size_of::<StackValue>() * count, 0).cast() }
}

#[inline]
unsafe fn free_stackvector(state: *mut lua_State, stack: *mut StackValue, count: usize) {
    unsafe { luaM_free_(state, stack.cast(), size_of::<StackValue>() * count) };
}

#[inline]
unsafe fn free_tstring_hash(state: *mut lua_State, hash: *mut *mut TString, count: usize) {
    unsafe { luaM_free_(state, hash.cast(), size_of::<*mut TString>() * count) };
}

unsafe extern "C" fn f_luaopen(state: *mut lua_State, ud: *mut c_void) {
    let _ = ud;
    let g = unsafe { g(state) };
    unsafe { stack_init(state, state) };
    unsafe { init_registry(state, g) };
    unsafe { luaS_init(state) };
    unsafe { luaT_init(state) };
    unsafe { luaX_init(state) };
    unsafe {
        (*g).gcstp = 0;
        setnilvalue(ptr::addr_of_mut!((*g).nilvalue));
    }
}

unsafe fn reset_ci(state: *mut lua_State) {
    let ci = unsafe { ptr::addr_of_mut!((*state).base_ci) };
    unsafe {
        (*state).ci = ci;
        (*ci).func.p = (*state).stack.p;
        setnilvalue(s2v((*ci).func.p));
        (*ci).top.p = (*ci).func.p.add(1 + LUA_MINSTACK);
        (*ci).u.c.k = None;
        (*ci).callstatus = CIST_C;
        (*state).status = LUA_OK;
        (*state).errfunc = 0;
    }
}

unsafe fn free_ci(state: *mut lua_State) {
    let ci = unsafe { (*state).ci };
    let mut next = unsafe { (*ci).next };
    unsafe { (*ci).next = ptr::null_mut() };
    while !next.is_null() {
        let current = next;
        next = unsafe { (*current).next };
        unsafe { free_callinfo(state, current) };
        unsafe { (*state).nci -= 1 };
    }
}

unsafe fn stack_init(thread: *mut lua_State, owner: *mut lua_State) {
    let size = BASIC_STACK_SIZE + EXTRA_STACK;
    let stack = unsafe { new_stackvector(owner, size) };
    unsafe {
        (*thread).stack.p = stack;
        (*thread).tbclist.p = stack;
    }
    for i in 0..size {
        unsafe { setnilvalue(s2v(stack.add(i))) };
    }
    unsafe {
        (*thread).stack_last.p = stack.add(BASIC_STACK_SIZE);
    }
    unsafe { reset_ci(thread) };
    unsafe {
        (*thread).top.p = stack.add(1);
    }
}

unsafe fn freestack(state: *mut lua_State) {
    if unsafe { (*state).stack.p.is_null() } {
        return;
    }
    unsafe { (*state).ci = ptr::addr_of_mut!((*state).base_ci) };
    unsafe { free_ci(state) };
    unsafe { free_stackvector(state, (*state).stack.p, (stacksize(state) as usize) + EXTRA_STACK) };
}

unsafe fn init_registry(state: *mut lua_State, g: *mut global_State) {
    let mut aux = TValue {
        value_: Value { ub: 0 },
        tt_: LUA_VNIL,
    };
    let registry = unsafe { luaH_new(state) };
    unsafe { sethvalue(ptr::addr_of_mut!((*g).l_registry), registry) };
    unsafe { luaH_resize(state, registry, LUA_RIDX_LAST as u32, 0) };
    unsafe { setbfvalue(ptr::addr_of_mut!(aux)) };
    unsafe { luaH_setint(state, registry, 1, ptr::addr_of_mut!(aux)) };
    unsafe { setthvalue(ptr::addr_of_mut!(aux), state) };
    unsafe { luaH_setint(state, registry, LUA_RIDX_MAINTHREAD, ptr::addr_of_mut!(aux)) };
    unsafe { sethvalue(ptr::addr_of_mut!(aux), luaH_new(state)) };
    unsafe { luaH_setint(state, registry, LUA_RIDX_GLOBALS, ptr::addr_of_mut!(aux)) };
}

unsafe fn preinit_thread(state: *mut lua_State, g: *mut global_State) {
    unsafe {
        (*state).l_g = g;
        (*state).stack.p = ptr::null_mut();
        (*state).ci = ptr::null_mut();
        (*state).nci = 0;
        (*state).twups = state;
        (*state).nCcalls = 0;
        (*state).errorJmp = ptr::null_mut();
        (*state).hook = None;
        (*state).hookmask = 0;
        (*state).basehookcount = 0;
        (*state).allowhook = 1;
        resethookcount(state);
        (*state).openupval = ptr::null_mut();
        (*state).status = LUA_OK;
        (*state).errfunc = 0;
        (*state).oldpc = 0;
        (*state).base_ci.previous = ptr::null_mut();
        (*state).base_ci.next = ptr::null_mut();
    }
}

unsafe fn close_state(state: *mut lua_State) {
    let g = unsafe { g(state) };
    if !unsafe { completestate(g) } {
        unsafe { luaC_freeallobjects(state) };
    } else {
        unsafe { reset_ci(state) };
        unsafe { luaD_closeprotected(state, 1, LUA_OK) };
        unsafe {
            (*state).top.p = (*state).stack.p.add(1);
        }
        unsafe { luaC_freeallobjects(state) };
    }
    unsafe { free_tstring_hash(state, (*g).strt.hash, (*g).strt.size.max(0) as usize) };
    unsafe { freestack(state) };
    if let Some(frealloc) = unsafe { (*g).frealloc } {
        unsafe { frealloc((*g).ud, g.cast(), size_of::<global_State>(), 0) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_setdebt(g: *mut global_State, mut debt: LMem) {
    let tb = unsafe { gettotalbytes(g) };
    if debt > MAX_LMEM - tb {
        debt = MAX_LMEM - tb;
    }
    unsafe {
        (*g).GCtotalbytes = tb + debt;
        (*g).GCdebt = debt;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_extendCI(state: *mut lua_State) -> *mut CallInfo {
    let ci = unsafe { new_callinfo(state) };
    unsafe {
        (*(*state).ci).next = ci;
        (*ci).previous = (*state).ci;
        (*ci).next = ptr::null_mut();
        (*ci).u.l.trap = 0;
        (*state).nci += 1;
    }
    ci
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_shrinkCI(state: *mut lua_State) {
    let mut ci = unsafe { (*(*state).ci).next };
    if ci.is_null() {
        return;
    }
    while {
        let next = unsafe { (*ci).next };
        !next.is_null()
    } {
        let next = unsafe { (*ci).next };
        let next2 = unsafe { (*next).next };
        unsafe {
            (*ci).next = next2;
            (*state).nci -= 1;
            free_callinfo(state, next);
        }
        if next2.is_null() {
            break;
        }
        unsafe {
            (*next2).previous = ci;
            ci = next2;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_checkcstack(state: *mut lua_State) {
    let calls = unsafe { get_ccalls(state) };
    if calls == LUAI_MAXCCALLS {
        unsafe { luaG_runerror(state, c"C stack overflow".as_ptr()) };
    } else if calls >= (LUAI_MAXCCALLS / 10 * 11) {
        unsafe { luaD_errerr(state) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_incCstack(state: *mut lua_State) {
    unsafe { (*state).nCcalls += 1 };
    if unsafe { get_ccalls(state) } >= LUAI_MAXCCALLS {
        unsafe { luaE_checkcstack(state) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_threadsize(state: *mut lua_State) -> LuMem {
    let mut sz = size_of::<LX>() + (unsafe { (*state).nci.max(0) as usize } * size_of::<CallInfo>());
    if unsafe { !(*state).stack.p.is_null() } {
        sz += ((unsafe { stacksize(state) as usize }) + EXTRA_STACK) * size_of::<StackValue>();
    }
    sz
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_newthread(state: *mut lua_State) -> *mut lua_State {
    let g = unsafe { g(state) };
    if unsafe { (*g).GCdebt <= 0 } {
        unsafe { luaC_step(state) };
    }
    let o = unsafe {
        luaC_newobjdt(
            state,
            LUA_TTHREAD,
            size_of::<LX>(),
            offset_of!(LX, l),
        )
    };
    let thread = o.cast::<lua_State>();
    unsafe { setthvalue2s(state, (*state).top.p, thread) };
    unsafe { api_incr_top(state) };
    unsafe { preinit_thread(thread, g) };
    unsafe {
        (*thread).hookmask = (*state).hookmask;
        (*thread).basehookcount = (*state).basehookcount;
        (*thread).hook = (*state).hook;
        resethookcount(thread);
    }
    unsafe {
        ptr::copy_nonoverlapping(
            lua_getextraspace(mainthread(g)),
            lua_getextraspace(thread),
            LUA_EXTRASPACE,
        );
    }
    unsafe { stack_init(thread, state) };
    thread
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_freethread(state: *mut lua_State, thread: *mut lua_State) {
    let l = unsafe { thread.cast::<u8>().sub(offset_of!(LX, l)).cast::<LX>() };
    unsafe { luaF_closeupval(thread, (*thread).stack.p) };
    unsafe { freestack(thread) };
    unsafe { luaM_free_(state, l.cast(), size_of::<LX>()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_resetthread(state: *mut lua_State, mut status: TStatus) -> TStatus {
    unsafe { reset_ci(state) };
    if status == LUA_YIELD {
        status = LUA_OK;
    }
    status = unsafe { luaD_closeprotected(state, 1, status) };
    if status != LUA_OK {
        unsafe { luaD_seterrorobj(state, status, (*state).stack.p.add(1)) };
    } else {
        unsafe { (*state).top.p = (*state).stack.p.add(1) };
    }
    unsafe {
        luaD_reallocstack(
            state,
            (*(*state).ci).top.p.offset_from((*state).stack.p) as c_int,
            0,
        )
    };
    status
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_closethread(state: *mut lua_State, from: *mut lua_State) -> c_int {
    unsafe {
        (*state).nCcalls = if from.is_null() { 0 } else { get_ccalls(from) };
    }
    let status = unsafe { luaE_resetthread(state, (*state).status) };
    if state == from {
        unsafe { luaD_throwbaselevel(state, status) };
    }
    status as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_newstate(
    f: LuaAlloc,
    ud: *mut c_void,
    seed: u32,
) -> *mut lua_State {
    let Some(frealloc) = f else {
        return ptr::null_mut();
    };
    let g = unsafe { frealloc(ud, ptr::null_mut(), LUA_TTHREAD as usize, size_of::<global_State>()) }
        .cast::<global_State>();
    if g.is_null() {
        return ptr::null_mut();
    }
    let state = unsafe { ptr::addr_of_mut!((*g).mainth.l) };
    unsafe {
        (*state).tt = LUA_VTHREAD;
        (*g).currentwhite = 1 << WHITE0BIT;
        (*state).marked = lua_c_white(g);
        preinit_thread(state, g);
        (*g).allgc = state.cast();
        (*state).next = ptr::null_mut();
        incnny(state);
        (*g).frealloc = Some(frealloc);
        (*g).ud = ud;
        (*g).warnf = None;
        (*g).ud_warn = ptr::null_mut();
        (*g).seed = seed;
        (*g).gcstp = GCSTPGC;
        (*g).strt.size = 0;
        (*g).strt.nuse = 0;
        (*g).strt.hash = ptr::null_mut();
        setnilvalue(ptr::addr_of_mut!((*g).l_registry));
        (*g).panic = None;
        (*g).gcstate = GCSpause;
        (*g).gckind = KGC_INC;
        (*g).gcstopem = 0;
        (*g).gcemergency = 0;
        (*g).finobj = ptr::null_mut();
        (*g).tobefnz = ptr::null_mut();
        (*g).fixedgc = ptr::null_mut();
        (*g).firstold1 = ptr::null_mut();
        (*g).survival = ptr::null_mut();
        (*g).old1 = ptr::null_mut();
        (*g).reallyold = ptr::null_mut();
        (*g).finobjsur = ptr::null_mut();
        (*g).finobjold1 = ptr::null_mut();
        (*g).finobjrold = ptr::null_mut();
        (*g).sweepgc = ptr::null_mut();
        (*g).gray = ptr::null_mut();
        (*g).grayagain = ptr::null_mut();
        (*g).weak = ptr::null_mut();
        (*g).ephemeron = ptr::null_mut();
        (*g).allweak = ptr::null_mut();
        (*g).twups = ptr::null_mut();
        (*g).GCtotalbytes = size_of::<global_State>() as LMem;
        (*g).GCmarked = 0;
        (*g).GCdebt = 0;
        setivalue(ptr::addr_of_mut!((*g).nilvalue), 0);
        (*g).gcparams[LUA_GCPPAUSE] = luaO_codeparam(LUAI_GCPAUSE);
        (*g).gcparams[LUA_GCPSTEPMUL] = luaO_codeparam(LUAI_GCMUL);
        (*g).gcparams[LUA_GCPSTEPSIZE] = luaO_codeparam((200 * size_of::<Table>()) as c_int);
        (*g).gcparams[LUA_GCPMINORMUL] = luaO_codeparam(LUAI_GENMINORMUL);
        (*g).gcparams[LUA_GCPMINORMAJOR] = luaO_codeparam(LUAI_MINORMAJOR);
        (*g).gcparams[LUA_GCPMAJORMINOR] = luaO_codeparam(LUAI_MAJORMINOR);
        for mt in &mut (*g).mt {
            *mt = ptr::null_mut();
        }
        for cache_set in &mut (*g).strcache {
            for item in cache_set {
                *item = ptr::null_mut();
            }
        }
        for name in &mut (*g).tmname {
            *name = ptr::null_mut();
        }
    }
    if unsafe { luaD_rawrunprotected(state, Some(f_luaopen), ptr::null_mut()) } != LUA_OK {
        unsafe { close_state(state) };
        ptr::null_mut()
    } else {
        state
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_close(state: *mut lua_State) {
    let main = unsafe { mainthread(g(state)) };
    unsafe { close_state(main) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_warning(state: *mut lua_State, msg: *const c_char, tocont: c_int) {
    let g = unsafe { g(state) };
    if let Some(warnf) = unsafe { (*g).warnf } {
        unsafe { warnf((*g).ud_warn, msg, tocont) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaE_warnerror(state: *mut lua_State, where_: *const c_char) {
    let errobj = unsafe { s2v((*state).top.p.sub(1)) };
    let msg = if unsafe { ((*errobj).tt_ & 0x0f) == 4 } {
        unsafe { crate::luaffi::lua_tolstring(state.cast(), -1, ptr::null_mut()) }
    } else {
        c"error object is not a string".as_ptr()
    };
    unsafe { luaE_warning(state, c"error in ".as_ptr(), 1) };
    unsafe { luaE_warning(state, where_, 1) };
    unsafe { luaE_warning(state, c" (".as_ptr(), 1) };
    unsafe { luaE_warning(state, msg, 1) };
    unsafe { luaE_warning(state, c")".as_ptr(), 0) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aux_rs::{luaL_checkversion_, luaL_newstate};
    use crate::luaffi::{LUAL_NUMSIZES, LUA_VERSION_NUM, lua_close};

    unsafe extern "C" fn test_hook(_: *mut lua_State, _: *mut lua_Debug) {}

    #[test]
    fn newthread_copies_hook_state_and_closes() {
        let state = unsafe { luaL_newstate() }.cast::<lua_State>();
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state.cast(), LUA_VERSION_NUM, LUAL_NUMSIZES);
            (*state).hookmask = 7;
            (*state).basehookcount = 11;
            (*state).hook = Some(test_hook);
            let main_top = (*state).top.p;

            let thread = lua_newthread(state);
            assert!(!thread.is_null());
            assert_eq!((*thread).hookmask, 7);
            assert_eq!((*thread).basehookcount, 11);
            assert!(matches!(
                (*thread).hook,
                Some(f) if f as *const () as usize == test_hook as *const () as usize
            ));
            assert!(!(*thread).stack.p.is_null());
            assert_eq!((*thread).top.p, (*thread).stack.p.add(1));
            assert_eq!((*state).top.p, main_top.add(1));

            let status = lua_closethread(thread, state);
            assert_eq!(status, LUA_OK as c_int);
        })();

        unsafe { lua_close(state.cast()) };
        result
    }

    #[test]
    fn callinfo_growth_and_threadsize_track_allocations() {
        let state = unsafe { luaL_newstate() }.cast::<lua_State>();
        assert!(!state.is_null());

        let result = (|| unsafe {
            let base_size = luaE_threadsize(state);
            let ci1 = luaE_extendCI(state);
            (*state).ci = ci1;
            let ci2 = luaE_extendCI(state);
            (*state).ci = ptr::addr_of_mut!((*state).base_ci);
            assert!(!ci1.is_null());
            assert!(!ci2.is_null());
            assert_eq!((*state).nci, 2);
            assert!(luaE_threadsize(state) >= base_size + (2 * size_of::<CallInfo>()));

            luaE_shrinkCI(state);
            assert_eq!((*state).nci, 1);
            assert!((*(*state).ci).next == ci1);
            assert!((*ci1).next.is_null());
        })();

        unsafe { lua_close(state.cast()) };
        result
    }
}
