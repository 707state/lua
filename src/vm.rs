#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

use crate::debug::*;
use crate::do_rs::*;
use crate::func::luaF_closeupval;
use crate::func::luaF_findupval;
use crate::func::luaF_newLclosure;
use crate::opcodes::luaP_isIT;
use crate::runtime::*;
use crate::string::luaS_createlngstrobj;
use crate::string::luaS_eqstr;
use crate::table::luaH_getshortstr;
use crate::table::luaH_psetshortstr;
use crate::table::luaH_resizearray;
use crate::tm::*;
use core::mem::size_of;

/// C strcoll 的 Rust 等价实现：按字节顺序比较（locale 无关）
#[inline]
unsafe fn strcoll(s1: *const c_char, s2: *const c_char) -> c_int {
    // Lua 字符串比较走此路径，safe 地使用 CStr
    let b1 = unsafe { core::ffi::CStr::from_ptr(s1) }.to_bytes();
    let b2 = unsafe { core::ffi::CStr::from_ptr(s2) }.to_bytes();
    b1.cmp(b2) as c_int
}

/// C strlen 的 Rust 等价实现
#[inline]
unsafe fn strlen(s: *const c_char) -> usize {
    unsafe { core::ffi::CStr::from_ptr(s) }.to_bytes().len()
}

#[inline]
unsafe fn get_nresults(cs: u32) -> c_int {
    (cs & CIST_NRESULTS) as c_int - 1
}

#[inline]
unsafe fn setclLvalue2s(_L: *mut lua_State, o: StkId, cl: *mut LClosure) {
    unsafe {
        (*s2v(o)).value_.gc = cl.cast();
        settt_(s2v(o), LUA_VLCL | BIT_ISCOLLECTABLE);
    }
}

#[inline]
unsafe fn chgivalue(o: *mut TValue, x: lua_Integer) {
    unsafe {
        (*o).value_.i = x;
    }
}

#[inline]
unsafe fn chgfltvalue(o: *mut TValue, x: lua_Number) {
    unsafe {
        (*o).value_.n = x;
    }
}

#[inline]
unsafe fn ttisfunction(o: *const TValue) -> bool {
    unsafe { ttype(o) == LUA_TFUNCTION }
}

#[inline]
unsafe fn cvt2num(o: *const TValue) -> bool {
    unsafe { ttisstring(o) }
}

#[inline]
unsafe fn tonumberns(o: *const TValue, n: *mut lua_Number) -> c_int {
    unsafe {
        if ttisfloat(o) {
            *n = fltvalue(o);
            1
        } else if ttisinteger(o) {
            *n = ivalue(o) as lua_Number;
            1
        } else {
            0
        }
    }
}

#[inline]
unsafe fn tointegerns(o: *const TValue, i: *mut lua_Integer) -> c_int {
    unsafe {
        if ttisinteger(o) {
            *i = ivalue(o);
            1
        } else {
            luaV_tointegerns(o, i, LUA_FLOORN2I)
        }
    }
}

#[inline]
unsafe fn tsslen(s: *mut TString) -> usize {
    unsafe {
        if strisshr(s) {
            (*s).shrlen as usize
        } else {
            (*s).u.lnglen
        }
    }
}

#[inline]
unsafe fn getlngstr(s: *mut TString) -> *mut c_char {
    unsafe { (*s).contents }
}

#[inline]
unsafe fn eqshrstr(a: *mut TString, b: *mut TString) -> c_int {
    c_int::from(core::ptr::eq(a, b))
}

#[inline]
unsafe fn checknoTM(mt: *mut Table, e: c_int) -> bool {
    unsafe { mt.is_null() || ((*mt).flags & (1u8 << e)) != 0 }
}

#[inline]
unsafe fn notm(tm: *const TValue) -> bool {
    unsafe { ttisnil(tm) }
}

#[inline]
unsafe fn fasttm(L: *mut lua_State, mt: *mut Table, e: c_int) -> *const TValue {
    unsafe {
        if checknoTM(mt, e) {
            core::ptr::null()
        } else {
            luaT_gettm(mt, e, (&mut (*G(L)).tmname)[e as usize])
        }
    }
}

#[inline]
unsafe fn invalidateTMcache(t: *mut Table) {
    unsafe {
        (*t).flags &= !MASKFLAGS;
    }
}

#[inline]
unsafe fn getArrTag(t: *mut Table, k: u32) -> *mut u8 {
    unsafe { (*t).array.cast::<u8>().add(size_of::<u32>() + k as usize) }
}

#[inline]
unsafe fn obj2arr(t: *mut Table, k: u32, value: *const TValue) {
    unsafe {
        *getArrTag(t, k) = (*value).tt_;
        *getArrVal(t, k) = (*value).value_;
    }
}

#[inline]
unsafe fn luaV_fastseti(t: *const TValue, key: lua_Integer, val: *mut TValue) -> c_int {
    unsafe {
        if !ttistable(t) {
            HNOTATABLE
        } else {
            let h = hvalue(t);
            let u = (key as lua_Unsigned).wrapping_sub(1);
            if u < (*h).asize as lua_Unsigned {
                let tag = getArrTag(h, u as u32);
                if checknoTM((*h).metatable, TM_NEWINDEX) || !tagisempty(*tag) {
                    *tag = (*val).tt_;
                    *getArrVal(h, u as u32) = (*val).value_;
                    HOK
                } else {
                    !(u as c_int)
                }
            } else {
                luaH_psetint(h, key, val)
            }
        }
    }
}

#[inline]
unsafe fn luaV_finishfastset(L: *mut lua_State, t: *const TValue, v: *const TValue) {
    unsafe {
        luaC_barrierback(L, gcvalue(t), v);
    }
}

#[inline]
unsafe fn lua_numbertointeger(n: lua_Number, p: *mut lua_Integer) -> c_int {
    unsafe {
        if n >= lua_Integer::MIN as lua_Number && n < -(lua_Integer::MIN as lua_Number) {
            *p = n as lua_Integer;
            1
        } else {
            0
        }
    }
}

#[inline]
unsafe fn l_intfitsf(i: lua_Integer) -> bool {
    const LIM: lua_Integer = 1_i64 << f64::MANTISSA_DIGITS;
    (-LIM..=LIM).contains(&i)
}

#[inline]
unsafe fn l_strton(obj: *const TValue, result: *mut TValue) -> c_int {
    unsafe {
        if !cvt2num(obj) {
            0
        } else {
            let st = tsvalue(obj);
            let mut stlen = 0usize;
            let s = getlstr(st, &mut stlen);
            c_int::from(luaO_str2num(s, result) == stlen + 1)
        }
    }
}

#[inline]
unsafe fn mask1(n: u32, p: u32) -> Instruction {
    ((!((!0u32) << n)) << p) as Instruction
}

#[inline]
unsafe fn getarg(i: Instruction, pos: u32, size: u32) -> c_int {
    unsafe { ((i >> pos) & mask1(size, 0)) as c_int }
}

#[inline]
unsafe fn GET_OPCODE(i: Instruction) -> c_int {
    unsafe { ((i >> POS_OP) & mask1(SIZE_OP, 0)) as c_int }
}

#[inline]
unsafe fn GETARG_A(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_A, SIZE_A) }
}
#[inline]
unsafe fn GETARG_B(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_B, SIZE_B) }
}
#[inline]
unsafe fn GETARG_VB(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_VB, SIZE_VB) }
}
#[inline]
unsafe fn GETARG_SB(i: Instruction) -> c_int {
    unsafe { GETARG_B(i) - OFFSET_SC }
}
#[inline]
unsafe fn GETARG_C(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_C, SIZE_C) }
}
#[inline]
unsafe fn GETARG_VC(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_VC, SIZE_VC) }
}
#[inline]
unsafe fn GETARG_SC(i: Instruction) -> c_int {
    unsafe { GETARG_C(i) - OFFSET_SC }
}
#[inline]
unsafe fn TESTARG_K(i: Instruction) -> bool {
    (i & (1u32 << POS_K)) != 0
}
#[inline]
unsafe fn GETARG_K(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_K, 1) }
}
#[inline]
unsafe fn GETARG_BX(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_BX, SIZE_BX) }
}
#[inline]
unsafe fn GETARG_AX(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_AX, SIZE_AX) }
}
#[inline]
unsafe fn GETARG_SBX(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_BX, SIZE_BX) - OFFSET_SBX }
}
#[inline]
unsafe fn GETARG_SJ(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_SJ, SIZE_SJ) - OFFSET_SJ }
}

#[inline]
unsafe fn savepc(ci: *mut CallInfo, pc: *const Instruction) {
    unsafe {
        (*ci).u.l.savedpc = pc;
    }
}

#[inline]
unsafe fn savestate(L: *mut lua_State, ci: *mut CallInfo, pc: *const Instruction) {
    unsafe {
        savepc(ci, pc);
        (*L).top.p = (*ci).top.p;
    }
}

#[inline]
unsafe fn updatetrap(ci: *mut CallInfo, trap: &mut c_int) {
    unsafe {
        *trap = (*ci).u.l.trap;
    }
}

#[inline]
unsafe fn updatebase(ci: *mut CallInfo, base: &mut StkId) {
    unsafe {
        *base = (*ci).func.p.add(1);
    }
}

#[inline]
unsafe fn RA(base: StkId, i: Instruction) -> StkId {
    unsafe { base.add(GETARG_A(i) as usize) }
}

#[inline]
unsafe fn RB(base: StkId, i: Instruction) -> StkId {
    unsafe { base.add(GETARG_B(i) as usize) }
}

#[inline]
unsafe fn RC(base: StkId, i: Instruction) -> StkId {
    unsafe { base.add(GETARG_C(i) as usize) }
}

#[inline]
unsafe fn KB<'a>(k: *mut TValue, i: Instruction) -> *mut TValue {
    unsafe { k.add(GETARG_B(i) as usize) }
}

#[inline]
unsafe fn KC<'a>(k: *mut TValue, i: Instruction) -> *mut TValue {
    unsafe { k.add(GETARG_C(i) as usize) }
}

#[inline]
unsafe fn RKC(base: StkId, k: *mut TValue, i: Instruction) -> *mut TValue {
    unsafe {
        if TESTARG_K(i) {
            k.add(GETARG_C(i) as usize)
        } else {
            s2v(base.add(GETARG_C(i) as usize))
        }
    }
}

#[inline]
unsafe fn dojump(
    ci: *mut CallInfo,
    i: Instruction,
    e: c_int,
    pc: &mut *const Instruction,
    trap: &mut c_int,
) {
    unsafe {
        *pc = (*pc).offset((GETARG_SJ(i) + e) as isize);
        updatetrap(ci, trap);
    }
}

#[inline]
unsafe fn donextjump(ci: *mut CallInfo, pc: &mut *const Instruction, trap: &mut c_int) {
    unsafe {
        let ni = **pc;
        dojump(ci, ni, 1, pc, trap);
    }
}

#[inline]
unsafe fn docondjump(
    cond: c_int,
    ci: *mut CallInfo,
    i: Instruction,
    pc: &mut *const Instruction,
    trap: &mut c_int,
) {
    unsafe {
        if cond != GETARG_K(i) {
            *pc = (*pc).add(1);
        } else {
            donextjump(ci, pc, trap);
        }
    }
}

#[inline]
unsafe fn checkGC(
    L: *mut lua_State,
    ci: *mut CallInfo,
    pc: *const Instruction,
    trap: &mut c_int,
    c: StkId,
) {
    unsafe {
        if (*G(L)).gcdebt <= 0 {
            savepc(ci, pc);
            (*L).top.p = c;
            luaC_step(L);
            updatetrap(ci, trap);
        }
    }
}

unsafe fn l_strcmp(ts1: *mut TString, ts2: *mut TString) -> c_int {
    unsafe {
        let mut rl1 = 0usize;
        let mut s1 = getlstr(ts1, &mut rl1);
        let mut rl2 = 0usize;
        let mut s2 = getlstr(ts2, &mut rl2);
        loop {
            let temp = strcoll(s1, s2);
            if temp != 0 {
                return temp;
            }
            let zl1 = strlen(s1);
            let zl2 = strlen(s2);
            if zl2 == rl2 {
                return if zl1 == rl1 { 0 } else { 1 };
            } else if zl1 == rl1 {
                return -1;
            }
            let step1 = zl1 + 1;
            let step2 = zl2 + 1;
            s1 = s1.add(step1);
            s2 = s2.add(step2);
            rl1 -= step1;
            rl2 -= step2;
        }
    }
}

pub(crate) unsafe fn luaV_tonumber_(obj: *const TValue, n: *mut lua_Number) -> c_int {
    unsafe {
        let mut v = TValue {
            value_: Value { i: 0 },
            tt_: LUA_VNIL,
        };
        if ttisinteger(obj) {
            *n = ivalue(obj) as lua_Number;
            1
        } else if l_strton(obj, &mut v) != 0 {
            *n = if ttisfloat(&v) {
                fltvalue(&v)
            } else {
                ivalue(&v) as lua_Number
            };
            1
        } else {
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe fn luaV_flttointeger(n: lua_Number, p: *mut lua_Integer, mode: c_int) -> c_int {
    unsafe {
        let mut f = n.floor();
        if n != f {
            if mode == F2Ieq {
                return 0;
            } else if mode == F2Iceil {
                f += 1.0;
            }
        }
        lua_numbertointeger(f, p)
    }
}

#[unsafe(no_mangle)]
pub unsafe fn luaV_tointegerns(obj: *const TValue, p: *mut lua_Integer, mode: c_int) -> c_int {
    unsafe {
        if ttisfloat(obj) {
            luaV_flttointeger(fltvalue(obj), p, mode)
        } else if ttisinteger(obj) {
            *p = ivalue(obj);
            1
        } else {
            0
        }
    }
}

pub(crate) unsafe fn luaV_tointeger(obj: *const TValue, p: *mut lua_Integer, mode: c_int) -> c_int {
    unsafe {
        let mut v = TValue {
            value_: Value { i: 0 },
            tt_: LUA_VNIL,
        };
        let mut o = obj;
        if l_strton(obj, &mut v) != 0 {
            o = &v;
        }
        luaV_tointegerns(o, p, mode)
    }
}

unsafe fn forlimit(
    L: *mut lua_State,
    init: lua_Integer,
    lim: *const TValue,
    p: *mut lua_Integer,
    step: lua_Integer,
) -> c_int {
    unsafe {
        if luaV_tointeger(lim, p, if step < 0 { F2Iceil } else { F2Ifloor }) == 0 {
            let mut flim = 0.0;
            if tonumber(lim, &mut flim) == 0 {
                luaG_forerror(L, lim, c"limit".as_ptr());
            }
            if 0.0 < flim {
                if step < 0 {
                    return 1;
                }
                *p = lua_Integer::MAX;
            } else {
                if step > 0 {
                    return 1;
                }
                *p = lua_Integer::MIN;
            }
        }
        c_int::from(if step > 0 { init > *p } else { init < *p })
    }
}

unsafe fn forprep(L: *mut lua_State, ra: StkId) -> c_int {
    unsafe {
        let pinit = s2v(ra);
        let plimit = s2v(ra.add(1));
        let pstep = s2v(ra.add(2));
        if ttisinteger(pinit) && ttisinteger(pstep) {
            let init = ivalue(pinit);
            let step = ivalue(pstep);
            let mut limit = 0;
            if step == 0 {
                luaG_runerror(L, "'for' step is zero");
            }
            if forlimit(L, init, plimit, &mut limit, step) != 0 {
                return 1;
            }
            let count = if step > 0 {
                let mut count = (limit as lua_Unsigned).wrapping_sub(init as lua_Unsigned);
                if step != 1 {
                    count /= step as lua_Unsigned;
                }
                count
            } else {
                let mut count = (init as lua_Unsigned).wrapping_sub(limit as lua_Unsigned);
                count /= ((-(step + 1)) as lua_Unsigned).wrapping_add(1);
                count
            };
            chgivalue(s2v(ra), count as lua_Integer);
            setivalue(s2v(ra.add(1)), step);
            chgivalue(s2v(ra.add(2)), init);
        } else {
            let (mut init, mut limit, mut step) = (0.0, 0.0, 0.0);
            if tonumber(plimit, &mut limit) == 0 {
                luaG_forerror(L, plimit, c"limit".as_ptr());
            }
            if tonumber(pstep, &mut step) == 0 {
                luaG_forerror(L, pstep, c"step".as_ptr());
            }
            if tonumber(pinit, &mut init) == 0 {
                luaG_forerror(L, pinit, c"initial value".as_ptr());
            }
            if step == 0.0 {
                luaG_runerror(L, "'for' step is zero");
            }
            if if 0.0 < step {
                limit < init
            } else {
                init < limit
            } {
                return 1;
            }
            setfltvalue(s2v(ra), limit);
            setfltvalue(s2v(ra.add(1)), step);
            setfltvalue(s2v(ra.add(2)), init);
        }
        0
    }
}

unsafe fn floatforloop(ra: StkId) -> c_int {
    unsafe {
        let step = fltvalue(s2v(ra.add(1)));
        let limit = fltvalue(s2v(ra));
        let mut idx = fltvalue(s2v(ra.add(2)));
        idx += step;
        if if 0.0 < step {
            idx <= limit
        } else {
            limit <= idx
        } {
            chgfltvalue(s2v(ra.add(2)), idx);
            1
        } else {
            0
        }
    }
}

pub(crate) unsafe fn luaV_finishget(
    L: *mut lua_State,
    mut t: *const TValue,
    key: *mut TValue,
    val: StkId,
    mut tag: lu_byte,
) -> lu_byte {
    unsafe {
        for _ in 0..MAXTAGLOOP {
            let tm = if tag == LUA_VNOTABLE {
                let tm = luaT_gettmbyobj(L, t, TM_INDEX);
                if notm(tm) {
                    luaG_typeerror(L, t, c"index".as_ptr());
                }
                tm
            } else {
                let tm = fasttm(L, (*hvalue(t)).metatable, TM_INDEX);
                if tm.is_null() {
                    setnilvalue(s2v(val));
                    return LUA_VNIL;
                }
                tm
            };
            if ttisfunction(tm) {
                tag = luaT_callTMres(L, tm, t, key, val);
                return tag;
            }
            t = tm;
            tag = if !ttistable(t) {
                LUA_VNOTABLE
            } else {
                luaH_get(hvalue(t), key, s2v(val))
            };
            if !tagisempty(tag) {
                return tag;
            }
        }
        luaG_runerror(L, "'__index' chain too long; possible loop")
    }
}

pub(crate) unsafe fn luaV_finishset(
    L: *mut lua_State,
    mut t: *const TValue,
    key: *mut TValue,
    val: *mut TValue,
    mut hres: c_int,
) {
    unsafe {
        for _ in 0..MAXTAGLOOP {
            let tm;
            if hres != HNOTATABLE {
                let h = hvalue(t);
                tm = fasttm(L, (*h).metatable, TM_NEWINDEX);
                if tm.is_null() {
                    sethvalue2s(L, (*L).top.p, h);
                    (*L).top.p = (*L).top.p.add(1);
                    luaH_finishset(L, h, key, val, hres);
                    (*L).top.p = (*L).top.p.sub(1);
                    invalidateTMcache(h);
                    luaC_barrierback(L, obj2gco(h), val);
                    return;
                }
            } else {
                tm = luaT_gettmbyobj(L, t, TM_NEWINDEX);
                if notm(tm) {
                    luaG_typeerror(L, t, c"index".as_ptr());
                }
            }
            if ttisfunction(tm) {
                luaT_callTM(L, tm, t, key, val);
                return;
            }
            t = tm;
            hres = if !ttistable(t) {
                HNOTATABLE
            } else {
                luaH_pset(hvalue(t), key, val)
            };
            if hres == HOK {
                luaV_finishfastset(L, t, val);
                return;
            }
        }
        luaG_runerror(L, "'__newindex' chain too long; possible loop")
    }
}

unsafe fn LTintfloat(i: lua_Integer, f: lua_Number) -> c_int {
    unsafe {
        if l_intfitsf(i) {
            c_int::from((i as lua_Number) < f)
        } else {
            let mut fi = 0;
            if luaV_flttointeger(f, &mut fi, F2Iceil) != 0 {
                c_int::from(i < fi)
            } else {
                c_int::from(f > 0.0)
            }
        }
    }
}

unsafe fn LEintfloat(i: lua_Integer, f: lua_Number) -> c_int {
    unsafe {
        if l_intfitsf(i) {
            c_int::from((i as lua_Number) <= f)
        } else {
            let mut fi = 0;
            if luaV_flttointeger(f, &mut fi, F2Ifloor) != 0 {
                c_int::from(i <= fi)
            } else {
                c_int::from(f > 0.0)
            }
        }
    }
}

unsafe fn LTfloatint(f: lua_Number, i: lua_Integer) -> c_int {
    unsafe {
        if l_intfitsf(i) {
            c_int::from(f < i as lua_Number)
        } else {
            let mut fi = 0;
            if luaV_flttointeger(f, &mut fi, F2Ifloor) != 0 {
                c_int::from(fi < i)
            } else {
                c_int::from(f < 0.0)
            }
        }
    }
}

unsafe fn LEfloatint(f: lua_Number, i: lua_Integer) -> c_int {
    unsafe {
        if l_intfitsf(i) {
            c_int::from(f <= i as lua_Number)
        } else {
            let mut fi = 0;
            if luaV_flttointeger(f, &mut fi, F2Iceil) != 0 {
                c_int::from(fi <= i)
            } else {
                c_int::from(f < 0.0)
            }
        }
    }
}

unsafe fn LTnum(l: *const TValue, r: *const TValue) -> c_int {
    unsafe {
        if ttisinteger(l) {
            let li = ivalue(l);
            if ttisinteger(r) {
                c_int::from(li < ivalue(r))
            } else {
                LTintfloat(li, fltvalue(r))
            }
        } else {
            let lf = fltvalue(l);
            if ttisfloat(r) {
                c_int::from(lf < fltvalue(r))
            } else {
                LTfloatint(lf, ivalue(r))
            }
        }
    }
}

unsafe fn LEnum(l: *const TValue, r: *const TValue) -> c_int {
    unsafe {
        if ttisinteger(l) {
            let li = ivalue(l);
            if ttisinteger(r) {
                c_int::from(li <= ivalue(r))
            } else {
                LEintfloat(li, fltvalue(r))
            }
        } else {
            let lf = fltvalue(l);
            if ttisfloat(r) {
                c_int::from(lf <= fltvalue(r))
            } else {
                LEfloatint(lf, ivalue(r))
            }
        }
    }
}

unsafe fn lessthanothers(L: *mut lua_State, l: *const TValue, r: *const TValue) -> c_int {
    unsafe {
        if ttisstring(l) && ttisstring(r) {
            c_int::from(l_strcmp(tsvalue(l), tsvalue(r)) < 0)
        } else {
            luaT_callorderTM(L, l, r, TM_LT)
        }
    }
}

unsafe fn lessequalothers(L: *mut lua_State, l: *const TValue, r: *const TValue) -> c_int {
    unsafe {
        if ttisstring(l) && ttisstring(r) {
            c_int::from(l_strcmp(tsvalue(l), tsvalue(r)) <= 0)
        } else {
            luaT_callorderTM(L, l, r, TM_LE)
        }
    }
}

pub(crate) unsafe fn luaV_lessthan(L: *mut lua_State, l: *const TValue, r: *const TValue) -> c_int {
    unsafe {
        if ttisnumber(l) && ttisnumber(r) {
            LTnum(l, r)
        } else {
            lessthanothers(L, l, r)
        }
    }
}

pub(crate) unsafe fn luaV_lessequal(
    L: *mut lua_State,
    l: *const TValue,
    r: *const TValue,
) -> c_int {
    unsafe {
        if ttisnumber(l) && ttisnumber(r) {
            LEnum(l, r)
        } else {
            lessequalothers(L, l, r)
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe fn luaV_equalobj(L: *mut lua_State, t1: *const TValue, t2: *const TValue) -> c_int {
    unsafe {
        let tm;
        if ttype(t1) != ttype(t2) {
            return 0;
        } else if ttypetag(t1) != ttypetag(t2) {
            match ttypetag(t1) {
                LUA_VNUMINT => {
                    let mut i2 = 0;
                    return c_int::from(
                        luaV_flttointeger(fltvalue(t2), &mut i2, F2Ieq) != 0 && ivalue(t1) == i2,
                    );
                }
                LUA_VNUMFLT => {
                    let mut i1 = 0;
                    return c_int::from(
                        luaV_flttointeger(fltvalue(t1), &mut i1, F2Ieq) != 0 && i1 == ivalue(t2),
                    );
                }
                LUA_VSHRSTR | LUA_VLNGSTR => return luaS_eqstr(tsvalue(t1), tsvalue(t2)),
                _ => return 0,
            }
        } else {
            match ttypetag(t1) {
                LUA_VNIL | LUA_VFALSE | LUA_VTRUE => return 1,
                LUA_VNUMINT => return c_int::from(ivalue(t1) == ivalue(t2)),
                LUA_VNUMFLT => return c_int::from(fltvalue(t1) == fltvalue(t2)),
                LUA_VLIGHTUSERDATA => return c_int::from(pvalue(t1) == pvalue(t2)),
                LUA_VSHRSTR => return eqshrstr(tsvalue(t1), tsvalue(t2)),
                LUA_VLNGSTR => return luaS_eqstr(tsvalue(t1), tsvalue(t2)),
                LUA_VUSERDATA => {
                    if uvalue(t1) == uvalue(t2) {
                        return 1;
                    } else if L.is_null() {
                        return 0;
                    }
                    tm = fasttm(L, (*uvalue(t1)).metatable, TM_EQ);
                    let tm = if tm.is_null() {
                        fasttm(L, (*uvalue(t2)).metatable, TM_EQ)
                    } else {
                        tm
                    };
                    if tm.is_null() {
                        return 0;
                    }
                    let tag = luaT_callTMres(L, tm, t1, t2, (*L).top.p);
                    return c_int::from(!l_isfalse(&TValue {
                        value_: Value { ub: tag },
                        tt_: tag,
                    }));
                }
                LUA_VTABLE => {
                    if hvalue(t1) == hvalue(t2) {
                        return 1;
                    } else if L.is_null() {
                        return 0;
                    }
                    tm = fasttm(L, (*hvalue(t1)).metatable, TM_EQ);
                    let tm = if tm.is_null() {
                        fasttm(L, (*hvalue(t2)).metatable, TM_EQ)
                    } else {
                        tm
                    };
                    if tm.is_null() {
                        return 0;
                    }
                    let tag = luaT_callTMres(L, tm, t1, t2, (*L).top.p);
                    return c_int::from(!tagisempty(tag) && tag != LUA_VFALSE);
                }
                LUA_VLCF => return c_int::from(fvalue(t1) == fvalue(t2)),
                _ => return c_int::from(gcvalue(t1) == gcvalue(t2)),
            }
        }
    }
}

unsafe fn tostring(L: *mut lua_State, o: *mut TValue) -> bool {
    unsafe {
        ttisstring(o)
            || (cvt2str(o) && {
                luaO_tostring(L, o);
                true
            })
    }
}

#[inline]
unsafe fn isemptystr(o: *const TValue) -> bool {
    unsafe { ttisshrstring(o) && (*tsvalue(o)).shrlen == 0 }
}

unsafe fn copy2buff(top: StkId, mut n: c_int, buff: *mut c_char) {
    unsafe {
        let mut tl = 0usize;
        loop {
            let st = tsvalue(s2v(top.sub(n as usize)));
            let mut l = 0usize;
            let s = getlstr(st, &mut l);
            memcpy(buff.add(tl).cast(), s.cast(), l);
            tl += l;
            n -= 1;
            if n <= 0 {
                break;
            }
        }
    }
}

pub(crate) unsafe fn luaV_concat(L: *mut lua_State, mut total: c_int) {
    unsafe {
        if total == 1 {
            return;
        }
        loop {
            let top = (*L).top.p;
            let mut n = 2;
            if !(ttisstring(s2v(top.sub(2))) || cvt2str(s2v(top.sub(2))))
                || !tostring(L, s2v(top.sub(1)))
            {
                luaT_tryconcatTM(L);
            } else if isemptystr(s2v(top.sub(1))) {
                let _ = tostring(L, s2v(top.sub(2)));
            } else if isemptystr(s2v(top.sub(2))) {
                setobjs2s(L, top.sub(2), top.sub(1));
            } else {
                let mut tl = tsslen(tsvalue(s2v(top.sub(1))));
                n = 1;
                while n < total && tostring(L, s2v(top.sub((n + 1) as usize))) {
                    let l = tsslen(tsvalue(s2v(top.sub((n + 1) as usize))));
                    if l >= MAX_SIZE - size_of::<TString>() - tl {
                        (*L).top.p = top.sub(total as usize);
                        luaG_runerror(L, "string length overflow");
                    }
                    tl += l;
                    n += 1;
                }
                let ts = if tl <= LUAI_MAXSHORTLEN {
                    let mut buff = [0i8; LUAI_MAXSHORTLEN];
                    copy2buff(top, n, buff.as_mut_ptr());
                    luaS_newlstr(L, buff.as_ptr(), tl)
                } else {
                    let ts = luaS_createlngstrobj(L, tl);
                    copy2buff(top, n, getlngstr(ts));
                    ts
                };
                setsvalue2s(L, top.sub(n as usize), ts);
            }
            total -= n - 1;
            (*L).top.p = (*L).top.p.sub((n - 1) as usize);
            if total <= 1 {
                return;
            }
        }
    }
}

pub(crate) unsafe fn luaV_objlen(L: *mut lua_State, ra: StkId, rb: *const TValue) {
    unsafe {
        let tm;
        match ttypetag(rb) {
            LUA_VTABLE => {
                let h = hvalue(rb);
                tm = fasttm(L, (*h).metatable, TM_LEN);
                if tm.is_null() {
                    setivalue(s2v(ra), luaH_getn(L, h) as lua_Integer);
                    return;
                }
            }
            LUA_VSHRSTR => {
                setivalue(s2v(ra), (*tsvalue(rb)).shrlen as lua_Integer);
                return;
            }
            LUA_VLNGSTR => {
                setivalue(s2v(ra), (*tsvalue(rb)).u.lnglen as lua_Integer);
                return;
            }
            _ => {
                tm = luaT_gettmbyobj(L, rb, TM_LEN);
                if notm(tm) {
                    luaG_typeerror(L, rb, c"get length of".as_ptr());
                }
            }
        }
        let _ = luaT_callTMres(L, tm, rb, rb, ra);
    }
}

pub unsafe fn luaV_idiv(L: *mut lua_State, m: lua_Integer, n: lua_Integer) -> lua_Integer {
    unsafe {
        if (n as lua_Unsigned).wrapping_add(1) <= 1 {
            if n == 0 {
                luaG_runerror(L, "attempt to divide by zero");
            }
            0i64.wrapping_sub(m)
        } else {
            let mut q = m / n;
            if (m ^ n) < 0 && m % n != 0 {
                q -= 1;
            }
            q
        }
    }
}
pub unsafe fn luaV_mod(L: *mut lua_State, m: lua_Integer, n: lua_Integer) -> lua_Integer {
    unsafe {
        if (n as lua_Unsigned).wrapping_add(1) <= 1 {
            if n == 0 {
                luaG_runerror(L, "attempt to perform 'n%0'");
            }
            0
        } else {
            let mut r = m % n;
            if r != 0 && (r ^ n) < 0 {
                r += n;
            }
            r
        }
    }
}

pub unsafe fn luaV_modf(_L: *mut lua_State, m: lua_Number, n: lua_Number) -> lua_Number {
    let mut r = m % n;
    if if r > 0.0 { n < 0.0 } else { r < 0.0 && n > 0.0 } {
        r += n;
    }
    r
}

pub unsafe fn luaV_shiftl(x: lua_Integer, y: lua_Integer) -> lua_Integer {
    let nbits = lua_Integer::BITS as lua_Integer;
    if y < 0 {
        if y <= -nbits {
            0
        } else {
            ((x as lua_Unsigned) >> (-y as u32)) as lua_Integer
        }
    } else if y >= nbits {
        0
    } else {
        ((x as lua_Unsigned) << (y as u32)) as lua_Integer
    }
}

unsafe fn pushclosure(
    L: *mut lua_State,
    p: *mut Proto,
    encup: *mut *mut UpVal,
    base: StkId,
    ra: StkId,
) {
    unsafe {
        let nup = (*p).sizeupvalues;
        let uv = (*p).upvalues;
        let ncl = luaF_newLclosure(L, nup);
        (*ncl).p = p;
        setclLvalue2s(L, ra, ncl);
        for i in 0..nup as usize {
            let uvd = uv.add(i);
            *(*ncl).upvals.as_mut_ptr().add(i) = if (*uvd).instack != 0 {
                luaF_findupval(L, base.add((*uvd).idx as usize))
            } else {
                *encup.add((*uvd).idx as usize)
            };
            luaC_objbarrier(L, obj2gco(ncl), obj2gco(*(*ncl).upvals.as_mut_ptr().add(i)));
        }
    }
}

pub unsafe fn luaV_finishOp(L: *mut lua_State) {
    unsafe {
        let ci = (*L).ci;
        let base = (*ci).func.p.add(1);
        let inst = *(*ci).u.l.savedpc.sub(1);
        let op = GET_OPCODE(inst);
        match op {
            OP_MMBIN | OP_MMBINI | OP_MMBINK => {
                setobjs2s(L, base.add(GETARG_A(*(*ci).u.l.savedpc.sub(2)) as usize), {
                    (*L).top.p = (*L).top.p.sub(1);
                    (*L).top.p
                });
            }
            OP_UNM | OP_BNOT | OP_LEN | OP_GETTABUP | OP_GETTABLE | OP_GETI | OP_GETFIELD
            | OP_SELF => {
                setobjs2s(L, base.add(GETARG_A(inst) as usize), {
                    (*L).top.p = (*L).top.p.sub(1);
                    (*L).top.p
                });
            }
            OP_LT | OP_LE | OP_LTI | OP_LEI | OP_GTI | OP_GEI | OP_EQ => {
                let res = c_int::from(!l_isfalse(s2v((*L).top.p.sub(1))));
                (*L).top.p = (*L).top.p.sub(1);
                if res != GETARG_K(inst) {
                    (*ci).u.l.savedpc = (*ci).u.l.savedpc.add(1);
                }
            }
            OP_CONCAT => {
                let top = (*L).top.p.sub(1);
                let a = GETARG_A(inst);
                let total = top.sub(1).offset_from(base.add(a as usize)) as c_int;
                setobjs2s(L, top.sub(2), top);
                (*L).top.p = top.sub(1);
                luaV_concat(L, total);
            }
            OP_CLOSE => {
                (*ci).u.l.savedpc = (*ci).u.l.savedpc.sub(1);
            }
            OP_RETURN => {
                let ra = base.add(GETARG_A(inst) as usize);
                (*L).top.p = ra.add((*ci).u2.nres as usize);
                (*ci).u.l.savedpc = (*ci).u.l.savedpc.sub(1);
            }
            _ => {}
        }
    }
}

pub unsafe fn luaV_execute(L: *mut lua_State, mut ci: *mut CallInfo) {
    unsafe {
        let mut trap = 0;
        let mut keep_trap = false;
        'newframe: loop {
            if !keep_trap {
                trap = (*L).hookmask;
            }
            keep_trap = false;
            let cl = ci_func(ci);
            let k = (*(*cl).p).k;
            let mut pc = (*ci).u.l.savedpc;
            if trap != 0 {
                trap = luaG_tracecall(L);
            }
            let mut base = (*ci).func.p.add(1);
            loop {
                if trap != 0 {
                    trap = luaG_traceexec(L, pc);
                    updatebase(ci, &mut base);
                }
                let i = *pc;
                pc = pc.add(1);
                debug_assert!(base == (*ci).func.p.add(1));
                debug_assert!(base <= (*L).top.p && (*L).top.p <= (*L).stack_last.p);
                debug_assert!(
                    luaP_isIT(i) != 0 || {
                        (*L).top.p = base;
                        true
                    }
                );
                match GET_OPCODE(i) {
                    OP_MOVE => {
                        let ra = RA(base, i);
                        setobjs2s(L, ra, RB(base, i));
                    }
                    OP_LOADI => {
                        setivalue(s2v(RA(base, i)), GETARG_SBX(i) as lua_Integer);
                    }
                    OP_LOADF => {
                        setfltvalue(s2v(RA(base, i)), GETARG_SBX(i) as lua_Number);
                    }
                    OP_LOADK => {
                        setobj2s(L, RA(base, i), k.add(GETARG_BX(i) as usize));
                    }
                    OP_LOADKX => {
                        setobj2s(L, RA(base, i), k.add(GETARG_AX(*pc) as usize));
                        pc = pc.add(1);
                    }
                    OP_LOADFALSE => setbfvalue(s2v(RA(base, i))),
                    OP_LFALSESKIP => {
                        setbfvalue(s2v(RA(base, i)));
                        pc = pc.add(1);
                    }
                    OP_LOADTRUE => setbtvalue(s2v(RA(base, i))),
                    OP_LOADNIL => {
                        let mut ra = RA(base, i);
                        let mut b = GETARG_B(i);
                        loop {
                            setnilvalue(s2v(ra));
                            if b == 0 {
                                break;
                            }
                            b -= 1;
                            ra = ra.add(1);
                        }
                    }
                    OP_GETUPVAL => {
                        let b = GETARG_B(i) as usize;
                        setobj2s(L, RA(base, i), (*(*(*cl).upvals.as_mut_ptr().add(b))).v.p);
                    }
                    OP_SETUPVAL => {
                        let ra = RA(base, i);
                        let uv = *(*cl).upvals.as_mut_ptr().add(GETARG_B(i) as usize);
                        setobj((*uv).v.p, s2v(ra));
                        luaC_barrier(L, obj2gco(uv), s2v(ra));
                    }
                    OP_GETTABUP => {
                        let ra = RA(base, i);
                        let upval = (*(*(*cl).upvals.as_mut_ptr().add(GETARG_B(i) as usize)))
                            .v
                            .p;
                        let rc = KC(k, i);
                        let mut tag = if !ttistable(upval) {
                            LUA_VNOTABLE
                        } else {
                            luaH_getshortstr(hvalue(upval), tsvalue(rc), s2v(ra))
                        };
                        if tagisempty(tag) {
                            savestate(L, ci, pc);
                            tag = luaV_finishget(L, upval, rc, ra, tag);
                            updatetrap(ci, &mut trap);
                            let _ = tag;
                        }
                    }
                    OP_GETTABLE => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        let rc = s2v(RC(base, i));
                        let mut tag = if ttisinteger(rc) {
                            if !ttistable(rb) {
                                LUA_VNOTABLE
                            } else {
                                luaH_getint(hvalue(rb), ivalue(rc), s2v(ra))
                            }
                        } else if !ttistable(rb) {
                            LUA_VNOTABLE
                        } else {
                            luaH_get(hvalue(rb), rc, s2v(ra))
                        };
                        if tagisempty(tag) {
                            savestate(L, ci, pc);
                            tag = luaV_finishget(L, rb, rc, ra, tag);
                            updatetrap(ci, &mut trap);
                            let _ = tag;
                        }
                    }
                    OP_GETI => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        let c = GETARG_C(i) as lua_Integer;
                        let mut tag = if !ttistable(rb) {
                            LUA_VNOTABLE
                        } else {
                            luaH_getint(hvalue(rb), c, s2v(ra))
                        };
                        if tagisempty(tag) {
                            let mut key = TValue {
                                value_: Value { i: 0 },
                                tt_: LUA_VNIL,
                            };
                            setivalue(&mut key, c);
                            savestate(L, ci, pc);
                            tag = luaV_finishget(L, rb, &mut key, ra, tag);
                            updatetrap(ci, &mut trap);
                            let _ = tag;
                        }
                    }
                    OP_GETFIELD => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        let rc = KC(k, i);
                        let mut tag = if !ttistable(rb) {
                            LUA_VNOTABLE
                        } else {
                            luaH_getshortstr(hvalue(rb), tsvalue(rc), s2v(ra))
                        };
                        if tagisempty(tag) {
                            savestate(L, ci, pc);
                            tag = luaV_finishget(L, rb, rc, ra, tag);
                            updatetrap(ci, &mut trap);
                            let _ = tag;
                        }
                    }
                    OP_SETTABUP => {
                        let upval = (*(*(*cl).upvals.as_mut_ptr().add(GETARG_A(i) as usize)))
                            .v
                            .p;
                        let rb = KB(k, i);
                        let rc = RKC(base, k, i);
                        let hres = if !ttistable(upval) {
                            HNOTATABLE
                        } else {
                            luaH_psetshortstr(hvalue(upval), tsvalue(rb), rc)
                        };
                        if hres == HOK {
                            luaV_finishfastset(L, upval, rc);
                        } else {
                            savestate(L, ci, pc);
                            luaV_finishset(L, upval, rb, rc, hres);
                            updatetrap(ci, &mut trap);
                        }
                    }
                    OP_SETTABLE => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        let rc = RKC(base, k, i);
                        let hres = if ttisinteger(rb) {
                            luaV_fastseti(s2v(ra), ivalue(rb), rc)
                        } else if !ttistable(s2v(ra)) {
                            HNOTATABLE
                        } else {
                            luaH_pset(hvalue(s2v(ra)), rb, rc)
                        };
                        if hres == HOK {
                            luaV_finishfastset(L, s2v(ra), rc);
                        } else {
                            savestate(L, ci, pc);
                            luaV_finishset(L, s2v(ra), rb, rc, hres);
                            updatetrap(ci, &mut trap);
                        }
                    }
                    OP_SETI => {
                        let ra = RA(base, i);
                        let b = GETARG_B(i) as lua_Integer;
                        let rc = RKC(base, k, i);
                        let hres = luaV_fastseti(s2v(ra), b, rc);
                        if hres == HOK {
                            luaV_finishfastset(L, s2v(ra), rc);
                        } else {
                            let mut key = TValue {
                                value_: Value { i: 0 },
                                tt_: LUA_VNIL,
                            };
                            setivalue(&mut key, b);
                            savestate(L, ci, pc);
                            luaV_finishset(L, s2v(ra), &mut key, rc, hres);
                            updatetrap(ci, &mut trap);
                        }
                    }
                    OP_SETFIELD => {
                        let ra = RA(base, i);
                        let rb = KB(k, i);
                        let rc = RKC(base, k, i);
                        let hres = if !ttistable(s2v(ra)) {
                            HNOTATABLE
                        } else {
                            luaH_psetshortstr(hvalue(s2v(ra)), tsvalue(rb), rc)
                        };
                        if hres == HOK {
                            luaV_finishfastset(L, s2v(ra), rc);
                        } else {
                            savestate(L, ci, pc);
                            luaV_finishset(L, s2v(ra), rb, rc, hres);
                            updatetrap(ci, &mut trap);
                        }
                    }
                    OP_NEWTABLE => {
                        let ra = RA(base, i);
                        let mut b = GETARG_VB(i) as u32;
                        let mut c = GETARG_VC(i) as u32;
                        if b > 0 {
                            b = 1u32 << (b - 1);
                        }
                        if TESTARG_K(i) {
                            c += GETARG_AX(*pc) as u32 * (MAXARG_VC as u32 + 1);
                        }
                        pc = pc.add(1);
                        (*L).top.p = ra.add(1);
                        let t = luaH_new(L);
                        sethvalue2s(L, ra, t);
                        if b != 0 || c != 0 {
                            luaH_resize(L, t, c, b);
                        }
                        checkGC(L, ci, pc, &mut trap, ra.add(1));
                    }
                    OP_SELF => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        let rc = KC(k, i);
                        setobj2s(L, ra.add(1), rb);
                        let mut tag = if !ttistable(rb) {
                            LUA_VNOTABLE
                        } else {
                            luaH_getshortstr(hvalue(rb), tsvalue(rc), s2v(ra))
                        };
                        if tagisempty(tag) {
                            savestate(L, ci, pc);
                            tag = luaV_finishget(L, rb, rc, ra, tag);
                            updatetrap(ci, &mut trap);
                            let _ = tag;
                        }
                    }
                    OP_ADDI | OP_ADDK | OP_SUBK | OP_MULK | OP_MODK | OP_POWK | OP_DIVK
                    | OP_IDIVK | OP_BANDK | OP_BORK | OP_BXORK | OP_SHLI | OP_SHRI | OP_ADD
                    | OP_SUB | OP_MUL | OP_MOD | OP_POW | OP_DIV | OP_IDIV | OP_BAND | OP_BOR
                    | OP_BXOR | OP_SHL | OP_SHR => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        let rc = if matches!(
                            GET_OPCODE(i),
                            OP_ADDK
                                | OP_SUBK
                                | OP_MULK
                                | OP_MODK
                                | OP_POWK
                                | OP_DIVK
                                | OP_IDIVK
                                | OP_BANDK
                                | OP_BORK
                                | OP_BXORK
                        ) {
                            KC(k, i)
                        } else if matches!(GET_OPCODE(i), OP_ADDI | OP_SHLI | OP_SHRI) {
                            core::ptr::null_mut()
                        } else {
                            s2v(RC(base, i))
                        };
                        let mut i1 = 0;
                        let mut i2 = 0;
                        let mut n1 = 0.0;
                        let mut n2 = 0.0;
                        let opcode = GET_OPCODE(i);
                        if matches!(opcode, OP_MODK | OP_IDIVK | OP_MOD | OP_IDIV) {
                            savestate(L, ci, pc);
                        }
                        match opcode {
                            OP_ADDI => {
                                let imm = GETARG_SC(i) as lua_Integer;
                                if ttisinteger(rb) {
                                    setivalue(s2v(ra), ivalue(rb).wrapping_add(imm));
                                    pc = pc.add(1);
                                } else if ttisfloat(rb) {
                                    setfltvalue(s2v(ra), fltvalue(rb) + imm as lua_Number);
                                    pc = pc.add(1);
                                }
                            }
                            OP_SHLI => {
                                let imm = GETARG_SC(i) as lua_Integer;
                                if tointegerns(rb, &mut i1) != 0 {
                                    setivalue(s2v(ra), luaV_shiftl(imm, i1));
                                    pc = pc.add(1);
                                }
                            }
                            OP_SHRI => {
                                let imm = GETARG_SC(i) as lua_Integer;
                                if tointegerns(rb, &mut i1) != 0 {
                                    setivalue(s2v(ra), luaV_shiftl(i1, -imm));
                                    pc = pc.add(1);
                                }
                            }
                            OP_ADDK | OP_SUBK | OP_MULK | OP_MODK | OP_IDIVK | OP_ADD | OP_SUB
                            | OP_MUL | OP_MOD | OP_IDIV => {
                                if ttisinteger(rb) && ttisinteger(rc) {
                                    i1 = ivalue(rb);
                                    i2 = ivalue(rc);
                                    let res = match opcode {
                                        OP_ADDK | OP_ADD => i1.wrapping_add(i2),
                                        OP_SUBK | OP_SUB => i1.wrapping_sub(i2),
                                        OP_MULK | OP_MUL => i1.wrapping_mul(i2),
                                        OP_MODK | OP_MOD => luaV_mod(L, i1, i2),
                                        _ => luaV_idiv(L, i1, i2),
                                    };
                                    setivalue(s2v(ra), res);
                                    pc = pc.add(1);
                                } else if tonumberns(rb, &mut n1) != 0
                                    && tonumberns(rc, &mut n2) != 0
                                {
                                    let res = match opcode {
                                        OP_ADDK | OP_ADD => n1 + n2,
                                        OP_SUBK | OP_SUB => n1 - n2,
                                        OP_MULK | OP_MUL => n1 * n2,
                                        OP_MODK | OP_MOD => luaV_modf(L, n1, n2),
                                        _ => (n1 / n2).floor(),
                                    };
                                    setfltvalue(s2v(ra), res);
                                    pc = pc.add(1);
                                }
                            }
                            OP_POWK | OP_DIVK | OP_POW | OP_DIV => {
                                if tonumberns(rb, &mut n1) != 0 && tonumberns(rc, &mut n2) != 0 {
                                    setfltvalue(
                                        s2v(ra),
                                        if matches!(opcode, OP_POWK | OP_POW) {
                                            if n2 == 2.0 { n1 * n1 } else { n1.powf(n2) }
                                        } else {
                                            n1 / n2
                                        },
                                    );
                                    pc = pc.add(1);
                                }
                            }
                            OP_BANDK | OP_BORK | OP_BXORK | OP_BAND | OP_BOR | OP_BXOR | OP_SHL
                            | OP_SHR => {
                                if tointegerns(rb, &mut i1) != 0 && tointegerns(rc, &mut i2) != 0 {
                                    let res = match opcode {
                                        OP_BANDK | OP_BAND => {
                                            ((i1 as lua_Unsigned) & (i2 as lua_Unsigned))
                                                as lua_Integer
                                        }
                                        OP_BORK | OP_BOR => {
                                            ((i1 as lua_Unsigned) | (i2 as lua_Unsigned))
                                                as lua_Integer
                                        }
                                        OP_BXORK | OP_BXOR => {
                                            ((i1 as lua_Unsigned) ^ (i2 as lua_Unsigned))
                                                as lua_Integer
                                        }
                                        OP_SHL => luaV_shiftl(i1, i2),
                                        _ => luaV_shiftl(i1, -i2),
                                    };
                                    setivalue(s2v(ra), res);
                                    pc = pc.add(1);
                                }
                            }
                            _ => {}
                        }
                    }
                    OP_MMBIN => {
                        let ra = RA(base, i);
                        let pi = *pc.sub(2);
                        let rb = s2v(RB(base, i));
                        savestate(L, ci, pc);
                        luaT_trybinTM(L, s2v(ra), rb, RA(base, pi), GETARG_C(i));
                        updatetrap(ci, &mut trap);
                    }
                    OP_MMBINI => {
                        let ra = RA(base, i);
                        let pi = *pc.sub(2);
                        savestate(L, ci, pc);
                        luaT_trybiniTM(
                            L,
                            s2v(ra),
                            GETARG_SB(i) as lua_Integer,
                            GETARG_K(i),
                            RA(base, pi),
                            GETARG_C(i),
                        );
                        updatetrap(ci, &mut trap);
                    }
                    OP_MMBINK => {
                        let ra = RA(base, i);
                        let pi = *pc.sub(2);
                        savestate(L, ci, pc);
                        luaT_trybinassocTM(
                            L,
                            s2v(ra),
                            KB(k, i),
                            GETARG_K(i),
                            RA(base, pi),
                            GETARG_C(i),
                        );
                        updatetrap(ci, &mut trap);
                    }
                    OP_UNM => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        let mut nb = 0.0;
                        if ttisinteger(rb) {
                            setivalue(s2v(ra), 0i64.wrapping_sub(ivalue(rb)));
                        } else if tonumberns(rb, &mut nb) != 0 {
                            setfltvalue(s2v(ra), -nb);
                        } else {
                            savestate(L, ci, pc);
                            luaT_trybinTM(L, rb, rb, ra, TM_UNM);
                            updatetrap(ci, &mut trap);
                        }
                    }
                    OP_BNOT => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        let mut ib = 0;
                        if tointegerns(rb, &mut ib) != 0 {
                            setivalue(
                                s2v(ra),
                                (!(0 as lua_Unsigned) ^ ib as lua_Unsigned) as lua_Integer,
                            );
                        } else {
                            savestate(L, ci, pc);
                            luaT_trybinTM(L, rb, rb, ra, TM_BNOT);
                            updatetrap(ci, &mut trap);
                        }
                    }
                    OP_NOT => {
                        if l_isfalse(s2v(RB(base, i))) {
                            setbtvalue(s2v(RA(base, i)));
                        } else {
                            setbfvalue(s2v(RA(base, i)));
                        }
                    }
                    OP_LEN => {
                        let ra = RA(base, i);
                        savestate(L, ci, pc);
                        luaV_objlen(L, ra, s2v(RB(base, i)));
                        updatetrap(ci, &mut trap);
                    }
                    OP_CONCAT => {
                        let ra = RA(base, i);
                        let n = GETARG_B(i);
                        (*L).top.p = ra.add(n as usize);
                        savepc(ci, pc);
                        luaV_concat(L, n);
                        updatetrap(ci, &mut trap);
                        checkGC(L, ci, pc, &mut trap, (*L).top.p);
                    }
                    OP_CLOSE => {
                        let ra = RA(base, i);
                        savestate(L, ci, pc);
                        let _ = luaF_close(L, ra, LUA_OK, 1);
                        updatetrap(ci, &mut trap);
                    }
                    OP_TBC => {
                        let ra = RA(base, i);
                        savestate(L, ci, pc);
                        luaF_newtbcupval(L, ra);
                    }
                    OP_JMP => dojump(ci, i, 0, &mut pc, &mut trap),
                    OP_EQ => {
                        savestate(L, ci, pc);
                        let cond = luaV_equalobj(L, s2v(RA(base, i)), s2v(RB(base, i)));
                        updatetrap(ci, &mut trap);
                        docondjump(cond, ci, i, &mut pc, &mut trap);
                    }
                    OP_LT => {
                        let ra = s2v(RA(base, i));
                        let rb = s2v(RB(base, i));
                        let cond = if ttisinteger(ra) && ttisinteger(rb) {
                            c_int::from(ivalue(ra) < ivalue(rb))
                        } else if ttisnumber(ra) && ttisnumber(rb) {
                            LTnum(ra, rb)
                        } else {
                            savestate(L, ci, pc);
                            let cond = lessthanothers(L, ra, rb);
                            updatetrap(ci, &mut trap);
                            cond
                        };
                        docondjump(cond, ci, i, &mut pc, &mut trap);
                    }
                    OP_LE => {
                        let ra = s2v(RA(base, i));
                        let rb = s2v(RB(base, i));
                        let cond = if ttisinteger(ra) && ttisinteger(rb) {
                            c_int::from(ivalue(ra) <= ivalue(rb))
                        } else if ttisnumber(ra) && ttisnumber(rb) {
                            LEnum(ra, rb)
                        } else {
                            savestate(L, ci, pc);
                            let cond = lessequalothers(L, ra, rb);
                            updatetrap(ci, &mut trap);
                            cond
                        };
                        docondjump(cond, ci, i, &mut pc, &mut trap);
                    }
                    OP_EQK => {
                        let cond = luaV_rawequalobj(s2v(RA(base, i)), KB(k, i));
                        docondjump(cond, ci, i, &mut pc, &mut trap);
                    }
                    OP_EQI | OP_LTI | OP_LEI | OP_GTI | OP_GEI => {
                        let ra = s2v(RA(base, i));
                        let im = GETARG_SB(i);
                        let cond = if ttisinteger(ra) {
                            let v = ivalue(ra);
                            match GET_OPCODE(i) {
                                OP_EQI => c_int::from(v == im as lua_Integer),
                                OP_LTI => c_int::from(v < im as lua_Integer),
                                OP_LEI => c_int::from(v <= im as lua_Integer),
                                OP_GTI => c_int::from(v > im as lua_Integer),
                                _ => c_int::from(v >= im as lua_Integer),
                            }
                        } else if ttisfloat(ra) {
                            let v = fltvalue(ra);
                            let imf = im as lua_Number;
                            match GET_OPCODE(i) {
                                OP_EQI => c_int::from(v == imf),
                                OP_LTI => c_int::from(v < imf),
                                OP_LEI => c_int::from(v <= imf),
                                OP_GTI => c_int::from(v > imf),
                                _ => c_int::from(v >= imf),
                            }
                        } else if GET_OPCODE(i) == OP_EQI {
                            0
                        } else {
                            savestate(L, ci, pc);
                            let cond = luaT_callorderiTM(
                                L,
                                ra,
                                im,
                                if matches!(GET_OPCODE(i), OP_GTI | OP_GEI) {
                                    1
                                } else {
                                    0
                                },
                                GETARG_C(i),
                                if matches!(GET_OPCODE(i), OP_LTI | OP_GTI) {
                                    TM_LT
                                } else {
                                    TM_LE
                                },
                            );
                            updatetrap(ci, &mut trap);
                            cond
                        };
                        docondjump(cond, ci, i, &mut pc, &mut trap);
                    }
                    OP_TEST => {
                        let cond = c_int::from(!l_isfalse(s2v(RA(base, i))));
                        docondjump(cond, ci, i, &mut pc, &mut trap);
                    }
                    OP_TESTSET => {
                        let ra = RA(base, i);
                        let rb = s2v(RB(base, i));
                        if c_int::from(l_isfalse(rb)) == GETARG_K(i) {
                            pc = pc.add(1);
                        } else {
                            setobj2s(L, ra, rb);
                            donextjump(ci, &mut pc, &mut trap);
                        }
                    }
                    OP_CALL => {
                        let ra = RA(base, i);
                        let b = GETARG_B(i);
                        let nresults = GETARG_C(i) - 1;
                        if b != 0 {
                            (*L).top.p = ra.add(b as usize);
                        }
                        savepc(ci, pc);
                        let newci = luaD_precall(L, ra, nresults);
                        if newci.is_null() {
                            updatetrap(ci, &mut trap);
                        } else {
                            ci = newci;
                            continue 'newframe;
                        }
                    }
                    OP_TAILCALL => {
                        let ra = RA(base, i);
                        let mut b = GETARG_B(i);
                        let nparams1 = GETARG_C(i);
                        let delta = if nparams1 != 0 {
                            (*ci).u.l.nextraargs + nparams1
                        } else {
                            0
                        };
                        if b != 0 {
                            (*L).top.p = ra.add(b as usize);
                        } else {
                            b = (*L).top.p.offset_from(ra) as c_int;
                        }
                        savepc(ci, pc);
                        if TESTARG_K(i) {
                            luaF_closeupval(L, base);
                        }
                        let n = luaD_pretailcall(L, ci, ra, b, delta);
                        if n < 0 {
                            continue 'newframe;
                        } else {
                            (*ci).func.p = (*ci).func.p.sub(delta as usize);
                            luaD_poscall(L, ci, n);
                            updatetrap(ci, &mut trap);
                            if (*ci).callstatus & CIST_FRESH != 0 {
                                return;
                            }
                            ci = (*ci).previous;
                            keep_trap = true;
                            continue 'newframe;
                        }
                    }
                    OP_RETURN => {
                        let ra = RA(base, i);
                        let mut n = GETARG_B(i) - 1;
                        let nparams1 = GETARG_C(i);
                        if n < 0 {
                            n = (*L).top.p.offset_from(ra) as c_int;
                        }
                        savepc(ci, pc);
                        if TESTARG_K(i) {
                            (*ci).u2.nres = n;
                            if (*L).top.p < (*ci).top.p {
                                (*L).top.p = (*ci).top.p;
                            }
                            let _ = luaF_close(L, base, CLOSEKTOP, 1);
                            updatetrap(ci, &mut trap);
                            updatebase(ci, &mut base);
                        }
                        if nparams1 != 0 {
                            (*ci).func.p =
                                (*ci).func.p.sub(((*ci).u.l.nextraargs + nparams1) as usize);
                        }
                        (*L).top.p = ra.add(n as usize);
                        luaD_poscall(L, ci, n);
                        updatetrap(ci, &mut trap);
                        if (*ci).callstatus & CIST_FRESH != 0 {
                            return;
                        }
                        ci = (*ci).previous;
                        keep_trap = true;
                        continue 'newframe;
                    }
                    OP_RETURN0 => {
                        if (*L).hookmask != 0 {
                            (*L).top.p = RA(base, i);
                            savepc(ci, pc);
                            luaD_poscall(L, ci, 0);
                            trap = 1;
                        } else {
                            let mut nres = get_nresults((*ci).callstatus);
                            (*L).ci = (*ci).previous;
                            (*L).top.p = base.sub(1);
                            while nres > 0 {
                                setnilvalue(s2v((*L).top.p));
                                (*L).top.p = (*L).top.p.add(1);
                                nres -= 1;
                            }
                        }
                        if (*ci).callstatus & CIST_FRESH != 0 {
                            return;
                        }
                        ci = (*ci).previous;
                        keep_trap = true;
                        continue 'newframe;
                    }
                    OP_RETURN1 => {
                        if (*L).hookmask != 0 {
                            let ra = RA(base, i);
                            (*L).top.p = ra.add(1);
                            savepc(ci, pc);
                            luaD_poscall(L, ci, 1);
                            trap = 1;
                        } else {
                            let nres = get_nresults((*ci).callstatus);
                            (*L).ci = (*ci).previous;
                            if nres == 0 {
                                (*L).top.p = base.sub(1);
                            } else {
                                let ra = RA(base, i);
                                setobjs2s(L, base.sub(1), ra);
                                (*L).top.p = base;
                                let mut left = nres;
                                while left > 1 {
                                    setnilvalue(s2v((*L).top.p));
                                    (*L).top.p = (*L).top.p.add(1);
                                    left -= 1;
                                }
                            }
                        }
                        if (*ci).callstatus & CIST_FRESH != 0 {
                            return;
                        }
                        ci = (*ci).previous;
                        keep_trap = true;
                        continue 'newframe;
                    }
                    OP_FORLOOP => {
                        let ra = RA(base, i);
                        if ttisinteger(s2v(ra.add(1))) {
                            let count = ivalue(s2v(ra)) as lua_Unsigned;
                            if count > 0 {
                                let step = ivalue(s2v(ra.add(1)));
                                let idx = ivalue(s2v(ra.add(2))).wrapping_add(step);
                                chgivalue(s2v(ra), (count - 1) as lua_Integer);
                                chgivalue(s2v(ra.add(2)), idx);
                                pc = pc.sub(GETARG_BX(i) as usize);
                            }
                        } else if floatforloop(ra) != 0 {
                            pc = pc.sub(GETARG_BX(i) as usize);
                        }
                        updatetrap(ci, &mut trap);
                    }
                    OP_FORPREP => {
                        let ra = RA(base, i);
                        savestate(L, ci, pc);
                        if forprep(L, ra) != 0 {
                            pc = pc.add(GETARG_BX(i) as usize + 1);
                        }
                    }
                    OP_TFORPREP => {
                        let ra = RA(base, i);
                        let mut temp = TValue {
                            value_: Value { i: 0 },
                            tt_: LUA_VNIL,
                        };
                        setobj(&mut temp, s2v(ra.add(3)));
                        setobjs2s(L, ra.add(3), ra.add(2));
                        setobj2s(L, ra.add(2), &temp);
                        savestate(L, ci, pc);
                        luaF_newtbcupval(L, ra.add(2));
                        pc = pc.add(GETARG_BX(i) as usize);
                        let i = *pc;
                        pc = pc.add(1);
                        debug_assert_eq!(GET_OPCODE(i), OP_TFORCALL);
                        setobjs2s(L, ra.add(5), ra.add(3));
                        setobjs2s(L, ra.add(4), ra.add(1));
                        setobjs2s(L, ra.add(3), ra);
                        (*L).top.p = ra.add(6);
                        savepc(ci, pc);
                        luaD_call(L, ra.add(3), GETARG_C(i));
                        updatetrap(ci, &mut trap);
                        updatebase(ci, &mut base);
                        let ni = *pc;
                        pc = pc.add(1);
                        debug_assert_eq!(GET_OPCODE(ni), OP_TFORLOOP);
                        if !ttisnil(s2v(RA(base, ni).add(3))) {
                            pc = pc.sub(GETARG_BX(ni) as usize);
                        }
                    }
                    OP_TFORCALL => {
                        let ra = RA(base, i);
                        setobjs2s(L, ra.add(5), ra.add(3));
                        setobjs2s(L, ra.add(4), ra.add(1));
                        setobjs2s(L, ra.add(3), ra);
                        (*L).top.p = ra.add(6);
                        savepc(ci, pc);
                        luaD_call(L, ra.add(3), GETARG_C(i));
                        updatetrap(ci, &mut trap);
                        updatebase(ci, &mut base);
                        let ni = *pc;
                        pc = pc.add(1);
                        debug_assert_eq!(GET_OPCODE(ni), OP_TFORLOOP);
                        if !ttisnil(s2v(RA(base, ni).add(3))) {
                            pc = pc.sub(GETARG_BX(ni) as usize);
                        }
                    }
                    OP_TFORLOOP => {
                        let ra = RA(base, i);
                        if !ttisnil(s2v(ra.add(3))) {
                            pc = pc.sub(GETARG_BX(i) as usize);
                        }
                    }
                    OP_SETLIST => {
                        let ra = RA(base, i);
                        let mut n = GETARG_VB(i) as u32;
                        let mut last = GETARG_VC(i) as u32;
                        let h = hvalue(s2v(ra));
                        if n == 0 {
                            n = (*L).top.p.offset_from(ra) as u32 - 1;
                        } else {
                            (*L).top.p = (*ci).top.p;
                        }
                        last += n;
                        if TESTARG_K(i) {
                            last += GETARG_AX(*pc) as u32 * (MAXARG_VC as u32 + 1);
                            pc = pc.add(1);
                        }
                        if last > (*h).asize {
                            luaH_resizearray(L, h, last);
                        }
                        while n > 0 {
                            let val = s2v(ra.add(n as usize));
                            obj2arr(h, last - 1, val);
                            luaC_barrierback(L, obj2gco(h), val);
                            last -= 1;
                            n -= 1;
                        }
                    }
                    OP_CLOSURE => {
                        let ra = RA(base, i);
                        let p = *(*(*cl).p).p.add(GETARG_BX(i) as usize);
                        savestate(L, ci, pc);
                        pushclosure(L, p, (*cl).upvals.as_mut_ptr(), base, ra);
                        checkGC(L, ci, pc, &mut trap, ra.add(1));
                    }
                    OP_VARARG => {
                        let ra = RA(base, i);
                        let n = GETARG_C(i) - 1;
                        let vatab = if GETARG_K(i) != 0 { GETARG_B(i) } else { -1 };
                        savestate(L, ci, pc);
                        luaT_getvarargs(L, ci, ra, n, vatab);
                        updatetrap(ci, &mut trap);
                    }
                    OP_GETVARG => {
                        luaT_getvararg(ci, RA(base, i), s2v(RC(base, i)));
                    }
                    OP_ERRNNIL => {
                        let ra = s2v(RA(base, i));
                        if !ttisnil(ra) {
                            savestate(L, ci, pc);
                            luaG_errnnil(L, cl, GETARG_BX(i));
                        }
                    }
                    OP_VARARGPREP => {
                        savepc(ci, pc);
                        luaT_adjustvarargs(L, ci, (*cl).p);
                        updatetrap(ci, &mut trap);
                        if trap != 0 {
                            luaD_hookcall(L, ci);
                            (*L).oldpc = 1;
                        }
                        updatebase(ci, &mut base);
                    }
                    OP_EXTRAARG => debug_assert!(false),
                    _ => unreachable!(),
                }
            }
        }
    }
}
