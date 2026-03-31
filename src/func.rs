use crate::lua_module::lua_State;
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr;

type TStatus = u8;
type Instruction = u32;
type LuaCFunction = Option<unsafe extern "C" fn(*mut lua_State) -> c_int>;

const LUA_OK: TStatus = 0;
const LUA_ERRERR: TStatus = 5;
const CLOSEKTOP: TStatus = LUA_ERRERR + 1;
const LUA_VUPVAL: u8 = 9;
const LUA_VPROTO: u8 = 10;
const LUA_VLCL: u8 = 6;
const LUA_VCCL: u8 = 38;
const BIT_ISCOLLECTABLE: u8 = 1 << 6;
const LUA_VNIL: u8 = 0;
const LUA_VFALSE: u8 = 1;
const TM_CLOSE: c_int = 24;
const PF_FIXED: u8 = 4;
const WHITEBITS: u8 = (1 << 3) | (1 << 4);
const BLACKBIT: u8 = 5;
const MAXDELTA: usize = u16::MAX as usize;

#[derive(Copy, Clone)]
#[repr(C)]
union Value {
    gc: *mut GCObject,
    p: *mut c_void,
    f: LuaCFunction,
    i: i64,
    n: f64,
    ub: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct TValue {
    value_: Value,
    tt_: u8,
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
    falloc: *mut c_void,
    ud: *mut c_void,
}

#[repr(C)]
struct Upvaldesc {
    name: *mut TString,
    instack: u8,
    idx: u8,
    kind: u8,
}

#[repr(C)]
struct LocVar {
    varname: *mut TString,
    startpc: c_int,
    endpc: c_int,
}

#[repr(C)]
struct AbsLineInfo {
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
    k: *mut TValue,
    code: *mut Instruction,
    p: *mut *mut Proto,
    upvalues: *mut Upvaldesc,
    lineinfo: *mut i8,
    abslineinfo: *mut AbsLineInfo,
    locvars: *mut LocVar,
    source: *mut TString,
    gclist: *mut GCObject,
}

#[repr(C)]
union UpValV {
    p: *mut TValue,
    offset: isize,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct UpValOpen {
    next: *mut UpVal,
    previous: *mut *mut UpVal,
}

#[repr(C)]
union UpValU {
    open: UpValOpen,
    value: TValue,
}

#[repr(C)]
pub struct UpVal {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    v: UpValV,
    u: UpValU,
}

#[repr(C)]
pub struct CClosure {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    nupvalues: u8,
    gclist: *mut GCObject,
    f: LuaCFunction,
    upvalue: [TValue; 1],
}

#[repr(C)]
pub struct LClosure {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    nupvalues: u8,
    gclist: *mut GCObject,
    p: *mut Proto,
    upvals: [*mut UpVal; 1],
}

#[derive(Copy, Clone)]
#[repr(C)]
union StkIdRel {
    p: *mut StackValue,
    offset: isize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union StackValue {
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

#[repr(C)]
struct CallInfo {
    func: StkIdRel,
}

#[repr(C)]
struct GlobalStatePrefix {
    _frealloc: *mut c_void,
    _ud: *mut c_void,
    _gctotalbytes: isize,
    _gcdebt: isize,
    _gcmarked: isize,
    _gcmajorminor: isize,
    _strt: [u8; 16],
    _l_registry: TValue,
    _nilvalue: TValue,
    _seed: u32,
    _gcparams: [u8; 6],
    currentwhite: u8,
    _gcstate: u8,
    _gckind: u8,
    _gcstopem: u8,
    _gcstp: u8,
    _gcemergency: u8,
    _allgc: *mut GCObject,
    _sweepgc: *mut *mut GCObject,
    _finobj: *mut GCObject,
    _gray: *mut GCObject,
    _grayagain: *mut GCObject,
    _weak: *mut GCObject,
    _ephemeron: *mut GCObject,
    _allweak: *mut GCObject,
    _tobefnz: *mut GCObject,
    _fixedgc: *mut GCObject,
    _survival: *mut GCObject,
    _old1: *mut GCObject,
    _reallyold: *mut GCObject,
    _firstold1: *mut GCObject,
    _finobjsur: *mut GCObject,
    _finobjold1: *mut GCObject,
    _finobjrold: *mut GCObject,
    twups: *mut lua_State,
}

#[repr(C)]
struct LuaStatePrefix {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    allowhook: u8,
    status: TStatus,
    top: StkIdRel,
    l_g: *mut GlobalStatePrefix,
    ci: *mut CallInfo,
    stack_last: StkIdRel,
    stack: StkIdRel,
    openupval: *mut UpVal,
    tbclist: StkIdRel,
    gclist: *mut GCObject,
    twups: *mut lua_State,
}

unsafe extern "C" {
    fn luaC_newobj(state: *mut lua_State, tt: u8, sz: usize) -> *mut GCObject;
    fn luaC_barrier_(state: *mut lua_State, o: *mut GCObject, v: *mut GCObject);
    fn luaT_gettmbyobj(state: *mut lua_State, o: *const TValue, event: c_int) -> *const TValue;
    fn luaD_call(state: *mut lua_State, func: StkId, nresults: c_int);
    fn luaD_callnoyield(state: *mut lua_State, func: StkId, nresults: c_int);
    fn luaD_seterrorobj(state: *mut lua_State, errcode: TStatus, oldtop: StkId);
    fn luaG_findlocal(
        state: *mut lua_State,
        ci: *mut CallInfo,
        n: c_int,
        pos: *mut StkId,
    ) -> *const c_char;
    fn luaG_runerror(state: *mut lua_State, fmt: *const c_char, ...) -> !;
    fn luaM_free_(state: *mut lua_State, block: *mut c_void, osize: usize);
}

pub(crate) unsafe fn raw_luaF_newproto(state: *mut c_void) -> *mut c_void {
    unsafe { luaF_newproto(state.cast()).cast() }
}

pub(crate) unsafe fn raw_luaF_newLclosure(state: *mut c_void, nupvals: c_int) -> *mut c_void {
    unsafe { luaF_newLclosure(state.cast(), nupvals).cast() }
}

#[inline]
unsafe fn lstate(state: *mut lua_State) -> *mut LuaStatePrefix {
    state.cast()
}

#[inline]
unsafe fn gstate(state: *mut lua_State) -> *mut GlobalStatePrefix {
    unsafe { (*lstate(state)).l_g }
}

#[inline]
unsafe fn s2v(stack: StkId) -> *mut TValue {
    unsafe { ptr::addr_of_mut!((*stack).val) }
}

#[inline]
unsafe fn settt(value: *mut TValue, tag: u8) {
    unsafe { (*value).tt_ = tag };
}

#[inline]
unsafe fn setnilvalue(value: *mut TValue) {
    unsafe { settt(value, LUA_VNIL) };
}

#[inline]
unsafe fn setobj(dst: *mut TValue, src: *const TValue) {
    unsafe {
        (*dst).value_ = (*src).value_;
        (*dst).tt_ = (*src).tt_;
    }
}

#[inline]
unsafe fn setobj2s(dst: StkId, src: *const TValue) {
    unsafe { setobj(s2v(dst), src) };
}

#[inline]
unsafe fn isfalse(value: *const TValue) -> bool {
    matches!(unsafe { (*value).tt_ }, LUA_VFALSE | LUA_VNIL)
}

#[inline]
unsafe fn ttisnil(value: *const TValue) -> bool {
    unsafe { ((*value).tt_ & 0x0f) == LUA_VNIL }
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
unsafe fn uplevel(uv: *const UpVal) -> StkId {
    unsafe { (*uv).v.p.cast::<StackValue>() }
}

#[inline]
unsafe fn isintwups(state: *mut lua_State) -> bool {
    unsafe { (*lstate(state)).twups != state }
}

#[inline]
unsafe fn savestack(state: *mut lua_State, pt: StkId) -> isize {
    let base = unsafe { (*lstate(state)).stack.p.cast::<u8>() };
    let pt = pt.cast::<u8>();
    unsafe { pt.offset_from(base) }
}

#[inline]
unsafe fn restorestack(state: *mut lua_State, n: isize) -> StkId {
    unsafe {
        (*lstate(state))
            .stack
            .p
            .cast::<u8>()
            .offset(n)
            .cast::<StackValue>()
    }
}

#[inline]
unsafe fn iswhite(obj: *const GCObject) -> bool {
    unsafe { ((*obj).marked & WHITEBITS) != 0 }
}

#[inline]
unsafe fn nw2black(obj: *mut GCObject) {
    unsafe { (*obj).marked |= 1 << BLACKBIT };
}

#[inline]
unsafe fn iscollectable(value: *const TValue) -> bool {
    unsafe { ((*value).tt_ & BIT_ISCOLLECTABLE) != 0 }
}

#[inline]
unsafe fn lua_c_objbarrier(state: *mut lua_State, parent: *mut GCObject, child: *mut GCObject) {
    if unsafe { !parent.is_null() && ((*parent).marked & (1 << BLACKBIT)) != 0 && iswhite(child) } {
        unsafe { luaC_barrier_(state, parent, child) };
    }
}

#[inline]
unsafe fn lua_c_barrier(state: *mut lua_State, parent: *mut GCObject, value: *const TValue) {
    if unsafe { iscollectable(value) } {
        unsafe { lua_c_objbarrier(state, parent, (*value).value_.gc) };
    }
}

#[inline]
fn size_cclosure(n: c_int) -> usize {
    offset_of!(CClosure, upvalue) + size_of::<TValue>() * n as usize
}

#[inline]
fn size_lclosure(n: c_int) -> usize {
    offset_of!(LClosure, upvals) + size_of::<*mut UpVal>() * n as usize
}

unsafe fn callclosemethod(state: *mut lua_State, obj: *mut TValue, err: *mut TValue, yy: c_int) {
    let mut top = unsafe { (*lstate(state)).top.p };
    let func = top;
    let tm = unsafe { luaT_gettmbyobj(state, obj, TM_CLOSE) };
    unsafe { setobj2s(top, tm) };
    top = unsafe { top.add(1) };
    unsafe { setobj2s(top, obj) };
    top = unsafe { top.add(1) };
    if !err.is_null() {
        unsafe { setobj2s(top, err) };
        top = unsafe { top.add(1) };
    }
    unsafe { (*lstate(state)).top.p = top };
    if yy != 0 {
        unsafe { luaD_call(state, func, 0) };
    } else {
        unsafe { luaD_callnoyield(state, func, 0) };
    }
}

unsafe fn checkclosemth(state: *mut lua_State, level: StkId) {
    let tm = unsafe { luaT_gettmbyobj(state, s2v(level), TM_CLOSE) };
    if unsafe { ttisnil(tm) } {
        let idx = unsafe { level.offset_from((*(*lstate(state)).ci).func.p) as c_int };
        let mut vname = unsafe { luaG_findlocal(state, (*lstate(state)).ci, idx, ptr::null_mut()) };
        if vname.is_null() {
            vname = c"?".as_ptr();
        }
        unsafe {
            luaG_runerror(
                state,
                c"variable '%s' got a non-closable value".as_ptr(),
                vname,
            )
        };
    }
}

unsafe fn prepcallclosemth(state: *mut lua_State, level: StkId, status: TStatus, yy: c_int) {
    let uv = unsafe { s2v(level) };
    let errobj = match status {
        LUA_OK => {
            unsafe { (*lstate(state)).top.p = level.add(1) };
            ptr::null_mut()
        }
        CLOSEKTOP => ptr::null_mut(),
        _ => {
            let err = unsafe { s2v(level.add(1)) };
            unsafe { luaD_seterrorobj(state, status, level.add(1)) };
            err
        }
    };
    unsafe { callclosemethod(state, uv, errobj, yy) };
}

unsafe fn poptbclist(state: *mut lua_State) {
    let mut tbc = unsafe { (*lstate(state)).tbclist.p };
    let delta = unsafe { (*tbc).tbclist.delta as usize };
    tbc = unsafe { tbc.sub(delta) };
    while tbc > unsafe { (*lstate(state)).stack.p } && unsafe { (*tbc).tbclist.delta == 0 } {
        tbc = unsafe { tbc.sub(MAXDELTA) };
    }
    unsafe { (*lstate(state)).tbclist.p = tbc };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_newCclosure(state: *mut lua_State, nupvals: c_int) -> *mut CClosure {
    let o = unsafe { luaC_newobj(state, LUA_VCCL, size_cclosure(nupvals)) };
    let c = o.cast::<CClosure>();
    unsafe { (*c).nupvalues = nupvals as u8 };
    c
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_newLclosure(
    state: *mut lua_State,
    mut nupvals: c_int,
) -> *mut LClosure {
    let o = unsafe { luaC_newobj(state, LUA_VLCL, size_lclosure(nupvals)) };
    let c = o.cast::<LClosure>();
    unsafe {
        (*c).p = ptr::null_mut();
        (*c).nupvalues = nupvals as u8;
    }
    let upvals = unsafe { (*c).upvals.as_mut_ptr() };
    while nupvals > 0 {
        nupvals -= 1;
        unsafe { *upvals.add(nupvals as usize) = ptr::null_mut() };
    }
    c
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_initupvals(state: *mut lua_State, cl: *mut LClosure) {
    let nup = unsafe { (*cl).nupvalues as usize };
    let upvals = unsafe { (*cl).upvals.as_mut_ptr() };
    for i in 0..nup {
        let o = unsafe { luaC_newobj(state, LUA_VUPVAL, size_of::<UpVal>()) };
        let uv = o.cast::<UpVal>();
        unsafe {
            (*uv).v.p = ptr::addr_of_mut!((*uv).u.value);
            setnilvalue((*uv).v.p);
            *upvals.add(i) = uv;
            lua_c_objbarrier(state, cl.cast(), uv.cast());
        }
    }
}

unsafe fn newupval(state: *mut lua_State, level: StkId, prev: *mut *mut UpVal) -> *mut UpVal {
    let o = unsafe { luaC_newobj(state, LUA_VUPVAL, size_of::<UpVal>()) };
    let uv = o.cast::<UpVal>();
    let next = unsafe { *prev };
    unsafe {
        (*uv).v.p = s2v(level);
        (*uv).u.open.next = next;
        (*uv).u.open.previous = prev;
        if !next.is_null() {
            (*next).u.open.previous = ptr::addr_of_mut!((*uv).u.open.next);
        }
        *prev = uv;
    }
    if unsafe { !isintwups(state) } {
        unsafe {
            (*lstate(state)).twups = (*gstate(state)).twups;
            (*gstate(state)).twups = state;
        }
    }
    uv
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_findupval(state: *mut lua_State, level: StkId) -> *mut UpVal {
    let mut pp = unsafe { ptr::addr_of_mut!((*lstate(state)).openupval) };
    let mut p = unsafe { *pp };
    while !p.is_null() && unsafe { uplevel(p) >= level } {
        if unsafe { uplevel(p) == level } {
            return p;
        }
        pp = unsafe { ptr::addr_of_mut!((*p).u.open.next) };
        p = unsafe { *pp };
    }
    unsafe { newupval(state, level, pp) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_newtbcupval(state: *mut lua_State, level: StkId) {
    if unsafe { isfalse(s2v(level)) } {
        return;
    }
    unsafe { checkclosemth(state, level) };
    while unsafe { level.offset_from((*lstate(state)).tbclist.p) as usize > MAXDELTA } {
        unsafe {
            (*lstate(state)).tbclist.p = (*lstate(state)).tbclist.p.add(MAXDELTA);
            (*(*lstate(state)).tbclist.p).tbclist.delta = 0;
        }
    }
    unsafe {
        (*level).tbclist.delta = level.offset_from((*lstate(state)).tbclist.p) as u16;
        (*lstate(state)).tbclist.p = level;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_unlinkupval(uv: *mut UpVal) {
    unsafe {
        *(*uv).u.open.previous = (*uv).u.open.next;
        if !(*uv).u.open.next.is_null() {
            (*(*uv).u.open.next).u.open.previous = (*uv).u.open.previous;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_closeupval(state: *mut lua_State, level: StkId) {
    let mut uv = unsafe { (*lstate(state)).openupval };
    while !uv.is_null() && unsafe { uplevel(uv) >= level } {
        let slot = unsafe { ptr::addr_of_mut!((*uv).u.value) };
        unsafe { luaF_unlinkupval(uv) };
        unsafe { setobj(slot, (*uv).v.p) };
        unsafe { (*uv).v.p = slot };
        if unsafe { !iswhite(uv.cast()) } {
            unsafe {
                nw2black(uv.cast());
                lua_c_barrier(state, uv.cast(), slot);
            }
        }
        uv = unsafe { (*lstate(state)).openupval };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_close(
    state: *mut lua_State,
    mut level: StkId,
    status: TStatus,
    yy: c_int,
) -> StkId {
    let levelrel = unsafe { savestack(state, level) };
    unsafe { luaF_closeupval(state, level) };
    while unsafe { (*lstate(state)).tbclist.p >= level } {
        let tbc = unsafe { (*lstate(state)).tbclist.p };
        unsafe { poptbclist(state) };
        unsafe { prepcallclosemth(state, tbc, status, yy) };
        level = unsafe { restorestack(state, levelrel) };
    }
    level
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_newproto(state: *mut lua_State) -> *mut Proto {
    let o = unsafe { luaC_newobj(state, LUA_VPROTO, size_of::<Proto>()) };
    let f = o.cast::<Proto>();
    unsafe {
        (*f).k = ptr::null_mut();
        (*f).sizek = 0;
        (*f).p = ptr::null_mut();
        (*f).sizep = 0;
        (*f).code = ptr::null_mut();
        (*f).sizecode = 0;
        (*f).lineinfo = ptr::null_mut();
        (*f).sizelineinfo = 0;
        (*f).abslineinfo = ptr::null_mut();
        (*f).sizeabslineinfo = 0;
        (*f).upvalues = ptr::null_mut();
        (*f).sizeupvalues = 0;
        (*f).numparams = 0;
        (*f).flag = 0;
        (*f).maxstacksize = 0;
        (*f).locvars = ptr::null_mut();
        (*f).sizelocvars = 0;
        (*f).linedefined = 0;
        (*f).lastlinedefined = 0;
        (*f).source = ptr::null_mut();
    }
    f
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_protosize(p: *mut Proto) -> usize {
    let mut sz = size_of::<Proto>()
        + unsafe { (*p).sizep.max(0) as usize } * size_of::<*mut Proto>()
        + unsafe { (*p).sizek.max(0) as usize } * size_of::<TValue>()
        + unsafe { (*p).sizelocvars.max(0) as usize } * size_of::<LocVar>()
        + unsafe { (*p).sizeupvalues.max(0) as usize } * size_of::<Upvaldesc>();
    if unsafe { (*p).flag & PF_FIXED } == 0 {
        sz += unsafe { (*p).sizecode.max(0) as usize } * size_of::<Instruction>();
        sz += unsafe { (*p).sizelineinfo.max(0) as usize } * size_of::<u8>();
        sz += unsafe { (*p).sizeabslineinfo.max(0) as usize } * size_of::<AbsLineInfo>();
    }
    sz
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_freeproto(state: *mut lua_State, f: *mut Proto) {
    if unsafe { (*f).flag & PF_FIXED } == 0 {
        unsafe {
            luaM_free_(
                state,
                (*f).code.cast(),
                (*f).sizecode.max(0) as usize * size_of::<Instruction>(),
            )
        };
        unsafe {
            luaM_free_(
                state,
                (*f).lineinfo.cast(),
                (*f).sizelineinfo.max(0) as usize * size_of::<u8>(),
            )
        };
        unsafe {
            luaM_free_(
                state,
                (*f).abslineinfo.cast(),
                (*f).sizeabslineinfo.max(0) as usize * size_of::<AbsLineInfo>(),
            )
        };
    }
    unsafe {
        luaM_free_(
            state,
            (*f).p.cast(),
            (*f).sizep.max(0) as usize * size_of::<*mut Proto>(),
        )
    };
    unsafe {
        luaM_free_(
            state,
            (*f).k.cast(),
            (*f).sizek.max(0) as usize * size_of::<TValue>(),
        )
    };
    unsafe {
        luaM_free_(
            state,
            (*f).locvars.cast(),
            (*f).sizelocvars.max(0) as usize * size_of::<LocVar>(),
        )
    };
    unsafe {
        luaM_free_(
            state,
            (*f).upvalues.cast(),
            (*f).sizeupvalues.max(0) as usize * size_of::<Upvaldesc>(),
        )
    };
    unsafe { luaM_free_(state, f.cast(), size_of::<Proto>()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaF_getlocalname(
    f: *const Proto,
    mut local_number: c_int,
    pc: c_int,
) -> *const c_char {
    let mut i = 0;
    while i < unsafe { (*f).sizelocvars }
        && unsafe { (*(*f).locvars.add(i as usize)).startpc <= pc }
    {
        let loc = unsafe { &*(*f).locvars.add(i as usize) };
        if pc < loc.endpc {
            local_number -= 1;
            if local_number == 0 {
                return unsafe { getstr(loc.varname) };
            }
        }
        i += 1;
    }
    ptr::null()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aux_rs::{luaL_checkversion_, luaL_newstate};
    use crate::luaffi::{LUA_VERSION_NUM, LUAL_NUMSIZES, lua_close};

    #[test]
    fn new_closures_and_proto_are_initialized() {
        let state = unsafe { luaL_newstate() };
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);

            let ccl = luaF_newCclosure(state, 3);
            assert_eq!((*ccl).nupvalues, 3);

            let lcl = luaF_newLclosure(state, 2);
            assert_eq!((*lcl).nupvalues, 2);
            assert!((*lcl).p.is_null());
            assert!((*lcl).upvals[0].is_null());
            assert!((*(*lcl).upvals.as_ptr().add(1)).is_null());

            let p = luaF_newproto(state);
            assert!((*p).k.is_null());
            assert_eq!((*p).sizek, 0);
            assert!(luaF_protosize(p) >= size_of::<Proto>());
        })();

        unsafe { lua_close(state) };
        result
    }
}
