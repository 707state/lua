#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

use crate::debug::*;
use crate::func::luaF_initupvals;
use crate::luaffi::strchr;
use crate::mem::luaM_realloc_;
use crate::parser_rs::luaY_parser;
use crate::runtime::*;
use crate::state::*;
use crate::undump::luaU_undump;
use crate::object::*;
use crate::string::*;
use crate::vm_rs::*;
use crate::zio::*;
use core::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::abort;

// 线程局部变量：记录当前线程处于 luaD_rawrunprotected 保护区的层数。
// panic hook 利用此标记来静默 LuaError/LuaErrorBase 产生的 panic 消息，
// 避免在正常的 Lua 错误处理流程中打印 "panicked at ..." 噪声。
std::thread_local! {
    static LUA_PROTECTED_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// 返回当前线程是否处于至少一层 luaD_rawrunprotected 保护区中。
#[inline]
pub(crate) fn in_lua_protected_call() -> bool {
    LUA_PROTECTED_DEPTH.with(|d| d.get() > 0)
}

/// 安装全局 panic hook（只需调用一次）：
/// 对 LuaError / LuaErrorBase 类型的 panic 且在 Lua 保护区内时，静默输出。
pub(crate) fn install_lua_panic_hook() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            // 如果当前线程正在 Lua 保护区内，且 payload 是 LuaError 或 LuaErrorBase，
            // 则静默处理（不打印 panic 消息），因为这是正常的 Lua 错误传播。
            if in_lua_protected_call() {
                if let Some(payload) = info.payload().downcast_ref::<LuaError>() {
                    let _ = payload; // 静默
                    return;
                }
                if let Some(payload) = info.payload().downcast_ref::<LuaErrorBase>() {
                    let _ = payload; // 静默
                    return;
                }
            }
            // 其他 panic（真实错误）照常走原来的 hook
            prev(info);
        }));
    });
}

#[repr(C)]
struct SParser {
    z: *mut ZIO,
    buff: Mbuffer,
    dyd: Dyndata,
    mode: *const c_char,
    name: *const c_char,
}

#[repr(C)]
struct CloseP {
    level: StkId,
    status: TStatus,
}

#[inline]
unsafe fn errorstatus(status: TStatus) -> bool {
    status > LUA_YIELD
}

#[inline]
unsafe fn stacksize(L: *mut lua_State) -> c_int {
    unsafe { (*L).stack_last.p.offset_from((*L).stack.p) as c_int }
}

#[inline]
unsafe fn getCcalls(L: *mut lua_State) -> u32 {
    unsafe { (*L).nCcalls & 0xffff }
}

#[inline]
unsafe fn incnny(L: *mut lua_State) {
    unsafe { (*L).nCcalls = (*L).nCcalls.wrapping_add(0x10000) };
}

#[inline]
unsafe fn decnny(L: *mut lua_State) {
    unsafe { (*L).nCcalls = (*L).nCcalls.wrapping_sub(0x10000) };
}

#[inline]
unsafe fn pcRel(pc: *const Instruction, p: *mut Proto) -> c_int {
    unsafe { pc.offset_from((*p).code) as c_int - 1 }
}

#[inline]
unsafe fn get_nresults(cs: u32) -> c_int {
    (cs & CIST_NRESULTS) as c_int - 1
}

#[inline]
unsafe fn getcistrecst(ci: *mut CallInfo) -> TStatus {
    unsafe { (((*ci).callstatus >> CIST_RECST) & 7) as TStatus }
}

#[inline]
unsafe fn setcistrecst(ci: *mut CallInfo, st: TStatus) {
    unsafe {
        api_check((st & 7) == st, "status overflow");
        (*ci).callstatus = ((*ci).callstatus & !(7u32 << CIST_RECST)) | ((st as u32) << CIST_RECST);
    }
}

#[inline]
unsafe fn getoah(ci: *mut CallInfo) -> u8 {
    unsafe {
        if (*ci).callstatus & CIST_OAH != 0 {
            1
        } else {
            0
        }
    }
}

#[inline]
unsafe fn checkstack(L: *mut lua_State, n: c_int) {
    if unsafe { (*L).stack_last.p.offset_from((*L).top.p) as c_int <= n } {
        unsafe { luaD_growstack(L, n, 1) };
    }
}

#[inline]
unsafe fn uplevel(up: *mut UpVal) -> StkId {
    unsafe { (*up).v.p.cast() }
}

pub unsafe fn luaD_seterrorobj(L: *mut lua_State, errcode: TStatus, oldtop: StkId) {
    if errcode == LUA_ERRMEM {
        unsafe { setsvalue2s(L, oldtop, (*G(L)).memerrmsg) };
    } else {
        unsafe {
            api_check(errorstatus(errcode), "real error expected");
            api_check(
                !ttisnil(s2v((*L).top.p.sub(1))),
                "non-nil error object expected",
            );
            setobjs2s(L, oldtop, (*L).top.p.sub(1));
        }
    }
    unsafe { (*L).top.p = oldtop.add(1) };
}

pub unsafe fn luaD_throw(L: *mut lua_State, mut errcode: TStatus) -> ! {
    if unsafe { (*L).nesting_level > 0 } {
        // 在 luaD_rawrunprotected 保护内：用 LuaError panic 跳出，由 catch_unwind 捕获
        std::panic::panic_any(LuaError(errcode));
    } else {
        // 没有保护点：尝试主线程或调用 panic handler
        let g = unsafe { G(L) };
        let mainth = unsafe { mainthread(g) };
        errcode = unsafe { luaE_resetthread(L, errcode) };
        unsafe {
            (*L).status = LuaStatus::from_u8(errcode).expect("lua_State.status must be valid")
        };
        if unsafe { (*mainth).nesting_level > 0 } {
            unsafe {
                setobjs2s(L, (*mainth).top.p, (*L).top.p.sub(1));
                (*mainth).top.p = (*mainth).top.p.add(1);
            }
            unsafe { luaD_throw(mainth, errcode) };
        } else {
            if let Some(panicf) = unsafe { (*g).panic } {
                unsafe { panicf(L) };
            }
            abort()
        }
    }
}

pub unsafe fn luaD_throwbaselevel(_L: *mut lua_State, errcode: TStatus) -> ! {
    // LuaErrorBase 会被每层 catch_unwind 识别并重新抛出，直到最外层才会被捕获并转为 LuaError
    std::panic::panic_any(LuaErrorBase(errcode));
}

pub unsafe fn luaD_rawrunprotected(L: *mut lua_State, f: Pfunc, ud: *mut c_void) -> TStatus {
    let oldnCcalls = unsafe { (*L).nCcalls };
    // 进入保护区：lua_State 嵌套层数 +1，thread_local 深度计数 +1
    unsafe { (*L).nesting_level = (*L).nesting_level.wrapping_add(1) };
    LUA_PROTECTED_DEPTH.with(|d| d.set(d.get().wrapping_add(1)));

    let result = catch_unwind(AssertUnwindSafe(|| {
        if let Some(f) = f {
            unsafe { f(L, ud) };
        }
    }));

    // 离开保护区：嵌套层数 -1，thread_local 深度计数 -1
    unsafe { (*L).nesting_level = (*L).nesting_level.wrapping_sub(1) };
    LUA_PROTECTED_DEPTH.with(|d| d.set(d.get().wrapping_sub(1)));
    unsafe { (*L).nCcalls = oldnCcalls };

    match result {
        Ok(()) => LUA_OK,
        Err(payload) => {
            // 尝试 downcast 为 LuaError（来自 luaD_throw）
            if let Some(&LuaError(code)) = payload.downcast_ref::<LuaError>() {
                return code;
            }
            // 尝试 downcast 为 LuaErrorBase（来自 luaD_throwbaselevel）
            if let Some(&LuaErrorBase(code)) = payload.downcast_ref::<LuaErrorBase>() {
                // 如果当前层已是最外层（nesting_level == 0），则在此层捕获
                if unsafe { (*L).nesting_level == 0 } {
                    return code;
                }
                // 否则继续向外传播
                std::panic::resume_unwind(Box::new(LuaErrorBase(code)));
            }
            // 其他 panic（非 Lua 错误）：继续传播
            std::panic::resume_unwind(payload);
        }
    }
}

pub unsafe fn luaD_errerr(L: *mut lua_State) -> ! {
    let msg = unsafe { luaS_new(L, c"error in error handling".as_ptr()) };
    unsafe {
        setsvalue2s(L, (*L).top.p, msg);
        (*L).top.p = (*L).top.p.add(1);
        luaD_throw(L, LUA_ERRERR);
    }
}

pub unsafe fn luaD_checkminstack(L: *mut lua_State) -> c_int {
    ((unsafe { stacksize(L) } < MAXSTACK - BASIC_STACK_SIZE as i32)
        && (unsafe { getCcalls(L) } < LUAI_MAXCCALLS - 2)) as c_int
}

unsafe fn relstack(L: *mut lua_State) {
    let mut ci = unsafe { (*L).ci };
    let mut up = unsafe { (*L).openupval };
    unsafe {
        (*L).top.offset = savestack(L, (*L).top.p);
        (*L).tbclist.offset = savestack(L, (*L).tbclist.p);
    }
    while !up.is_null() {
        unsafe {
            (*up).v.offset = savestack(L, uplevel(up));
            up = (*up).u.open.next;
        }
    }
    while !ci.is_null() {
        unsafe {
            (*ci).top.offset = savestack(L, (*ci).top.p);
            (*ci).func.offset = savestack(L, (*ci).func.p);
            ci = (*ci).previous;
        }
    }
}

unsafe fn correctstack(L: *mut lua_State, _oldstack: StkId) {
    let mut ci = unsafe { (*L).ci };
    let mut up = unsafe { (*L).openupval };
    unsafe {
        (*L).top.p = restorestack(L, (*L).top.offset);
        (*L).tbclist.p = restorestack(L, (*L).tbclist.offset);
    }
    while !up.is_null() {
        unsafe {
            (*up).v.p = s2v(restorestack(L, (*up).v.offset));
            up = (*up).u.open.next;
        }
    }
    while !ci.is_null() {
        unsafe {
            (*ci).top.p = restorestack(L, (*ci).top.offset);
            (*ci).func.p = restorestack(L, (*ci).func.offset);
            if isLua(ci) {
                (*ci).u.l.trap = 1;
            }
            ci = (*ci).previous;
        }
    }
}

pub unsafe fn luaD_reallocstack(L: *mut lua_State, newsize: c_int, raiseerror: c_int) -> c_int {
    let oldsize = unsafe { stacksize(L) };
    let oldstack = unsafe { (*L).stack.p };
    let oldgcstop = unsafe { (*G(L)).gcstopem };
    unsafe {
        api_check(
            newsize <= MAXSTACK || newsize == ERRORSTACKSIZE,
            "invalid stack size",
        )
    };
    unsafe { relstack(L) };
    unsafe { (*G(L)).gcstopem = 1 };
    let newstack = unsafe {
        crate::mem::luaM_realloc_(
            L,
            oldstack.cast(),
            (oldsize + EXTRA_STACK as i32) as usize * size_of::<StackValue>(),
            (newsize + EXTRA_STACK as i32) as usize * size_of::<StackValue>(),
        )
        .cast::<StackValue>()
    };
    unsafe { (*G(L)).gcstopem = oldgcstop };
    if newstack.is_null() {
        unsafe { correctstack(L, oldstack) };
        if raiseerror != 0 {
            unsafe { luaD_throw(L, LUA_ERRMEM) };
        }
        return 0;
    }
    unsafe {
        (*L).stack.p = newstack;
        correctstack(L, oldstack);
        (*L).stack_last.p = (*L).stack.p.add(newsize as usize);
    }
    for i in (oldsize + EXTRA_STACK as i32)..(newsize + EXTRA_STACK as i32) {
        unsafe { setnilvalue(s2v(newstack.add(i as usize))) };
    }
    1
}

pub unsafe fn luaD_growstack(L: *mut lua_State, n: c_int, raiseerror: c_int) -> c_int {
    let size = unsafe { stacksize(L) };
    if size > MAXSTACK {
        unsafe { api_check(stacksize(L) == ERRORSTACKSIZE, "error stack expected") };
        if raiseerror != 0 {
            unsafe { luaD_errerr(L) };
        }
        0
    } else if n < MAXSTACK {
        let mut newsize = size + (size >> 1);
        let needed = unsafe { (*L).top.p.offset_from((*L).stack.p) as c_int + n };
        if newsize > MAXSTACK {
            newsize = MAXSTACK;
        }
        if newsize < needed {
            newsize = needed;
        }
        if newsize <= MAXSTACK {
            return unsafe { luaD_reallocstack(L, newsize, raiseerror) };
        }
        unsafe { luaD_reallocstack(L, ERRORSTACKSIZE, raiseerror) };
        if raiseerror != 0 {
            unsafe { luaG_runerror(L, "stack overflow") };
        }
        0
    } else {
        unsafe { luaD_reallocstack(L, ERRORSTACKSIZE, raiseerror) };
        if raiseerror != 0 {
            unsafe { luaG_runerror(L, "stack overflow") };
        }
        0
    }
}

unsafe fn stackinuse(L: *mut lua_State) -> c_int {
    let mut ci = unsafe { (*L).ci };
    let mut lim = unsafe { (*L).top.p };
    while !ci.is_null() {
        if lim < unsafe { (*ci).top.p } {
            lim = unsafe { (*ci).top.p };
        }
        ci = unsafe { (*ci).previous };
    }
    let mut res = unsafe { lim.offset_from((*L).stack.p) as c_int + 1 };
    if res < LUA_MINSTACK as i32 {
        res = LUA_MINSTACK as i32;
    }
    res
}

pub unsafe fn luaD_shrinkstack(L: *mut lua_State) {
    let inuse = unsafe { stackinuse(L) };
    let max = if inuse > MAXSTACK / 3 {
        MAXSTACK
    } else {
        inuse * 3
    };
    if inuse <= MAXSTACK && unsafe { stacksize(L) } > max {
        let nsize = if inuse > MAXSTACK / 2 {
            MAXSTACK
        } else {
            inuse * 2
        };
        unsafe { luaD_reallocstack(L, nsize, 0) };
    }
    unsafe { luaE_shrinkCI(L) };
}

pub unsafe fn luaD_inctop(L: *mut lua_State) {
    unsafe {
        (*L).top.p = (*L).top.p.add(1);
        checkstack(L, 1);
    }
}

pub unsafe fn luaD_hook(
    L: *mut lua_State,
    event: c_int,
    line: c_int,
    ftransfer: c_int,
    ntransfer: c_int,
) {
    let hook = unsafe { (*L).hook };
    if let Some(hookf) = hook {
        if unsafe { (*L).allowhook } != 0 {
            let ci = unsafe { (*L).ci };
            let top = unsafe { savestack(L, (*L).top.p) };
            let ci_top = unsafe { savestack(L, (*ci).top.p) };
            let mut ar = lua_Debug {
                event,
                name: ptr::null(),
                namewhat: ptr::null(),
                what: ptr::null(),
                source: ptr::null(),
                srclen: 0,
                currentline: line,
                linedefined: 0,
                lastlinedefined: 0,
                nups: 0,
                nparams: 0,
                isvararg: 0,
                extraargs: 0,
                istailcall: 0,
                ftransfer: 0,
                ntransfer: 0,
                short_src: [0; 60],
                i_ci: ci,
            };
            unsafe {
                (*L).transferinfo.ftransfer = ftransfer;
                (*L).transferinfo.ntransfer = ntransfer;
                if isLua(ci) && (*L).top.p < (*ci).top.p {
                    (*L).top.p = (*ci).top.p;
                }
                checkstack(L, LUA_MINSTACK as i32);
                if (*ci).top.p < (*L).top.p.add(LUA_MINSTACK as usize) {
                    (*ci).top.p = (*L).top.p.add(LUA_MINSTACK as usize);
                }
                (*L).allowhook = 0;
                (*ci).callstatus |= CIST_HOOKED;
            }
            unsafe { hookf(L, ptr::addr_of_mut!(ar).cast()) };
            unsafe {
                (*L).allowhook = 1;
                (*ci).top.p = restorestack(L, ci_top);
                (*L).top.p = restorestack(L, top);
                (*ci).callstatus &= !CIST_HOOKED;
            }
        }
    }
}

pub unsafe fn luaD_hookcall(L: *mut lua_State, ci: *mut CallInfo) {
    unsafe { (*L).oldpc = 0 };
    if unsafe { (*L).hookmask & LUA_MASKCALL } != 0 {
        let event = if unsafe { (*ci).callstatus & CIST_TAIL } != 0 {
            LUA_HOOKTAILCALL
        } else {
            LUA_HOOKCALL
        };
        let p = unsafe { ci_func(ci) };
        unsafe {
            (*ci).u.l.savedpc = (*ci).u.l.savedpc.add(1);
            luaD_hook(L, event, -1, 1, (*(*p).p).numparams as c_int);
            (*ci).u.l.savedpc = (*ci).u.l.savedpc.sub(1);
        }
    }
}

unsafe fn rethook(L: *mut lua_State, mut ci: *mut CallInfo, nres: c_int) {
    if unsafe { (*L).hookmask & LUA_MASKRET } != 0 {
        let firstres = unsafe { (*L).top.p.sub(nres as usize) };
        let mut delta = 0;
        if unsafe { isLua(ci) } {
            let p = unsafe { (*ci_func(ci)).p };
            if unsafe { (*p).flag & PF_VAHID } != 0 {
                delta = unsafe { (*ci).u.l.nextraargs + (*p).numparams as c_int + 1 };
            }
        }
        unsafe {
            (*ci).func.p = (*ci).func.p.add(delta as usize);
            luaD_hook(
                L,
                LUA_HOOKRET,
                -1,
                firstres.offset_from((*ci).func.p) as c_int,
                nres,
            );
            (*ci).func.p = (*ci).func.p.sub(delta as usize);
        }
    }
    ci = unsafe { (*ci).previous };
    if !ci.is_null() && unsafe { isLua(ci) } {
        unsafe { (*L).oldpc = pcRel((*ci).u.l.savedpc, (*ci_func(ci)).p) };
    }
}

unsafe fn tryfuncTM(L: *mut lua_State, func: StkId, status: u32) -> u32 {
    let tm = unsafe { crate::tm::luaT_gettmbyobj(L, s2v(func), TagMethod::Call) };
    if unsafe { ttisnil(tm) } {
        unsafe { luaG_callerror(L, s2v(func)) };
    }
    let mut p = unsafe { (*L).top.p };
    while p > func {
        unsafe {
            setobjs2s(L, p, p.sub(1));
            p = p.sub(1);
        }
    }
    unsafe {
        (*L).top.p = (*L).top.p.add(1);
        setobj2s(L, func, tm);
    }
    if status & MAX_CCMT == MAX_CCMT {
        unsafe { luaG_runerror(L, "'__call' chain too long") };
    }
    status + (1u32 << CIST_CCMT)
}

unsafe fn genmoveresults(L: *mut lua_State, res: StkId, mut nres: c_int, wanted: c_int) {
    let firstresult = unsafe { (*L).top.p.sub(nres as usize) };
    if nres > wanted {
        nres = wanted;
    }
    let mut i = 0;
    while i < nres {
        unsafe { setobjs2s(L, res.add(i as usize), firstresult.add(i as usize)) };
        i += 1;
    }
    while i < wanted {
        unsafe { setnilvalue(s2v(res.add(i as usize))) };
        i += 1;
    }
    unsafe { (*L).top.p = res.add(wanted as usize) };
}

unsafe fn moveresults(L: *mut lua_State, mut res: StkId, nres: c_int, fwanted: u32) {
    match fwanted {
        1 => unsafe { (*L).top.p = res },
        2 => {
            if nres == 0 {
                unsafe { setnilvalue(s2v(res)) };
            } else {
                unsafe { setobjs2s(L, res, (*L).top.p.sub(nres as usize)) };
            }
            unsafe { (*L).top.p = res.add(1) };
        }
        x if x == (LUA_MULTRET + 1) as u32 => unsafe { genmoveresults(L, res, nres, nres) },
        _ => {
            let mut wanted = unsafe { get_nresults(fwanted) };
            if fwanted & CIST_TBC != 0 {
                unsafe {
                    (*(*L).ci).u2.nres = nres;
                    (*(*L).ci).callstatus |= CIST_CLSRET;
                    res = luaF_close(L, res, CLOSEKTOP, 1);
                    (*(*L).ci).callstatus &= !CIST_CLSRET;
                }
                if unsafe { (*L).hookmask } != 0 {
                    let savedres = unsafe { savestack(L, res) };
                    unsafe { rethook(L, (*L).ci, nres) };
                    res = unsafe { restorestack(L, savedres) };
                }
                if wanted == LUA_MULTRET {
                    wanted = nres;
                }
            }
            unsafe { genmoveresults(L, res, nres, wanted) };
        }
    }
}

pub unsafe fn luaD_poscall(L: *mut lua_State, ci: *mut CallInfo, nres: c_int) {
    let fwanted = unsafe { (*ci).callstatus & (CIST_TBC | CIST_NRESULTS) };
    if unsafe { (*L).hookmask } != 0 && fwanted & CIST_TBC == 0 {
        unsafe { rethook(L, ci, nres) };
    }
    unsafe { moveresults(L, (*ci).func.p, nres, fwanted) };
    unsafe {
        api_check(
            (*ci).callstatus & (CIST_HOOKED | CIST_YPCALL | CIST_FIN | CIST_CLSRET) == 0,
            "invalid call status on return",
        );
        (*L).ci = (*ci).previous;
    }
}

unsafe fn next_ci(L: *mut lua_State) -> *mut CallInfo {
    let next = unsafe { (*(*L).ci).next };
    if next.is_null() {
        unsafe { luaE_extendCI(L) }
    } else {
        next
    }
}

unsafe fn prepCallInfo(L: *mut lua_State, func: StkId, status: u32, top: StkId) -> *mut CallInfo {
    let ci = unsafe { next_ci(L) };
    unsafe {
        (*L).ci = ci;
        (*ci).func.p = func;
        (*ci).callstatus = status;
        (*ci).top.p = top;
    }
    ci
}

unsafe fn precallC(L: *mut lua_State, mut func: StkId, status: u32, f: lua_CFunction) -> c_int {
    unsafe { checkstackp(L, LUA_MINSTACK as i32, &mut func) };
    let ci = unsafe {
        prepCallInfo(
            L,
            func,
            status | CIST_C,
            (*L).top.p.add(LUA_MINSTACK as usize),
        )
    };
    if unsafe { (*L).hookmask & LUA_MASKCALL } != 0 {
        let narg = unsafe { (*L).top.p.offset_from(func) as c_int - 1 };
        unsafe { luaD_hook(L, LUA_HOOKCALL, -1, 1, narg) };
    }
    let n = unsafe { f.expect("C function")(L) };
    unsafe { api_checknelems(L, n) };
    unsafe { luaD_poscall(L, ci, n) };
    n
}

pub unsafe fn luaD_pretailcall(
    L: *mut lua_State,
    ci: *mut CallInfo,
    mut func: StkId,
    mut narg1: c_int,
    delta: c_int,
) -> c_int {
    let mut status = (LUA_MULTRET + 1) as u32;
    loop {
        match unsafe { ttypetag(s2v(func)) } {
            LUA_VCCL => {
                return unsafe { precallC(L, func, status, (*clCvalue(s2v(func))).f) };
            }
            LUA_VLCF => return unsafe { precallC(L, func, status, fvalue(s2v(func))) },
            LUA_VLCL => {
                let p = unsafe { (*clLvalue(s2v(func))).p };
                let fsize = unsafe { (*p).maxstacksize as c_int };
                let nfixparams = unsafe { (*p).numparams as c_int };
                unsafe { checkstackp(L, fsize - delta, &mut func) };
                unsafe { (*ci).func.p = (*ci).func.p.sub(delta as usize) };
                for i in 0..narg1 {
                    unsafe { setobjs2s(L, (*ci).func.p.add(i as usize), func.add(i as usize)) };
                }
                func = unsafe { (*ci).func.p };
                while narg1 <= nfixparams {
                    unsafe { setnilvalue(s2v(func.add(narg1 as usize))) };
                    narg1 += 1;
                }
                unsafe {
                    (*ci).top.p = func.add(1 + fsize as usize);
                    (*ci).u.l.savedpc = (*p).code;
                    (*ci).callstatus |= CIST_TAIL;
                    (*L).top.p = func.add(narg1 as usize);
                }
                return -1;
            }
            _ => {
                unsafe { checkstackp(L, 1, &mut func) };
                status = unsafe { tryfuncTM(L, func, status) };
                narg1 += 1;
            }
        }
    }
}

pub unsafe fn luaD_precall(L: *mut lua_State, mut func: StkId, nresults: c_int) -> *mut CallInfo {
    let mut status = (nresults + 1) as u32;
    unsafe { api_check(status <= (MAXRESULTS + 1) as u32, "invalid result count") };
    loop {
        match unsafe { ttypetag(s2v(func)) } {
            LUA_VCCL => {
                unsafe { precallC(L, func, status, (*clCvalue(s2v(func))).f) };
                return ptr::null_mut();
            }
            LUA_VLCF => {
                unsafe { precallC(L, func, status, fvalue(s2v(func))) };
                return ptr::null_mut();
            }
            LUA_VLCL => {
                let p = unsafe { (*clLvalue(s2v(func))).p };
                let mut narg = unsafe { (*L).top.p.offset_from(func) as c_int - 1 };
                let nfixparams = unsafe { (*p).numparams as c_int };
                let fsize = unsafe { (*p).maxstacksize as c_int };
                unsafe { checkstackp(L, fsize, &mut func) };
                let ci = unsafe { prepCallInfo(L, func, status, func.add(1 + fsize as usize)) };
                unsafe { (*ci).u.l.savedpc = (*p).code };
                while narg < nfixparams {
                    unsafe { setnilvalue(s2v((*L).top.p)) };
                    unsafe { (*L).top.p = (*L).top.p.add(1) };
                    narg += 1;
                }
                return ci;
            }
            _ => {
                unsafe { checkstackp(L, 1, &mut func) };
                status = unsafe { tryfuncTM(L, func, status) };
            }
        }
    }
}

unsafe fn ccall(L: *mut lua_State, func: StkId, nResults: c_int, inc: u32) {
    unsafe { (*L).nCcalls = (*L).nCcalls.wrapping_add(inc) };
    if unsafe { getCcalls(L) } >= LUAI_MAXCCALLS {
        let mut func = func;
        unsafe { checkstackp(L, 0, &mut func) };
        unsafe { luaE_checkcstack(L) };
    }
    let ci = unsafe { luaD_precall(L, func, nResults) };
    if !ci.is_null() {
        unsafe {
            (*ci).callstatus |= CIST_FRESH;
            luaV_execute(L, ci);
        }
    }
    unsafe { (*L).nCcalls = (*L).nCcalls.wrapping_sub(inc) };
}

pub unsafe fn luaD_call(L: *mut lua_State, func: StkId, nResults: c_int) {
    unsafe { ccall(L, func, nResults, 1) };
}

pub unsafe fn luaD_callnoyield(L: *mut lua_State, func: StkId, nResults: c_int) {
    unsafe { ccall(L, func, nResults, NYCI) };
}

unsafe fn finishpcallk(L: *mut lua_State, ci: *mut CallInfo) -> TStatus {
    let mut status = unsafe { getcistrecst(ci) };
    if status == LUA_OK {
        status = LUA_YIELD;
    } else {
        let mut func = unsafe { restorestack(L, (*ci).u2.funcidx as isize) };
        unsafe {
            (*L).allowhook = getoah(ci);
            func = luaF_close(L, func, status, 1);
            luaD_seterrorobj(L, status, func);
            luaD_shrinkstack(L);
            setcistrecst(ci, LUA_OK);
        }
    }
    unsafe {
        (*ci).callstatus &= !CIST_YPCALL;
        (*L).errfunc = (*ci).u.c.old_errfunc;
    }
    status
}

unsafe fn finishCcall(L: *mut lua_State, ci: *mut CallInfo) {
    let n;
    if unsafe { (*ci).callstatus & CIST_CLSRET } != 0 {
        n = unsafe { (*ci).u2.nres };
    } else {
        let mut status = LUA_YIELD;
        let kf = unsafe { (*ci).u.c.k };
        unsafe { api_check(kf.is_some() && yieldable(L), "invalid continuation") };
        if unsafe { (*ci).callstatus & CIST_YPCALL } != 0 {
            status = unsafe { finishpcallk(L, ci) };
        }
        unsafe { adjustresults(L, LUA_MULTRET) };
        n = unsafe { kf.expect("continuation")(L, APIstatus(status), (*ci).u.c.ctx) };
        unsafe { api_checknelems(L, n) };
    }
    unsafe { luaD_poscall(L, ci, n) };
}

unsafe fn unroll(L: *mut lua_State, _ud: *mut c_void) {
    while unsafe { (*L).ci } != ptr::addr_of_mut!(unsafe { &mut *L }.base_ci) {
        let ci = unsafe { (*L).ci };
        if unsafe { !isLua(ci) } {
            unsafe { finishCcall(L, ci) };
        } else {
            unsafe {
                luaV_finishOp(L);
                luaV_execute(L, ci);
            }
        }
    }
}

unsafe fn findpcall(L: *mut lua_State) -> *mut CallInfo {
    let mut ci = unsafe { (*L).ci };
    while !ci.is_null() {
        if unsafe { (*ci).callstatus & CIST_YPCALL } != 0 {
            return ci;
        }
        ci = unsafe { (*ci).previous };
    }
    ptr::null_mut()
}

unsafe fn resume_error(L: *mut lua_State, msg: *const c_char, narg: c_int) -> c_int {
    unsafe {
        api_checkpop(L, narg);
        (*L).top.p = (*L).top.p.sub(narg as usize);
        setsvalue2s(L, (*L).top.p, luaS_new(L, msg));
        api_incr_top(L);
    }
    LUA_ERRRUN as c_int
}

unsafe fn resume(L: *mut lua_State, ud: *mut c_void) {
    let mut n = unsafe { *(ud.cast::<c_int>()) };
    let firstArg = unsafe { (*L).top.p.sub(n as usize) };
    let ci = unsafe { (*L).ci };
    if unsafe { (*L).status } == LuaStatus::Ok {
        unsafe { ccall(L, firstArg.sub(1), LUA_MULTRET, 0) };
    } else {
        unsafe {
            api_check((*L).status == LuaStatus::Yield, "yielded status expected");
            (*L).status = LuaStatus::Ok;
        }
        if unsafe { isLua(ci) } {
            unsafe {
                api_check(
                    (*ci).callstatus & CIST_HOOKYIELD != 0,
                    "hook yield expected",
                );
                (*ci).u.l.savedpc = (*ci).u.l.savedpc.sub(1);
                (*L).top.p = firstArg;
                luaV_execute(L, ci);
            }
        } else {
            if let Some(k) = unsafe { (*ci).u.c.k } {
                n = unsafe { k(L, LUA_YIELD as c_int, (*ci).u.c.ctx) };
                unsafe { api_checknelems(L, n) };
            }
            unsafe { luaD_poscall(L, ci, n) };
        }
        unsafe { unroll(L, ptr::null_mut()) };
    }
}

unsafe fn precover(L: *mut lua_State, mut status: TStatus) -> TStatus {
    while unsafe { errorstatus(status) } {
        let ci = unsafe { findpcall(L) };
        if ci.is_null() {
            break;
        }
        unsafe {
            (*L).ci = ci;
            setcistrecst(ci, status);
            status = luaD_rawrunprotected(L, Some(unroll), ptr::null_mut());
        }
    }
    status
}

pub unsafe fn lua_resume(
    L: *mut lua_State,
    from: *mut lua_State,
    nargs: c_int,
    nresults: *mut c_int,
) -> c_int {
    let status;
    if unsafe { (*L).status } == LuaStatus::Ok {
        if unsafe { (*L).ci } != ptr::addr_of_mut!(unsafe { &mut *L }.base_ci) {
            return unsafe {
                resume_error(L, c"cannot resume non-suspended coroutine".as_ptr(), nargs)
            };
        } else if unsafe { (*L).top.p.offset_from((*(*L).ci).func.p.add(1)) as c_int == nargs } {
            return unsafe { resume_error(L, c"cannot resume dead coroutine".as_ptr(), nargs) };
        }
    } else if unsafe { (*L).status } != LuaStatus::Yield {
        return unsafe { resume_error(L, c"cannot resume dead coroutine".as_ptr(), nargs) };
    }
    unsafe { (*L).nCcalls = if from.is_null() { 0 } else { getCcalls(from) } };
    if unsafe { getCcalls(L) } >= LUAI_MAXCCALLS {
        return unsafe { resume_error(L, c"C stack overflow".as_ptr(), nargs) };
    }
    unsafe {
        (*L).nCcalls = (*L).nCcalls.wrapping_add(1);
        api_checkpop(
            L,
            if (*L).status == LuaStatus::Ok {
                nargs + 1
            } else {
                nargs
            },
        );
    }
    status =
        unsafe { luaD_rawrunprotected(L, Some(resume), ptr::addr_of!(nargs).cast_mut().cast()) };
    let status = unsafe { precover(L, status) };
    if unsafe { errorstatus(status) } {
        unsafe {
            (*L).status = LuaStatus::from_u8(status).expect("lua_State.status must be valid");
            luaD_seterrorobj(L, status, (*L).top.p);
            (*(*L).ci).top.p = (*L).top.p;
        }
    }
    unsafe {
        *nresults = if status == LUA_YIELD {
            (*(*L).ci).u2.nyield
        } else {
            (*L).top.p.offset_from((*(*L).ci).func.p.add(1)) as c_int
        };
    }
    unsafe { APIstatus(status) }
}

pub unsafe fn lua_isyieldable(L: *mut lua_State) -> c_int {
    unsafe { yieldable(L) as c_int }
}

pub unsafe fn lua_yieldk(
    L: *mut lua_State,
    nresults: c_int,
    ctx: lua_KContext,
    k: lua_KFunction,
) -> c_int {
    let ci = unsafe { (*L).ci };
    unsafe { api_checkpop(L, nresults) };
    if unsafe { !yieldable(L) } {
        if L != unsafe { mainthread(G(L)) } {
            unsafe { luaG_runerror(L, "attempt to yield across a C-call boundary") };
        } else {
            unsafe { luaG_runerror(L, "attempt to yield from outside a coroutine") };
        }
    }
    unsafe {
        (*L).status = LuaStatus::Yield;
        (*ci).u2.nyield = nresults;
    }
    if unsafe { isLua(ci) } {
        unsafe {
            api_check(!isLuacode(ci), "hooks cannot yield from Lua code");
            api_check(nresults == 0, "hooks cannot yield values");
            api_check(k.is_none(), "hooks cannot continue after yielding");
        }
    } else {
        unsafe {
            if let Some(kf) = k {
                (*ci).u.c.k = Some(kf);
                (*ci).u.c.ctx = ctx;
            } else {
                (*ci).u.c.k = None;
            }
            luaD_throw(L, LUA_YIELD);
        }
    }
    unsafe {
        api_check(
            (*ci).callstatus & CIST_HOOKED != 0,
            "hooked status expected",
        )
    };
    0
}

unsafe fn closepaux(L: *mut lua_State, ud: *mut c_void) {
    let pcl = unsafe { &mut *ud.cast::<CloseP>() };
    unsafe { luaF_close(L, pcl.level, pcl.status, 0) };
}

pub unsafe fn luaD_closeprotected(L: *mut lua_State, level: isize, mut status: TStatus) -> TStatus {
    let old_ci = unsafe { (*L).ci };
    let old_allowhooks = unsafe { (*L).allowhook };
    loop {
        let mut pcl = CloseP {
            level: unsafe { restorestack(L, level) },
            status,
        };
        status = unsafe { luaD_rawrunprotected(L, Some(closepaux), ptr::addr_of_mut!(pcl).cast()) };
        if status == LUA_OK {
            return pcl.status;
        }
        unsafe {
            (*L).ci = old_ci;
            (*L).allowhook = old_allowhooks;
        }
    }
}

pub(crate) unsafe fn luaD_pcall(
    L: *mut lua_State,
    func: Pfunc,
    u: *mut c_void,
    old_top: isize,
    ef: isize,
) -> TStatus {
    let old_ci = unsafe { (*L).ci };
    let old_allowhooks = unsafe { (*L).allowhook };
    let old_errfunc = unsafe { (*L).errfunc };
    unsafe { (*L).errfunc = ef };
    let mut status = unsafe { luaD_rawrunprotected(L, func, u) };
    if status != LUA_OK {
        unsafe {
            (*L).ci = old_ci;
            (*L).allowhook = old_allowhooks;
            status = luaD_closeprotected(L, old_top, status);
            luaD_seterrorobj(L, status, restorestack(L, old_top));
            luaD_shrinkstack(L);
        }
    }
    unsafe { (*L).errfunc = old_errfunc };
    status
}

unsafe fn checkmode(L: *mut lua_State, mode: *const c_char, x: *const c_char) {
    if unsafe { strchr(mode, *x as c_int) }.is_null() {
        let x_s = unsafe { std::ffi::CStr::from_ptr(x) }.to_string_lossy();
        let mode_s = unsafe { std::ffi::CStr::from_ptr(mode) }.to_string_lossy();
        unsafe {
            luaO_pushstr(
                L,
                &format!("attempt to load a {x_s} chunk (mode is '{mode_s}')"),
            )
        };
        unsafe { luaD_throw(L, LUA_ERRSYNTAX) };
    }
}

#[inline]
unsafe fn zgetc(z: *mut ZIO) -> c_int {
    if unsafe { (*z).n } > 0 {
        unsafe {
            (*z).n -= 1;
            let c = *(*z).p as u8 as c_int;
            (*z).p = (*z).p.add(1);
            c
        }
    } else {
        unsafe { luaZ_fill(z) }
    }
}

unsafe fn f_parser(L: *mut lua_State, ud: *mut c_void) {
    let p = unsafe { &mut *ud.cast::<SParser>() };
    let mode = if p.mode.is_null() {
        c"bt".as_ptr()
    } else {
        p.mode
    };
    let c = unsafe { zgetc(p.z) };
    let cl = if c as c_char == LUA_SIGNATURE_0 {
        let fixed = if unsafe { !strchr(mode, b'B' as c_int).is_null() } {
            1
        } else {
            unsafe { checkmode(L, mode, c"binary".as_ptr()) };
            0
        };
        unsafe { luaU_undump(L, p.z, p.name, fixed) }
    } else {
        unsafe { checkmode(L, mode, c"text".as_ptr()) };
        unsafe {
            luaY_parser(
                L,
                p.z,
                ptr::addr_of_mut!(p.buff),
                ptr::addr_of_mut!(p.dyd),
                p.name,
                c,
            )
        }
    };
    unsafe {
        api_check(
            (*cl).nupvalues as c_int == (*(*cl).p).sizeupvalues,
            "upvalue count mismatch",
        );
        luaF_initupvals(L, cl);
    }
}

pub(crate) unsafe fn luaD_protectedparser(
    L: *mut lua_State,
    z: *mut ZIO,
    name: *const c_char,
    mode: *const c_char,
) -> TStatus {
    let mut p = SParser {
        z,
        buff: Mbuffer {
            buffer: ptr::null_mut(),
            n: 0,
            buffsize: 0,
        },
        dyd: Dyndata {
            actvar: VardescList {
                arr: ptr::null_mut(),
                n: 0,
                size: 0,
            },
            gt: Labellist {
                arr: ptr::null_mut(),
                n: 0,
                size: 0,
            },
            label: Labellist {
                arr: ptr::null_mut(),
                n: 0,
                size: 0,
            },
        },
        mode,
        name,
    };
    unsafe { incnny(L) };
    unsafe { luaZ_initbuffer(L, ptr::addr_of_mut!(p.buff)) };
    let status = unsafe {
        luaD_pcall(
            L,
            Some(f_parser),
            ptr::addr_of_mut!(p).cast(),
            savestack(L, (*L).top.p),
            (*L).errfunc,
        )
    };
    unsafe { luaZ_freebuffer(L, ptr::addr_of_mut!(p.buff)) };
    unsafe {
        luaM_realloc_(
            L,
            p.dyd.actvar.arr.cast(),
            p.dyd.actvar.size as usize * size_of::<Vardesc>(),
            0,
        );
        luaM_realloc_(
            L,
            p.dyd.gt.arr.cast(),
            p.dyd.gt.size as usize * size_of::<Labeldesc>(),
            0,
        );
        luaM_realloc_(
            L,
            p.dyd.label.arr.cast(),
            p.dyd.label.size as usize * size_of::<Labeldesc>(),
            0,
        );
        decnny(L);
    }
    status
}
