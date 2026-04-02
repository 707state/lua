#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

use crate::runtime::*;

pub static lua_ident: [u8; 129] =
    *b"$LuaVersion: Lua 5.5.0  Copyright (C) 1994-2025 Lua.org, PUC-Rio $$LuaAuthors: R. Ierusalimschy, L. H. de Figueiredo, W. Celes $\0";

use crate::debug::luaG_errormsg;
use crate::runtime::luaE_setdebt;

pub unsafe fn lua_checkstack(L: *mut lua_State, n: c_int) -> c_int {
    let ci = unsafe { (*L).ci };
    unsafe { api_check(n >= 0, "negative 'n'") };
    let res = if unsafe { (*L).stack_last.p.offset_from((*L).top.p) > n as isize } {
        1
    } else {
        unsafe { luaD_growstack(L, n, 0) }
    };
    if res != 0 && unsafe { (*ci).top.p < (*L).top.p.add(n as usize) } {
        unsafe { (*ci).top.p = (*L).top.p.add(n as usize) };
    }
    res
}

pub unsafe fn lua_xmove(from: *mut lua_State, to: *mut lua_State, n: c_int) {
    if from == to {
        return;
    }
    unsafe { api_checkpop(from, n) };
    unsafe { api_check(G(from) == G(to), "moving among independent states") };
    unsafe {
        api_check(
            (*(*to).ci).top.p.offset_from((*to).top.p) >= n as isize,
            "stack overflow",
        )
    };
    unsafe { (*from).top.p = (*from).top.p.sub(n as usize) };
    for i in 0..n as usize {
        unsafe {
            setobjs2s(to, (*to).top.p, (*from).top.p.add(i));
            (*to).top.p = (*to).top.p.add(1);
        }
    }
}

pub unsafe fn lua_atpanic(L: *mut lua_State, panicf: lua_CFunction) -> lua_CFunction {
    let old = unsafe { (*G(L)).panic };
    unsafe { (*G(L)).panic = panicf };
    old
}

pub unsafe fn lua_version(_L: *mut lua_State) -> lua_Number {
    LUA_VERSION_NUM
}

pub unsafe fn lua_absindex(L: *mut lua_State, idx: c_int) -> c_int {
    if idx > 0 || unsafe { ispseudo(idx) } {
        idx
    } else {
        unsafe { ((*L).top.p.offset_from((*(*L).ci).func.p) as c_int) + idx }
    }
}

pub unsafe fn lua_gettop(L: *mut lua_State) -> c_int {
    unsafe { (*L).top.p.offset_from((*(*L).ci).func.p.add(1)) as c_int }
}

pub unsafe fn lua_settop(L: *mut lua_State, idx: c_int) {
    let ci = unsafe { (*L).ci };
    let func = unsafe { (*ci).func.p };
    let diff = if idx >= 0 {
        unsafe {
            api_check(
                idx as isize <= (*ci).top.p.offset_from(func.add(1)),
                "new top too large",
            )
        };
        let mut diff = unsafe { (func.add(1 + idx as usize)).offset_from((*L).top.p) };
        while diff > 0 {
            unsafe { setnilvalue(s2v((*L).top.p)) };
            unsafe { (*L).top.p = (*L).top.p.add(1) };
            diff -= 1;
        }
        unsafe { (func.add(1 + idx as usize)).offset_from((*L).top.p) }
    } else {
        unsafe {
            api_check(
                (-(idx + 1)) as isize <= (*L).top.p.offset_from(func.add(1)),
                "invalid new top",
            )
        };
        (idx + 1) as isize
    };
    let mut newtop = unsafe { (*L).top.p.offset(diff) };
    if diff < 0 && unsafe { (*L).tbclist.p >= newtop } {
        unsafe { newtop = luaF_close(L, newtop, CLOSEKTOP, 0) };
    }
    unsafe { (*L).top.p = newtop };
}

pub unsafe fn lua_closeslot(L: *mut lua_State, idx: c_int) {
    let mut level = unsafe { index2stack(L, idx) };
    unsafe {
        api_check(
            ((*(*L).ci).callstatus & CIST_TBC) != 0 && (*L).tbclist.p == level,
            "no variable to close at given level",
        )
    };
    level = unsafe { luaF_close(L, level, CLOSEKTOP, 0) };
    unsafe { setnilvalue(s2v(level)) };
}

pub unsafe fn lua_rotate(L: *mut lua_State, idx: c_int, n: c_int) {
    let t = unsafe { (*L).top.p.sub(1) };
    let p = unsafe { index2stack(L, idx) };
    unsafe { api_check((*L).tbclist.p < p, "moving a to-be-closed slot") };
    unsafe {
        api_check(
            (if n >= 0 { n } else { -n }) as isize <= t.offset_from(p) + 1,
            "invalid 'n'",
        )
    };
    let m = if n >= 0 {
        unsafe { t.sub(n as usize) }
    } else {
        unsafe { p.add((-n - 1) as usize) }
    };
    unsafe { reverse(L, p, m) };
    unsafe { reverse(L, m.add(1), t) };
    unsafe { reverse(L, p, t) };
}

pub unsafe fn lua_copy(L: *mut lua_State, fromidx: c_int, toidx: c_int) {
    let fr = unsafe { index2value(L, fromidx) };
    let to = unsafe { index2value(L, toidx) };
    unsafe { api_check(isvalid(L, to), "invalid index") };
    unsafe { setobj(to, fr) };
    if unsafe { isupvalue(toidx) } {
        unsafe { luaC_barrier(L, obj2gco(clCvalue(s2v((*(*L).ci).func.p))), fr) };
    }
}

pub unsafe fn lua_pushvalue(L: *mut lua_State, idx: c_int) {
    unsafe { setobj2s(L, (*L).top.p, index2value(L, idx)) };
    unsafe { api_incr_top(L) };
}

#[unsafe(no_mangle)]
pub unsafe fn lua_type(L: *mut lua_State, idx: c_int) -> c_int {
    let o = unsafe { index2value(L, idx) };
    if unsafe { isvalid(L, o) } {
        unsafe { ttype(o) as c_int }
    } else {
        LUA_TNONE
    }
}

pub unsafe fn lua_typename(_L: *mut lua_State, t: c_int) -> *const c_char {
    unsafe { api_check(LUA_TNONE <= t && t < LUA_NUMTYPES, "invalid type") };
    static NAMES: [&[u8]; 10] = [
        b"no value\0",
        b"nil\0",
        b"boolean\0",
        b"userdata\0",
        b"number\0",
        b"string\0",
        b"table\0",
        b"function\0",
        b"userdata\0",
        b"thread\0",
    ];
    NAMES[(t + 1) as usize].as_ptr().cast()
}

pub unsafe fn lua_iscfunction(L: *mut lua_State, idx: c_int) -> c_int {
    let o = unsafe { index2value(L, idx) };
    (unsafe { ttislcf(o) || ttisCclosure(o) }) as c_int
}

pub unsafe fn lua_isinteger(L: *mut lua_State, idx: c_int) -> c_int {
    unsafe { ttisinteger(index2value(L, idx)) as c_int }
}

pub unsafe fn lua_isnumber(L: *mut lua_State, idx: c_int) -> c_int {
    let mut n = 0.0;
    unsafe { tonumber(index2value(L, idx), ptr::addr_of_mut!(n)) }
}

pub unsafe fn lua_isstring(L: *mut lua_State, idx: c_int) -> c_int {
    let o = unsafe { index2value(L, idx) };
    (unsafe { ttisstring(o) || cvt2str(o) }) as c_int
}

pub unsafe fn lua_isuserdata(L: *mut lua_State, idx: c_int) -> c_int {
    let o = unsafe { index2value(L, idx) };
    (unsafe { ttisfulluserdata(o) || ttislightuserdata(o) }) as c_int
}

pub unsafe fn lua_rawequal(L: *mut lua_State, index1: c_int, index2: c_int) -> c_int {
    let o1 = unsafe { index2value(L, index1) };
    let o2 = unsafe { index2value(L, index2) };
    if unsafe { isvalid(L, o1) && isvalid(L, o2) } {
        unsafe { luaV_equalobj(ptr::null_mut(), o1, o2) }
    } else {
        0
    }
}

pub unsafe fn lua_arith(L: *mut lua_State, op: c_int) {
    if op != LUA_OPUNM && op != LUA_OPBNOT {
        unsafe { api_checkpop(L, 2) };
    } else {
        unsafe { api_checkpop(L, 1) };
        unsafe { setobjs2s(L, (*L).top.p, (*L).top.p.sub(1)) };
        unsafe { api_incr_top(L) };
    }
    unsafe {
        luaO_arith(
            L,
            op,
            s2v((*L).top.p.sub(2)),
            s2v((*L).top.p.sub(1)),
            (*L).top.p.sub(2),
        )
    };
    unsafe { (*L).top.p = (*L).top.p.sub(1) };
}

pub unsafe fn lua_compare(L: *mut lua_State, index1: c_int, index2: c_int, op: c_int) -> c_int {
    let o1 = unsafe { index2value(L, index1) };
    let o2 = unsafe { index2value(L, index2) };
    if !(unsafe { isvalid(L, o1) && isvalid(L, o2) }) {
        return 0;
    }
    match op {
        LUA_OPEQ => unsafe { luaV_equalobj(L, o1, o2) },
        LUA_OPLT => unsafe { luaV_lessthan(L, o1, o2) },
        LUA_OPLE => unsafe { luaV_lessequal(L, o1, o2) },
        _ => {
            unsafe { api_check(false, "invalid option") };
            0
        }
    }
}

pub unsafe fn lua_numbertocstring(L: *mut lua_State, idx: c_int, buff: *mut c_char) -> c_uint {
    let o = unsafe { index2value(L, idx) };
    if unsafe { ttisnumber(o) } {
        let mut len = unsafe { luaO_tostringbuff(o, buff) };
        unsafe { *buff.add(len as usize) = 0 };
        len += 1;
        len
    } else {
        0
    }
}

pub unsafe fn lua_stringtonumber(L: *mut lua_State, s: *const c_char) -> usize {
    let sz = unsafe { luaO_str2num(s, s2v((*L).top.p)) };
    if sz != 0 {
        unsafe { api_incr_top(L) };
    }
    sz
}

pub unsafe fn lua_tonumberx(L: *mut lua_State, idx: c_int, pisnum: *mut c_int) -> lua_Number {
    let mut n = 0.0;
    let isnum = unsafe { tonumber(index2value(L, idx), ptr::addr_of_mut!(n)) };
    if !pisnum.is_null() {
        unsafe { *pisnum = isnum };
    }
    n
}

#[unsafe(no_mangle)]
pub unsafe fn lua_tointegerx(L: *mut lua_State, idx: c_int, pisnum: *mut c_int) -> lua_Integer {
    let mut i = 0;
    let isnum = unsafe { tointeger(index2value(L, idx), ptr::addr_of_mut!(i)) };
    if !pisnum.is_null() {
        unsafe { *pisnum = isnum };
    }
    i
}

#[unsafe(no_mangle)]
pub unsafe fn lua_toboolean(L: *mut lua_State, idx: c_int) -> c_int {
    (!unsafe { l_isfalse(index2value(L, idx)) }) as c_int
}

pub unsafe fn lua_tolstring(L: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char {
    let mut o = unsafe { index2value(L, idx) };
    if !unsafe { ttisstring(o) } {
        if !unsafe { cvt2str(o) } {
            if !len.is_null() {
                unsafe { *len = 0 };
            }
            return ptr::null();
        }
        unsafe { luaO_tostring(L, o) };
        unsafe { luaC_checkGC(L) };
        o = unsafe { index2value(L, idx) };
    }
    if !len.is_null() {
        unsafe { getlstr(tsvalue(o), &mut *len) }
    } else {
        unsafe { getstr(tsvalue(o)) }
    }
}

pub unsafe fn lua_rawlen(L: *mut lua_State, idx: c_int) -> lua_Unsigned {
    let o = unsafe { index2value(L, idx) };
    match unsafe { ttypetag(o) } {
        LUA_VSHRSTR => unsafe { (*tsvalue(o)).shrlen as lua_Unsigned },
        LUA_VLNGSTR => unsafe { (*tsvalue(o)).u.lnglen as lua_Unsigned },
        LUA_VUSERDATA => unsafe { (*uvalue(o)).len as lua_Unsigned },
        LUA_VTABLE => unsafe { luaH_getn(L, hvalue(o)) },
        _ => 0,
    }
}

pub unsafe fn lua_tocfunction(L: *mut lua_State, idx: c_int) -> lua_CFunction {
    let o = unsafe { index2value(L, idx) };
    if unsafe { ttislcf(o) } {
        unsafe { fvalue(o) }
    } else if unsafe { ttisCclosure(o) } {
        unsafe { (*clCvalue(o)).f }
    } else {
        None
    }
}

pub unsafe fn lua_touserdata(L: *mut lua_State, idx: c_int) -> *mut c_void {
    unsafe { touserdata(index2value(L, idx)) }
}

pub unsafe fn lua_tothread(L: *mut lua_State, idx: c_int) -> *mut lua_State {
    let o = unsafe { index2value(L, idx) };
    if unsafe { ttisthread(o) } {
        unsafe { thvalue(o) }
    } else {
        ptr::null_mut()
    }
}

pub unsafe fn lua_topointer(L: *mut lua_State, idx: c_int) -> *const c_void {
    let o = unsafe { index2value(L, idx) };
    match unsafe { ttypetag(o) } {
        LUA_VLCF => unsafe { fvalue(o).map(|f| f as *const c_void).unwrap_or(ptr::null()) },
        LUA_VUSERDATA | LUA_VLIGHTUSERDATA => unsafe { touserdata(o).cast_const() },
        _ => {
            if unsafe { iscollectable(o) } {
                unsafe { gcvalue(o).cast() }
            } else {
                ptr::null()
            }
        }
    }
}

pub unsafe fn lua_pushnil(L: *mut lua_State) {
    unsafe { setnilvalue(s2v((*L).top.p)) };
    unsafe { api_incr_top(L) };
}

pub unsafe fn lua_pushnumber(L: *mut lua_State, n: lua_Number) {
    unsafe { setfltvalue(s2v((*L).top.p), n) };
    unsafe { api_incr_top(L) };
}

pub unsafe fn lua_pushinteger(L: *mut lua_State, n: lua_Integer) {
    unsafe { setivalue(s2v((*L).top.p), n) };
    unsafe { api_incr_top(L) };
}

pub unsafe fn lua_pushlstring(L: *mut lua_State, s: *const c_char, len: usize) -> *const c_char {
    let ts = if len == 0 {
        unsafe { luaS_new(L, c"".as_ptr()) }
    } else {
        unsafe { luaS_newlstr(L, s, len) }
    };
    unsafe { setsvalue2s(L, (*L).top.p, ts) };
    unsafe { api_incr_top(L) };
    unsafe { luaC_checkGC(L) };
    unsafe { getstr(ts) }
}

pub unsafe fn lua_pushexternalstring(
    L: *mut lua_State,
    s: *const c_char,
    len: usize,
    falloc: lua_Alloc,
    ud: *mut c_void,
) -> *const c_char {
    unsafe { api_check(len <= MAX_SIZE, "string too large") };
    unsafe { api_check(*s.add(len) == 0, "string not ending with zero") };
    let ts = unsafe { luaS_newextlstr(L, s, len, falloc, ud) };
    unsafe { setsvalue2s(L, (*L).top.p, ts) };
    unsafe { api_incr_top(L) };
    unsafe { luaC_checkGC(L) };
    unsafe { getstr(ts) }
}

pub unsafe fn lua_pushstring(L: *mut lua_State, mut s: *const c_char) -> *const c_char {
    if s.is_null() {
        unsafe { setnilvalue(s2v((*L).top.p)) };
    } else {
        let ts = unsafe { luaS_new(L, s) };
        unsafe { setsvalue2s(L, (*L).top.p, ts) };
        s = unsafe { getstr(ts) };
    }
    unsafe { api_incr_top(L) };
    unsafe { luaC_checkGC(L) };
    s
}

pub unsafe fn lua_pushvfstring(
    L: *mut lua_State,
    fmt: *const c_char,
    argp: VaList<'_>,
) -> *const c_char {
    let ret = unsafe { luaO_pushvfstring(L, fmt, argp) };
    unsafe { luaC_checkGC(L) };
    ret
}

pub unsafe extern "C" fn lua_pushfstring(
    L: *mut lua_State,
    fmt: *const c_char,
    argp: ...
) -> *const c_char {
    let ret = unsafe { luaO_pushvfstring(L, fmt, argp) };
    if ret.is_null() {
        unsafe { luaD_throw(L, LUA_ERRMEM) };
    }
    unsafe { luaC_checkGC(L) };
    ret
}

pub unsafe fn lua_pushcclosure(L: *mut lua_State, fn_: lua_CFunction, n: c_int) {
    if n == 0 {
        unsafe { setfvalue(s2v((*L).top.p), fn_) };
        unsafe { api_incr_top(L) };
    } else {
        unsafe { api_checkpop(L, n) };
        unsafe { api_check(n <= MAXUPVAL, "upvalue index too large") };
        let cl = unsafe { luaF_newCclosure(L, n) };
        unsafe { (*cl).f = fn_ };
        for i in 0..n as usize {
            unsafe {
                setobj2n(
                    L,
                    ptr::addr_of_mut!((*cl).upvalue).cast::<TValue>().add(i),
                    s2v((*L).top.p.sub(n as usize).add(i)),
                )
            };
        }
        unsafe { (*L).top.p = (*L).top.p.sub(n as usize) };
        unsafe { setclCvalue(L, s2v((*L).top.p), cl) };
        unsafe { api_incr_top(L) };
        unsafe { luaC_checkGC(L) };
    }
}

pub unsafe fn lua_pushboolean(L: *mut lua_State, b: c_int) {
    if b != 0 {
        unsafe { setbtvalue(s2v((*L).top.p)) };
    } else {
        unsafe { setbfvalue(s2v((*L).top.p)) };
    }
    unsafe { api_incr_top(L) };
}

pub unsafe fn lua_pushlightuserdata(L: *mut lua_State, p: *mut c_void) {
    unsafe { setpvalue(s2v((*L).top.p), p) };
    unsafe { api_incr_top(L) };
}

pub unsafe fn lua_pushthread(L: *mut lua_State) -> c_int {
    unsafe { setthvalue(L, s2v((*L).top.p), L) };
    unsafe { api_incr_top(L) };
    (unsafe { mainthread(G(L)) == L }) as c_int
}

pub unsafe fn lua_getglobal(L: *mut lua_State, name: *const c_char) -> c_int {
    let mut gt = TValue {
        value_: Value { ub: 0 },
        tt_: 0,
    };
    unsafe { getGlobalTable(L, ptr::addr_of_mut!(gt)) };
    unsafe { auxgetstr(L, ptr::addr_of!(gt), name) }
}

pub unsafe fn lua_gettable(L: *mut lua_State, idx: c_int) -> c_int {
    unsafe { api_checkpop(L, 1) };
    let t = unsafe { index2value(L, idx) };
    let mut tag = if unsafe { ttistable(t) } {
        unsafe { luaH_get(hvalue(t), s2v((*L).top.p.sub(1)), s2v((*L).top.p.sub(1))) }
    } else {
        LUA_TNIL | (3 << 4)
    };
    if unsafe { tagisempty(tag) } {
        tag = unsafe { luaV_finishget(L, t, s2v((*L).top.p.sub(1)), (*L).top.p.sub(1), tag) };
    }
    unsafe { novariant(tag) as c_int }
}

#[unsafe(no_mangle)]
pub unsafe fn lua_getfield(L: *mut lua_State, idx: c_int, k: *const c_char) -> c_int {
    unsafe { auxgetstr(L, index2value(L, idx), k) }
}

pub unsafe fn lua_geti(L: *mut lua_State, idx: c_int, n: lua_Integer) -> c_int {
    let t = unsafe { index2value(L, idx) };
    let mut tag = if unsafe { ttistable(t) } {
        unsafe { luaH_getint(hvalue(t), n, s2v((*L).top.p)) }
    } else {
        LUA_TNIL | (3 << 4)
    };
    if unsafe { tagisempty(tag) } {
        let mut key = TValue {
            value_: Value { ub: 0 },
            tt_: 0,
        };
        unsafe { setivalue(ptr::addr_of_mut!(key), n) };
        tag = unsafe { luaV_finishget(L, t, ptr::addr_of_mut!(key), (*L).top.p, tag) };
    }
    unsafe { api_incr_top(L) };
    unsafe { novariant(tag) as c_int }
}

pub unsafe fn lua_rawget(L: *mut lua_State, idx: c_int) -> c_int {
    unsafe { api_checkpop(L, 1) };
    let t = unsafe { gettable(L, idx) };
    let tag = unsafe { luaH_get(t, s2v((*L).top.p.sub(1)), s2v((*L).top.p.sub(1))) };
    unsafe { (*L).top.p = (*L).top.p.sub(1) };
    unsafe { finishrawget(L, tag) }
}

pub unsafe fn lua_rawgeti(L: *mut lua_State, idx: c_int, n: lua_Integer) -> c_int {
    let t = unsafe { gettable(L, idx) };
    let tag = unsafe { luaH_getint(t, n, s2v((*L).top.p)) };
    unsafe { finishrawget(L, tag) }
}

pub unsafe fn lua_rawgetp(L: *mut lua_State, idx: c_int, p: *const c_void) -> c_int {
    let t = unsafe { gettable(L, idx) };
    let mut k = TValue {
        value_: Value { ub: 0 },
        tt_: 0,
    };
    unsafe { setpvalue(ptr::addr_of_mut!(k), p as *mut c_void) };
    unsafe { finishrawget(L, luaH_get(t, ptr::addr_of!(k), s2v((*L).top.p))) }
}

pub unsafe fn lua_createtable(L: *mut lua_State, narray: c_int, nrec: c_int) {
    let t = unsafe { luaH_new(L) };
    unsafe { sethvalue2s(L, (*L).top.p, t) };
    unsafe { api_incr_top(L) };
    if narray > 0 || nrec > 0 {
        unsafe { luaH_resize(L, t, narray as c_uint, nrec as c_uint) };
    }
    unsafe { luaC_checkGC(L) };
}

pub unsafe fn lua_getmetatable(L: *mut lua_State, objindex: c_int) -> c_int {
    let obj = unsafe { index2value(L, objindex) };
    let mt = match unsafe { ttype(obj) } {
        LUA_TTABLE => unsafe { (*hvalue(obj)).metatable },
        LUA_TUSERDATA => unsafe { (*uvalue(obj)).metatable },
        other => unsafe { (*G(L)).mt[other as usize] },
    };
    if !mt.is_null() {
        unsafe { sethvalue2s(L, (*L).top.p, mt) };
        unsafe { api_incr_top(L) };
        1
    } else {
        0
    }
}

pub unsafe fn lua_getiuservalue(L: *mut lua_State, idx: c_int, n: c_int) -> c_int {
    let o = unsafe { index2value(L, idx) };
    unsafe { api_check(ttisfulluserdata(o), "full userdata expected") };
    let t = if n <= 0 || n > unsafe { (*uvalue(o)).nuvalue as c_int } {
        unsafe { setnilvalue(s2v((*L).top.p)) };
        LUA_TNONE
    } else {
        unsafe {
            setobj2s(
                L,
                (*L).top.p,
                ptr::addr_of!((*(*uvalue(o)).uv.as_ptr().add((n - 1) as usize)).uv),
            )
        };
        unsafe { ttype(s2v((*L).top.p)) as c_int }
    };
    unsafe { api_incr_top(L) };
    t
}

pub unsafe fn lua_setglobal(L: *mut lua_State, name: *const c_char) {
    let mut gt = TValue {
        value_: Value { ub: 0 },
        tt_: 0,
    };
    unsafe { getGlobalTable(L, ptr::addr_of_mut!(gt)) };
    unsafe { auxsetstr(L, ptr::addr_of!(gt), name) };
}

pub unsafe fn lua_settable(L: *mut lua_State, idx: c_int) {
    unsafe { api_checkpop(L, 2) };
    let t = unsafe { index2value(L, idx) };
    let hres = if unsafe { ttistable(t) } {
        unsafe { luaH_pset(hvalue(t), s2v((*L).top.p.sub(2)), s2v((*L).top.p.sub(1))) }
    } else {
        HNOTATABLE
    };
    if hres == HOK {
        unsafe { luaC_barrierback(L, gcvalue(t), s2v((*L).top.p.sub(1))) };
    } else {
        unsafe { luaV_finishset(L, t, s2v((*L).top.p.sub(2)), s2v((*L).top.p.sub(1)), hres) };
    }
    unsafe { (*L).top.p = (*L).top.p.sub(2) };
}

pub unsafe fn lua_setfield(L: *mut lua_State, idx: c_int, k: *const c_char) {
    unsafe { auxsetstr(L, index2value(L, idx), k) };
}

pub unsafe fn lua_seti(L: *mut lua_State, idx: c_int, n: lua_Integer) {
    unsafe { api_checkpop(L, 1) };
    let t = unsafe { index2value(L, idx) };
    let hres = if unsafe { ttistable(t) } {
        let h = unsafe { hvalue(t) };
        let u = (n as lua_Unsigned).wrapping_sub(1);
        if u < unsafe { (*h).asize as lua_Unsigned } {
            let tag = unsafe { getArrTag(h, u as u32) };
            if unsafe { checknoTM((*h).metatable, TM_NEWINDEX) } || !unsafe { tagisempty(*tag) } {
                unsafe { fval2arr(h, u as u32, tag, s2v((*L).top.p.sub(1))) };
                HOK
            } else {
                !(u as c_int)
            }
        } else {
            unsafe { luaH_psetint(h, n, s2v((*L).top.p.sub(1))) }
        }
    } else {
        HNOTATABLE
    };
    if hres == HOK {
        unsafe { luaC_barrierback(L, gcvalue(t), s2v((*L).top.p.sub(1))) };
    } else {
        let mut temp = TValue {
            value_: Value { ub: 0 },
            tt_: 0,
        };
        unsafe { setivalue(ptr::addr_of_mut!(temp), n) };
        unsafe { luaV_finishset(L, t, ptr::addr_of_mut!(temp), s2v((*L).top.p.sub(1)), hres) };
    }
    unsafe { (*L).top.p = (*L).top.p.sub(1) };
}

pub unsafe fn lua_rawset(L: *mut lua_State, idx: c_int) {
    unsafe { aux_rawset(L, idx, s2v((*L).top.p.sub(2)), 2) };
}

pub unsafe fn lua_rawsetp(L: *mut lua_State, idx: c_int, p: *const c_void) {
    let mut k = TValue {
        value_: Value { ub: 0 },
        tt_: 0,
    };
    unsafe { setpvalue(ptr::addr_of_mut!(k), p as *mut c_void) };
    unsafe { aux_rawset(L, idx, ptr::addr_of_mut!(k), 1) };
}

pub unsafe fn lua_rawseti(L: *mut lua_State, idx: c_int, n: lua_Integer) {
    unsafe { api_checkpop(L, 1) };
    let t = unsafe { gettable(L, idx) };
    unsafe { luaH_setint(L, t, n, s2v((*L).top.p.sub(1))) };
    unsafe { luaC_barrierback(L, obj2gco(t), s2v((*L).top.p.sub(1))) };
    unsafe { (*L).top.p = (*L).top.p.sub(1) };
}

pub unsafe fn lua_setmetatable(L: *mut lua_State, objindex: c_int) -> c_int {
    unsafe { api_checkpop(L, 1) };
    let obj = unsafe { index2value(L, objindex) };
    let mt = if unsafe { ttisnil(s2v((*L).top.p.sub(1))) } {
        ptr::null_mut()
    } else {
        unsafe { api_check(ttistable(s2v((*L).top.p.sub(1))), "table expected") };
        unsafe { hvalue(s2v((*L).top.p.sub(1))) }
    };
    match unsafe { ttype(obj) } {
        LUA_TTABLE => {
            unsafe { (*hvalue(obj)).metatable = mt };
            if !mt.is_null() {
                unsafe { luaC_objbarrier(L, gcvalue(obj), obj2gco(mt)) };
                unsafe { luaC_checkfinalizer(L, gcvalue(obj), mt) };
            }
        }
        LUA_TUSERDATA => {
            unsafe { (*uvalue(obj)).metatable = mt };
            if !mt.is_null() {
                unsafe { luaC_objbarrier(L, obj2gco(uvalue(obj)), obj2gco(mt)) };
                unsafe { luaC_checkfinalizer(L, gcvalue(obj), mt) };
            }
        }
        other => unsafe { (*G(L)).mt[other as usize] = mt },
    }
    unsafe { (*L).top.p = (*L).top.p.sub(1) };
    1
}

pub unsafe fn lua_setiuservalue(L: *mut lua_State, idx: c_int, n: c_int) -> c_int {
    unsafe { api_checkpop(L, 1) };
    let o = unsafe { index2value(L, idx) };
    unsafe { api_check(ttisfulluserdata(o), "full userdata expected") };
    let res = if !(((n as u32).wrapping_sub(1)) < unsafe { (*uvalue(o)).nuvalue as u32 }) {
        0
    } else {
        unsafe {
            setobj(
                ptr::addr_of_mut!((*(*uvalue(o)).uv.as_mut_ptr().add((n - 1) as usize)).uv),
                s2v((*L).top.p.sub(1)),
            )
        };
        unsafe { luaC_barrierback(L, gcvalue(o), s2v((*L).top.p.sub(1))) };
        1
    };
    unsafe { (*L).top.p = (*L).top.p.sub(1) };
    res
}

pub unsafe fn lua_callk(
    L: *mut lua_State,
    nargs: c_int,
    nresults: c_int,
    ctx: lua_KContext,
    k: lua_KFunction,
) {
    unsafe {
        api_check(
            k.is_none() || !isLua((*L).ci),
            "cannot use continuations inside hooks",
        )
    };
    unsafe { api_checkpop(L, nargs + 1) };
    unsafe {
        api_check(
            (*L).status == LUA_OK,
            "cannot do calls on non-normal thread",
        )
    };
    unsafe { checkresults(L, nargs, nresults) };
    let func = unsafe { (*L).top.p.sub((nargs + 1) as usize) };
    if k.is_some() && unsafe { yieldable(L) } {
        unsafe {
            (*(*L).ci).u.c.k = k;
            (*(*L).ci).u.c.ctx = ctx;
            luaD_call(L, func, nresults);
        }
    } else {
        unsafe { luaD_callnoyield(L, func, nresults) };
    }
    unsafe { adjustresults(L, nresults) };
}

pub unsafe fn lua_pcallk(
    L: *mut lua_State,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
    ctx: lua_KContext,
    k: lua_KFunction,
) -> c_int {
    let mut c = CallS {
        func: ptr::null_mut(),
        nresults: 0,
    };
    unsafe {
        api_check(
            k.is_none() || !isLua((*L).ci),
            "cannot use continuations inside hooks",
        )
    };
    unsafe { api_checkpop(L, nargs + 1) };
    unsafe {
        api_check(
            (*L).status == LUA_OK,
            "cannot do calls on non-normal thread",
        )
    };
    unsafe { checkresults(L, nargs, nresults) };
    let funcidx = if errfunc == 0 {
        0
    } else {
        let o = unsafe { index2stack(L, errfunc) };
        unsafe {
            api_check(
                ttype(s2v(o)) == LUA_TFUNCTION,
                "error handler must be a function",
            )
        };
        unsafe { savestack(L, o) }
    };
    c.func = unsafe { (*L).top.p.sub((nargs + 1) as usize) };
    let status = if k.is_none() || !unsafe { yieldable(L) } {
        c.nresults = nresults;
        unsafe {
            luaD_pcall(
                L,
                Some(f_call),
                ptr::addr_of_mut!(c).cast(),
                savestack(L, c.func),
                funcidx,
            )
        }
    } else {
        let ci = unsafe { (*L).ci };
        unsafe {
            (*ci).u.c.k = k;
            (*ci).u.c.ctx = ctx;
            (*ci).u2.funcidx = savestack(L, c.func) as c_int;
            (*ci).u.c.old_errfunc = (*L).errfunc;
            (*L).errfunc = funcidx;
            if (*L).allowhook != 0 {
                (*ci).callstatus |= CIST_OAH;
            } else {
                (*ci).callstatus &= !CIST_OAH;
            }
            (*ci).callstatus |= CIST_YPCALL;
            luaD_call(L, c.func, nresults);
            (*ci).callstatus &= !CIST_YPCALL;
            (*L).errfunc = (*ci).u.c.old_errfunc;
        }
        LUA_OK
    };
    unsafe { adjustresults(L, nresults) };
    unsafe { APIstatus(status) }
}

pub unsafe fn lua_load(
    L: *mut lua_State,
    reader: lua_Reader,
    data: *mut c_void,
    chunkname: *const c_char,
    mode: *const c_char,
) -> c_int {
    let mut z = ZIO {
        n: 0,
        p: ptr::null(),
        reader,
        data,
        L,
    };
    let chunkname = if chunkname.is_null() {
        c"?".as_ptr()
    } else {
        chunkname
    };
    unsafe { luaZ_init(L, ptr::addr_of_mut!(z), reader, data) };
    let status = unsafe { luaD_protectedparser(L, ptr::addr_of_mut!(z), chunkname, mode) };
    if status == LUA_OK {
        let f = unsafe { clLvalue(s2v((*L).top.p.sub(1))) };
        if unsafe { (*f).nupvalues >= 1 } {
            let mut gt = TValue {
                value_: Value { ub: 0 },
                tt_: 0,
            };
            unsafe { getGlobalTable(L, ptr::addr_of_mut!(gt)) };
            unsafe { setobj((*(*(*f).upvals.as_ptr())).v.p, ptr::addr_of!(gt)) };
            unsafe { luaC_barrier(L, obj2gco(*(*f).upvals.as_ptr()), ptr::addr_of!(gt)) };
        }
    }
    unsafe { APIstatus(status) }
}

pub unsafe fn lua_dump(
    L: *mut lua_State,
    writer: lua_Writer,
    data: *mut c_void,
    strip: c_int,
) -> c_int {
    let otop = unsafe { savestack(L, (*L).top.p) };
    let f = unsafe { s2v((*L).top.p.sub(1)) };
    unsafe { api_checkpop(L, 1) };
    unsafe { api_check(ttisLclosure(f), "Lua function expected") };
    let status = unsafe { luaU_dump(L, (*clLvalue(f)).p, writer, data, strip) };
    unsafe { (*L).top.p = restorestack(L, otop) };
    status
}

pub unsafe fn lua_status(L: *mut lua_State) -> c_int {
    unsafe { APIstatus((*L).status) }
}

pub unsafe extern "C" fn lua_gc(L: *mut lua_State, what: c_int, mut args: ...) -> c_int {
    let g = unsafe { G(L) };
    if unsafe { (*g).gcstp & (GCSTPGC | GCSTPCLS) } != 0 {
        return -1;
    }
    match what {
        LUA_GCSTOP => {
            unsafe { (*g).gcstp = GCSTPUSR };
            0
        }
        LUA_GCRESTART => {
            unsafe { luaE_setdebt(g, 0) };
            unsafe { (*g).gcstp = 0 };
            0
        }
        LUA_GCCOLLECT => {
            unsafe { luaC_fullgc(L, 0) };
            0
        }
        LUA_GCCOUNT => unsafe { (gettotalbytes(g) >> 10) as c_int },
        LUA_GCCOUNTB => unsafe { (gettotalbytes(g) & 0x3ff) as c_int },
        LUA_GCSTEP => {
            let oldstp = unsafe { (*g).gcstp };
            let mut n = unsafe { args.arg::<usize>() as l_mem };
            let mut res = 0;
            unsafe { (*g).gcstp = 0 };
            if n <= 0 {
                n = unsafe { (*g).gcdebt };
            }
            unsafe { luaE_setdebt(g, (*g).gcdebt - n) };
            let work = unsafe { (*g).gcdebt <= 0 };
            if work {
                unsafe { luaC_step(L) };
                if unsafe { (*g).gcstate == GCSpause } {
                    res = 1;
                }
            }
            unsafe { (*g).gcstp = oldstp };
            res
        }
        LUA_GCISRUNNING => (unsafe { (*g).gcstp == 0 }) as c_int,
        LUA_GCGEN => {
            let res = if unsafe { (*g).gckind == KGC_INC } {
                LUA_GCINC
            } else {
                LUA_GCGEN
            };
            unsafe { luaC_changemode(L, KGC_GENMINOR) };
            res
        }
        LUA_GCINC => {
            let res = if unsafe { (*g).gckind == KGC_INC } {
                LUA_GCINC
            } else {
                LUA_GCGEN
            };
            unsafe { luaC_changemode(L, KGC_INC as c_int) };
            res
        }
        LUA_GCPARAM => {
            let param = unsafe { args.arg::<c_int>() };
            let value = unsafe { args.arg::<c_int>() };
            unsafe {
                api_check(
                    0 <= param && (param as usize) < LUA_GCPN,
                    "invalid parameter",
                )
            };
            let res = unsafe { luaO_applyparam((*g).gcparams[param as usize], 100) as c_int };
            if value >= 0 {
                unsafe { (*g).gcparams[param as usize] = luaO_codeparam(value as u32) };
            }
            res
        }
        _ => -1,
    }
}

pub unsafe fn lua_error(L: *mut lua_State) -> c_int {
    let errobj = unsafe { s2v((*L).top.p.sub(1)) };
    unsafe { api_checkpop(L, 1) };
    if unsafe { ttisshrstring(errobj) && ptr::eq(tsvalue(errobj), (*G(L)).memerrmsg) } {
        unsafe { luaD_throw(L, LUA_ERRMEM) };
    } else {
        unsafe { luaG_errormsg(L) }
    }
}

pub unsafe fn lua_next(L: *mut lua_State, idx: c_int) -> c_int {
    unsafe { api_checkpop(L, 1) };
    let t = unsafe { gettable(L, idx) };
    let more = unsafe { luaH_next(L, t, (*L).top.p.sub(1)) };
    if more != 0 {
        unsafe { api_incr_top(L) };
    } else {
        unsafe { (*L).top.p = (*L).top.p.sub(1) };
    }
    more
}

pub unsafe fn lua_toclose(L: *mut lua_State, idx: c_int) {
    let o = unsafe { index2stack(L, idx) };
    unsafe {
        api_check(
            (*L).tbclist.p < o,
            "given index below or equal a marked one",
        )
    };
    unsafe { luaF_newtbcupval(L, o) };
    unsafe { (*(*L).ci).callstatus |= CIST_TBC };
}

pub unsafe fn lua_concat(L: *mut lua_State, n: c_int) {
    unsafe { api_checknelems(L, n) };
    if n > 0 {
        unsafe { luaV_concat(L, n) };
        unsafe { luaC_checkGC(L) };
    } else {
        unsafe { setsvalue2s(L, (*L).top.p, luaS_newlstr(L, c"".as_ptr(), 0)) };
        unsafe { api_incr_top(L) };
    }
}

pub unsafe fn lua_len(L: *mut lua_State, idx: c_int) {
    let t = unsafe { index2value(L, idx) };
    unsafe { luaV_objlen(L, (*L).top.p, t) };
    unsafe { api_incr_top(L) };
}

pub unsafe fn lua_getallocf(L: *mut lua_State, ud: *mut *mut c_void) -> lua_Alloc {
    if !ud.is_null() {
        unsafe { *ud = (*G(L)).ud };
    }
    unsafe { (*G(L)).frealloc }
}

pub unsafe fn lua_setallocf(L: *mut lua_State, f: lua_Alloc, ud: *mut c_void) {
    unsafe {
        (*G(L)).ud = ud;
        (*G(L)).frealloc = f;
    }
}

pub unsafe fn lua_setwarnf(L: *mut lua_State, f: lua_WarnFunction, ud: *mut c_void) {
    unsafe {
        (*G(L)).ud_warn = ud;
        (*G(L)).warnf = f;
    }
}

pub unsafe fn lua_warning(L: *mut lua_State, msg: *const c_char, tocont: c_int) {
    unsafe { luaE_warning(L, msg, tocont) };
}

pub unsafe fn lua_newuserdatauv(L: *mut lua_State, size: usize, nuvalue: c_int) -> *mut c_void {
    unsafe { api_check((0..SHRT_MAX).contains(&nuvalue), "invalid value") };
    let u = unsafe { luaS_newudata(L, size, nuvalue as u16) };
    unsafe { setuvalue(L, s2v((*L).top.p), u) };
    unsafe { api_incr_top(L) };
    unsafe { luaC_checkGC(L) };
    unsafe { u.cast::<u8>().add(udatamemoffset((*u).nuvalue)).cast() }
}

unsafe fn aux_upvalue(
    fi: *mut TValue,
    n: c_int,
    val: *mut *mut TValue,
    owner: *mut *mut GCObject,
) -> *const c_char {
    match unsafe { ttypetag(fi) } {
        LUA_VCCL => {
            let f = unsafe { clCvalue(fi) };
            if !(((n as u32).wrapping_sub(1)) < unsafe { (*f).nupvalues as u32 }) {
                return ptr::null();
            }
            unsafe {
                *val = ptr::addr_of_mut!((*f).upvalue)
                    .cast::<TValue>()
                    .add((n - 1) as usize)
            };
            if !owner.is_null() {
                unsafe { *owner = obj2gco(f) };
            }
            c"".as_ptr()
        }
        LUA_VLCL => {
            let f = unsafe { clLvalue(fi) };
            let p = unsafe { (*f).p };
            if !(((n as u32).wrapping_sub(1)) < unsafe { (*p).sizeupvalues as u32 }) {
                return ptr::null();
            }
            unsafe { *val = (*(*(*f).upvals.as_ptr().add((n - 1) as usize))).v.p };
            if !owner.is_null() {
                unsafe { *owner = obj2gco(*(*f).upvals.as_ptr().add((n - 1) as usize)) };
            }
            let name = unsafe { (*(*p).upvalues.add((n - 1) as usize)).name };
            if name.is_null() {
                c"(no name)".as_ptr()
            } else {
                unsafe { getstr(name) }
            }
        }
        _ => ptr::null(),
    }
}

pub unsafe fn lua_getupvalue(L: *mut lua_State, funcindex: c_int, n: c_int) -> *const c_char {
    let mut val = ptr::null_mut();
    let name = unsafe {
        aux_upvalue(
            index2value(L, funcindex),
            n,
            ptr::addr_of_mut!(val),
            ptr::null_mut(),
        )
    };
    if !name.is_null() {
        unsafe { setobj2s(L, (*L).top.p, val) };
        unsafe { api_incr_top(L) };
    }
    name
}

pub unsafe fn lua_setupvalue(L: *mut lua_State, funcindex: c_int, n: c_int) -> *const c_char {
    let mut val = ptr::null_mut();
    let mut owner = ptr::null_mut();
    let fi = unsafe { index2value(L, funcindex) };
    unsafe { api_checknelems(L, 1) };
    let name = unsafe { aux_upvalue(fi, n, ptr::addr_of_mut!(val), ptr::addr_of_mut!(owner)) };
    if !name.is_null() {
        unsafe { (*L).top.p = (*L).top.p.sub(1) };
        unsafe { setobj(val, s2v((*L).top.p)) };
        unsafe { luaC_barrier(L, owner, val) };
    }
    name
}

unsafe fn getupvalref(
    L: *mut lua_State,
    fidx: c_int,
    n: c_int,
    pf: *mut *mut LClosure,
) -> *mut *mut UpVal {
    static mut NULLUP: *mut UpVal = ptr::null_mut();
    let fi = unsafe { index2value(L, fidx) };
    unsafe { api_check(ttisLclosure(fi), "Lua function expected") };
    let f = unsafe { clLvalue(fi) };
    if !pf.is_null() {
        unsafe { *pf = f };
    }
    if 1 <= n && n <= unsafe { (*(*f).p).sizeupvalues } {
        unsafe { (*f).upvals.as_mut_ptr().add((n - 1) as usize) }
    } else {
        ptr::addr_of_mut!(NULLUP)
    }
}

pub unsafe fn lua_upvalueid(L: *mut lua_State, fidx: c_int, n: c_int) -> *mut c_void {
    let fi = unsafe { index2value(L, fidx) };
    match unsafe { ttypetag(fi) } {
        LUA_VLCL => unsafe { *getupvalref(L, fidx, n, ptr::null_mut()) as *mut c_void },
        LUA_VCCL => {
            let f = unsafe { clCvalue(fi) };
            if 1 <= n && n <= unsafe { (*f).nupvalues as c_int } {
                unsafe {
                    ptr::addr_of_mut!((*f).upvalue)
                        .cast::<TValue>()
                        .add((n - 1) as usize)
                        .cast()
                }
            } else {
                ptr::null_mut()
            }
        }
        LUA_VLCF => ptr::null_mut(),
        _ => {
            unsafe { api_check(false, "function expected") };
            ptr::null_mut()
        }
    }
}

pub unsafe fn lua_upvaluejoin(L: *mut lua_State, fidx1: c_int, n1: c_int, fidx2: c_int, n2: c_int) {
    let mut f1 = ptr::null_mut();
    let up1 = unsafe { getupvalref(L, fidx1, n1, ptr::addr_of_mut!(f1)) };
    let up2 = unsafe { getupvalref(L, fidx2, n2, ptr::null_mut()) };
    unsafe {
        api_check(
            !(*up1).is_null() && !(*up2).is_null(),
            "invalid upvalue index",
        )
    };
    unsafe { *up1 = *up2 };
    unsafe { luaC_objbarrier(L, obj2gco(f1), obj2gco(*up1)) };
}
