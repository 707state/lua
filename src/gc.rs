#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::{luavm::GlobalState, runtime::*};
use core::mem::size_of;

const GCSWEEPMAX: l_mem = 20;
const CWUFIN: l_mem = 10;

const GCSpropagate: u8 = 0;
const GCSenteratomic: u8 = 1;
const GCSatomic: u8 = 2;
const GCSswpallgc: u8 = 3;
const GCSswpfinobj: u8 = 4;
const GCSswptobefnz: u8 = 5;
const GCSswpend: u8 = 6;
const GCScallfin: u8 = 7;

const FINALIZEDBIT: u8 = 6;
const TESTBIT: u8 = 7;
const AGEBITS: u8 = 7;

const G_NEW: u8 = 0;
const G_SURVIVAL: u8 = 1;
const G_OLD0: u8 = 2;
const G_OLD1: u8 = 3;
const G_OLD: u8 = 4;
const G_TOUCHED1: u8 = 5;
const G_TOUCHED2: u8 = 6;

const TM_GC: usize = 2;
const TM_MODE: usize = 3;
const LSTRMEM: i8 = -3;
const KGC_GENMAJOR: u8 = 2;

#[inline] unsafe fn luaM_malloc_(s: *mut lua_State, size: usize, tag: c_int) -> *mut c_void { unsafe { crate::mem::luaM_malloc_(s, size, tag) } }
#[inline] unsafe fn luaM_free_(s: *mut lua_State, b: *mut c_void, os: usize) { unsafe { crate::mem::luaM_free_(s, b, os) } }
#[inline] unsafe fn luaH_size(t: *mut Table) -> usize { unsafe { crate::table::luaH_size(t) } }
#[inline] unsafe fn luaH_free(s: *mut lua_State, t: *mut Table) { unsafe { crate::table::luaH_free(s, t) } }
#[inline] unsafe fn luaF_protosize(p: *mut Proto) -> usize { unsafe { crate::func::luaF_protosize(p) } }
#[inline] unsafe fn luaF_freeproto(s: *mut lua_State, f: *mut Proto) { unsafe { crate::func::luaF_freeproto(s, f) } }
#[inline] unsafe fn luaE_threadsize(s: *mut lua_State) -> usize { unsafe { crate::state::luaE_threadsize(s) } }
#[inline] unsafe fn luaE_freethread(s: *mut lua_State, t: *mut lua_State) { unsafe { crate::state::luaE_freethread(s, t) } }
#[inline] unsafe fn luaE_setdebt(g: *mut GlobalState, d: l_mem) { unsafe { crate::state::luaE_setdebt(g, d) } }
#[inline] unsafe fn luaE_warnerror(s: *mut lua_State, w: *const c_char) { unsafe { crate::state::luaE_warnerror(s, w) } }
#[inline] unsafe fn luaS_sizelngstr(len: usize, kind: c_int) -> usize { unsafe { crate::string::luaS_sizelngstr(len, kind) } }
#[inline] unsafe fn luaS_resize(s: *mut lua_State, n: c_int) { unsafe { crate::string::luaS_resize(s, n) } }
#[inline] unsafe fn luaS_remove(s: *mut lua_State, ts: *mut TString) { unsafe { crate::string::luaS_remove(s, ts) } }
#[inline] unsafe fn luaS_clearcache(g: *mut GlobalState) { unsafe { crate::string::luaS_clearcache(g) } }
#[inline] unsafe fn luaT_gettm(e: *mut Table, ev: c_int, en: *mut TString) -> *const TValue { unsafe { crate::tm::luaT_gettm(e, ev, en) } }
#[inline] unsafe fn luaT_gettmbyobj(s: *mut lua_State, o: *const TValue, ev: c_int) -> *const TValue { unsafe { crate::tm::luaT_gettmbyobj(s, o, ev) } }
#[inline] unsafe fn luaD_shrinkstack(L: *mut lua_State) { unsafe { crate::do_rs::luaD_shrinkstack(L) } }
#[inline] unsafe fn luaD_checkminstack(L: *mut lua_State) -> c_int { unsafe { crate::do_rs::luaD_checkminstack(L) } }
#[inline] unsafe fn luaF_unlinkupval(uv: *mut UpVal) { unsafe { crate::func::luaF_unlinkupval(uv) } }

#[inline]
unsafe fn bitmask(b: u8) -> u8 {
    1u8 << b
}

#[inline]
unsafe fn otherwhite(g: *mut GlobalState) -> u8 {
    (*g).currentwhite ^ WHITEBITS
}

#[inline]
unsafe fn luaC_white(g: *mut GlobalState) -> u8 {
    (*g).currentwhite & WHITEBITS
}

#[inline]
unsafe fn isgray(o: *mut GCObject) -> bool {
    (*o).marked & (WHITEBITS | bitmask(BLACKBIT)) == 0
}

#[inline]
unsafe fn isdeadm(ow: u8, marked: u8) -> bool {
    marked & ow != 0
}

#[inline]
unsafe fn isdead(g: *mut GlobalState, o: *mut GCObject) -> bool {
    isdeadm(otherwhite(g), (*o).marked)
}

#[inline]
unsafe fn getage(o: *mut GCObject) -> u8 {
    (*o).marked & AGEBITS
}

#[inline]
unsafe fn setage(o: *mut GCObject, age: u8) {
    (*o).marked = ((*o).marked & !AGEBITS) | age;
}

#[inline]
unsafe fn isold(o: *mut GCObject) -> bool {
    getage(o) > G_SURVIVAL
}

#[inline]
unsafe fn makewhite(g: *mut GlobalState, o: *mut GCObject) {
    (*o).marked = ((*o).marked & !((1 << BLACKBIT) | WHITEBITS)) | luaC_white(g);
}

#[inline]
unsafe fn set2gray(o: *mut GCObject) {
    (*o).marked &= !((1 << BLACKBIT) | WHITEBITS);
}

#[inline]
unsafe fn set2black(o: *mut GCObject) {
    (*o).marked = ((*o).marked & !WHITEBITS) | bitmask(BLACKBIT);
}

#[inline]
unsafe fn nw2black(o: *mut GCObject) {
    debug_assert!(!iswhite(o));
    (*o).marked |= bitmask(BLACKBIT);
}

#[inline]
unsafe fn tofinalize(o: *mut GCObject) -> bool {
    (*o).marked & bitmask(FINALIZEDBIT) != 0
}

#[inline]
unsafe fn resetbit(x: &mut u8, b: u8) {
    *x &= !bitmask(b);
}

#[inline]
unsafe fn l_setbit(x: &mut u8, b: u8) {
    *x |= bitmask(b);
}

#[inline]
unsafe fn gcrunning(g: *mut GlobalState) -> bool {
    (*g).gcstp == 0
}

#[inline]
unsafe fn issweepphase(g: *mut GlobalState) -> bool {
    GCSswpallgc <= (*g).gcstate && (*g).gcstate <= GCSswpend
}

#[inline]
unsafe fn keepinvariant(g: *mut GlobalState) -> bool {
    (*g).gcstate <= GCSatomic
}

#[inline]
unsafe fn upisopen(uv: *mut UpVal) -> bool {
    !ptr::eq((*uv).v.p, ptr::addr_of_mut!((*uv).u.value))
}

#[inline]
unsafe fn isintwups(th: *mut lua_State) -> bool {
    !ptr::eq((*th).twups, th)
}

#[inline]
unsafe fn gco2t(o: *mut GCObject) -> *mut Table {
    o.cast()
}

#[inline]
unsafe fn gco2lcl(o: *mut GCObject) -> *mut LClosure {
    o.cast()
}

#[inline]
unsafe fn gco2ccl(o: *mut GCObject) -> *mut CClosure {
    o.cast()
}

#[inline]
unsafe fn gco2u(o: *mut GCObject) -> *mut Udata {
    o.cast()
}

#[inline]
unsafe fn gco2p(o: *mut GCObject) -> *mut Proto {
    o.cast()
}

#[inline]
unsafe fn gco2th(o: *mut GCObject) -> *mut lua_State {
    o.cast()
}

#[inline]
unsafe fn gco2ts(o: *mut GCObject) -> *mut TString {
    o.cast()
}

#[inline]
unsafe fn gco2upv(o: *mut GCObject) -> *mut UpVal {
    o.cast()
}

#[inline]
unsafe fn sizeCclosure(n: u8) -> usize {
    size_of::<CClosure>() + (n as usize).saturating_sub(1) * size_of::<TValue>()
}

#[inline]
unsafe fn sizeLclosure(n: u8) -> usize {
    size_of::<LClosure>() + (n as usize).saturating_sub(1) * size_of::<*mut UpVal>()
}

#[inline]
unsafe fn sizestrshr(len: u32) -> usize {
    size_of::<TString>() + len as usize + 1
}

#[inline]
unsafe fn sizeudata(nuv: u16, nb: usize) -> usize {
    udatamemoffset(nuv) + nb
}

#[inline]
unsafe fn gnodelast(h: *mut Table) -> *mut Node {
    gnode(h, sizenode(h))
}

#[inline]
unsafe fn gnode(t: *mut Table, i: u32) -> *mut Node {
    (*t).node.add(i as usize)
}

#[inline]
unsafe fn sizenode(t: *mut Table) -> u32 {
    1u32 << (*t).lsizenode
}

#[inline]
unsafe fn gval(n: *mut Node) -> *mut TValue {
    ptr::addr_of_mut!((*n).i_val)
}

#[inline]
unsafe fn keytt(n: *mut Node) -> u8 {
    (*n).u.key_tt
}

#[inline]
unsafe fn keyisnil(n: *mut Node) -> bool {
    keytt(n) == LUA_TNIL
}

#[inline]
unsafe fn keyiscollectable(n: *mut Node) -> bool {
    keytt(n) & BIT_ISCOLLECTABLE != 0
}

#[inline]
unsafe fn gckey(n: *mut Node) -> *mut GCObject {
    (*n).u.key_val.gc
}

#[inline]
unsafe fn gckeyN(n: *mut Node) -> *mut GCObject {
    if keyiscollectable(n) {
        gckey(n)
    } else {
        ptr::null_mut()
    }
}

#[inline]
unsafe fn setdeadkey(n: *mut Node) {
    (*n).u.key_tt = 11;
}

#[inline]
unsafe fn isempty(v: *const TValue) -> bool {
    ttisnil(v)
}

#[inline]
unsafe fn setempty(v: *mut TValue) {
    (*v).tt_ = LUA_VEMPTY;
    (*v).value_.gc = ptr::null_mut();
}

#[inline]
unsafe fn valiswhite(v: *const TValue) -> bool {
    iscollectable(v) && iswhite(gcvalue(v))
}

#[inline]
unsafe fn keyiswhite(n: *mut Node) -> bool {
    keyiscollectable(n) && iswhite(gckey(n))
}

#[inline]
unsafe fn gcvalueN(o: *const TValue) -> *mut GCObject {
    if iscollectable(o) {
        gcvalue(o)
    } else {
        ptr::null_mut()
    }
}

#[inline]
unsafe fn gcvalarr(t: *mut Table, i: u32) -> *mut GCObject {
    if *getArrTag(t, i) & BIT_ISCOLLECTABLE != 0 {
        (*getArrVal(t, i)).gc
    } else {
        ptr::null_mut()
    }
}

#[inline]
unsafe fn markvalue(g: *mut GlobalState, o: *mut TValue) {
    if valiswhite(o) {
        reallymarkobject(g, gcvalue(o));
    }
}

#[inline]
unsafe fn markkey(g: *mut GlobalState, n: *mut Node) {
    if keyiswhite(n) {
        reallymarkobject(g, gckey(n));
    }
}

#[inline]
unsafe fn markobject<T>(g: *mut GlobalState, t: *mut T) {
    let o = obj2gco(t);
    if iswhite(o) {
        reallymarkobject(g, o);
    }
}

#[inline]
unsafe fn markobjectN<T>(g: *mut GlobalState, t: *mut T) {
    if !t.is_null() {
        markobject(g, t);
    }
}

#[inline]
unsafe fn getgclist(o: *mut GCObject) -> *mut *mut GCObject {
    match (*o).tt {
        LUA_VTABLE => ptr::addr_of_mut!((*gco2t(o)).gclist),
        LUA_VLCL => ptr::addr_of_mut!((*gco2lcl(o)).gclist),
        LUA_VCCL => ptr::addr_of_mut!((*gco2ccl(o)).gclist),
        LUA_VTHREAD => ptr::addr_of_mut!((*gco2th(o)).gclist),
        LUA_VPROTO => ptr::addr_of_mut!((*gco2p(o)).gclist),
        LUA_VUSERDATA => ptr::addr_of_mut!((*gco2u(o)).gclist),
        _ => panic!("invalid gclist object"),
    }
}

#[inline]
unsafe fn linkgclist_(o: *mut GCObject, pnext: *mut *mut GCObject, list: *mut *mut GCObject) {
    debug_assert!(!isgray(o));
    *pnext = *list;
    *list = o;
    set2gray(o);
}

#[inline]
unsafe fn linkobjgclist(o: *mut GCObject, list: *mut *mut GCObject) {
    linkgclist_(o, getgclist(o), list);
}

#[inline]
unsafe fn linkgclist_table(o: *mut Table, list: *mut *mut GCObject) {
    linkgclist_(obj2gco(o), ptr::addr_of_mut!((*o).gclist), list);
}

#[inline]
unsafe fn linkgclist_udata(o: *mut Udata, list: *mut *mut GCObject) {
    linkgclist_(obj2gco(o), ptr::addr_of_mut!((*o).gclist), list);
}

#[inline]
unsafe fn linkgclist_lclosure(o: *mut LClosure, list: *mut *mut GCObject) {
    linkgclist_(obj2gco(o), ptr::addr_of_mut!((*o).gclist), list);
}

#[inline]
unsafe fn linkgclist_cclosure(o: *mut CClosure, list: *mut *mut GCObject) {
    linkgclist_(obj2gco(o), ptr::addr_of_mut!((*o).gclist), list);
}

#[inline]
unsafe fn linkgclist_thread(o: *mut lua_State, list: *mut *mut GCObject) {
    linkgclist_(obj2gco(o), ptr::addr_of_mut!((*o).gclist), list);
}

#[inline]
unsafe fn linkgclist_proto(o: *mut Proto, list: *mut *mut GCObject) {
    linkgclist_(obj2gco(o), ptr::addr_of_mut!((*o).gclist), list);
}

unsafe fn objsize(o: *mut GCObject) -> l_mem {
    match (*o).tt {
        LUA_VTABLE => luaH_size(gco2t(o)) as l_mem,
        LUA_VLCL => sizeLclosure((*gco2lcl(o)).nupvalues) as l_mem,
        LUA_VCCL => sizeCclosure((*gco2ccl(o)).nupvalues) as l_mem,
        LUA_VUSERDATA => {
            let u = gco2u(o);
            sizeudata((*u).nuvalue, (*u).len) as l_mem
        }
        LUA_VPROTO => luaF_protosize(gco2p(o)) as l_mem,
        LUA_VTHREAD => luaE_threadsize(gco2th(o)) as l_mem,
        LUA_VSHRSTR => sizestrshr((*gco2ts(o)).shrlen as u32) as l_mem,
        LUA_VLNGSTR => {
            let ts = gco2ts(o);
            luaS_sizelngstr((*ts).u.lnglen, (*ts).shrlen as c_int) as l_mem
        }
        LUA_VUPVAL => size_of::<UpVal>() as l_mem,
        _ => panic!("invalid gc object"),
    }
}

unsafe fn clearkey(n: *mut Node) {
    debug_assert!(isempty(gval(n)));
    if keyiscollectable(n) {
        setdeadkey(n);
    }
}

unsafe fn iscleared(g: *mut GlobalState, o: *mut GCObject) -> bool {
    if o.is_null() {
        false
    } else if novariant((*o).tt) == LUA_TSTRING {
        markobject(g, o);
        false
    } else {
        iswhite(o)
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaC_barrier_(
    L: *mut lua_State,
    o: *mut GCObject,
    v: *mut GCObject,
) {
    let g = G(L);
    debug_assert!(isblack(o) && iswhite(v) && !isdead(g, v) && !isdead(g, o));
    if keepinvariant(g) {
        reallymarkobject(g, v);
        if isold(o) {
            debug_assert!(!isold(v));
            setage(v, G_OLD0);
        }
    } else if issweepphase(g) && (*g).gckind != KGC_GENMINOR as u8 {
        makewhite(g, o);
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaC_barrierback_(L: *mut lua_State, o: *mut GCObject) {
    let g = G(L);
    debug_assert!(isblack(o) && !isdead(g, o));
    if getage(o) == G_TOUCHED2 {
        set2gray(o);
    } else {
        linkobjgclist(o, ptr::addr_of_mut!((*g).grayagain));
    }
    if isold(o) {
        setage(o, G_TOUCHED1);
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaC_fix(L: *mut lua_State, o: *mut GCObject) {
    let g = G(L);
    debug_assert!((*g).allgc == o);
    set2gray(o);
    setage(o, G_OLD);
    (*g).allgc = (*o).next;
    (*o).next = (*g).fixedgc;
    (*g).fixedgc = o;
}

#[unsafe(no_mangle)]
pub unsafe  fn luaC_newobjdt(
    L: *mut lua_State,
    tt: lu_byte,
    sz: usize,
    offset: usize,
) -> *mut GCObject {
    let g = G(L);
    let p = luaM_malloc_(L, sz, novariant(tt) as c_int).cast::<u8>();
    let o = p.add(offset).cast::<GCObject>();
    (*o).marked = luaC_white(g);
    (*o).tt = tt;
    (*o).next = (*g).allgc;
    (*g).allgc = o;
    o
}

#[unsafe(no_mangle)]
pub unsafe  fn luaC_newobj(
    L: *mut lua_State,
    tt: lu_byte,
    sz: usize,
) -> *mut GCObject {
    luaC_newobjdt(L, tt, sz, 0)
}

unsafe fn reallymarkobject(g: *mut GlobalState, o: *mut GCObject) {
    (*g).gcmarked += objsize(o);
    match (*o).tt {
        LUA_VSHRSTR | LUA_VLNGSTR => set2black(o),
        LUA_VUPVAL => {
            let uv = gco2upv(o);
            if upisopen(uv) {
                set2gray(uv.cast());
            } else {
                set2black(uv.cast());
            }
            markvalue(g, (*uv).v.p);
        }
        LUA_VUSERDATA => {
            let u = gco2u(o);
            if (*u).nuvalue == 0 {
                markobjectN(g, (*u).metatable);
                set2black(u.cast());
            } else {
                linkgclist_udata(u, ptr::addr_of_mut!((*g).gray));
            }
        }
        LUA_VLCL => linkgclist_lclosure(gco2lcl(o), ptr::addr_of_mut!((*g).gray)),
        LUA_VCCL => linkgclist_cclosure(gco2ccl(o), ptr::addr_of_mut!((*g).gray)),
        LUA_VTABLE => linkgclist_table(gco2t(o), ptr::addr_of_mut!((*g).gray)),
        LUA_VTHREAD => linkgclist_thread(gco2th(o), ptr::addr_of_mut!((*g).gray)),
        LUA_VPROTO => linkgclist_proto(gco2p(o), ptr::addr_of_mut!((*g).gray)),
        _ => panic!("invalid mark object"),
    }
}

unsafe fn markmt(g: *mut GlobalState) {
    for i in 0..LUA_NUMTYPES as usize {
        markobjectN(g, (&mut (*g).mt)[i]);
    }
}

unsafe fn markbeingfnz(g: *mut GlobalState) {
    let mut o = (*g).tobefnz;
    while !o.is_null() {
        markobject(g, o);
        o = (*o).next;
    }
}

unsafe fn remarkupvals(g: *mut GlobalState) {
    let p = ptr::addr_of_mut!((*g).twups);
    let mut pcur = p;
    while !(*pcur).is_null() {
        let thread = *pcur;
        if !iswhite(thread.cast()) && !(*thread).openupval.is_null() {
            pcur = ptr::addr_of_mut!((*thread).twups);
        } else {
            let mut uv = (*thread).openupval;
            *pcur = (*thread).twups;
            (*thread).twups = thread;
            while !uv.is_null() {
                if !iswhite(uv.cast()) {
                    markvalue(g, (*uv).v.p);
                }
                uv = (*uv).u.open.next;
            }
        }
    }
}

unsafe fn cleargraylists(g: *mut GlobalState) {
    (*g).gray = ptr::null_mut();
    (*g).grayagain = ptr::null_mut();
    (*g).weak = ptr::null_mut();
    (*g).allweak = ptr::null_mut();
    (*g).ephemeron = ptr::null_mut();
}

unsafe fn restartcollection(g: *mut GlobalState) {
    cleargraylists(g);
    (*g).gcmarked = 0;
    markobject(g, mainthread(g));
    markvalue(g, ptr::addr_of_mut!((*g).l_registry));
    markmt(g);
    markbeingfnz(g);
}

unsafe fn genlink(g: *mut GlobalState, o: *mut GCObject) {
    debug_assert!(isblack(o));
    if getage(o) == G_TOUCHED1 {
        linkobjgclist(o, ptr::addr_of_mut!((*g).grayagain));
    } else if getage(o) == G_TOUCHED2 {
        setage(o, G_OLD);
    }
}

unsafe fn traverseweakvalue(g: *mut GlobalState, h: *mut Table) {
    let limit = gnodelast(h);
    let mut hasclears = (*h).asize > 0;
    let mut n = gnode(h, 0);
    while n < limit {
        if isempty(gval(n)) {
            clearkey(n);
        } else {
            debug_assert!(!keyisnil(n));
            markkey(g, n);
            if !hasclears && iscleared(g, gcvalueN(gval(n))) {
                hasclears = true;
            }
        }
        n = n.add(1);
    }
    if (*g).gcstate == GCSpropagate {
        linkgclist_table(h, ptr::addr_of_mut!((*g).grayagain));
    } else if hasclears {
        linkgclist_table(h, ptr::addr_of_mut!((*g).weak));
    } else {
        genlink(g, obj2gco(h));
    }
}

unsafe fn traversearray(g: *mut GlobalState, h: *mut Table) -> c_int {
    let mut marked = 0;
    for i in 0..(*h).asize {
        let o = gcvalarr(h, i);
        if !o.is_null() && iswhite(o) {
            marked = 1;
            reallymarkobject(g, o);
        }
    }
    marked
}

unsafe fn traverseephemeron(g: *mut GlobalState, h: *mut Table, inv: c_int) -> c_int {
    let mut hasclears = 0;
    let mut hasww = 0;
    let nsize = sizenode(h);
    let mut marked = traversearray(g, h);
    for i in 0..nsize {
        let n = if inv != 0 {
            gnode(h, nsize - 1 - i)
        } else {
            gnode(h, i)
        };
        if isempty(gval(n)) {
            clearkey(n);
        } else if iscleared(g, gckeyN(n)) {
            hasclears = 1;
            if valiswhite(gval(n)) {
                hasww = 1;
            }
        } else if valiswhite(gval(n)) {
            marked = 1;
            reallymarkobject(g, gcvalue(gval(n)));
        }
    }
    if (*g).gcstate == GCSpropagate {
        linkgclist_table(h, ptr::addr_of_mut!((*g).grayagain));
    } else if hasww != 0 {
        linkgclist_table(h, ptr::addr_of_mut!((*g).ephemeron));
    } else if hasclears != 0 {
        linkgclist_table(h, ptr::addr_of_mut!((*g).allweak));
    } else {
        genlink(g, obj2gco(h));
    }
    marked
}

unsafe fn traversestrongtable(g: *mut GlobalState, h: *mut Table) {
    let limit = gnodelast(h);
    traversearray(g, h);
    let mut n = gnode(h, 0);
    while n < limit {
        if isempty(gval(n)) {
            clearkey(n);
        } else {
            debug_assert!(!keyisnil(n));
            markkey(g, n);
            markvalue(g, gval(n));
        }
        n = n.add(1);
    }
    genlink(g, obj2gco(h));
}

unsafe fn gfasttm(g: *mut GlobalState, mt: *mut Table, e: usize) -> *const TValue {
    if checknoTM(mt, e) {
        ptr::null()
    } else {
        luaT_gettm(mt, e as c_int, (&mut (*g).tmname)[e])
    }
}

unsafe fn notm(tm: *const TValue) -> bool {
    ttisnil(tm)
}

unsafe fn getmode(g: *mut GlobalState, h: *mut Table) -> c_int {
    let mode = gfasttm(g, (*h).metatable, TM_MODE);
    if mode.is_null() || !ttisstring(mode) {
        0
    } else {
        let smode = getstr(tsvalue(mode));
        let weakkey = libc_strchr(smode, b'k' as c_int);
        let weakvalue = libc_strchr(smode, b'v' as c_int);
        ((if !weakkey.is_null() { 1 } else { 0 }) << 1) | if !weakvalue.is_null() { 1 } else { 0 }
    }
}

unsafe fn traversetable(g: *mut GlobalState, h: *mut Table) -> l_mem {
    markobjectN(g, (*h).metatable);
    match getmode(g, h) {
        0 => traversestrongtable(g, h),
        1 => traverseweakvalue(g, h),
        2 => {
            traverseephemeron(g, h, 0);
        }
        3 => {
            if (*g).gcstate == GCSpropagate {
                linkgclist_table(h, ptr::addr_of_mut!((*g).grayagain));
            } else {
                linkgclist_table(h, ptr::addr_of_mut!((*g).allweak));
            }
        }
        _ => {}
    }
    (1 + 2 * sizenode(h) + (*h).asize) as l_mem
}

unsafe fn traverseudata(g: *mut GlobalState, u: *mut Udata) -> l_mem {
    markobjectN(g, (*u).metatable);
    for i in 0..(*u).nuvalue as usize {
        let uv = ptr::addr_of_mut!((*(*u).uv.as_mut_ptr().add(i)).uv);
        markvalue(g, uv);
    }
    genlink(g, obj2gco(u));
    (1 + (*u).nuvalue as usize) as l_mem
}

unsafe fn traverseproto(g: *mut GlobalState, f: *mut Proto) -> l_mem {
    markobjectN(g, (*f).source);
    for i in 0..(*f).sizek.max(0) as usize {
        markvalue(g, (*f).k.add(i));
    }
    for i in 0..(*f).sizeupvalues.max(0) as usize {
        markobjectN(g, (*(*f).upvalues.add(i)).name);
    }
    for i in 0..(*f).sizep.max(0) as usize {
        markobjectN(g, *(*f).p.add(i));
    }
    let locvars = (*f).locvars.cast::<LocVar>();
    for i in 0..(*f).sizelocvars.max(0) as usize {
        markobjectN(g, (*locvars.add(i)).varname);
    }
    (1 + (*f).sizek + (*f).sizeupvalues + (*f).sizep + (*f).sizelocvars) as l_mem
}

unsafe fn traverseCclosure(g: *mut GlobalState, cl: *mut CClosure) -> l_mem {
    for i in 0..(*cl).nupvalues as usize {
        markvalue(g, (*cl).upvalue.as_mut_ptr().add(i));
    }
    (1 + (*cl).nupvalues as usize) as l_mem
}

unsafe fn traverseLclosure(g: *mut GlobalState, cl: *mut LClosure) -> l_mem {
    markobjectN(g, (*cl).p);
    for i in 0..(*cl).nupvalues as usize {
        markobjectN(g, *(*cl).upvals.as_mut_ptr().add(i));
    }
    (1 + (*cl).nupvalues as usize) as l_mem
}

unsafe fn traversethread(g: *mut GlobalState, th: *mut lua_State) -> l_mem {
    let mut o = (*th).stack.p;
    if isold(th.cast()) || (*g).gcstate == GCSpropagate {
        linkgclist_thread(th, ptr::addr_of_mut!((*g).grayagain));
    }
    if o.is_null() {
        return 0;
    }
    while o < (*th).top.p {
        markvalue(g, s2v(o));
        o = o.add(1);
    }
    let mut uv = (*th).openupval;
    while !uv.is_null() {
        markobject(g, uv);
        uv = (*uv).u.open.next;
    }
    if (*g).gcstate == GCSatomic {
        if (*g).gcemergency == 0 {
            luaD_shrinkstack(th);
        }
        let mut o2 = (*th).top.p;
        let limit = (*th).stack_last.p.add(EXTRA_STACK);
        while o2 < limit {
            setnilvalue(s2v(o2));
            o2 = o2.add(1);
        }
        if !isintwups(th) && !(*th).openupval.is_null() {
            (*th).twups = (*g).twups;
            (*g).twups = th;
        }
    }
    (1 + (*th).top.p.offset_from((*th).stack.p) as usize) as l_mem
}

unsafe fn propagatemark(g: *mut GlobalState) -> l_mem {
    let o = (*g).gray;
    nw2black(o);
    (*g).gray = *getgclist(o);
    match (*o).tt {
        LUA_VTABLE => traversetable(g, gco2t(o)),
        LUA_VUSERDATA => traverseudata(g, gco2u(o)),
        LUA_VLCL => traverseLclosure(g, gco2lcl(o)),
        LUA_VCCL => traverseCclosure(g, gco2ccl(o)),
        LUA_VPROTO => traverseproto(g, gco2p(o)),
        LUA_VTHREAD => traversethread(g, gco2th(o)),
        _ => 0,
    }
}

unsafe fn propagateall(g: *mut GlobalState) {
    while !(*g).gray.is_null() {
        propagatemark(g);
    }
}

unsafe fn convergeephemerons(g: *mut GlobalState) {
    let mut changed;
    let mut dir = 0;
    loop {
        let mut next = (*g).ephemeron;
        (*g).ephemeron = ptr::null_mut();
        changed = 0;
        while !next.is_null() {
            let w = next;
            let h = gco2t(w);
            next = (*h).gclist;
            nw2black(h.cast());
            if traverseephemeron(g, h, dir) != 0 {
                propagateall(g);
                changed = 1;
            }
        }
        dir = if dir == 0 { 1 } else { 0 };
        if changed == 0 {
            break;
        }
    }
}

unsafe fn clearbykeys(g: *mut GlobalState, mut l: *mut GCObject) {
    while !l.is_null() {
        let h = gco2t(l);
        let limit = gnodelast(h);
        let mut n = gnode(h, 0);
        while n < limit {
            if iscleared(g, gckeyN(n)) {
                setempty(gval(n));
            }
            if isempty(gval(n)) {
                clearkey(n);
            }
            n = n.add(1);
        }
        l = (*h).gclist;
    }
}

unsafe fn clearbyvalues(g: *mut GlobalState, mut l: *mut GCObject, f: *mut GCObject) {
    while l != f {
        let h = gco2t(l);
        for i in 0..(*h).asize {
            let o = gcvalarr(h, i);
            if iscleared(g, o) {
                *getArrTag(h, i) = LUA_VEMPTY;
            }
        }
        let limit = gnodelast(h);
        let mut n = gnode(h, 0);
        while n < limit {
            if iscleared(g, gcvalueN(gval(n))) {
                setempty(gval(n));
            }
            if isempty(gval(n)) {
                clearkey(n);
            }
            n = n.add(1);
        }
        l = (*h).gclist;
    }
}

unsafe fn freeupval(L: *mut lua_State, uv: *mut UpVal) {
    if upisopen(uv) {
        luaF_unlinkupval(uv);
    }
    luaM_free_(L, uv.cast(), size_of::<UpVal>());
}

unsafe fn freeobj(L: *mut lua_State, o: *mut GCObject) {
    match (*o).tt {
        LUA_VPROTO => luaF_freeproto(L, gco2p(o)),
        LUA_VUPVAL => freeupval(L, gco2upv(o)),
        LUA_VLCL => {
            let cl = gco2lcl(o);
            luaM_free_(L, cl.cast(), sizeLclosure((*cl).nupvalues));
        }
        LUA_VCCL => {
            let cl = gco2ccl(o);
            luaM_free_(L, cl.cast(), sizeCclosure((*cl).nupvalues));
        }
        LUA_VTABLE => luaH_free(L, gco2t(o)),
        LUA_VTHREAD => luaE_freethread(L, gco2th(o)),
        LUA_VUSERDATA => {
            let u = gco2u(o);
            luaM_free_(L, o.cast(), sizeudata((*u).nuvalue, (*u).len));
        }
        LUA_VSHRSTR => {
            let ts = gco2ts(o);
            luaS_remove(L, ts);
            luaM_free_(L, ts.cast(), sizestrshr((*ts).shrlen as u32));
        }
        LUA_VLNGSTR => {
            let ts = gco2ts(o);
            if (*ts).shrlen == LSTRMEM {
                if let Some(falloc) = (*ts).falloc {
                    falloc((*ts).ud, (*ts).contents.cast(), (*ts).u.lnglen + 1, 0);
                }
            }
            luaM_free_(
                L,
                ts.cast(),
                luaS_sizelngstr((*ts).u.lnglen, (*ts).shrlen as c_int),
            );
        }
        _ => panic!("invalid freeobj"),
    }
}

unsafe fn sweeplist(
    L: *mut lua_State,
    mut p: *mut *mut GCObject,
    mut countin: l_mem,
) -> *mut *mut GCObject {
    let g = G(L);
    let ow = otherwhite(g);
    let white = luaC_white(g);
    while !(*p).is_null() && countin > 0 {
        countin -= 1;
        let curr = *p;
        let marked = (*curr).marked;
        if isdeadm(ow, marked) {
            *p = (*curr).next;
            freeobj(L, curr);
        } else {
            (*curr).marked = (marked & !(((1 << BLACKBIT) | WHITEBITS) | AGEBITS)) | white | G_NEW;
            p = ptr::addr_of_mut!((*curr).next);
        }
    }
    if (*p).is_null() { ptr::null_mut() } else { p }
}

unsafe fn sweeptolive(L: *mut lua_State, mut p: *mut *mut GCObject) -> *mut *mut GCObject {
    loop {
        let old = p;
        p = sweeplist(L, p, 1);
        if p != old {
            return p;
        }
    }
}

unsafe fn checkSizes(L: *mut lua_State, g: *mut GlobalState) {
    if (*g).gcemergency == 0 && (*g).strt.nuse < (*g).strt.size / 4 {
        luaS_resize(L, (*g).strt.size / 2);
    }
}

unsafe fn udata2finalize(g: *mut GlobalState) -> *mut GCObject {
    let o = (*g).tobefnz;
    (*g).tobefnz = (*o).next;
    (*o).next = (*g).allgc;
    (*g).allgc = o;
    resetbit(&mut (*o).marked, FINALIZEDBIT);
    if issweepphase(g) {
        makewhite(g, o);
    } else if getage(o) == G_OLD1 {
        (*g).firstold1 = o;
    }
    o
}

unsafe fn dothecall(L: *mut lua_State, _ud: *mut c_void) {
    luaD_callnoyield(L, (*L).top.p.sub(2), 0);
}

unsafe fn setgcovalue(L: *mut lua_State, obj: *mut TValue, gcobj: *mut GCObject) {
    (*obj).value_.gc = gcobj;
    (*obj).tt_ = (*gcobj).tt | BIT_ISCOLLECTABLE;
    let _ = L;
}

unsafe fn GCTM(L: *mut lua_State) {
    let g = G(L);
    let mut v = TValue {
        value_: Value {
            gc: ptr::null_mut(),
        },
        tt_: 0,
    };
    setgcovalue(L, ptr::addr_of_mut!(v), udata2finalize(g));
    let tm = luaT_gettmbyobj(L, ptr::addr_of!(v), TM_GC as c_int);
    if !notm(tm) {
        let oldah = (*L).allowhook;
        let oldgcstp = (*g).gcstp;
        (*g).gcstp |= GCSTPGC;
        (*L).allowhook = 0;
        setobj2s(L, (*L).top.p, tm);
        (*L).top.p = (*L).top.p.add(1);
        setobj2s(L, (*L).top.p, ptr::addr_of!(v));
        (*L).top.p = (*L).top.p.add(1);
        (*(*L).ci).callstatus |= CIST_FIN;
        let status = luaD_pcall(
            L,
            Some(dothecall),
            ptr::null_mut(),
            savestack(L, (*L).top.p.sub(2)),
            0,
        );
        (*(*L).ci).callstatus &= !CIST_FIN;
        (*L).allowhook = oldah;
        (*g).gcstp = oldgcstp;
        if status != LUA_OK {
            luaE_warnerror(L, c"__gc".as_ptr());
            (*L).top.p = (*L).top.p.sub(1);
        }
    }
}

unsafe fn callallpendingfinalizers(L: *mut lua_State) {
    while !(*G(L)).tobefnz.is_null() {
        GCTM(L);
    }
}

unsafe fn findlast(mut p: *mut *mut GCObject) -> *mut *mut GCObject {
    while !(*p).is_null() {
        p = ptr::addr_of_mut!((**p).next);
    }
    p
}

unsafe fn separatetobefnz(g: *mut GlobalState, all: c_int) {
    let mut p = ptr::addr_of_mut!((*g).finobj);
    let mut lastnext = findlast(ptr::addr_of_mut!((*g).tobefnz));
    while *p != (*g).finobjold1 {
        let curr = *p;
        if !(iswhite(curr) || all != 0) {
            p = ptr::addr_of_mut!((*curr).next);
        } else {
            if curr == (*g).finobjsur {
                (*g).finobjsur = (*curr).next;
            }
            *p = (*curr).next;
            (*curr).next = *lastnext;
            *lastnext = curr;
            lastnext = ptr::addr_of_mut!((*curr).next);
        }
    }
}

unsafe fn checkpointer(p: *mut *mut GCObject, o: *mut GCObject) {
    if *p == o {
        *p = (*o).next;
    }
}

unsafe fn correctpointers(g: *mut GlobalState, o: *mut GCObject) {
    checkpointer(ptr::addr_of_mut!((*g).survival), o);
    checkpointer(ptr::addr_of_mut!((*g).old1), o);
    checkpointer(ptr::addr_of_mut!((*g).reallyold), o);
    checkpointer(ptr::addr_of_mut!((*g).firstold1), o);
}

pub(crate) unsafe fn luaC_checkfinalizer(L: *mut lua_State, o: *mut GCObject, mt: *mut Table) {
    let g = G(L);
    if tofinalize(o) || gfasttm(g, mt, TM_GC).is_null() || ((*g).gcstp & GCSTPCLS) != 0 {
        return;
    }
    if issweepphase(g) {
        makewhite(g, o);
        if ptr::eq((*g).sweepgc, ptr::addr_of_mut!((*o).next)) {
            (*g).sweepgc = sweeptolive(L, (*g).sweepgc);
        }
    } else {
        correctpointers(g, o);
    }
    let mut p = ptr::addr_of_mut!((*g).allgc);
    while *p != o {
        p = ptr::addr_of_mut!((**p).next);
    }
    *p = (*o).next;
    (*o).next = (*g).finobj;
    (*g).finobj = o;
    l_setbit(&mut (*o).marked, FINALIZEDBIT);
}

unsafe fn setpause(g: *mut GlobalState) {
    let threshold = luaO_applyparam((*g).gcparams[3], (*g).gcmarked);
    let mut debt = threshold - gettotalbytes(g);
    if debt < 0 {
        debt = 0;
    }
    luaE_setdebt(g, debt);
}

unsafe fn sweep2old(L: *mut lua_State, mut p: *mut *mut GCObject) {
    let g = G(L);
    while !(*p).is_null() {
        let curr = *p;
        if iswhite(curr) {
            *p = (*curr).next;
            freeobj(L, curr);
        } else {
            setage(curr, G_OLD);
            if (*curr).tt == LUA_VTHREAD {
                linkgclist_thread(gco2th(curr), ptr::addr_of_mut!((*g).grayagain));
            } else if (*curr).tt == LUA_VUPVAL && upisopen(gco2upv(curr)) {
                set2gray(curr);
            } else {
                nw2black(curr);
            }
            p = ptr::addr_of_mut!((*curr).next);
        }
    }
}

unsafe fn sweepgen(
    L: *mut lua_State,
    g: *mut GlobalState,
    mut p: *mut *mut GCObject,
    limit: *mut GCObject,
    pfirstold1: *mut *mut GCObject,
    paddedold: *mut l_mem,
) -> *mut *mut GCObject {
    const NEXTAGE: [u8; 7] = [
        G_SURVIVAL, G_OLD1, G_OLD1, G_OLD, G_OLD, G_TOUCHED1, G_TOUCHED2,
    ];
    let white = luaC_white(g);
    let mut addedold = 0;
    while *p != limit {
        let curr = *p;
        if iswhite(curr) {
            *p = (*curr).next;
            freeobj(L, curr);
        } else {
            let age = getage(curr);
            if age == G_NEW {
                let marked = (*curr).marked & !(((1 << BLACKBIT) | WHITEBITS) | AGEBITS);
                (*curr).marked = marked | G_SURVIVAL | white;
            } else {
                setage(curr, NEXTAGE[age as usize]);
                if getage(curr) == G_OLD1 {
                    addedold += objsize(curr);
                    if (*pfirstold1).is_null() {
                        *pfirstold1 = curr;
                    }
                }
            }
            p = ptr::addr_of_mut!((*curr).next);
        }
    }
    *paddedold += addedold;
    p
}

unsafe fn correctgraylist(mut p: *mut *mut GCObject) -> *mut *mut GCObject {
    while !(*p).is_null() {
        let curr = *p;
        let next = getgclist(curr);
        if iswhite(curr) {
            *p = *next;
        } else if getage(curr) == G_TOUCHED1 {
            nw2black(curr);
            setage(curr, G_TOUCHED2);
            p = next;
        } else if (*curr).tt == LUA_VTHREAD {
            p = next;
        } else {
            if getage(curr) == G_TOUCHED2 {
                setage(curr, G_OLD);
            }
            nw2black(curr);
            *p = *next;
        }
    }
    p
}

unsafe fn correctgraylists(g: *mut GlobalState) {
    let mut list = correctgraylist(ptr::addr_of_mut!((*g).grayagain));
    *list = (*g).weak;
    (*g).weak = ptr::null_mut();
    list = correctgraylist(list);
    *list = (*g).allweak;
    (*g).allweak = ptr::null_mut();
    list = correctgraylist(list);
    *list = (*g).ephemeron;
    (*g).ephemeron = ptr::null_mut();
    correctgraylist(list);
}

unsafe fn markold(g: *mut GlobalState, from: *mut GCObject, to: *mut GCObject) {
    let mut p = from;
    while p != to {
        if getage(p) == G_OLD1 {
            setage(p, G_OLD);
            if isblack(p) {
                reallymarkobject(g, p);
            }
        }
        p = (*p).next;
    }
}

unsafe fn finishgencycle(L: *mut lua_State, g: *mut GlobalState) {
    correctgraylists(g);
    checkSizes(L, g);
    (*g).gcstate = GCSpropagate;
    if (*g).gcemergency == 0 && luaD_checkminstack(L) != 0 {
        callallpendingfinalizers(L);
    }
}

unsafe fn minor2inc(L: *mut lua_State, g: *mut GlobalState, kind: u8) {
    (*g).gcmajorminor = (*g).gcmarked;
    (*g).gckind = kind;
    (*g).reallyold = ptr::null_mut();
    (*g).old1 = ptr::null_mut();
    (*g).survival = ptr::null_mut();
    (*g).finobjrold = ptr::null_mut();
    (*g).finobjold1 = ptr::null_mut();
    (*g).finobjsur = ptr::null_mut();
    entersweep(L);
    luaE_setdebt(g, luaO_applyparam((*g).gcparams[5], 100));
}

unsafe fn checkminormajor(g: *mut GlobalState) -> c_int {
    let limit = luaO_applyparam((*g).gcparams[2], (*g).gcmajorminor);
    if limit == 0 {
        return 0;
    }
    ((*g).gcmarked >= limit) as c_int
}

unsafe fn youngcollection(L: *mut lua_State, g: *mut GlobalState) {
    let mut addedold1 = 0;
    let marked = (*g).gcmarked;
    let mut dummy = ptr::null_mut();
    if !(*g).firstold1.is_null() {
        markold(g, (*g).firstold1, (*g).reallyold);
        (*g).firstold1 = ptr::null_mut();
    }
    markold(g, (*g).finobj, (*g).finobjrold);
    markold(g, (*g).tobefnz, ptr::null_mut());
    atomic(L);
    (*g).gcstate = GCSswpallgc;
    let psurvival = sweepgen(
        L,
        g,
        ptr::addr_of_mut!((*g).allgc),
        (*g).survival,
        ptr::addr_of_mut!((*g).firstold1),
        ptr::addr_of_mut!(addedold1),
    );
    sweepgen(
        L,
        g,
        psurvival,
        (*g).old1,
        ptr::addr_of_mut!((*g).firstold1),
        ptr::addr_of_mut!(addedold1),
    );
    (*g).reallyold = (*g).old1;
    (*g).old1 = *psurvival;
    (*g).survival = (*g).allgc;
    let psurvival2 = sweepgen(
        L,
        g,
        ptr::addr_of_mut!((*g).finobj),
        (*g).finobjsur,
        ptr::addr_of_mut!(dummy),
        ptr::addr_of_mut!(addedold1),
    );
    sweepgen(
        L,
        g,
        psurvival2,
        (*g).finobjold1,
        ptr::addr_of_mut!(dummy),
        ptr::addr_of_mut!(addedold1),
    );
    (*g).finobjrold = (*g).finobjold1;
    (*g).finobjold1 = *psurvival2;
    (*g).finobjsur = (*g).finobj;
    sweepgen(
        L,
        g,
        ptr::addr_of_mut!((*g).tobefnz),
        ptr::null_mut(),
        ptr::addr_of_mut!(dummy),
        ptr::addr_of_mut!(addedold1),
    );
    (*g).gcmarked = marked + addedold1;
    if checkminormajor(g) != 0 {
        minor2inc(L, g, KGC_GENMAJOR);
        (*g).gcmarked = 0;
    } else {
        finishgencycle(L, g);
    }
}

unsafe fn atomic2gen(L: *mut lua_State, g: *mut GlobalState) {
    cleargraylists(g);
    (*g).gcstate = GCSswpallgc;
    sweep2old(L, ptr::addr_of_mut!((*g).allgc));
    (*g).reallyold = (*g).allgc;
    (*g).old1 = (*g).allgc;
    (*g).survival = (*g).allgc;
    (*g).firstold1 = ptr::null_mut();
    sweep2old(L, ptr::addr_of_mut!((*g).finobj));
    (*g).finobjrold = (*g).finobj;
    (*g).finobjold1 = (*g).finobj;
    (*g).finobjsur = (*g).finobj;
    sweep2old(L, ptr::addr_of_mut!((*g).tobefnz));
    (*g).gckind = KGC_GENMINOR as u8;
    (*g).gcmajorminor = (*g).gcmarked;
    (*g).gcmarked = 0;
    finishgencycle(L, g);
}

unsafe fn setminordebt(g: *mut GlobalState) {
    luaE_setdebt(g, luaO_applyparam((*g).gcparams[0], (*g).gcmajorminor));
}

unsafe fn entergen(L: *mut lua_State, g: *mut GlobalState) {
    luaC_runtilstate(L, GCSpause as c_int, 1);
    luaC_runtilstate(L, GCSpropagate as c_int, 1);
    atomic(L);
    atomic2gen(L, g);
    setminordebt(g);
}

pub(crate) unsafe fn luaC_changemode(L: *mut lua_State, newmode: c_int) {
    let g = G(L);
    if (*g).gckind == KGC_GENMAJOR {
        (*g).gckind = KGC_INC;
    }
    if newmode as u8 != (*g).gckind {
        if newmode as u8 == KGC_INC {
            minor2inc(L, g, KGC_INC);
        } else {
            entergen(L, g);
        }
    }
}

unsafe fn fullgen(L: *mut lua_State, g: *mut GlobalState) {
    minor2inc(L, g, KGC_INC);
    entergen(L, g);
}

unsafe fn checkmajorminor(L: *mut lua_State, g: *mut GlobalState) -> c_int {
    if (*g).gckind == KGC_GENMAJOR {
        let numbytes = gettotalbytes(g);
        let addedbytes = numbytes - (*g).gcmajorminor;
        let limit = luaO_applyparam((*g).gcparams[1], addedbytes);
        let tobecollected = numbytes - (*g).gcmarked;
        if tobecollected > limit {
            atomic2gen(L, g);
            setminordebt(g);
            return 1;
        }
    }
    (*g).gcmajorminor = (*g).gcmarked;
    0
}

unsafe fn entersweep(L: *mut lua_State) {
    let g = G(L);
    (*g).gcstate = GCSswpallgc;
    (*g).sweepgc = sweeptolive(L, ptr::addr_of_mut!((*g).allgc));
}

unsafe fn deletelist(L: *mut lua_State, mut p: *mut GCObject, limit: *mut GCObject) {
    while p != limit {
        let next = (*p).next;
        freeobj(L, p);
        p = next;
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaC_freeallobjects(L: *mut lua_State) {
    let g = G(L);
    (*g).gcstp = GCSTPCLS;
    luaC_changemode(L, KGC_INC as c_int);
    separatetobefnz(g, 1);
    callallpendingfinalizers(L);
    deletelist(L, (*g).allgc, obj2gco(mainthread(g)));
    deletelist(L, (*g).fixedgc, ptr::null_mut());
}

unsafe fn atomic(L: *mut lua_State) {
    let g = G(L);
    let grayagain = (*g).grayagain;
    (*g).grayagain = ptr::null_mut();
    (*g).gcstate = GCSatomic;
    markobject(g, L);
    markvalue(g, ptr::addr_of_mut!((*g).l_registry));
    markmt(g);
    propagateall(g);
    remarkupvals(g);
    propagateall(g);
    (*g).gray = grayagain;
    propagateall(g);
    convergeephemerons(g);
    clearbyvalues(g, (*g).weak, ptr::null_mut());
    clearbyvalues(g, (*g).allweak, ptr::null_mut());
    let origweak = (*g).weak;
    let origall = (*g).allweak;
    separatetobefnz(g, 0);
    markbeingfnz(g);
    propagateall(g);
    convergeephemerons(g);
    clearbykeys(g, (*g).ephemeron);
    clearbykeys(g, (*g).allweak);
    clearbyvalues(g, (*g).weak, origweak);
    clearbyvalues(g, (*g).allweak, origall);
    luaS_clearcache(g);
    (*g).currentwhite = otherwhite(g);
}

unsafe fn sweepstep(
    L: *mut lua_State,
    g: *mut GlobalState,
    nextstate: u8,
    nextlist: *mut *mut GCObject,
    fast: c_int,
) {
    if !(*g).sweepgc.is_null() {
        (*g).sweepgc = sweeplist(
            L,
            (*g).sweepgc,
            if fast != 0 { MAX_LMEM } else { GCSWEEPMAX },
        );
    } else {
        (*g).gcstate = nextstate;
        (*g).sweepgc = nextlist;
    }
}

const step2pause: l_mem = -3;
const atomicstep: l_mem = -2;
const step2minor: l_mem = -1;

unsafe fn singlestep(L: *mut lua_State, fast: c_int) -> l_mem {
    let g = G(L);
    (*g).gcstopem = 1;
    let stepresult = match (*g).gcstate {
        GCSpause => {
            restartcollection(g);
            (*g).gcstate = GCSpropagate;
            1
        }
        GCSpropagate => {
            if fast != 0 || (*g).gray.is_null() {
                (*g).gcstate = GCSenteratomic;
                1
            } else {
                propagatemark(g)
            }
        }
        GCSenteratomic => {
            atomic(L);
            if checkmajorminor(L, g) != 0 {
                step2minor
            } else {
                entersweep(L);
                atomicstep
            }
        }
        GCSswpallgc => {
            sweepstep(L, g, GCSswpfinobj, ptr::addr_of_mut!((*g).finobj), fast);
            GCSWEEPMAX
        }
        GCSswpfinobj => {
            sweepstep(L, g, GCSswptobefnz, ptr::addr_of_mut!((*g).tobefnz), fast);
            GCSWEEPMAX
        }
        GCSswptobefnz => {
            sweepstep(L, g, GCSswpend, ptr::null_mut(), fast);
            GCSWEEPMAX
        }
        GCSswpend => {
            checkSizes(L, g);
            (*g).gcstate = GCScallfin;
            GCSWEEPMAX
        }
        GCScallfin => {
            if !(*g).tobefnz.is_null() && (*g).gcemergency == 0 && luaD_checkminstack(L) != 0 {
                (*g).gcstopem = 0;
                GCTM(L);
                CWUFIN
            } else {
                (*g).gcstate = GCSpause;
                step2pause
            }
        }
        _ => 0,
    };
    (*g).gcstopem = 0;
    stepresult
}

pub(crate) unsafe fn luaC_runtilstate(L: *mut lua_State, state: c_int, fast: c_int) {
    let g = G(L);
    while state as u8 != (*g).gcstate {
        singlestep(L, fast);
    }
}

unsafe fn incstep(L: *mut lua_State, g: *mut GlobalState) {
    let stepsize = luaO_applyparam((*g).gcparams[5], 100);
    let mut work2do = luaO_applyparam(
        (*g).gcparams[4],
        stepsize / size_of::<*const c_void>() as isize,
    );
    let fast = (work2do == 0) as c_int;
    loop {
        let stres = singlestep(L, fast);
        if stres == step2minor {
            return;
        } else if stres == step2pause || (stres == atomicstep && fast == 0) {
            break;
        } else {
            work2do -= stres;
        }
        if fast == 0 && work2do <= 0 {
            break;
        }
    }
    if (*g).gcstate == GCSpause {
        setpause(g);
    } else {
        luaE_setdebt(g, stepsize);
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaC_step(L: *mut lua_State) {
    let g = G(L);
    if !gcrunning(g) {
        if (*g).gcstp & GCSTPUSR != 0 {
            luaE_setdebt(g, 20000);
        }
    } else {
        match (*g).gckind {
            KGC_INC | KGC_GENMAJOR => incstep(L, g),
            x if x == KGC_GENMINOR as u8 => {
                youngcollection(L, g);
                setminordebt(g);
            }
            _ => {}
        }
    }
}

unsafe fn fullinc(L: *mut lua_State, g: *mut GlobalState) {
    if keepinvariant(g) {
        entersweep(L);
    }
    luaC_runtilstate(L, GCSpause as c_int, 1);
    luaC_runtilstate(L, GCScallfin as c_int, 1);
    luaC_runtilstate(L, GCSpause as c_int, 1);
    setpause(g);
}

#[unsafe(no_mangle)]
pub unsafe  fn luaC_fullgc(L: *mut lua_State, isemergency: c_int) {
    let g = G(L);
    (*g).gcemergency = isemergency as u8;
    match (*g).gckind {
        x if x == KGC_GENMINOR as u8 => fullgen(L, g),
        KGC_INC => fullinc(L, g),
        KGC_GENMAJOR => {
            (*g).gckind = KGC_INC;
            fullinc(L, g);
            (*g).gckind = KGC_GENMAJOR;
        }
        _ => {}
    }
    (*g).gcemergency = 0;
}

unsafe fn libc_strchr(s: *const c_char, c: c_int) -> *const c_char {
    let mut p = s;
    while !p.is_null() && *p != 0 {
        if *p as c_int == c {
            return p;
        }
        p = p.add(1);
    }
    ptr::null()
}
