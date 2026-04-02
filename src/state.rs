use crate::api::lua_tolstring;
use crate::debug::*;
use crate::do_rs::*;
use crate::func::luaF_closeupval;
use crate::gc::*;
use crate::lex::raw_luaX_init;
use crate::luavm::GlobalState;
use crate::mem::{luaM_free_, luaM_malloc_};
use crate::object::luaO_codeparam;
use crate::runtime::*;
use crate::string::raw_luaS_init;
use crate::table::{raw_luaH_new, raw_luaH_resize, raw_luaH_setint};
use crate::tm::raw_luaT_init;
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr;

/// Local alias for `G()` from runtime.rs (lowercase for consistency with original code).
#[inline]
unsafe fn g(state: *mut lua_State) -> *mut GlobalState {
    unsafe { G(state) }
}

#[inline]
unsafe fn setthvalue2s(state: *mut lua_State, stack: StkId, thread: *mut lua_State) {
    let _ = state;
    unsafe { setthvalue(state, s2v(stack), thread) };
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
unsafe fn lua_c_white(g: *mut GlobalState) -> u8 {
    unsafe { (*g).currentwhite & WHITEBITS }
}

#[inline]
unsafe fn completestate(g: *mut GlobalState) -> bool {
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

unsafe  fn f_luaopen(state: *mut lua_State, ud: *mut c_void) {
    let _ = ud;
    let g = unsafe { g(state) };
    unsafe { stack_init(state, state) };
    unsafe { init_registry(state, g) };
    unsafe { raw_luaS_init(state.cast()) };
    unsafe { raw_luaT_init(state.cast()) };
    unsafe { raw_luaX_init(state.cast()) };
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
    unsafe {
        free_stackvector(
            state,
            (*state).stack.p,
            (stacksize(state) as usize) + EXTRA_STACK,
        )
    };
}

unsafe fn init_registry(state: *mut lua_State, g: *mut GlobalState) {
    let mut aux = TValue {
        value_: Value { ub: 0 },
        tt_: LUA_VNIL,
    };
    let registry = unsafe { raw_luaH_new(state.cast()).cast::<Table>() };
    unsafe { sethvalue(ptr::addr_of_mut!((*g).l_registry), registry) };
    unsafe { raw_luaH_resize(state.cast(), registry.cast(), LUA_RIDX_LAST as u32, 0) };
    unsafe { setbfvalue(ptr::addr_of_mut!(aux)) };
    unsafe {
        raw_luaH_setint(
            state.cast(),
            registry.cast(),
            1,
            ptr::addr_of_mut!(aux).cast(),
        )
    };
    unsafe { setthvalue(state, ptr::addr_of_mut!(aux), state) };
    unsafe {
        raw_luaH_setint(
            state.cast(),
            registry.cast(),
            LUA_RIDX_MAINTHREAD,
            ptr::addr_of_mut!(aux).cast(),
        )
    };
    unsafe {
        sethvalue(
            ptr::addr_of_mut!(aux),
            raw_luaH_new(state.cast()).cast::<Table>(),
        )
    };
    unsafe {
        raw_luaH_setint(
            state.cast(),
            registry.cast(),
            LUA_RIDX_GLOBALS,
            ptr::addr_of_mut!(aux).cast(),
        )
    };
}

unsafe fn preinit_thread(state: *mut lua_State, g: *mut GlobalState) {
    unsafe {
        (*state).l_G = g;
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
        unsafe { frealloc((*g).ud, g.cast(), size_of::<GlobalState>(), 0) };
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaE_setdebt(g: *mut GlobalState, mut debt: l_mem) {
    let tb = unsafe { gettotalbytes(g) };
    if debt > MAX_LMEM - tb {
        debt = MAX_LMEM - tb;
    }
    unsafe {
        (*g).gctotalbytes = tb + debt;
        (*g).gcdebt = debt;
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaE_extendCI(state: *mut lua_State) -> *mut CallInfo {
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
pub unsafe  fn luaE_shrinkCI(state: *mut lua_State) {
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
pub unsafe  fn luaE_checkcstack(state: *mut lua_State) {
    let calls = unsafe { get_ccalls(state) };
    if calls == LUAI_MAXCCALLS {
        unsafe { luaG_runerror(state, c"C stack overflow".as_ptr()) };
    } else if calls >= (LUAI_MAXCCALLS / 10 * 11) {
        unsafe { luaD_errerr(state) };
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaE_incCstack(state: *mut lua_State) {
    unsafe { (*state).nCcalls += 1 };
    if unsafe { get_ccalls(state) } >= LUAI_MAXCCALLS {
        unsafe { luaE_checkcstack(state) };
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaE_threadsize(state: *mut lua_State) -> usize {
    let mut sz =
        size_of::<LX>() + (unsafe { (*state).nci.max(0) as usize } * size_of::<CallInfo>());
    if unsafe { !(*state).stack.p.is_null() } {
        sz += ((unsafe { stacksize(state) as usize }) + EXTRA_STACK) * size_of::<StackValue>();
    }
    sz
}

pub unsafe fn lua_newthread(state: *mut lua_State) -> *mut lua_State {
    let g = unsafe { g(state) };
    if unsafe { (*g).gcdebt <= 0 } {
        unsafe { luaC_step(state) };
    }
    let o = unsafe { luaC_newobjdt(state, LUA_TTHREAD, size_of::<LX>(), offset_of!(LX, l)) };
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
pub unsafe  fn luaE_freethread(state: *mut lua_State, thread: *mut lua_State) {
    let l = unsafe { thread.cast::<u8>().sub(offset_of!(LX, l)).cast::<LX>() };
    unsafe { luaF_closeupval(thread, (*thread).stack.p) };
    unsafe { freestack(thread) };
    unsafe { luaM_free_(state, l.cast(), size_of::<LX>()) };
}

#[unsafe(no_mangle)]
pub unsafe  fn luaE_resetthread(
    state: *mut lua_State,
    mut status: TStatus,
) -> TStatus {
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

pub(crate) unsafe  fn lua_closethread(
    state: *mut lua_State,
    from: *mut lua_State,
) -> c_int {
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
pub unsafe  fn lua_newstate(
    f: lua_Alloc,
    ud: *mut c_void,
    seed: u32,
) -> *mut lua_State {
    let Some(frealloc) = f else {
        return ptr::null_mut();
    };
    let g = unsafe {
        frealloc(
            ud,
            ptr::null_mut(),
            LUA_TTHREAD as usize,
            size_of::<GlobalState>(),
        )
    }
    .cast::<GlobalState>();
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
        (*g).gcstate = GCSPAUSE;
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
        (*g).gctotalbytes = size_of::<GlobalState>() as l_mem;
        (*g).gcmarked = 0;
        (*g).gcdebt = 0;
        setivalue(ptr::addr_of_mut!((*g).nilvalue), 0);
        (*g).gcparams[LUA_GCPPAUSE] = luaO_codeparam(LUAI_GCPAUSE as u32);
        (*g).gcparams[LUA_GCPSTEPMUL] = luaO_codeparam(LUAI_GCMUL as u32);
        (*g).gcparams[LUA_GCPSTEPSIZE] = luaO_codeparam((200 * size_of::<Table>()) as c_int as u32);
        (*g).gcparams[LUA_GCPMINORMUL] = luaO_codeparam(LUAI_GENMINORMUL as u32);
        (*g).gcparams[LUA_GCPMINORMAJOR] = luaO_codeparam(LUAI_MINORMAJOR as u32);
        (*g).gcparams[LUA_GCPMAJORMINOR] = luaO_codeparam(LUAI_MAJORMINOR as u32);
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
pub unsafe  fn lua_close(state: *mut lua_State) {
    let main = unsafe { mainthread(g(state)) };
    unsafe { close_state(main) };
}

pub(crate) unsafe fn luaE_warning(state: *mut lua_State, msg: *const c_char, tocont: c_int) {
    let g = unsafe { g(state) };
    if let Some(warnf) = unsafe { (*g).warnf } {
        unsafe { warnf((*g).ud_warn, msg, tocont) };
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaE_warnerror(state: *mut lua_State, where_: *const c_char) {
    let errobj = unsafe { s2v((*state).top.p.sub(1)) };
    let msg = if unsafe { ((*errobj).tt_ & 0x0f) == 4 } {
        unsafe { lua_tolstring(state.cast(), -1, ptr::null_mut()) }
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
    use crate::{aux_rs::{luaL_checkversion_, luaL_newstate}, luaffi::*};

    unsafe fn test_hook(_: *mut lua_State, _: *mut lua_Debug) {}

    #[test]
    fn newthread_copies_hook_state_and_closes() {
        let state = { luaL_newstate() }.cast::<lua_State>();
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
        let state = { luaL_newstate() }.cast::<lua_State>();
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
