#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::luavm::GlobalState;
use crate::runtime::*;
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr;
use std::ffi::CStr;

const LUAI_MAXSHORTLEN: usize = 40;
const MEMERRMSG: &[u8] = b"not enough memory\0";
const MINSTRTABSIZE: c_int = 128;
const LSTRFIX: i8 = -2;
const LSTRMEM: i8 = -3;
const MAXSTRTB: c_int = (c_int::MAX as usize / size_of::<*mut TString>()) as c_int;

#[repr(C)]
struct Udata0 {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    nuvalue: u16,
    len: usize,
    metatable: *mut Table,
    bindata: UValue,
}

#[repr(C)]
struct NewExt {
    kind: i8,
    s: *const c_char,
    len: usize,
    ts: *mut TString,
}

/// C memcmp 的 Rust 等价：比较两段内存
#[inline]
unsafe fn memcmp(lhs: *const c_void, rhs: *const c_void, n: usize) -> c_int {
    let l = unsafe { core::slice::from_raw_parts(lhs.cast::<u8>(), n) };
    let r = unsafe { core::slice::from_raw_parts(rhs.cast::<u8>(), n) };
    l.cmp(r) as c_int
}

/// C memcpy 的 Rust 等价
#[inline]
unsafe fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) {
    unsafe { core::ptr::copy_nonoverlapping(src.cast::<u8>(), dst.cast::<u8>(), n) };
}

/// C strcmp 的 Rust 等价
#[inline]
unsafe fn strcmp(lhs: *const c_char, rhs: *const c_char) -> c_int {
    let l = unsafe { core::ffi::CStr::from_ptr(lhs) }.to_bytes();
    let r = unsafe { core::ffi::CStr::from_ptr(rhs) }.to_bytes();
    l.cmp(r) as c_int
}

#[inline]
unsafe fn luaC_fix(state: *mut lua_State, o: *mut GCObject) {
    unsafe { crate::gc::luaC_fix(state, o) }
}
#[inline]
unsafe fn luaC_fullgc(state: *mut lua_State, isemergency: c_int) {
    unsafe { crate::gc::luaC_fullgc(state, isemergency) }
}
#[inline]
unsafe fn luaC_newobj(state: *mut lua_State, tt: u8, sz: usize) -> *mut GCObject {
    unsafe { crate::gc::luaC_newobj(state, tt, sz) }
}
#[inline]
unsafe fn luaD_rawrunprotected(
    state: *mut lua_State,
    f: Option<unsafe fn(*mut lua_State, *mut c_void)>,
    ud: *mut c_void,
) -> TStatus {
    unsafe { crate::do_rs::luaD_rawrunprotected(state, f, ud) }
}
#[inline]
unsafe fn luaD_throw(state: *mut lua_State, errcode: TStatus) -> ! {
    unsafe { crate::do_rs::luaD_throw(state, errcode) }
}
#[inline]
unsafe fn luaM_malloc_(state: *mut lua_State, size: usize, tag: c_int) -> *mut c_void {
    unsafe { crate::mem::luaM_malloc_(state, size, tag) }
}
#[inline]
unsafe fn luaM_realloc_(
    state: *mut lua_State,
    block: *mut c_void,
    oldsize: usize,
    size: usize,
) -> *mut c_void {
    unsafe { crate::mem::luaM_realloc_(state, block, oldsize, size) }
}

pub(crate) unsafe fn raw_luaS_init(state: *mut c_void) {
    unsafe { luaS_init(state.cast()) };
}

pub(crate) unsafe fn raw_luaS_new(state: *mut c_void, s: *const c_char) -> *mut c_void {
    unsafe { luaS_new(state.cast(), s).cast() }
}

pub(crate) unsafe fn raw_luaS_newlstr(
    state: *mut c_void,
    s: *const c_char,
    len: usize,
) -> *mut c_void {
    unsafe { luaS_newlstr(state.cast(), s, len).cast() }
}

/// Local alias for `G()` from runtime.rs.
#[inline]
unsafe fn g(state: *mut lua_State) -> *mut GlobalState {
    unsafe { G(state) }
}

#[inline]
unsafe fn gco2ts(o: *mut GCObject) -> *mut TString {
    o.cast()
}

#[inline]
unsafe fn gco2u(o: *mut GCObject) -> *mut Udata {
    o.cast()
}

#[inline]
unsafe fn rawgetshrstr(ts: *const TString) -> *mut c_char {
    unsafe { ptr::addr_of!((*ts).contents).cast_mut().cast() }
}

#[inline]
unsafe fn getshrstr(ts: *const TString) -> *mut c_char {
    unsafe { rawgetshrstr(ts) }
}

#[inline]
unsafe fn getlngstr(ts: *const TString) -> *mut c_char {
    unsafe { (*ts).contents }
}

/// Local version returning *mut c_char (runtime.rs returns *const c_char).
#[inline]
unsafe fn getstr_mut(ts: *const TString) -> *mut c_char {
    if unsafe { strisshr(ts) } {
        unsafe { rawgetshrstr(ts) }
    } else {
        unsafe { (*ts).contents }
    }
}

#[inline]
unsafe fn tsslen(ts: *const TString) -> usize {
    if unsafe { strisshr(ts) } {
        unsafe { (*ts).shrlen as u8 as usize }
    } else {
        unsafe { (*ts).u.lnglen }
    }
}

#[inline]
unsafe fn iswhite<T>(obj: *const T) -> bool {
    let o = obj.cast::<GCObject>();
    unsafe { ((*o).marked & WHITEBITS) != 0 }
}

#[inline]
unsafe fn otherwhite(g: *mut GlobalState) -> u8 {
    unsafe { (*g).currentwhite ^ WHITEBITS }
}

#[inline]
unsafe fn isdead(g: *mut GlobalState, ts: *mut TString) -> bool {
    unsafe { ((*ts).marked & otherwhite(g)) != 0 }
}

#[inline]
unsafe fn changewhite(ts: *mut TString) {
    unsafe { (*ts).marked ^= WHITEBITS };
}

#[inline]
fn lmod(hash: u32, size: c_int) -> usize {
    debug_assert!(size > 0 && (size & (size - 1)) == 0);
    (hash & ((size - 1) as u32)) as usize
}

#[inline]
fn sizestrshr(len: usize) -> usize {
    offset_of!(TString, contents) + len + 1
}

#[inline]
fn udatamemoffset(nuv: u16) -> usize {
    if nuv == 0 {
        offset_of!(Udata0, bindata)
    } else {
        offset_of!(Udata, uv) + (size_of::<UValue>() * nuv as usize)
    }
}

#[inline]
fn sizeudata(nuv: u16, nb: usize) -> usize {
    udatamemoffset(nuv) + nb
}

#[inline]
unsafe fn point2uint(p: *const c_char) -> u32 {
    (p as usize & u32::MAX as usize) as u32
}

unsafe fn luaM_error(state: *mut lua_State) -> ! {
    unsafe { luaD_throw(state, LUA_ERRMEM) }
}

unsafe fn luaM_reallocvector_tstring(
    state: *mut lua_State,
    block: *mut *mut TString,
    oldn: c_int,
    n: c_int,
) -> *mut *mut TString {
    unsafe {
        luaM_realloc_(
            state,
            block.cast(),
            oldn.max(0) as usize * size_of::<*mut TString>(),
            n.max(0) as usize * size_of::<*mut TString>(),
        )
        .cast()
    }
}

unsafe fn luaM_newvector_tstring(state: *mut lua_State, n: c_int) -> *mut *mut TString {
    unsafe { luaM_malloc_(state, n.max(0) as usize * size_of::<*mut TString>(), 0).cast() }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_eqstr(a: *mut TString, b: *mut TString) -> c_int {
    let len1 = unsafe { tsslen(a) };
    let len2 = unsafe { tsslen(b) };
    let s1 = unsafe { getstr_mut(a) };
    let s2 = unsafe { getstr_mut(b) };
    ((len1 == len2) && unsafe { memcmp(s1.cast(), s2.cast(), len1) == 0 }) as c_int
}

unsafe fn luaS_hash(str_: *const c_char, mut l: usize, seed: u32) -> u32 {
    let mut h = seed ^ l as u32;
    while l > 0 {
        l -= 1;
        let ch = unsafe { *str_.add(l) as u8 as u32 };
        h ^= (h << 5).wrapping_add(h >> 2).wrapping_add(ch);
    }
    h
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_hashlongstr(ts: *mut TString) -> u32 {
    if unsafe { (*ts).extra } == 0 {
        let len = unsafe { (*ts).u.lnglen };
        let seed = unsafe { (*ts).hash };
        unsafe {
            (*ts).hash = luaS_hash(getlngstr(ts), len, seed);
            (*ts).extra = 1;
        }
    }
    unsafe { (*ts).hash }
}

unsafe fn tablerehash(vect: *mut *mut TString, osize: c_int, nsize: c_int) {
    for i in osize..nsize {
        unsafe { *vect.add(i as usize) = ptr::null_mut() };
    }
    for i in 0..osize {
        let mut p = unsafe { *vect.add(i as usize) };
        unsafe { *vect.add(i as usize) = ptr::null_mut() };
        while !p.is_null() {
            let hnext = unsafe { (*p).u.hnext };
            let h = lmod(unsafe { (*p).hash }, nsize);
            unsafe {
                (*p).u.hnext = *vect.add(h);
                *vect.add(h) = p;
            }
            p = hnext;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_resize(state: *mut lua_State, nsize: c_int) {
    let tb = unsafe { ptr::addr_of_mut!((*g(state)).strt) };
    let osize = unsafe { (*tb).size };
    if nsize < osize {
        unsafe { tablerehash((*tb).hash, osize, nsize) };
    }
    let newvect = unsafe { luaM_reallocvector_tstring(state, (*tb).hash, osize, nsize) };
    if newvect.is_null() {
        if nsize < osize {
            unsafe { tablerehash((*tb).hash, nsize, osize) };
        }
    } else {
        unsafe {
            (*tb).hash = newvect;
            (*tb).size = nsize;
        }
        if nsize > osize {
            unsafe { tablerehash(newvect, osize, nsize) };
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_clearcache(g: *mut GlobalState) {
    for i in 0..STRCACHE_N {
        for j in 0..STRCACHE_M {
            let ts = unsafe { (&mut (*g).strcache)[i][j] };
            if !ts.is_null() && unsafe { iswhite(ts.cast_const()) } {
                unsafe { (&mut (*g).strcache)[i][j] = (*g).memerrmsg };
            }
        }
    }
}

pub(crate) unsafe fn luaS_init(state: *mut lua_State) {
    let g = unsafe { g(state) };
    let tb = unsafe { ptr::addr_of_mut!((*g).strt) };
    unsafe {
        (*tb).hash = luaM_newvector_tstring(state, MINSTRTABSIZE);
        tablerehash((*tb).hash, 0, MINSTRTABSIZE);
        (*tb).size = MINSTRTABSIZE;
    }
    let memerrmsg = unsafe { luaS_newlstr(state, MEMERRMSG.as_ptr().cast(), MEMERRMSG.len() - 1) };
    unsafe {
        (*g).memerrmsg = memerrmsg;
        luaC_fix(state, memerrmsg.cast());
    }
    for i in 0..STRCACHE_N {
        for j in 0..STRCACHE_M {
            unsafe { (&mut (*g).strcache)[i][j] = (*g).memerrmsg };
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_sizelngstr(len: usize, kind: c_int) -> usize {
    match kind as i8 {
        LSTRREG => offset_of!(TString, falloc) + len + 1,
        LSTRFIX => offset_of!(TString, falloc),
        _ => size_of::<TString>(),
    }
}

unsafe fn createstrobj(state: *mut lua_State, totalsize: usize, tag: u8, h: u32) -> *mut TString {
    let o = unsafe { luaC_newobj(state, tag, totalsize) };
    let ts = unsafe { gco2ts(o) };
    unsafe {
        (*ts).hash = h;
        (*ts).extra = 0;
    }
    ts
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_createlngstrobj(
    state: *mut lua_State,
    l: usize,
) -> *mut TString {
    let totalsize = unsafe { luaS_sizelngstr(l, LSTRREG as c_int) };
    let ts = unsafe { createstrobj(state, totalsize, LUA_VLNGSTR, (*g(state)).seed) };
    unsafe {
        (*ts).u.lnglen = l;
        (*ts).shrlen = LSTRREG;
        (*ts).contents = ts.cast::<u8>().add(offset_of!(TString, falloc)).cast();
        *(*ts).contents.add(l) = 0;
    }
    ts
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_remove(state: *mut lua_State, ts: *mut TString) {
    let tb = unsafe { ptr::addr_of_mut!((*g(state)).strt) };
    let mut p = unsafe { (*tb).hash.add(lmod((*ts).hash, (*tb).size)) };
    while unsafe { *p } != ts {
        p = unsafe { ptr::addr_of_mut!((**p).u.hnext) };
    }
    unsafe {
        *p = (**p).u.hnext;
        (*tb).nuse -= 1;
    }
}

unsafe fn growstrtab(state: *mut lua_State, tb: *mut stringtable) {
    if unsafe { (*tb).nuse } == c_int::MAX {
        unsafe { luaC_fullgc(state, 1) };
        if unsafe { (*tb).nuse } == c_int::MAX {
            unsafe { luaM_error(state) };
        }
    }
    if unsafe { (*tb).size } <= MAXSTRTB / 2 {
        unsafe { luaS_resize(state, (*tb).size * 2) };
    }
}

unsafe fn internshrstr(state: *mut lua_State, str_: *const c_char, l: usize) -> *mut TString {
    let g = unsafe { g(state) };
    let tb = unsafe { ptr::addr_of_mut!((*g).strt) };
    let h = unsafe { luaS_hash(str_, l, (*g).seed) };
    let mut list = unsafe { (*tb).hash.add(lmod(h, (*tb).size)) };
    let mut ts = unsafe { *list };
    while !ts.is_null() {
        if l == unsafe { (*ts).shrlen as u8 as usize }
            && unsafe { memcmp(str_.cast(), getshrstr(ts).cast(), l) == 0 }
        {
            if unsafe { isdead(g, ts) } {
                unsafe { changewhite(ts) };
            }
            return ts;
        }
        ts = unsafe { (*ts).u.hnext };
    }
    if unsafe { (*tb).nuse >= (*tb).size } {
        unsafe { growstrtab(state, tb) };
        list = unsafe { (*tb).hash.add(lmod(h, (*tb).size)) };
    }
    ts = unsafe { createstrobj(state, sizestrshr(l), LUA_VSHRSTR, h) };
    unsafe {
        (*ts).shrlen = l as i8;
        *getshrstr(ts).add(l) = 0;
        memcpy(getshrstr(ts).cast(), str_.cast(), l);
        (*ts).u.hnext = *list;
        *list = ts;
        (*tb).nuse += 1;
    }
    ts
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_newlstr(
    state: *mut lua_State,
    str_: *const c_char,
    l: usize,
) -> *mut TString {
    if l <= LUAI_MAXSHORTLEN {
        unsafe { internshrstr(state, str_, l) }
    } else {
        if l >= MAX_SIZE - size_of::<TString>() {
            unsafe { crate::mem::luaM_toobig(state.cast()) };
        }
        let ts = unsafe { luaS_createlngstrobj(state, l) };
        unsafe { memcpy(getlngstr(ts).cast(), str_.cast(), l) };
        ts
    }
}

pub(crate) unsafe fn luaS_new(state: *mut lua_State, str_: *const c_char) -> *mut TString {
    let i = unsafe { point2uint(str_) as usize % STRCACHE_N };
    let p = unsafe { &mut (*g(state)).strcache[i] };
    for item in p.iter() {
        if unsafe { strcmp(str_, getstr_mut(*item).cast()) == 0 } {
            return *item;
        }
    }
    for j in (1..STRCACHE_M).rev() {
        p[j] = p[j - 1];
    }
    let len = unsafe { CStr::from_ptr(str_).to_bytes().len() };
    p[0] = unsafe { luaS_newlstr(state, str_, len) };
    p[0]
}

pub(crate) unsafe fn luaS_newudata(state: *mut lua_State, s: usize, nuvalue: u16) -> *mut Udata {
    if s > MAX_SIZE - udatamemoffset(nuvalue) {
        unsafe { crate::mem::luaM_toobig(state.cast()) };
    }
    let o = unsafe { luaC_newobj(state, LUA_VUSERDATA, sizeudata(nuvalue, s)) };
    let u = unsafe { gco2u(o) };
    unsafe {
        (*u).len = s;
        (*u).nuvalue = nuvalue;
        (*u).metatable = ptr::null_mut();
    }
    for i in 0..nuvalue as usize {
        unsafe { setnilvalue(ptr::addr_of_mut!((*(*u).uv.as_mut_ptr().add(i)).uv)) };
    }
    u
}

unsafe  fn f_newext(state: *mut lua_State, ud: *mut c_void) {
    let ne = unsafe { &mut *ud.cast::<NewExt>() };
    let size = unsafe { luaS_sizelngstr(0, ne.kind as c_int) };
    ne.ts = unsafe { createstrobj(state, size, LUA_VLNGSTR, (*g(state)).seed) };
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_newextlstr(
    state: *mut lua_State,
    s: *const c_char,
    len: usize,
    falloc: lua_Alloc,
    ud: *mut c_void,
) -> *mut TString {
    let mut ne = NewExt {
        kind: 0,
        s,
        len,
        ts: ptr::null_mut(),
    };
    if falloc.is_none() {
        ne.kind = LSTRFIX;
        unsafe { f_newext(state, ptr::addr_of_mut!(ne).cast()) };
    } else {
        ne.kind = LSTRMEM;
        if unsafe { luaD_rawrunprotected(state, Some(f_newext), ptr::addr_of_mut!(ne).cast()) }
            != LUA_OK
        {
            if let Some(free) = falloc {
                unsafe { free(ud, s.cast_mut().cast(), len + 1, 0) };
            }
            unsafe { luaM_error(state) };
        }
        unsafe {
            (*ne.ts).falloc = falloc;
            (*ne.ts).ud = ud;
        }
    }
    unsafe {
        (*ne.ts).shrlen = ne.kind;
        (*ne.ts).u.lnglen = len;
        (*ne.ts).contents = s.cast_mut();
    }
    ne.ts
}

#[unsafe(no_mangle)]
pub unsafe  fn luaS_normstr(
    state: *mut lua_State,
    ts: *mut TString,
) -> *mut TString {
    let len = unsafe { (*ts).u.lnglen };
    if len > LUAI_MAXSHORTLEN {
        ts
    } else {
        unsafe { internshrstr(state, getlngstr(ts), len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{aux_rs::{luaL_checkversion_, luaL_newstate}, luaffi::LUAL_NUMSIZES, state::lua_close};

    #[test]
    fn strings_and_userdata_behave_like_c_runtime() {
        let state = { luaL_newstate() }.cast::<lua_State>();
        assert!(!state.is_null());

        let result = (|| unsafe  {
            luaL_checkversion_(state.cast(), LUA_VERSION_NUM, LUAL_NUMSIZES);

            let short1 = luaS_newlstr(state, c"abc".as_ptr(), 3);
            let short2 = luaS_newlstr(state, c"abc".as_ptr(), 3);
            assert_eq!(short1, short2);

            let long_bytes = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\0";
            let long1 = luaS_newlstr(state, long_bytes.as_ptr().cast(), long_bytes.len() - 1);
            let long2 = luaS_newlstr(state, long_bytes.as_ptr().cast(), long_bytes.len() - 1);
            assert_ne!(long1, long2);
            assert_eq!(luaS_eqstr(long1, long2), 1);
            assert_eq!(luaS_hashlongstr(long1), luaS_hashlongstr(long2));

            let ud = luaS_newudata(state, 16, 2);
            assert_eq!((*ud).len, 16);
            assert_eq!((*ud).nuvalue, 2);
            assert!((*ud).metatable.is_null());
            assert_eq!((*(*ud).uv.as_ptr()).uv.tt_, LUA_VNIL);
            assert_eq!((*(*ud).uv.as_ptr().add(1)).uv.tt_, LUA_VNIL);
        })();

        unsafe { lua_close(state.cast()) };
        result
    }
}
