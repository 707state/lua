use crate::debug::{luaG_findlocal, luaG_runerror};
use crate::do_rs::luaD_seterrorobj;
use crate::gc::luaC_newobj;
use crate::mem::luaM_free_;
use crate::runtime::*;
use crate::tm::luaT_gettmbyobj;
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr;

#[repr(C)]
union UpValV {
    p: *mut TValue,
    offset: isize,
}

#[repr(C)]
union UpValU {
    open: UpValOpen,
    value: TValue,
}

pub(crate) unsafe fn raw_luaF_newproto(state: *mut c_void) -> *mut c_void {
    unsafe { luaF_newproto(state.cast()).cast() }
}

pub(crate) unsafe fn raw_luaF_newLclosure(state: *mut c_void, nupvals: c_int) -> *mut c_void {
    unsafe { luaF_newLclosure(state.cast(), nupvals).cast() }
}

#[inline]
unsafe fn isfalse(value: *const TValue) -> bool {
    matches!(unsafe { (*value).tt_ }, LUA_VFALSE | LUA_VNIL)
}

#[inline]
unsafe fn uplevel(uv: *const UpVal) -> StkId {
    unsafe { (*uv).v.p.cast::<StackValue>() }
}

#[inline]
unsafe fn isintwups(state: *mut lua_State) -> bool {
    unsafe { (*state).twups != state }
}

#[inline]
unsafe fn savestack(state: *mut lua_State, pt: StkId) -> isize {
    let base = unsafe { (*state).stack.p.cast::<u8>() };
    let pt = pt.cast::<u8>();
    unsafe { pt.offset_from(base) }
}

#[inline]
unsafe fn restorestack(state: *mut lua_State, n: isize) -> StkId {
    unsafe {
        (*state)
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
    let mut top = unsafe { (*state).top.p };
    let func = top;
    let tm = unsafe { luaT_gettmbyobj(state, obj, TM_CLOSE) };
    unsafe { setobj2s(state, top, tm) };
    top = unsafe { top.add(1) };
    unsafe { setobj2s(state, top, obj) };
    top = unsafe { top.add(1) };
    if !err.is_null() {
        unsafe { setobj2s(state, top, err) };
        top = unsafe { top.add(1) };
    }
    unsafe { (*state).top.p = top };
    if yy != 0 {
        unsafe { luaD_call(state, func, 0) };
    } else {
        unsafe { luaD_callnoyield(state, func, 0) };
    }
}

unsafe fn checkclosemth(state: *mut lua_State, level: StkId) {
    let tm = unsafe { luaT_gettmbyobj(state, s2v(level), TM_CLOSE) };
    if unsafe { ttisnil(tm) } {
        let idx = unsafe { level.offset_from((*(*state).ci).func.p) as c_int };
        let mut vname = unsafe { luaG_findlocal(state, (*state).ci, idx, ptr::null_mut()) };
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
            unsafe { (*state).top.p = level.add(1) };
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
    let mut tbc = unsafe { (*state).tbclist.p };
    let delta = unsafe { (*tbc).tbclist.delta as usize };
    tbc = unsafe { tbc.sub(delta) };
    while tbc > unsafe { (*state).stack.p } && unsafe { (*tbc).tbclist.delta == 0 } {
        tbc = unsafe { tbc.sub(MAXDELTA) };
    }
    unsafe { (*state).tbclist.p = tbc };
}

pub(crate) unsafe fn luaF_newCclosure(state: *mut lua_State, nupvals: c_int) -> *mut CClosure {
    let o = unsafe { luaC_newobj(state, LUA_VCCL, size_cclosure(nupvals)) };
    let c = o.cast::<CClosure>();
    unsafe { (*c).nupvalues = nupvals as u8 };
    c
}

#[unsafe(no_mangle)]
pub unsafe  fn luaF_newLclosure(
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
pub unsafe  fn luaF_initupvals(state: *mut lua_State, cl: *mut LClosure) {
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
            (*state).twups = (*G(state)).twups;
            (*G(state)).twups = state;
        }
    }
    uv
}

#[unsafe(no_mangle)]
pub unsafe  fn luaF_findupval(state: *mut lua_State, level: StkId) -> *mut UpVal {
    let mut pp = unsafe { ptr::addr_of_mut!((*state).openupval) };
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

pub(crate) unsafe fn luaF_newtbcupval(state: *mut lua_State, level: StkId) {
    if unsafe { isfalse(s2v(level)) } {
        return;
    }
    unsafe { checkclosemth(state, level) };
    while unsafe { level.offset_from((*state).tbclist.p) as usize > MAXDELTA } {
        unsafe {
            (*state).tbclist.p = (*state).tbclist.p.add(MAXDELTA);
            (*(*state).tbclist.p).tbclist.delta = 0;
        }
    }
    unsafe {
        (*level).tbclist.delta = level.offset_from((*state).tbclist.p) as u16;
        (*state).tbclist.p = level;
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaF_unlinkupval(uv: *mut UpVal) {
    unsafe {
        *(*uv).u.open.previous = (*uv).u.open.next;
        if !(*uv).u.open.next.is_null() {
            (*(*uv).u.open.next).u.open.previous = (*uv).u.open.previous;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaF_closeupval(state: *mut lua_State, level: StkId) {
    let mut uv = unsafe { (*state).openupval };
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
        uv = unsafe { (*state).openupval };
    }
}

pub(crate) unsafe fn luaF_close(
    state: *mut lua_State,
    mut level: StkId,
    status: TStatus,
    yy: c_int,
) -> StkId {
    let levelrel = unsafe { savestack(state, level) };
    unsafe { luaF_closeupval(state, level) };
    while unsafe { (*state).tbclist.p >= level } {
        let tbc = unsafe { (*state).tbclist.p };
        unsafe { poptbclist(state) };
        unsafe { prepcallclosemth(state, tbc, status, yy) };
        level = unsafe { restorestack(state, levelrel) };
    }
    level
}

#[unsafe(no_mangle)]
pub unsafe  fn luaF_newproto(state: *mut lua_State) -> *mut Proto {
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
pub unsafe  fn luaF_protosize(p: *mut Proto) -> usize {
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
pub unsafe fn luaF_freeproto(state: *mut lua_State, f: *mut Proto) {
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

pub unsafe fn luaF_getlocalname(
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
    use crate::{aux_rs::{luaL_checkversion_, luaL_newstate}, luaffi::LUAL_NUMSIZES, state::lua_close};

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
