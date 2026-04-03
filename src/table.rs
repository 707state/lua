use crate::api::{lua_callk, lua_checkstack, lua_compare, lua_geti, lua_getmetatable, lua_isstring, lua_rawget, lua_seti, lua_toboolean, lua_tolstring, lua_type};
use crate::aux_rs::{
    luaL_checkinteger, luaL_checktype, luaL_len, luaL_makeseed, luaL_optinteger, luaL_optlstring,
};
use crate::lua_module::{
    argcheck, create_library, lua_Integer, lua_Unsigned, lua_gettop, lua_pop, lua_pushinteger,
    lua_pushlstring, lua_pushnil, lua_pushstring, lua_pushvalue, lua_setfield, lua_settop,
    luaL_Reg,
};
use crate::runtime::Value;
use crate::runtime::*;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

static TABLE_OVERFLOW_ERR: &[u8] = b"table overflow\0";
static INVALID_NEXT_KEY_ERR: &[u8] = b"invalid key to 'next'\0";
static TABLE_INDEX_NIL_ERR: &[u8] = b"table index is nil\0";
static TABLE_INDEX_NAN_ERR: &[u8] = b"table index is NaN\0";

#[repr(C)]
struct DummyNodeSync(Node);
unsafe impl Sync for DummyNodeSync {}
#[repr(C)]
struct TValueSync(TValue);
unsafe impl Sync for TValueSync {}

static DUMMYNODE: DummyNodeSync = DummyNodeSync(Node {
    u: NodeKey {
        value_: Value {
            gc: ptr::null_mut(),
        },
        tt_: LUA_VEMPTY,
        key_tt: LUA_TDEADKEY,
        next: 0,
        key_val: Value {
            gc: ptr::null_mut(),
        },
    },
});

static ABSENTKEY: TValueSync = TValueSync(TValue {
    value_: Value {
        gc: ptr::null_mut(),
    },
    tt_: LUA_VABSTKEY,
});

#[repr(C)]
struct Counters {
    total: u32,
    na: u32,
    deleted: c_int,
    nums: [u32; (MAXABITS as usize) + 1],
}

#[inline] unsafe fn luaC_newobj(s: *mut lua_State, tt: u8, sz: usize) -> *mut GCObject { unsafe { crate::gc::luaC_newobj(s, tt, sz) } }
#[inline] unsafe fn luaC_barrierback_(s: *mut lua_State, o: *mut GCObject) { unsafe { crate::gc::luaC_barrierback_(s, o) } }
#[inline] unsafe fn luaD_throw(s: *mut lua_State, e: u8) -> ! { unsafe { crate::do_rs::luaD_throw(s, e) } }
/// 单字符串 runerror，避免变参
#[inline] unsafe fn luaG_runerror(s: *mut lua_State, msg: *const c_char) -> ! { unsafe { crate::debug::luaG_runerror1(s, msg) } }
#[inline] unsafe fn luaM_malloc_(s: *mut lua_State, sz: usize, t: c_int) -> *mut c_void { unsafe { crate::mem::luaM_malloc_(s, sz, t) } }
#[inline] unsafe fn luaM_realloc_(s: *mut lua_State, b: *mut c_void, os: usize, ns: usize) -> *mut c_void { unsafe { crate::mem::luaM_realloc_(s, b, os, ns) } }
#[inline] unsafe fn luaM_free_(s: *mut lua_State, b: *mut c_void, os: usize) { unsafe { crate::mem::luaM_free_(s, b, os) } }
#[inline] unsafe fn luaO_ceillog2(x: u32) -> u8 { unsafe { crate::object::luaO_ceillog2(x) }}
#[inline] unsafe fn luaS_eqstr(a: *mut TString, b: *mut TString) -> c_int { unsafe { crate::string::luaS_eqstr(a, b) } }
#[inline] unsafe fn luaS_hashlongstr(ts: *mut TString) -> u32 { unsafe { crate::string::luaS_hashlongstr(ts) } }
#[inline] unsafe fn luaS_normstr(s: *mut lua_State, ts: *mut TString) -> *mut TString { unsafe { crate::string::luaS_normstr(s, ts) } }
#[inline] unsafe fn luaV_flttointeger(n: lua_Number, p: *mut lua_Integer, m: c_int) -> c_int { unsafe { crate::vm_rs::luaV_flttointeger(n, p, m) } }

#[inline]
fn ctb(tag: u8) -> u8 {
    tag | BIT_ISCOLLECTABLE
}

#[inline]
unsafe fn getArrTag(t: *mut Table, k: u32) -> *mut u8 {
    unsafe { (*t).array.cast::<u8>().add(size_of::<u32>() + k as usize) }
}

#[inline]
unsafe fn lenhint(t: *mut Table) -> *mut u32 {
    unsafe { (*t).array.cast() }
}

#[inline]
unsafe fn farr2val(t: *mut Table, k: u32, tag: u8, res: *mut TValue) {
    unsafe {
        (*res).tt_ = tag;
        (*res).value_ = *getArrVal(t, k);
    }
}

#[inline]
unsafe fn obj2arr(t: *mut Table, k: u32, value: *const TValue) {
    unsafe {
        *getArrTag(t, k) = (*value).tt_;
        *getArrVal(t, k) = (*value).value_;
    }
}

#[inline]
unsafe fn fval2arr(t: *mut Table, k: u32, tag: *mut u8, value: *const TValue) {
    unsafe {
        *tag = (*value).tt_;
        *getArrVal(t, k) = (*value).value_;
    }
}

#[inline]
unsafe fn gnode(t: *mut Table, i: u32) -> *mut Node {
    unsafe { (*t).node.add(i as usize) }
}

#[inline]
unsafe fn gval(n: *mut Node) -> *mut TValue {
    unsafe { ptr::addr_of_mut!((*n).i_val) }
}

#[inline]
unsafe fn gnext(n: *mut Node) -> c_int {
    unsafe { (*n).u.next }
}

#[inline]
unsafe fn set_gnext(n: *mut Node, next: c_int) {
    unsafe { (*n).u.next = next };
}

#[inline]
unsafe fn keytt(n: *mut Node) -> u8 {
    unsafe { (*n).u.key_tt }
}

#[inline]
unsafe fn set_keytt(n: *mut Node, tt: u8) {
    unsafe { (*n).u.key_tt = tt };
}

#[inline]
unsafe fn keyval(n: *mut Node) -> Value {
    unsafe { (*n).u.key_val }
}

#[inline]
unsafe fn keyisnil(n: *mut Node) -> bool {
    unsafe { keytt(n) == LUA_TNIL }
}

#[inline]
unsafe fn keyisinteger(n: *mut Node) -> bool {
    unsafe { keytt(n) == LUA_VNUMINT }
}

#[inline]
unsafe fn keyival(n: *mut Node) -> lua_Integer {
    unsafe { (*n).u.key_val.i }
}

#[inline]
unsafe fn keyisshrstr(n: *mut Node) -> bool {
    unsafe { keytt(n) == ctb(LUA_VSHRSTR) }
}

#[inline]
unsafe fn keystrval(n: *mut Node) -> *mut TString {
    unsafe { (*n).u.key_val.gc.cast() }
}

#[inline]
unsafe fn keyisdead(n: *mut Node) -> bool {
    unsafe { keytt(n) == LUA_TDEADKEY }
}

#[inline]
unsafe fn keyiscollectable(n: *mut Node) -> bool {
    unsafe { (keytt(n) & BIT_ISCOLLECTABLE) != 0 }
}

#[inline]
unsafe fn gckey(n: *mut Node) -> *mut GCObject {
    unsafe { (*n).u.key_val.gc }
}

#[inline]
unsafe fn setnilkey(node: *mut Node) {
    unsafe { set_keytt(node, LUA_TNIL) };
}

#[inline]
unsafe fn setdeadkey(node: *mut Node) {
    unsafe { set_keytt(node, LUA_TDEADKEY) };
}

#[inline]
unsafe fn setnodekey(node: *mut Node, obj: *const TValue) {
    unsafe {
        (*node).u.key_val = (*obj).value_;
        (*node).u.key_tt = (*obj).tt_;
    }
}

#[inline]
unsafe fn getnodekey(obj: *mut TValue, node: *mut Node) {
    unsafe {
        (*obj).value_ = (*node).u.key_val;
        (*obj).tt_ = (*node).u.key_tt;
    }
}

#[inline]
unsafe fn isextstr(value: *const TValue) -> bool {
    unsafe { ttislngstring(value) && (*tsvalue(value)).shrlen != LSTRREG }
}

#[inline]
unsafe fn isempty(value: *const TValue) -> bool {
    unsafe { ttisnil(value) }
}

#[inline]
unsafe fn isabstkey(value: *const TValue) -> bool {
    unsafe { rawtt(value) == LUA_VABSTKEY }
}

#[inline]
unsafe fn iswhite_gc(obj: *mut GCObject) -> bool {
    unsafe { ((*obj).marked & WHITEBITS) != 0 }
}

#[inline]
unsafe fn isblack_table(t: *mut Table) -> bool {
    unsafe { ((*t).marked & (1 << BLACKBIT)) != 0 }
}

#[inline]
unsafe fn iswhite_value(value: *const TValue) -> bool {
    unsafe { iscollectable(value) && iswhite_gc(gcvalue(value)) }
}

#[inline]
unsafe fn luaC_barrierback(state: *mut lua_State, table: *mut Table, value: *const TValue) {
    if unsafe { isblack_table(table) && iswhite_value(value) } {
        unsafe { luaC_barrierback_(state, table.cast()) };
    }
}

#[inline]
unsafe fn luaM_error(state: *mut lua_State) -> ! {
    unsafe { luaD_throw(state, LUA_ERRMEM) }
}

#[inline]
unsafe fn luaM_newvector<T>(state: *mut lua_State, n: usize) -> *mut T {
    unsafe { luaM_malloc_(state, n.saturating_mul(size_of::<T>()), 0).cast() }
}

#[inline]
unsafe fn luaM_newblock(state: *mut lua_State, size: usize) -> *mut c_void {
    unsafe { luaM_malloc_(state, size, 0) }
}

#[inline]
unsafe fn luaM_freemem(state: *mut lua_State, block: *mut c_void, size: usize) {
    unsafe { luaM_free_(state, block, size) };
}

#[inline]
unsafe fn luaM_freearray<T>(state: *mut lua_State, block: *mut T, n: usize) {
    unsafe { luaM_free_(state, block.cast(), n.saturating_mul(size_of::<T>())) };
}

#[inline]
unsafe fn isdummy(t: *mut Table) -> bool {
    unsafe { ((*t).flags & BITDUMMY) != 0 }
}

#[inline]
unsafe fn setnodummy(t: *mut Table) {
    unsafe { (*t).flags &= NOTBITDUMMY };
}

#[inline]
unsafe fn setdummy(t: *mut Table) {
    unsafe { (*t).flags |= BITDUMMY };
}

#[inline]
unsafe fn sizenode(t: *mut Table) -> u32 {
    1u32 << unsafe { (*t).lsizenode }
}

#[inline]
unsafe fn allocsizenode(t: *mut Table) -> u32 {
    if unsafe { isdummy(t) } {
        0
    } else {
        unsafe { sizenode(t) }
    }
}

#[inline]
fn lmod(s: u32, size: u32) -> u32 {
    s & (size - 1)
}

#[inline]
fn point2uint<T>(p: *const T) -> u32 {
    ((p as usize) & u32::MAX as usize) as u32
}

#[inline]
unsafe fn hashpow2(t: *mut Table, n: u32) -> *mut Node {
    unsafe { gnode(t, lmod(n, sizenode(t))) }
}

#[inline]
unsafe fn hashmod(t: *mut Table, n: u32) -> *mut Node {
    unsafe { gnode(t, n % ((sizenode(t) - 1) | 1)) }
}

#[inline]
unsafe fn hashstr(t: *mut Table, ts: *mut TString) -> *mut Node {
    unsafe { hashpow2(t, (*ts).hash) }
}

#[inline]
unsafe fn hashboolean(t: *mut Table, b: u32) -> *mut Node {
    unsafe { hashpow2(t, b) }
}

#[inline]
unsafe fn hashpointer<T>(t: *mut Table, p: *const T) -> *mut Node {
    unsafe { hashmod(t, point2uint(p)) }
}

unsafe fn hashint(t: *mut Table, i: lua_Integer) -> *mut Node {
    let ui = i as lua_Unsigned;
    if ui <= i32::MAX as lua_Unsigned {
        unsafe { gnode(t, (ui as u32) % ((sizenode(t) - 1) | 1)) }
    } else {
        unsafe { hashmod(t, ui as u32) }
    }
}

unsafe fn l_hashfloat(n: lua_Number) -> u32 {
    let bits = n.to_bits();
    (bits ^ (bits >> 32)) as u32
}

unsafe fn mainpositionTV(t: *mut Table, key: *const TValue) -> *mut Node {
    match unsafe { ttypetag(key) } {
        LUA_VNUMINT => unsafe { hashint(t, ivalue(key)) },
        LUA_VNUMFLT => unsafe { hashmod(t, l_hashfloat(fltvalue(key))) },
        LUA_VSHRSTR => unsafe { hashstr(t, tsvalue(key)) },
        LUA_VLNGSTR => unsafe { hashpow2(t, luaS_hashlongstr(tsvalue(key))) },
        LUA_VFALSE => unsafe { hashboolean(t, 0) },
        LUA_VTRUE => unsafe { hashboolean(t, 1) },
        LUA_VLIGHTUSERDATA => unsafe { hashpointer(t, pvalue(key)) },
        LUA_VLCF => unsafe {
            let f = fvalue(key).map(|fp| fp as *const ()).unwrap_or(ptr::null());
            hashpointer(t, f)
        },
        _ => unsafe { hashpointer(t, gcvalue(key)) },
    }
}

unsafe fn mainpositionfromnode(t: *mut Table, nd: *mut Node) -> *mut Node {
    let mut key = TValue {
        value_: Value {
            gc: ptr::null_mut(),
        },
        tt_: LUA_VNIL,
    };
    unsafe { getnodekey(&mut key, nd) };
    unsafe { mainpositionTV(t, &key) }
}

unsafe fn equalkey(k1: *const TValue, n2: *mut Node, deadok: bool) -> bool {
    if unsafe { rawtt(k1) } != unsafe { keytt(n2) } {
        if unsafe { keyisshrstr(n2) } && unsafe { ttislngstring(k1) } {
            unsafe { luaS_eqstr(tsvalue(k1), keystrval(n2)) != 0 }
        } else if deadok && unsafe { keyisdead(n2) } && unsafe { iscollectable(k1) } {
            unsafe { gcvalue(k1) == gckey(n2) }
        } else {
            false
        }
    } else {
        match unsafe { keytt(n2) } {
            LUA_VNIL | LUA_VFALSE | LUA_VTRUE => true,
            LUA_VNUMINT => unsafe { ivalue(k1) == keyival(n2) },
            LUA_VNUMFLT => unsafe { fltvalue(k1) == keyval(n2).n },
            LUA_VLIGHTUSERDATA => unsafe { pvalue(k1) == keyval(n2).p },
            LUA_VLCF => unsafe { fvalue(k1) == keyval(n2).f },
            x if x == ctb(LUA_VLNGSTR) => unsafe { luaS_eqstr(tsvalue(k1), keystrval(n2)) != 0 },
            _ => unsafe { gcvalue(k1) == keyval(n2).gc },
        }
    }
}

unsafe fn getgeneric(t: *mut Table, key: *const TValue, deadok: bool) -> *const TValue {
    let mut n = unsafe { mainpositionTV(t, key) };
    loop {
        if unsafe { equalkey(key, n, deadok) } {
            return unsafe { gval(n) };
        }
        let nx = unsafe { gnext(n) };
        if nx == 0 {
            return &ABSENTKEY.0;
        }
        n = unsafe { n.add(nx as usize) };
    }
}

#[inline]
fn checkrange(k: lua_Integer, limit: u32) -> u32 {
    let uk = k as lua_Unsigned;
    if uk.wrapping_sub(1) < limit as lua_Unsigned {
        uk as u32
    } else {
        0
    }
}

#[inline]
fn arrayindex(k: lua_Integer) -> u32 {
    checkrange(k, MAXASIZE)
}

#[inline]
unsafe fn ikeyinarray(t: *mut Table, k: lua_Integer) -> u32 {
    unsafe { checkrange(k, (*t).asize) }
}

#[inline]
unsafe fn keyinarray(t: *mut Table, key: *const TValue) -> u32 {
    if unsafe { ttisinteger(key) } {
        unsafe { ikeyinarray(t, ivalue(key)) }
    } else {
        0
    }
}

unsafe fn findindex(state: *mut lua_State, t: *mut Table, key: *mut TValue, asize: u32) -> u32 {
    if unsafe { ttisnil(key) } {
        return 0;
    }
    let i = unsafe { keyinarray(t, key) };
    if i != 0 {
        i
    } else {
        let n = unsafe { getgeneric(t, key, true) };
        if unsafe { isabstkey(n) } {
            unsafe { luaG_runerror(state, INVALID_NEXT_KEY_ERR.as_ptr().cast()) };
        }
        let base = unsafe { (*t).node } as usize;
        let idx = (n as usize - base) / size_of::<Node>();
        idx as u32 + 1 + asize
    }
}

unsafe fn sizehash(t: *mut Table) -> usize {
    unsafe { sizenode(t) as usize * size_of::<Node>() }
}

unsafe fn freehash(state: *mut lua_State, t: *mut Table) {
    if !unsafe { isdummy(t) } {
        unsafe { luaM_freearray(state, (*t).node, sizenode(t) as usize) };
    }
}

#[inline]
fn arrayXhash(na: u32, nh: u32) -> bool {
    (na as usize) <= (nh as usize).saturating_mul(3)
}

unsafe fn countint(key: lua_Integer, ct: &mut Counters) {
    let k = arrayindex(key);
    if k != 0 {
        ct.nums[unsafe { luaO_ceillog2(k) } as usize] += 1;
        ct.na += 1;
    }
}

unsafe fn arraykeyisempty(t: *mut Table, key: u32) -> bool { unsafe {
    let tag = *getArrTag(t, key - 1);
    tagisempty(tag)
}}

unsafe fn numusearray(t: *mut Table, ct: &mut Counters) {
    let mut ause = 0u32;
    let mut i = 1u32;
    let asize = unsafe { (*t).asize };
    let mut ttlg = 1u32;
    for lg in 0..=MAXABITS as usize {
        let mut lc = 0u32;
        let mut lim = ttlg;
        if lim > asize {
            lim = asize;
            if i > lim {
                break;
            }
        }
        while i <= lim {
            if !unsafe { arraykeyisempty(t, i) } {
                lc += 1;
            }
            i += 1;
        }
        ct.nums[lg] += lc;
        ause += lc;
        ttlg = ttlg.wrapping_mul(2);
    }
    ct.total += ause;
    ct.na += ause;
}

unsafe fn numusehash(t: *mut Table, ct: &mut Counters) {
    let mut total = 0u32;
    let mut i = unsafe { sizenode(t) };
    while i > 0 {
        i -= 1;
        let n = unsafe { gnode(t, i) };
        if unsafe { isempty(gval(n)) } {
            ct.deleted = 1;
        } else {
            total += 1;
            if unsafe { keyisinteger(n) } {
                unsafe { countint(keyival(n), ct) };
            }
        }
    }
    ct.total += total;
}

unsafe fn computesizes(ct: &mut Counters) -> u32 {
    let mut a = 0u32;
    let mut na = 0u32;
    let mut optimal = 0u32;
    let mut twotoi = 1u32;
    let mut i = 0usize;
    while twotoi > 0 && arrayXhash(twotoi, ct.na) {
        let nums = ct.nums[i];
        a += nums;
        if nums > 0 && arrayXhash(twotoi, a) {
            optimal = twotoi;
            na = a;
        }
        i += 1;
        if i >= ct.nums.len() {
            break;
        }
        twotoi = twotoi.wrapping_mul(2);
    }
    ct.na = na;
    optimal
}

#[inline]
fn concretesize(size: u32) -> usize {
    if size == 0 {
        0
    } else {
        size as usize * (size_of::<Value>() + 1) + size_of::<u32>()
    }
}

unsafe fn resizearray(
    state: *mut lua_State,
    t: *mut Table,
    oldasize: u32,
    newasize: u32,
) -> *mut Value {
    if oldasize == newasize {
        return unsafe { (*t).array };
    }
    if newasize == 0 {
        let op = unsafe { (*t).array.sub(oldasize as usize) };
        unsafe { luaM_freemem(state, op.cast(), concretesize(oldasize)) };
        return ptr::null_mut();
    }

    let newasizeb = concretesize(newasize);
    let np = unsafe { luaM_realloc_(state, ptr::null_mut(), 0, newasizeb).cast::<Value>() };
    if np.is_null() {
        return ptr::null_mut();
    }
    let np = unsafe { np.add(newasize as usize) };
    if oldasize > 0 {
        let op = unsafe { (*t).array };
        let tomove = oldasize.min(newasize);
        let tomoveb = if oldasize < newasize {
            concretesize(oldasize)
        } else {
            newasizeb
        };
        unsafe {
            ptr::copy_nonoverlapping(
                op.sub(tomove as usize).cast::<u8>(),
                np.sub(tomove as usize).cast::<u8>(),
                tomoveb,
            );
            luaM_freemem(
                state,
                op.sub(oldasize as usize).cast(),
                concretesize(oldasize),
            );
        }
    }
    np
}

unsafe fn setnodevector(state: *mut lua_State, t: *mut Table, mut size: u32) {
    if size == 0 {
        unsafe {
            (*t).node = ptr::addr_of!(DUMMYNODE.0).cast_mut();
            (*t).lsizenode = 0;
            setdummy(t);
        }
        return;
    }
    let lsize = unsafe { luaO_ceillog2(size) } as u32;
    if lsize > MAXHBITS || (1u32 << lsize) > MAXHSIZE {
        unsafe { luaG_runerror(state, TABLE_OVERFLOW_ERR.as_ptr().cast()) };
    }
    size = 1u32 << lsize;
    let node = unsafe { luaM_newvector::<Node>(state, size as usize) };
    if node.is_null() {
        unsafe { luaM_error(state) };
    }
    unsafe {
        (*t).node = node;
        (*t).lsizenode = lsize as u8;
        setnodummy(t);
    }
    for i in 0..size {
        let n = unsafe { gnode(t, i) };
        unsafe {
            set_gnext(n, 0);
            setnilkey(n);
            settt(gval(n), LUA_VEMPTY);
            (*gval(n)).value_.gc = ptr::null_mut();
        }
    }
}

unsafe fn reinserthash(_state: *mut lua_State, ot: *mut Table, t: *mut Table) {
    for j in 0..unsafe { sizenode(ot) } {
        let old = unsafe { gnode(ot, j) };
        if !unsafe { isempty(gval(old)) } {
            let mut k = TValue {
                value_: Value {
                    gc: ptr::null_mut(),
                },
                tt_: LUA_VNIL,
            };
            unsafe { getnodekey(&mut k, old) };
            unsafe { newcheckedkey(t, &k, gval(old)) };
        }
    }
}

unsafe fn exchangehashpart(t1: *mut Table, t2: *mut Table) {
    unsafe {
        core::mem::swap(&mut (*t1).lsizenode, &mut (*t2).lsizenode);
        core::mem::swap(&mut (*t1).node, &mut (*t2).node);
        let bitdummy1 = (*t1).flags & BITDUMMY;
        (*t1).flags = ((*t1).flags & NOTBITDUMMY) | ((*t2).flags & BITDUMMY);
        (*t2).flags = ((*t2).flags & NOTBITDUMMY) | bitdummy1;
    }
}

unsafe fn reinsertOldSlice(t: *mut Table, oldasize: u32, newasize: u32) { unsafe {
    for i in newasize..oldasize {
        let tag = *getArrTag(t, i);
        if !tagisempty(tag) {
            let mut key = TValue {
                value_: Value {
                    gc: ptr::null_mut(),
                },
                tt_: LUA_VNIL,
            };
            let mut aux = TValue {
                value_: Value {
                    gc: ptr::null_mut(),
                },
                tt_: LUA_VNIL,
            };
                setivalue(&mut key, i as lua_Integer + 1);
                farr2val(t, i, tag, &mut aux);
                insertkey(t, &key, &mut aux);
        }
    }
}}

unsafe fn clearNewSlice(t: *mut Table, mut oldasize: u32, newasize: u32) {
    while oldasize < newasize {
        unsafe { *getArrTag(t, oldasize) = LUA_VEMPTY };
        oldasize += 1;
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_resize(
    state: *mut lua_State,
    t: *mut Table,
    newasize: u32,
    nhsize: u32,
) {
    let mut newt = Table {
        next: ptr::null_mut(),
        tt: 0,
        marked: 0,
        flags: 0,
        lsizenode: 0,
        asize: 0,
        array: ptr::null_mut(),
        node: ptr::null_mut(),
        metatable: ptr::null_mut(),
        gclist: ptr::null_mut(),
    };
    let oldasize = unsafe { (*t).asize };
    if newasize > MAXASIZE {
        unsafe { luaG_runerror(state, TABLE_OVERFLOW_ERR.as_ptr().cast()) };
    }
    unsafe { setnodevector(state, &mut newt, nhsize) };
    if newasize < oldasize {
        unsafe {
            exchangehashpart(t, &mut newt);
            reinsertOldSlice(t, oldasize, newasize);
            exchangehashpart(t, &mut newt);
        }
    }
    let newarray = unsafe { resizearray(state, t, oldasize, newasize) };
    if newarray.is_null() && newasize > 0 {
        unsafe {
            freehash(state, &mut newt);
            luaM_error(state);
        }
    }
    unsafe {
        exchangehashpart(t, &mut newt);
        (*t).array = newarray;
        (*t).asize = newasize;
        if !newarray.is_null() {
            *lenhint(t) = newasize / 2;
        }
        clearNewSlice(t, oldasize, newasize);
        reinserthash(state, &mut newt, t);
        freehash(state, &mut newt);
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_resizearray(
    state: *mut lua_State,
    t: *mut Table,
    nasize: u32,
) {
    let nsize = unsafe { allocsizenode(t) };
    unsafe { luaH_resize(state, t, nasize, nsize) };
}

unsafe fn rehash(state: *mut lua_State, t: *mut Table, ek: *const TValue) {
    let mut ct = Counters {
        total: 1,
        na: 0,
        deleted: 0,
        nums: [0; (MAXABITS as usize) + 1],
    };
    if unsafe { ttisinteger(ek) } {
        unsafe { countint(ivalue(ek), &mut ct) };
    }
    unsafe { numusehash(t, &mut ct) };
    let asize = if ct.na == 0 {
        unsafe { (*t).asize }
    } else {
        unsafe { numusearray(t, &mut ct) };
        unsafe { computesizes(&mut ct) }
    };
    let mut nsize = ct.total - ct.na;
    if ct.deleted != 0 {
        nsize += nsize >> 2;
    }
    unsafe { luaH_resize(state, t, asize, nsize) };
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_new(state: *mut lua_State) -> *mut Table {
    let o = unsafe { luaC_newobj(state, LUA_VTABLE, size_of::<Table>()) };
    let t = o.cast::<Table>();
    unsafe {
        (*t).metatable = ptr::null_mut();
        (*t).flags = MASKFLAGS;
        (*t).array = ptr::null_mut();
        (*t).asize = 0;
        setnodevector(state, t, 0);
    }
    t
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_size(t: *mut Table) -> usize {
    let mut sz = size_of::<Table>() + concretesize(unsafe { (*t).asize });
    if !unsafe { isdummy(t) } {
        sz += unsafe { sizehash(t) };
    }
    sz
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_free(state: *mut lua_State, t: *mut Table) {
    unsafe {
        freehash(state, t);
        resizearray(state, t, (*t).asize, 0);
        luaM_free_(state, t.cast(), size_of::<Table>());
    }
}

unsafe fn getfreepos(t: *mut Table) -> *mut Node {
    let mut i = unsafe { sizenode(t) };
    while i > 0 {
        i -= 1;
        let free = unsafe { gnode(t, i) };
        if unsafe { keyisnil(free) } {
            return free;
        }
    }
    ptr::null_mut()
}

unsafe fn insertkey(t: *mut Table, key: *const TValue, value: *mut TValue) -> bool {
    let mut mp = unsafe { mainpositionTV(t, key) };
    if !unsafe { isempty(gval(mp)) } || unsafe { isdummy(t) } {
        let f = unsafe { getfreepos(t) };
        if f.is_null() {
            return false;
        }
        let othern = unsafe { mainpositionfromnode(t, mp) };
        if othern != mp {
            let mut prev = othern;
            while unsafe { prev.add(gnext(prev) as usize) } != mp {
                prev = unsafe { prev.add(gnext(prev) as usize) };
            }
            unsafe {
                set_gnext(prev, f.offset_from(prev) as c_int);
                *f = ptr::read(mp);
                if gnext(mp) != 0 {
                    set_gnext(f, gnext(f) + mp.offset_from(f) as c_int);
                    set_gnext(mp, 0);
                }
                settt(gval(mp), LUA_VEMPTY);
            }
        } else {
            unsafe {
                if gnext(mp) != 0 {
                    set_gnext(f, mp.add(gnext(mp) as usize).offset_from(f) as c_int);
                } else {
                    set_gnext(f, 0);
                }
                set_gnext(mp, f.offset_from(mp) as c_int);
            }
            mp = f;
        }
    }
    unsafe {
        setnodekey(mp, key);
        setobj(gval(mp), value);
    }
    true
}

unsafe fn newcheckedkey(t: *mut Table, key: *const TValue, value: *mut TValue) {
    let i = unsafe { keyinarray(t, key) };
    if i > 0 {
        unsafe { obj2arr(t, i - 1, value) };
    } else {
        let done = unsafe { insertkey(t, key, value) };
        debug_assert!(done);
    }
}

unsafe fn luaH_newkey(
    state: *mut lua_State,
    t: *mut Table,
    key: *const TValue,
    value: *mut TValue,
) {
    if !unsafe { ttisnil(value) } {
        if !unsafe { insertkey(t, key, value) } {
            unsafe {
                rehash(state, t, key);
                newcheckedkey(t, key, value);
            }
        }
        unsafe { luaC_barrierback(state, t, key) };
    }
}

unsafe fn getintfromhash(t: *mut Table, key: lua_Integer) -> *const TValue {
    let mut n = unsafe { hashint(t, key) };
    loop {
        if unsafe { keyisinteger(n) && keyival(n) == key } {
            return unsafe { gval(n) };
        }
        let nx = unsafe { gnext(n) };
        if nx == 0 {
            break;
        }
        n = unsafe { n.add(nx as usize) };
    }
    &ABSENTKEY.0
}

unsafe fn hashkeyisempty(t: *mut Table, key: lua_Unsigned) -> bool {
    let val = unsafe { getintfromhash(t, key as lua_Integer) };
    unsafe { isempty(val) }
}

unsafe fn finishnodeget(val: *const TValue, res: *mut TValue) -> u8 {
    if !unsafe { ttisnil(val) } {
        unsafe { setobj(res, val) };
    }
    unsafe { ttypetag(val) }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_getint(
    t: *mut Table,
    key: lua_Integer,
    res: *mut TValue,
) -> u8 { unsafe {
    let k = ikeyinarray(t, key);
    if k > 0 {
        let tag = *getArrTag(t, k - 1);
        if !tagisempty(tag) {
            farr2val(t, k - 1, tag, res);
        }
        tag
    } else {
        finishnodeget(getintfromhash(t, key), res)
    }
}}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_Hgetshortstr(
    t: *mut Table,
    key: *mut TString,
) -> *const TValue {
    let mut n = unsafe { hashstr(t, key) };
    loop {
        if unsafe { keyisshrstr(n) } && unsafe { keystrval(n) == key } {
            return unsafe { gval(n) };
        }
        let nx = unsafe { gnext(n) };
        if nx == 0 {
            return &ABSENTKEY.0;
        }
        n = unsafe { n.add(nx as usize) };
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_getshortstr(
    t: *mut Table,
    key: *mut TString,
    res: *mut TValue,
) -> u8 {
    unsafe { finishnodeget(luaH_Hgetshortstr(t, key), res) }
}

unsafe fn Hgetlongstr(t: *mut Table, key: *mut TString) -> *const TValue {
    let mut ko = TValue {
        value_: Value {
            gc: ptr::null_mut(),
        },
        tt_: LUA_VNIL,
    };
    unsafe { setsvalue(&mut ko, key) };
    unsafe { getgeneric(t, &ko, false) }
}

unsafe fn Hgetstr(t: *mut Table, key: *mut TString) -> *const TValue {
    if unsafe { strisshr(key) } {
        unsafe { luaH_Hgetshortstr(t, key) }
    } else {
        unsafe { Hgetlongstr(t, key) }
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_getstr(
    t: *mut Table,
    key: *mut TString,
    res: *mut TValue,
) -> u8 {
    unsafe { finishnodeget(Hgetstr(t, key), res) }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_get(
    t: *mut Table,
    key: *const TValue,
    res: *mut TValue,
) -> u8 {
    let slot = match unsafe { ttypetag(key) } {
        LUA_VSHRSTR => unsafe { luaH_Hgetshortstr(t, tsvalue(key)) },
        LUA_VNUMINT => return unsafe { luaH_getint(t, ivalue(key), res) },
        LUA_VNIL => &ABSENTKEY.0,
        LUA_VNUMFLT => {
            let mut k = 0;
            if unsafe { luaV_flttointeger(fltvalue(key), &mut k, F2IEQ) } != 0 {
                return unsafe { luaH_getint(t, k, res) };
            }
            unsafe { getgeneric(t, key, false) }
        }
        _ => unsafe { getgeneric(t, key, false) },
    };
    unsafe { finishnodeget(slot, res) }
}

unsafe fn retpsetcode(t: *mut Table, slot: *const TValue) -> c_int {
    if unsafe { isabstkey(slot) } {
        HNOTFOUND
    } else {
        let idx = (slot as usize - unsafe { (*t).node } as usize) / size_of::<Node>();
        idx as c_int + HFIRSTNODE
    }
}

unsafe fn finishnodeset(t: *mut Table, slot: *const TValue, val: *mut TValue) -> c_int {
    if !unsafe { ttisnil(slot) } {
        unsafe { setobj(slot as *mut TValue, val) };
        HOK
    } else {
        unsafe { retpsetcode(t, slot) }
    }
}

unsafe fn rawfinishnodeset(slot: *const TValue, val: *mut TValue) -> bool {
    if unsafe { isabstkey(slot) } {
        false
    } else {
        unsafe { setobj(slot as *mut TValue, val) };
        true
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_psetint(
    t: *mut Table,
    key: lua_Integer,
    val: *mut TValue,
) -> c_int {
    debug_assert_eq!(unsafe { ikeyinarray(t, key) }, 0);
    unsafe { finishnodeset(t, getintfromhash(t, key), val) }
}

unsafe fn psetint(t: *mut Table, key: lua_Integer, val: *mut TValue) -> c_int { unsafe {
    let u = (key as lua_Unsigned).wrapping_sub(1);
    if u < (*t).asize as lua_Unsigned {
        let tag = getArrTag(t, u as u32);
        if checknoTM((*t).metatable, TM_NEWINDEX as usize) || !tagisempty(*tag) {
            fval2arr(t, u as u32, tag, val);
            HOK
        } else {
            !(u as c_int)
        }
    } else {
        luaH_psetint(t, key, val)
    }
}}

#[inline]
unsafe fn checknoTM(mt: *mut Table, e: usize) -> bool {
    mt.is_null() || unsafe { ((*mt).flags & (1u8 << e)) != 0 }
}

#[inline]
unsafe fn invalidateTMcache(t: *mut Table) {
    unsafe { (*t).flags &= !MASKFLAGS };
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_psetshortstr(
    t: *mut Table,
    key: *mut TString,
    val: *mut TValue,
) -> c_int {
    let slot = unsafe { luaH_Hgetshortstr(t, key) };
    if !unsafe { ttisnil(slot) } {
        unsafe { setobj(slot as *mut TValue, val) };
        HOK
    } else if unsafe { checknoTM((*t).metatable, TM_NEWINDEX as usize) } {
        if unsafe { ttisnil(val) } {
            HOK
        } else if unsafe { isabstkey(slot) }
            && !(unsafe { isblack_table(t) } && unsafe { iswhite_gc(key.cast()) })
        {
            let mut tk = TValue {
                value_: Value {
                    gc: ptr::null_mut(),
                },
                tt_: LUA_VNIL,
            };
            unsafe { setsvalue(&mut tk, key) };
            if unsafe { insertkey(t, &tk, val) } {
                unsafe { invalidateTMcache(t) };
                HOK
            } else {
                unsafe { retpsetcode(t, slot) }
            }
        } else {
            unsafe { retpsetcode(t, slot) }
        }
    } else {
        unsafe { retpsetcode(t, slot) }
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_psetstr(
    t: *mut Table,
    key: *mut TString,
    val: *mut TValue,
) -> c_int {
    if unsafe { strisshr(key) } {
        unsafe { luaH_psetshortstr(t, key, val) }
    } else {
        unsafe { finishnodeset(t, Hgetlongstr(t, key), val) }
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_pset(
    t: *mut Table,
    key: *const TValue,
    val: *mut TValue,
) -> c_int {
    match unsafe { ttypetag(key) } {
        LUA_VSHRSTR => unsafe { luaH_psetshortstr(t, tsvalue(key), val) },
        LUA_VNUMINT => unsafe { psetint(t, ivalue(key), val) },
        LUA_VNIL => HNOTFOUND,
        LUA_VNUMFLT => {
            let mut k = 0;
            if unsafe { luaV_flttointeger(fltvalue(key), &mut k, F2IEQ) } != 0 {
                unsafe { psetint(t, k, val) }
            } else {
                unsafe { finishnodeset(t, getgeneric(t, key, false), val) }
            }
        }
        _ => unsafe { finishnodeset(t, getgeneric(t, key, false), val) },
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_finishset(
    state: *mut lua_State,
    t: *mut Table,
    mut key: *const TValue,
    value: *mut TValue,
    hres: c_int,
) {
    if hres == HNOTFOUND {
        let mut aux = TValue {
            value_: Value {
                gc: ptr::null_mut(),
            },
            tt_: LUA_VNIL,
        };
        if unsafe { ttisnil(key) } {
            unsafe { luaG_runerror(state, TABLE_INDEX_NIL_ERR.as_ptr().cast()) };
        } else if unsafe { ttisfloat(key) } {
            let f = unsafe { fltvalue(key) };
            let mut k = 0;
            if unsafe { luaV_flttointeger(f, &mut k, F2IEQ) } != 0 {
                unsafe {
                    setivalue(&mut aux, k);
                    key = &aux;
                }
            } else if f.is_nan() {
                unsafe { luaG_runerror(state, TABLE_INDEX_NAN_ERR.as_ptr().cast()) };
            }
        } else if unsafe { isextstr(key) } {
            let ts = unsafe { luaS_normstr(state, tsvalue(key)) };
            let top = unsafe { (*state).top.p };
            unsafe {
                setsvalue(s2v(top), ts);
                (*state).top.p = top.add(1);
                luaH_newkey(state, t, s2v(top), value);
                (*state).top.p = top;
            }
            return;
        }
        unsafe { luaH_newkey(state, t, key, value) };
    } else if hres > 0 {
        unsafe { setobj(gval(gnode(t, (hres - HFIRSTNODE) as u32)), value) };
    } else {
        unsafe { obj2arr(t, (!hres) as u32, value) };
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_set(
    state: *mut lua_State,
    t: *mut Table,
    key: *const TValue,
    value: *mut TValue,
) {
    let hres = unsafe { luaH_pset(t, key, value) };
    if hres != HOK {
        unsafe { luaH_finishset(state, t, key, value, hres) };
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_setint(
    state: *mut lua_State,
    t: *mut Table,
    key: lua_Integer,
    value: *mut TValue,
) {
    let ik = unsafe { ikeyinarray(t, key) };
    if ik > 0 {
        unsafe { obj2arr(t, ik - 1, value) };
    } else if !unsafe { rawfinishnodeset(getintfromhash(t, key), value) } {
        let mut k = TValue {
            value_: Value {
                gc: ptr::null_mut(),
            },
            tt_: LUA_VNIL,
        };
        unsafe {
            setivalue(&mut k, key);
            luaH_newkey(state, t, &k, value);
        }
    }
}

unsafe fn hash_search(state: *mut lua_State, t: *mut Table, asize: u32) -> lua_Unsigned {
    let mut i = asize as lua_Unsigned + 1;
    let mut rnd = unsafe { (*G(state)).seed };
    let n = if asize > 0 {
        unsafe { luaO_ceillog2(asize) as u32 }
    } else {
        0
    };
    let mask = if n == 0 { 0 } else { (1u32 << n) - 1 };
    let incr = (rnd & mask) + 1;
    let mut j = if incr as lua_Unsigned <= LUA_MAXINTEGER as lua_Unsigned - i {
        i + incr as lua_Unsigned
    } else {
        i + 1
    };
    rnd >>= n;
    while !unsafe { hashkeyisempty(t, j) } {
        i = j;
        if j <= (LUA_MAXINTEGER as lua_Unsigned) / 2 - 1 {
            j = j * 2 + (rnd & 1) as lua_Unsigned;
            rnd >>= 1;
        } else {
            j = LUA_MAXINTEGER as lua_Unsigned;
            if unsafe { hashkeyisempty(t, j) } {
                break;
            } else {
                return j;
            }
        }
    }
    while j - i > 1 {
        let m = (i + j) / 2;
        if unsafe { hashkeyisempty(t, m) } {
            j = m;
        } else {
            i = m;
        }
    }
    i
}

unsafe fn binsearch(t: *mut Table, mut i: u32, mut j: u32) -> u32 {
    while j - i > 1 {
        let m = (i + j) / 2;
        if unsafe { arraykeyisempty(t, m) } {
            j = m;
        } else {
            i = m;
        }
    }
    i
}

unsafe fn newhint(t: *mut Table, hint: u32) -> lua_Unsigned {
    unsafe { *lenhint(t) = hint };
    hint as lua_Unsigned
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_getn(state: *mut lua_State, t: *mut Table) -> lua_Unsigned {
    let asize = unsafe { (*t).asize };
    if asize > 0 {
        let maxvicinity = 4u32;
        let mut limit = unsafe { *lenhint(t) };
        if limit == 0 {
            limit = 1;
        }
        if unsafe { arraykeyisempty(t, limit) } {
            for _ in 0..maxvicinity {
                if limit <= 1 {
                    break;
                }
                limit -= 1;
                if !unsafe { arraykeyisempty(t, limit) } {
                    return unsafe { newhint(t, limit) };
                }
            }
            return unsafe { newhint(t, binsearch(t, 0, limit)) };
        } else {
            for _ in 0..maxvicinity {
                if limit >= asize {
                    break;
                }
                limit += 1;
                if unsafe { arraykeyisempty(t, limit) } {
                    return unsafe { newhint(t, limit - 1) };
                }
            }
            if unsafe { arraykeyisempty(t, asize) } {
                return unsafe { newhint(t, binsearch(t, limit, asize)) };
            }
        }
        unsafe { *lenhint(t) = asize };
    }
    if unsafe { isdummy(t) } || unsafe { hashkeyisempty(t, asize as lua_Unsigned + 1) } {
        asize as lua_Unsigned
    } else {
        unsafe { hash_search(state, t, asize) }
    }
}

#[unsafe(no_mangle)]
pub unsafe  fn luaH_next(
    state: *mut lua_State,
    t: *mut Table,
    key: StkId,
) -> c_int { unsafe {
    let asize = (*t).asize;
    let mut i = findindex(state, t, s2v(key), asize);
    while i < asize {
        let tag = *getArrTag(t, i);
        if !tagisempty(tag) {
                setivalue(s2v(key), i as lua_Integer + 1);
                farr2val(t, i, tag, s2v(key.add(1)));
            return 1;
        }
        i += 1;
    }
    i -= asize;
    while i < sizenode(t) {
        let n = gnode(t, i);
        if !isempty(gval(n)) {
                getnodekey(s2v(key), n);
                setobj2s(state, key.add(1), gval(n));
            return 1;
        }
        i += 1;
    }
    0
}}

pub(crate) unsafe fn raw_luaH_new(state: *mut c_void) -> *mut c_void {
    unsafe { crate::runtime::luaH_new(state.cast()).cast() }
}

pub(crate) unsafe fn raw_luaH_resize(state: *mut c_void, table: *mut c_void, narr: u32, nrec: u32) {
    unsafe { crate::runtime::luaH_resize(state.cast(), table.cast(), narr, nrec) };
}

pub(crate) unsafe fn raw_luaH_setint(
    state: *mut c_void,
    table: *mut c_void,
    key: lua_Integer,
    value: *mut c_void,
) {
    unsafe { crate::runtime::luaH_setint(state.cast(), table.cast(), key, value.cast()) };
}

pub(crate) unsafe fn raw_luaH_getint(
    table: *mut c_void,
    key: lua_Integer,
    result: *mut c_void,
) -> u8 {
    unsafe { crate::runtime::luaH_getint(table.cast(), key, result.cast()) }
}

pub(crate) unsafe fn raw_luaH_getstr(
    table: *mut c_void,
    key: *mut c_void,
    result: *mut c_void,
) -> u8 {
    unsafe { crate::runtime::luaH_getstr(table.cast(), key.cast(), result.cast()) }
}

pub(crate) unsafe fn raw_luaH_Hgetshortstr(table: *mut c_void, key: *mut c_void) -> *const c_void {
    unsafe { luaH_Hgetshortstr(table.cast(), key.cast()).cast() }
}

pub(crate) unsafe fn raw_luaH_getshortstr(
    table: *mut c_void,
    key: *mut c_void,
    result: *mut c_void,
) -> u8 {
    unsafe { luaH_getshortstr(table.cast(), key.cast(), result.cast()) }
}

pub(crate) unsafe fn raw_luaH_set(
    state: *mut c_void,
    table: *mut c_void,
    key: *const c_void,
    value: *mut c_void,
) {
    unsafe { luaH_set(state.cast(), table.cast(), key.cast(), value.cast()) };
}

type IdxT = u32;

static TAB_FUNCS: [luaL_Reg; 10] = [
    luaL_Reg {
        name: NAME_CONCAT.as_ptr().cast(),
        func: Some(tconcat),
    },
    luaL_Reg {
        name: NAME_CREATE.as_ptr().cast(),
        func: Some(tcreate),
    },
    luaL_Reg {
        name: NAME_INSERT.as_ptr().cast(),
        func: Some(tinsert),
    },
    luaL_Reg {
        name: NAME_PACK.as_ptr().cast(),
        func: Some(tpack),
    },
    luaL_Reg {
        name: NAME_UNPACK.as_ptr().cast(),
        func: Some(tunpack),
    },
    luaL_Reg {
        name: NAME_REMOVE.as_ptr().cast(),
        func: Some(tremove),
    },
    luaL_Reg {
        name: NAME_MOVE.as_ptr().cast(),
        func: Some(tmove),
    },
    luaL_Reg {
        name: NAME_SORT.as_ptr().cast(),
        func: Some(sort),
    },
    luaL_Reg {
        name: NAME_GETN.as_ptr().cast(),
        func: Some(getn),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

#[inline]
unsafe fn lua_call(state: *mut lua_State, nargs: c_int, nresults: c_int) {
    unsafe { lua_callk(state, nargs, nresults, 0, None) };
}

#[inline]
fn is_none_or_nil(state: *mut lua_State, index: c_int) -> bool {
    unsafe { lua_type(state, index) <= LUA_TNIL.into() }
}

#[inline]
unsafe fn runtime_error(state: *mut lua_State, message: &[u8]) -> c_int {
    let len = message.len().saturating_sub(1);
    unsafe { lua_pushlstring(state, message.as_ptr().cast(), len) };
    unsafe { crate::lua_module::lua_error(state) }
}

#[inline]
unsafe fn checkfield(state: *mut lua_State, key: &[u8], n: c_int) -> bool {
    unsafe { lua_pushstring(state, key.as_ptr().cast()) };
    unsafe { lua_rawget(state, -n) != LUA_TNIL.into() }
}

unsafe fn checktab(state: *mut lua_State, arg: c_int, what: c_int) {
    if unsafe { lua_type(state, arg) } != LUA_TTABLE.into() {
        let mut n = 1;
        if unsafe { lua_getmetatable(state, arg) } != 0
            && (!(what & TAB_R != 0) || {
                n += 1;
                unsafe { checkfield(state, FIELD_INDEX, n) }
            })
            && (!(what & TAB_W != 0) || {
                n += 1;
                unsafe { checkfield(state, FIELD_NEWINDEX, n) }
            })
            && (!(what & TAB_L != 0) || {
                n += 1;
                unsafe { checkfield(state, FIELD_LEN, n) }
            })
        {
            unsafe { lua_pop(state, n) };
        } else {
            {
                luaL_checktype(state, arg, LUA_TTABLE.into())
            };
        }
    }
}

#[inline]
unsafe fn aux_getn(state: *mut lua_State, n: c_int, what: c_int) -> lua_Integer {
    unsafe { checktab(state, n, what | TAB_L) };
    luaL_len(state, n)
}

unsafe  fn tcreate(state: *mut lua_State) -> c_int {
    let sizeseq = { luaL_checkinteger(state, 1) } as lua_Unsigned;
    let sizerest = { luaL_optinteger(state, 2, 0) } as lua_Unsigned;
    unsafe {
        argcheck(
            state,
            sizeseq <= i32::MAX as lua_Unsigned,
            1,
            ERR_OUT_OF_RANGE,
        )
    };
    unsafe {
        argcheck(
            state,
            sizerest <= i32::MAX as lua_Unsigned,
            2,
            ERR_OUT_OF_RANGE,
        )
    };
    unsafe { crate::lua_module::lua_createtable(state, sizeseq as c_int, sizerest as c_int) };
    1
}

unsafe  fn tinsert(state: *mut lua_State) -> c_int {
    let pos;
    let mut e = unsafe { aux_getn(state, 1, TAB_RW) };
    e = match e.checked_add(1) {
        Some(v) => v,
        None => return unsafe { runtime_error(state, ERR_ARRAY_TOO_BIG) },
    };
    match unsafe { lua_gettop(state) } {
        2 => {
            pos = e;
        }
        3 => {
            pos = luaL_checkinteger(state, 2);
            unsafe { argcheck(state, pos >= 1 && pos <= e, 2, ERR_POSITION_OUT_OF_BOUNDS) };
            let mut i = e;
            while i > pos {
                unsafe { lua_geti(state, 1, i - 1) };
                unsafe { lua_seti(state, 1, i) };
                i -= 1;
            }
        }
        _ => return unsafe { runtime_error(state, ERR_WRONG_INSERT_ARGS) },
    }
    unsafe { lua_seti(state, 1, pos) };
    0
}

unsafe  fn tremove(state: *mut lua_State) -> c_int {
    let size = unsafe { aux_getn(state, 1, TAB_RW) };
    let mut pos = { luaL_optinteger(state, 2, size) };
    if pos != size {
        unsafe {
            argcheck(
                state,
                pos >= 1 && pos <= size,
                2,
                ERR_POSITION_OUT_OF_BOUNDS,
            )
        };
    }
    unsafe { lua_geti(state, 1, pos) };
    while pos < size {
        unsafe { lua_geti(state, 1, pos + 1) };
        unsafe { lua_seti(state, 1, pos) };
        pos += 1;
    }
    unsafe { lua_pushnil(state) };
    unsafe { lua_seti(state, 1, pos) };
    1
}

unsafe  fn tmove(state: *mut lua_State) -> c_int {
    let f = { luaL_checkinteger(state, 2) };
    let e = { luaL_checkinteger(state, 3) };
    let t = { luaL_checkinteger(state, 4) };
    let tt = if !is_none_or_nil(state, 5) { 5 } else { 1 };
    unsafe { checktab(state, 1, TAB_R) };
    unsafe { checktab(state, tt, TAB_W) };
    if e >= f {
        let n128 = e as i128 - f as i128 + 1;
        unsafe {
            argcheck(
                state,
                n128 <= lua_Integer::MAX as i128,
                3,
                ERR_TOO_MANY_ELEMENTS_TO_MOVE,
            )
        };
        unsafe {
            argcheck(
                state,
                (t as i128) + n128 - 1 <= lua_Integer::MAX as i128,
                4,
                ERR_DEST_WRAP_AROUND,
            )
        };
        let n = n128 as lua_Integer;
        let same_table = tt == 1 || unsafe { lua_compare(state, 1, tt, LUA_OPEQ) } != 0;
        if t > e || t <= f || !same_table {
            let mut i = 0;
            while i < n {
                unsafe { lua_geti(state, 1, f + i) };
                unsafe { lua_seti(state, tt, t + i) };
                i += 1;
            }
        } else {
            let mut i = n - 1;
            loop {
                unsafe { lua_geti(state, 1, f + i) };
                unsafe { lua_seti(state, tt, t + i) };
                if i == 0 {
                    break;
                }
                i -= 1;
            }
        }
    }
    unsafe { lua_pushvalue(state, tt) };
    1
}

unsafe fn addfield(state: *mut lua_State, out: &mut Vec<u8>, i: lua_Integer) -> Result<(), c_int> {
    unsafe { lua_geti(state, 1, i) };
    if unsafe { lua_isstring(state, -1) } == 0 {
        unsafe { lua_pop(state, 1) };
        return Err(unsafe { runtime_error(state, ERR_INVALID_CONCAT_VALUE) });
    }
    let mut len = 0usize;
    let ptr = unsafe { lua_tolstring(state, -1, &mut len) };
    if ptr.is_null() {
        unsafe { lua_pop(state, 1) };
        return Err(unsafe { runtime_error(state, ERR_INVALID_CONCAT_VALUE) });
    }
    let bytes = unsafe { core::slice::from_raw_parts(ptr.cast::<u8>(), len) };
    out.extend_from_slice(bytes);
    unsafe { lua_pop(state, 1) };
    Ok(())
}

unsafe  fn tconcat(state: *mut lua_State) -> c_int {
    let mut last = unsafe { aux_getn(state, 1, TAB_R) };
    let mut lsep = 0usize;
    let sep = { luaL_optlstring(state, 2, b"\0".as_ptr().cast(), &mut lsep) };
    let mut i = { luaL_optinteger(state, 3, 1) };
    last = luaL_optinteger(state, 4, last);
    let mut out = Vec::new();
    while i < last {
        if let Err(code) = unsafe { addfield(state, &mut out, i) } {
            return code;
        }
        let sep_bytes = unsafe { core::slice::from_raw_parts(sep.cast::<u8>(), lsep) };
        out.extend_from_slice(sep_bytes);
        i += 1;
    }
    if i == last {
        if let Err(code) = unsafe { addfield(state, &mut out, i) } {
            return code;
        }
    }
    unsafe { lua_pushlstring(state, out.as_ptr().cast(), out.len()) };
    1
}

unsafe  fn tpack(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    unsafe { crate::lua_module::lua_createtable(state, n, 1) };
    unsafe { crate::luaffi::lua_insert(state, 1) };
    let mut i = n;
    while i >= 1 {
        unsafe { lua_seti(state, 1, i as lua_Integer) };
        i -= 1;
    }
    unsafe { lua_pushinteger(state, n as lua_Integer) };
    unsafe { lua_setfield(state, 1, NAME_N.as_ptr().cast()) };
    1
}

unsafe  fn tunpack(state: *mut lua_State) -> c_int {
    let i = { luaL_optinteger(state, 2, 1) };
    let e = if is_none_or_nil(state, 3) {
        luaL_len(state, 1)
    } else {
        luaL_checkinteger(state, 3)
    };
    if i > e {
        return 0;
    }
    let n = e as i128 - i as i128;
    let results = n + 1;
    if n >= i32::MAX as i128 || unsafe { lua_checkstack(state, results as c_int) } == 0 {
        return unsafe { runtime_error(state, ERR_TOO_MANY_RESULTS_TO_UNPACK) };
    }
    let mut idx = i;
    while idx < e {
        unsafe { lua_geti(state, 1, idx) };
        idx += 1;
    }
    unsafe { lua_geti(state, 1, e) };
    results as c_int
}

#[inline]
unsafe fn set2(state: *mut lua_State, i: IdxT, j: IdxT) {
    unsafe { lua_seti(state, 1, i as lua_Integer) };
    unsafe { lua_seti(state, 1, j as lua_Integer) };
}

unsafe fn sort_comp(state: *mut lua_State, a: c_int, b: c_int) -> bool {
    if is_none_or_nil(state, 2) {
        unsafe { lua_compare(state, a, b, LUA_OPLT) != 0 }
    } else {
        unsafe { lua_pushvalue(state, 2) };
        unsafe { lua_pushvalue(state, a - 1) };
        unsafe { lua_pushvalue(state, b - 2) };
        unsafe { lua_call(state, 2, 1) };
        let res = unsafe { lua_toboolean(state, -1) } != 0;
        unsafe { lua_pop(state, 1) };
        res
    }
}

unsafe fn partition(state: *mut lua_State, lo: IdxT, up: IdxT) -> Result<IdxT, c_int> {
    let mut i = lo;
    let mut j = up - 1;
    loop {
        loop {
            i += 1;
            unsafe { lua_geti(state, 1, i as lua_Integer) };
            if !unsafe { sort_comp(state, -1, -2) } {
                break;
            }
            if i == up - 1 {
                return Err(unsafe { runtime_error(state, ERR_INVALID_ORDER_FUNCTION) });
            }
            unsafe { lua_pop(state, 1) };
        }
        loop {
            j -= 1;
            unsafe { lua_geti(state, 1, j as lua_Integer) };
            if !unsafe { sort_comp(state, -3, -1) } {
                break;
            }
            if j < i {
                return Err(unsafe { runtime_error(state, ERR_INVALID_ORDER_FUNCTION) });
            }
            unsafe { lua_pop(state, 1) };
        }
        if j < i {
            unsafe { lua_pop(state, 1) };
            unsafe { set2(state, up - 1, i) };
            return Ok(i);
        }
        unsafe { set2(state, i, j) };
    }
}

#[inline]
fn choose_pivot(lo: IdxT, up: IdxT, rnd: u32) -> IdxT {
    let r4 = (up - lo) / 4;
    (rnd ^ lo ^ up) % (r4 * 2) + (lo + r4)
}

unsafe fn auxsort(
    state: *mut lua_State,
    mut lo: IdxT,
    mut up: IdxT,
    mut rnd: u32,
) -> Result<(), c_int> {
    while lo < up {
        unsafe { lua_geti(state, 1, lo as lua_Integer) };
        unsafe { lua_geti(state, 1, up as lua_Integer) };
        if unsafe { sort_comp(state, -1, -2) } {
            unsafe { set2(state, lo, up) };
        } else {
            unsafe { lua_pop(state, 2) };
        }
        if up - lo == 1 {
            return Ok(());
        }
        let mut p = if up - lo < RANLIMIT || rnd == 0 {
            (lo + up) / 2
        } else {
            choose_pivot(lo, up, rnd)
        };
        unsafe { lua_geti(state, 1, p as lua_Integer) };
        unsafe { lua_geti(state, 1, lo as lua_Integer) };
        if unsafe { sort_comp(state, -2, -1) } {
            unsafe { set2(state, p, lo) };
        } else {
            unsafe { lua_pop(state, 1) };
            unsafe { lua_geti(state, 1, up as lua_Integer) };
            if unsafe { sort_comp(state, -1, -2) } {
                unsafe { set2(state, p, up) };
            } else {
                unsafe { lua_pop(state, 2) };
            }
        }
        if up - lo == 2 {
            return Ok(());
        }
        unsafe { lua_geti(state, 1, p as lua_Integer) };
        unsafe { lua_pushvalue(state, -1) };
        unsafe { lua_geti(state, 1, (up - 1) as lua_Integer) };
        unsafe { set2(state, p, up - 1) };
        p = match unsafe { partition(state, lo, up) } {
            Ok(p) => p,
            Err(code) => return Err(code),
        };
        let n;
        if p - lo < up - p {
            unsafe { auxsort(state, lo, p - 1, rnd)? };
            n = p - lo;
            lo = p + 1;
        } else {
            unsafe { auxsort(state, p + 1, up, rnd)? };
            n = up - p;
            up = p - 1;
        }
        if (up - lo) / 128 > n {
            rnd = luaL_makeseed(state);
        }
    }
    Ok(())
}

unsafe  fn sort(state: *mut lua_State) -> c_int {
    let n = unsafe { aux_getn(state, 1, TAB_RW) };
    if n > 1 {
        unsafe { argcheck(state, n < i32::MAX as lua_Integer, 1, ERR_ARRAY_TOO_BIG) };
        if !is_none_or_nil(state, 2) {
            {
                luaL_checktype(state, 2, LUA_TFUNCTION.into())
            };
        }
        unsafe { lua_settop(state, 2) };
        if let Err(code) = unsafe { auxsort(state, 1, n as IdxT, 0) } {
            return code;
        }
    }
    0
}

unsafe  fn getn(state: *mut lua_State) -> c_int {
    let n = unsafe { aux_getn(state, 1, TAB_R) };
    unsafe { lua_pushinteger(state, n) };
    1
}

#[unsafe(no_mangle)]
pub unsafe  fn luaopen_table(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &TAB_FUNCS) };
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aux_rs::{luaL_checkversion_, luaL_loadbufferx, luaL_newstate};
    use crate::init::luaL_openselectedlibs;
    use crate::luaffi::*;
    use crate::state::lua_close;
    use crate::test_support::run_lua_test;
    use std::ptr;

    fn load_and_run(state: *mut lua_State, source: &str) -> Result<(), String> {
        unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
            luaL_openselectedlibs(state, !0, 0);
            let name = b"@table_rs_test\0";
            let status = luaL_loadbufferx(
                state,
                source.as_ptr().cast(),
                source.len(),
                name.as_ptr().cast(),
                ptr::null(),
            );
            if status != LUA_OK.into() {
                return Err(lua_error_string(state));
            }
            let status = lua_pcall(state, 0, 0, 0);
            if status != LUA_OK.into() {
                return Err(lua_error_string(state));
            }
        }
        Ok(())
    }

    fn lua_error_string(state: *mut lua_State) -> String {
        unsafe {
            let mut len = 0usize;
            let ptr = lua_tolstring(state, -1, &mut len);
            if ptr.is_null() {
                return "<non-string error>".to_string();
            }
            String::from_utf8_lossy(core::slice::from_raw_parts(ptr.cast::<u8>(), len)).into()
        }
    }

    #[test]
    fn table_library_behaves_like_builtin() {
        let state = { luaL_newstate() };
        assert!(!state.is_null());
        let script = r#"
            assert(type(table) == "table")

            local t = table.create(3, 0)
            assert(#t == 0)

            table.insert(t, "a")
            table.insert(t, "c")
            table.insert(t, 2, "b")
            assert(table.concat(t, ",") == "a,b,c")

            local removed = table.remove(t, 2)
            assert(removed == "b")
            assert(table.concat(t, ",") == "a,c")

            local moved = table.move({10, 20, 30}, 1, 3, 2, {0, 0, 0, 0, 0})
            assert(moved[1] == 0 and moved[2] == 10 and moved[3] == 20 and moved[4] == 30)

            local packed = table.pack("x", nil, "z")
            assert(packed.n == 3)
            local a, b, c = table.unpack(packed, 1, packed.n)
            assert(a == "x" and b == nil and c == "z")

            local sortable = {4, 1, 3, 2}
            table.sort(sortable)
            assert(table.concat(sortable, ",") == "1,2,3,4")

            table.sort(sortable, function(x, y) return x > y end)
            assert(table.concat(sortable, ",") == "4,3,2,1")

            assert(table.getn({11, 22, 33}) == 3)
        "#;

        let result = load_and_run(state, script);
        unsafe { lua_close(state) };
        if let Err(err) = result {
            panic!("{err}");
        }
    }

    #[test]
    fn table_builtin_script() {
        run_lua_test(
            "test/table_builtin.lua",
            include_str!("../test/table_builtin.lua"),
        );
    }
}
