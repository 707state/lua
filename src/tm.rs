#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::luavm::GlobalState;
use crate::runtime::*;
use crate::string::raw_luaS_new;
use crate::table::{
    raw_luaH_Hgetshortstr, raw_luaH_getint, raw_luaH_getshortstr, raw_luaH_new, raw_luaH_resize,
    raw_luaH_set, raw_luaH_setint,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

#[derive(Copy, Clone)]
#[repr(transparent)]
struct SyncCharPtr(*const c_char);

unsafe impl Sync for SyncCharPtr {}

#[inline] unsafe fn luaC_fix(s: *mut lua_State, o: *mut GCObject) { unsafe { crate::gc::luaC_fix(s, o) } }
#[inline] unsafe fn luaC_step(s: *mut lua_State) { unsafe { crate::gc::luaC_step(s) } }
#[inline] unsafe fn luaD_call(s: *mut lua_State, f: StkId, n: c_int) { unsafe { crate::do_rs::luaD_call(s, f, n) } }
#[inline] unsafe fn luaD_callnoyield(s: *mut lua_State, f: StkId, n: c_int) { unsafe { crate::do_rs::luaD_callnoyield(s, f, n) } }
#[inline] unsafe fn luaD_growstack(s: *mut lua_State, n: c_int, r: c_int) -> c_int { unsafe { crate::do_rs::luaD_growstack(s, n, r) } }
#[inline] unsafe fn luaG_tointerror(s: *mut lua_State, p1: *const TValue, p2: *const TValue) -> ! { unsafe { crate::debug::luaG_tointerror(s, p1, p2) } }
#[inline] unsafe fn luaG_opinterror(s: *mut lua_State, p1: *const TValue, p2: *const TValue, m: *const c_char) -> ! { unsafe { crate::debug::luaG_opinterror(s, p1, p2, m) } }
#[inline] unsafe fn luaG_concaterror(s: *mut lua_State, p1: *const TValue, p2: *const TValue) -> ! { unsafe { crate::debug::luaG_concaterror(s, p1, p2) } }
#[inline] unsafe fn luaG_ordererror(s: *mut lua_State, p1: *const TValue, p2: *const TValue) -> ! { unsafe { crate::debug::luaG_ordererror(s, p1, p2) } }
/// 单字符串版本，避免变参
#[inline] unsafe fn luaG_runerror(s: *mut lua_State, msg: *const c_char) -> ! { unsafe { crate::debug::luaG_runerror1(s, msg) } }
#[inline] unsafe fn luaV_tointegerns(o: *const TValue, p: *mut lua_Integer, m: c_int) -> c_int { unsafe { crate::vm_rs::luaV_tointegerns(o, p, m) } }

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
fn tagisfalse(tag: u8) -> bool {
    tag == LUA_VFALSE || (tag & 0x0f) == LUA_TNIL
}

pub(crate) unsafe fn luaT_init(state: *mut lua_State) {
    static EVENT_NAMES: [&[u8]; TM_N as usize] = [
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
pub unsafe  fn luaT_gettm(
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
pub unsafe  fn luaT_gettmbyobj(
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

pub(crate) unsafe fn luaT_objtypename(state: *mut lua_State, o: *const TValue) -> *const c_char {
    let mut mt = ptr::null_mut();
    if unsafe { ttistable(o) } {
        mt = unsafe { (*hvalue(o)).metatable };
    } else if unsafe { ttisfulluserdata(o) } {
        mt = unsafe { (*uvalue(o)).metatable };
    }
    if !mt.is_null() {
        let name = unsafe{
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
pub unsafe  fn luaT_callTM(
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
pub unsafe  fn luaT_callTMres(
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

pub unsafe fn luaT_trybinTM(
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
pub unsafe  fn luaT_tryconcatTM(state: *mut lua_State) {
    let p1 = unsafe { (*state).top.p.sub(2) };
    if unsafe { callbinTM(state, s2v(p1), s2v(p1.add(1)), p1, TM_CONCAT) } < 0 {
        unsafe { luaG_concaterror(state, s2v(p1), s2v(p1.add(1))) };
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaT_trybinassocTM(
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
pub unsafe  fn luaT_trybiniTM(
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
pub unsafe  fn luaT_callorderTM(
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
pub unsafe  fn luaT_callorderiTM(
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
        sethvalue(s2v((*state).top.p), t);
        (*state).top.p = (*state).top.p.add(1);
        raw_luaH_resize(state.cast(), t.cast(), n as u32, 1);
        setsvalue(
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
pub unsafe  fn luaT_adjustvarargs(
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
pub unsafe  fn luaT_getvararg(ci: *mut CallInfo, ra: StkId, rc: *mut TValue) {
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
pub unsafe  fn luaT_getvarargs(
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
    use crate::luaffi::LUAL_NUMSIZES;
    use crate::state::lua_close;
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
