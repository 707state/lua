#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

use crate::opcodes::luaP_isIT;
use crate::runtime::*;
use core::mem::size_of;

const MAXTAGLOOP: c_int = 2000;

const F2Ieq: c_int = 0;
const F2Ifloor: c_int = 1;
const F2Iceil: c_int = 2;
const LUA_FLOORN2I: c_int = F2Ieq;

const LUA_VEMPTY: u8 = LUA_TNIL | (1 << 4);
const LUA_VABSTKEY: u8 = LUA_TNIL | (2 << 4);
const LUA_VNOTABLE: u8 = LUA_TNIL | (3 << 4);

const TM_INDEX: c_int = 0;
const TM_NEWINDEX: c_int = 1;
const TM_LEN: c_int = 4;
const TM_EQ: c_int = 5;
const TM_ADD: c_int = 6;
const TM_SUB: c_int = 7;
const TM_MUL: c_int = 8;
const TM_MOD: c_int = 9;
const TM_POW: c_int = 10;
const TM_DIV: c_int = 11;
const TM_IDIV: c_int = 12;
const TM_BAND: c_int = 13;
const TM_BOR: c_int = 14;
const TM_BXOR: c_int = 15;
const TM_SHL: c_int = 16;
const TM_SHR: c_int = 17;
const TM_UNM: c_int = 18;
const TM_BNOT: c_int = 19;
const TM_LT: c_int = 20;
const TM_LE: c_int = 21;

const PF_VATAB: u8 = 2;

const CIST_NRESULTS: u32 = 0xff;
const CIST_FRESH: u32 = CIST_C << 1;
const CIST_HOOKED: u32 = CIST_OAH << 1;
const CIST_TAIL: u32 = CIST_YPCALL << 1;
const CIST_HOOKYIELD: u32 = CIST_TAIL << 1;
const CIST_FIN: u32 = CIST_HOOKYIELD << 1;
const CLOSEKTOP: TStatus = LUA_ERRERR + 1;

const OP_MOVE: c_int = 0;
const OP_LOADI: c_int = 1;
const OP_LOADF: c_int = 2;
const OP_LOADK: c_int = 3;
const OP_LOADKX: c_int = 4;
const OP_LOADFALSE: c_int = 5;
const OP_LFALSESKIP: c_int = 6;
const OP_LOADTRUE: c_int = 7;
const OP_LOADNIL: c_int = 8;
const OP_GETUPVAL: c_int = 9;
const OP_SETUPVAL: c_int = 10;
const OP_GETTABUP: c_int = 11;
const OP_GETTABLE: c_int = 12;
const OP_GETI: c_int = 13;
const OP_GETFIELD: c_int = 14;
const OP_SETTABUP: c_int = 15;
const OP_SETTABLE: c_int = 16;
const OP_SETI: c_int = 17;
const OP_SETFIELD: c_int = 18;
const OP_NEWTABLE: c_int = 19;
const OP_SELF: c_int = 20;
const OP_ADDI: c_int = 21;
const OP_ADDK: c_int = 22;
const OP_SUBK: c_int = 23;
const OP_MULK: c_int = 24;
const OP_MODK: c_int = 25;
const OP_POWK: c_int = 26;
const OP_DIVK: c_int = 27;
const OP_IDIVK: c_int = 28;
const OP_BANDK: c_int = 29;
const OP_BORK: c_int = 30;
const OP_BXORK: c_int = 31;
const OP_SHLI: c_int = 32;
const OP_SHRI: c_int = 33;
const OP_ADD: c_int = 34;
const OP_SUB: c_int = 35;
const OP_MUL: c_int = 36;
const OP_MOD: c_int = 37;
const OP_POW: c_int = 38;
const OP_DIV: c_int = 39;
const OP_IDIV: c_int = 40;
const OP_BAND: c_int = 41;
const OP_BOR: c_int = 42;
const OP_BXOR: c_int = 43;
const OP_SHL: c_int = 44;
const OP_SHR: c_int = 45;
const OP_MMBIN: c_int = 46;
const OP_MMBINI: c_int = 47;
const OP_MMBINK: c_int = 48;
const OP_UNM: c_int = 49;
const OP_BNOT: c_int = 50;
const OP_NOT: c_int = 51;
const OP_LEN: c_int = 52;
const OP_CONCAT: c_int = 53;
const OP_CLOSE: c_int = 54;
const OP_TBC: c_int = 55;
const OP_JMP: c_int = 56;
const OP_EQ: c_int = 57;
const OP_LT: c_int = 58;
const OP_LE: c_int = 59;
const OP_EQK: c_int = 60;
const OP_EQI: c_int = 61;
const OP_LTI: c_int = 62;
const OP_LEI: c_int = 63;
const OP_GTI: c_int = 64;
const OP_GEI: c_int = 65;
const OP_TEST: c_int = 66;
const OP_TESTSET: c_int = 67;
const OP_CALL: c_int = 68;
const OP_TAILCALL: c_int = 69;
const OP_RETURN: c_int = 70;
const OP_RETURN0: c_int = 71;
const OP_RETURN1: c_int = 72;
const OP_FORLOOP: c_int = 73;
const OP_FORPREP: c_int = 74;
const OP_TFORPREP: c_int = 75;
const OP_TFORCALL: c_int = 76;
const OP_TFORLOOP: c_int = 77;
const OP_SETLIST: c_int = 78;
const OP_CLOSURE: c_int = 79;
const OP_VARARG: c_int = 80;
const OP_GETVARG: c_int = 81;
const OP_ERRNNIL: c_int = 82;
const OP_VARARGPREP: c_int = 83;
const OP_EXTRAARG: c_int = 84;

const SIZE_C: u32 = 8;
const SIZE_VC: u32 = 10;
const SIZE_B: u32 = 8;
const SIZE_VB: u32 = 6;
const SIZE_BX: u32 = SIZE_C + SIZE_B + 1;
const SIZE_A: u32 = 8;
const SIZE_AX: u32 = SIZE_BX + SIZE_A;
const SIZE_SJ: u32 = SIZE_BX + SIZE_A;
const SIZE_OP: u32 = 7;

const POS_OP: u32 = 0;
const POS_A: u32 = POS_OP + SIZE_OP;
const POS_K: u32 = POS_A + SIZE_A;
const POS_B: u32 = POS_K + 1;
const POS_VB: u32 = POS_K + 1;
const POS_C: u32 = POS_B + SIZE_B;
const POS_VC: u32 = POS_VB + SIZE_VB;
const POS_BX: u32 = POS_K;
const POS_AX: u32 = POS_A;
const POS_SJ: u32 = POS_A;

const MAXARG_BX: c_int = ((1u32 << SIZE_BX) - 1) as c_int;
const MAXARG_VC: c_int = ((1u32 << SIZE_VC) - 1) as c_int;
const OFFSET_SBX: c_int = MAXARG_BX >> 1;
const MAXARG_SJ: c_int = ((1u32 << SIZE_SJ) - 1) as c_int;
const OFFSET_SJ: c_int = MAXARG_SJ >> 1;
const MAXARG_C: c_int = ((1u32 << SIZE_C) - 1) as c_int;
const OFFSET_SC: c_int = MAXARG_C >> 1;

const LUAI_MAXSHORTLEN: usize = 40;

unsafe extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcoll(s1: *const c_char, s2: *const c_char) -> c_int;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    fn luaD_hookcall(L: *mut lua_State, ci: *mut CallInfo);
    fn luaD_pretailcall(
        L: *mut lua_State,
        ci: *mut CallInfo,
        func: StkId,
        narg1: c_int,
        delta: c_int,
    ) -> c_int;
    fn luaD_precall(L: *mut lua_State, func: StkId, nresults: c_int) -> *mut CallInfo;
    fn luaD_poscall(L: *mut lua_State, ci: *mut CallInfo, nres: c_int);

    fn luaG_typeerror(L: *mut lua_State, o: *const TValue, op: *const c_char) -> !;
    fn luaG_forerror(L: *mut lua_State, o: *const TValue, what: *const c_char) -> !;
    fn luaG_runerror(L: *mut lua_State, fmt: *const c_char, ...) -> !;
    fn luaG_tracecall(L: *mut lua_State) -> c_int;
    fn luaG_traceexec(L: *mut lua_State, pc: *const Instruction) -> c_int;
    fn luaG_errnnil(L: *mut lua_State, cl: *mut LClosure, k: c_int) -> !;

    fn luaT_gettm(events: *mut Table, event: c_int, ename: *mut TString) -> *const TValue;
    fn luaT_gettmbyobj(L: *mut lua_State, o: *const TValue, event: c_int) -> *const TValue;
    fn luaT_callTM(L: *mut lua_State, f: *const TValue, p1: *const TValue, p2: *const TValue, p3: *const TValue);
    fn luaT_callTMres(
        L: *mut lua_State,
        f: *const TValue,
        p1: *const TValue,
        p2: *const TValue,
        p3: StkId,
    ) -> lu_byte;
    fn luaT_trybinTM(L: *mut lua_State, p1: *const TValue, p2: *const TValue, res: StkId, event: c_int);
    fn luaT_trybiniTM(L: *mut lua_State, p1: *const TValue, i2: lua_Integer, inv: c_int, res: StkId, event: c_int);
    fn luaT_trybinassocTM(L: *mut lua_State, p1: *const TValue, p2: *const TValue, inv: c_int, res: StkId, event: c_int);
    fn luaT_tryconcatTM(L: *mut lua_State);
    fn luaT_callorderTM(L: *mut lua_State, p1: *const TValue, p2: *const TValue, event: c_int) -> c_int;
    fn luaT_callorderiTM(L: *mut lua_State, p1: *const TValue, v2: c_int, inv: c_int, isfloat: c_int, event: c_int) -> c_int;
    fn luaT_adjustvarargs(L: *mut lua_State, ci: *mut CallInfo, p: *const Proto);
    fn luaT_getvararg(ci: *mut CallInfo, ra: StkId, rc: *mut TValue);
    fn luaT_getvarargs(L: *mut lua_State, ci: *mut CallInfo, where_: StkId, wanted: c_int, vatab: c_int);

    fn luaF_newLclosure(L: *mut lua_State, nupvals: c_int) -> *mut LClosure;
    fn luaF_findupval(L: *mut lua_State, level: StkId) -> *mut UpVal;
    fn luaF_closeupval(L: *mut lua_State, level: StkId);

    fn luaS_eqstr(a: *mut TString, b: *mut TString) -> c_int;
    fn luaS_createlngstrobj(L: *mut lua_State, len: usize) -> *mut TString;

    fn luaH_resizearray(L: *mut lua_State, t: *mut Table, nasize: u32);
    fn luaH_getshortstr(t: *mut Table, key: *mut TString, res: *mut TValue) -> lu_byte;
    fn luaH_psetshortstr(t: *mut Table, key: *mut TString, val: *mut TValue) -> c_int;
}

#[inline]
unsafe fn ci_func(ci: *mut CallInfo) -> *mut LClosure {
    clLvalue(s2v((*ci).func.p))
}

#[inline]
unsafe fn get_nresults(cs: u32) -> c_int {
    (cs & CIST_NRESULTS) as c_int - 1
}

#[inline]
unsafe fn setclLvalue2s(_L: *mut lua_State, o: StkId, cl: *mut LClosure) {
    (*s2v(o)).value_.gc = cl.cast();
    settt_(s2v(o), LUA_VLCL | BIT_ISCOLLECTABLE);
}

#[inline]
unsafe fn setsvalue(o: *mut TValue, s: *mut TString) {
    (*o).value_.gc = s.cast();
    settt_(o, (*s).tt | BIT_ISCOLLECTABLE);
}

#[inline]
unsafe fn chgivalue(o: *mut TValue, x: lua_Integer) {
    (*o).value_.i = x;
}

#[inline]
unsafe fn chgfltvalue(o: *mut TValue, x: lua_Number) {
    (*o).value_.n = x;
}

#[inline]
unsafe fn ttisfunction(o: *const TValue) -> bool {
    ttype(o) == LUA_TFUNCTION
}

#[inline]
unsafe fn cvt2num(o: *const TValue) -> bool {
    ttisstring(o)
}

#[inline]
unsafe fn tonumberns(o: *const TValue, n: *mut lua_Number) -> c_int {
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

#[inline]
unsafe fn tointegerns(o: *const TValue, i: *mut lua_Integer) -> c_int {
    if ttisinteger(o) {
        *i = ivalue(o);
        1
    } else {
        luaV_tointegerns(o, i, LUA_FLOORN2I)
    }
}

#[inline]
unsafe fn strisshr(s: *mut TString) -> bool {
    (*s).shrlen >= 0
}

#[inline]
unsafe fn tsslen(s: *mut TString) -> usize {
    if strisshr(s) {
        (*s).shrlen as usize
    } else {
        (*s).u.lnglen
    }
}

#[inline]
unsafe fn getlngstr(s: *mut TString) -> *mut c_char {
    (*s).contents
}

#[inline]
unsafe fn eqshrstr(a: *mut TString, b: *mut TString) -> c_int {
    c_int::from(core::ptr::eq(a, b))
}

#[inline]
unsafe fn checknoTM(mt: *mut Table, e: c_int) -> bool {
    mt.is_null() || ((*mt).flags & (1u8 << e)) != 0
}

#[inline]
unsafe fn notm(tm: *const TValue) -> bool {
    ttisnil(tm)
}

#[inline]
unsafe fn fasttm(L: *mut lua_State, mt: *mut Table, e: c_int) -> *const TValue {
    if checknoTM(mt, e) {
        core::ptr::null()
    } else {
        luaT_gettm(mt, e, (&mut (*G(L)).tmname)[e as usize])
    }
}

#[inline]
unsafe fn invalidateTMcache(t: *mut Table) {
    (*t).flags &= !MASKFLAGS;
}

#[inline]
unsafe fn getArrTag(t: *mut Table, k: u32) -> *mut u8 {
    (*t).array.cast::<u8>().add(size_of::<u32>() + k as usize)
}

#[inline]
unsafe fn getArrVal(t: *mut Table, k: u32) -> *mut Value {
    (*t).array.sub(1 + k as usize)
}

#[inline]
unsafe fn obj2arr(t: *mut Table, k: u32, value: *const TValue) {
    *getArrTag(t, k) = (*value).tt_;
    *getArrVal(t, k) = (*value).value_;
}

#[inline]
unsafe fn luaV_fastseti(t: *const TValue, key: lua_Integer, val: *mut TValue) -> c_int {
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

#[inline]
unsafe fn luaV_finishfastset(L: *mut lua_State, t: *const TValue, v: *const TValue) {
    luaC_barrierback(L, gcvalue(t), v);
}

#[inline]
unsafe fn luaV_rawequalobj(t1: *const TValue, t2: *const TValue) -> c_int {
    luaV_equalobj(core::ptr::null_mut(), t1, t2)
}

#[inline]
unsafe fn lua_numbertointeger(n: lua_Number, p: *mut lua_Integer) -> c_int {
    if n >= lua_Integer::MIN as lua_Number && n < -(lua_Integer::MIN as lua_Number) {
        *p = n as lua_Integer;
        1
    } else {
        0
    }
}

#[inline]
unsafe fn l_intfitsf(i: lua_Integer) -> bool {
    const LIM: lua_Integer = 1_i64 << f64::MANTISSA_DIGITS;
    (-LIM..=LIM).contains(&i)
}

#[inline]
unsafe fn l_strton(obj: *const TValue, result: *mut TValue) -> c_int {
    if !cvt2num(obj) {
        0
    } else {
        let st = tsvalue(obj);
        let mut stlen = 0usize;
        let s = getlstr(st, &mut stlen);
        c_int::from(luaO_str2num(s, result) == stlen + 1)
    }
}

#[inline]
unsafe fn mask1(n: u32, p: u32) -> Instruction {
    ((!((!0u32) << n)) << p) as Instruction
}

#[inline]
unsafe fn getarg(i: Instruction, pos: u32, size: u32) -> c_int {
    ((i >> pos) & mask1(size, 0)) as c_int
}

#[inline]
unsafe fn GET_OPCODE(i: Instruction) -> c_int {
    ((i >> POS_OP) & mask1(SIZE_OP, 0)) as c_int
}

#[inline]
unsafe fn GETARG_A(i: Instruction) -> c_int { getarg(i, POS_A, SIZE_A) }
#[inline]
unsafe fn GETARG_B(i: Instruction) -> c_int { getarg(i, POS_B, SIZE_B) }
#[inline]
unsafe fn GETARG_VB(i: Instruction) -> c_int { getarg(i, POS_VB, SIZE_VB) }
#[inline]
unsafe fn GETARG_SB(i: Instruction) -> c_int { GETARG_B(i) - OFFSET_SC }
#[inline]
unsafe fn GETARG_C(i: Instruction) -> c_int { getarg(i, POS_C, SIZE_C) }
#[inline]
unsafe fn GETARG_VC(i: Instruction) -> c_int { getarg(i, POS_VC, SIZE_VC) }
#[inline]
unsafe fn GETARG_SC(i: Instruction) -> c_int { GETARG_C(i) - OFFSET_SC }
#[inline]
unsafe fn TESTARG_K(i: Instruction) -> bool { (i & (1u32 << POS_K)) != 0 }
#[inline]
unsafe fn GETARG_K(i: Instruction) -> c_int { getarg(i, POS_K, 1) }
#[inline]
unsafe fn GETARG_BX(i: Instruction) -> c_int { getarg(i, POS_BX, SIZE_BX) }
#[inline]
unsafe fn GETARG_AX(i: Instruction) -> c_int { getarg(i, POS_AX, SIZE_AX) }
#[inline]
unsafe fn GETARG_SBX(i: Instruction) -> c_int { getarg(i, POS_BX, SIZE_BX) - OFFSET_SBX }
#[inline]
unsafe fn GETARG_SJ(i: Instruction) -> c_int { getarg(i, POS_SJ, SIZE_SJ) - OFFSET_SJ }

#[inline]
unsafe fn savepc(ci: *mut CallInfo, pc: *const Instruction) {
    (*ci).u.l.savedpc = pc;
}

#[inline]
unsafe fn savestate(L: *mut lua_State, ci: *mut CallInfo, pc: *const Instruction) {
    savepc(ci, pc);
    (*L).top.p = (*ci).top.p;
}

#[inline]
unsafe fn updatetrap(ci: *mut CallInfo, trap: &mut c_int) {
    *trap = (*ci).u.l.trap;
}

#[inline]
unsafe fn updatebase(ci: *mut CallInfo, base: &mut StkId) {
    *base = (*ci).func.p.add(1);
}

#[inline]
unsafe fn RA(base: StkId, i: Instruction) -> StkId {
    base.add(GETARG_A(i) as usize)
}

#[inline]
unsafe fn RB(base: StkId, i: Instruction) -> StkId {
    base.add(GETARG_B(i) as usize)
}

#[inline]
unsafe fn RC(base: StkId, i: Instruction) -> StkId {
    base.add(GETARG_C(i) as usize)
}

#[inline]
unsafe fn KB<'a>(k: *mut TValue, i: Instruction) -> *mut TValue {
    k.add(GETARG_B(i) as usize)
}

#[inline]
unsafe fn KC<'a>(k: *mut TValue, i: Instruction) -> *mut TValue {
    k.add(GETARG_C(i) as usize)
}

#[inline]
unsafe fn RKC(base: StkId, k: *mut TValue, i: Instruction) -> *mut TValue {
    if TESTARG_K(i) {
        k.add(GETARG_C(i) as usize)
    } else {
        s2v(base.add(GETARG_C(i) as usize))
    }
}

#[inline]
unsafe fn dojump(ci: *mut CallInfo, i: Instruction, e: c_int, pc: &mut *const Instruction, trap: &mut c_int) {
    *pc = (*pc).offset((GETARG_SJ(i) + e) as isize);
    updatetrap(ci, trap);
}

#[inline]
unsafe fn donextjump(ci: *mut CallInfo, pc: &mut *const Instruction, trap: &mut c_int) {
    let ni = **pc;
    dojump(ci, ni, 1, pc, trap);
}

#[inline]
unsafe fn docondjump(cond: c_int, ci: *mut CallInfo, i: Instruction, pc: &mut *const Instruction, trap: &mut c_int) {
    if cond != GETARG_K(i) {
        *pc = (*pc).add(1);
    } else {
        donextjump(ci, pc, trap);
    }
}

#[inline]
unsafe fn checkGC(L: *mut lua_State, ci: *mut CallInfo, pc: *const Instruction, trap: &mut c_int, c: StkId) {
    if (*G(L)).GCdebt <= 0 {
        savepc(ci, pc);
        (*L).top.p = c;
        luaC_step(L);
        updatetrap(ci, trap);
    }
}

unsafe fn l_strcmp(ts1: *mut TString, ts2: *mut TString) -> c_int {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_tonumber_(obj: *const TValue, n: *mut lua_Number) -> c_int {
    let mut v = TValue { value_: Value { i: 0 }, tt_: LUA_VNIL };
    if ttisinteger(obj) {
        *n = ivalue(obj) as lua_Number;
        1
    } else if l_strton(obj, &mut v) != 0 {
        *n = if ttisfloat(&v) { fltvalue(&v) } else { ivalue(&v) as lua_Number };
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_flttointeger(n: lua_Number, p: *mut lua_Integer, mode: c_int) -> c_int {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_tointegerns(obj: *const TValue, p: *mut lua_Integer, mode: c_int) -> c_int {
    if ttisfloat(obj) {
        luaV_flttointeger(fltvalue(obj), p, mode)
    } else if ttisinteger(obj) {
        *p = ivalue(obj);
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_tointeger(obj: *const TValue, p: *mut lua_Integer, mode: c_int) -> c_int {
    let mut v = TValue { value_: Value { i: 0 }, tt_: LUA_VNIL };
    let mut o = obj;
    if l_strton(obj, &mut v) != 0 {
        o = &v;
    }
    luaV_tointegerns(o, p, mode)
}

unsafe fn forlimit(L: *mut lua_State, init: lua_Integer, lim: *const TValue, p: *mut lua_Integer, step: lua_Integer) -> c_int {
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

unsafe fn forprep(L: *mut lua_State, ra: StkId) -> c_int {
    let pinit = s2v(ra);
    let plimit = s2v(ra.add(1));
    let pstep = s2v(ra.add(2));
    if ttisinteger(pinit) && ttisinteger(pstep) {
        let init = ivalue(pinit);
        let step = ivalue(pstep);
        let mut limit = 0;
        if step == 0 {
            luaG_runerror(L, c"'for' step is zero".as_ptr());
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
            luaG_runerror(L, c"'for' step is zero".as_ptr());
        }
        if if 0.0 < step { limit < init } else { init < limit } {
            return 1;
        }
        setfltvalue(s2v(ra), limit);
        setfltvalue(s2v(ra.add(1)), step);
        setfltvalue(s2v(ra.add(2)), init);
    }
    0
}

unsafe fn floatforloop(ra: StkId) -> c_int {
    let step = fltvalue(s2v(ra.add(1)));
    let limit = fltvalue(s2v(ra));
    let mut idx = fltvalue(s2v(ra.add(2)));
    idx += step;
    if if 0.0 < step { idx <= limit } else { limit <= idx } {
        chgfltvalue(s2v(ra.add(2)), idx);
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_finishget(
    L: *mut lua_State,
    mut t: *const TValue,
    key: *mut TValue,
    val: StkId,
    mut tag: lu_byte,
) -> lu_byte {
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
    luaG_runerror(L, c"'__index' chain too long; possible loop".as_ptr())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_finishset(
    L: *mut lua_State,
    mut t: *const TValue,
    key: *mut TValue,
    val: *mut TValue,
    mut hres: c_int,
) {
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
        hres = if !ttistable(t) { HNOTATABLE } else { luaH_pset(hvalue(t), key, val) };
        if hres == HOK {
            luaV_finishfastset(L, t, val);
            return;
        }
    }
    luaG_runerror(L, c"'__newindex' chain too long; possible loop".as_ptr())
}

unsafe fn LTintfloat(i: lua_Integer, f: lua_Number) -> c_int {
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

unsafe fn LEintfloat(i: lua_Integer, f: lua_Number) -> c_int {
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

unsafe fn LTfloatint(f: lua_Number, i: lua_Integer) -> c_int {
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

unsafe fn LEfloatint(f: lua_Number, i: lua_Integer) -> c_int {
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

unsafe fn LTnum(l: *const TValue, r: *const TValue) -> c_int {
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

unsafe fn LEnum(l: *const TValue, r: *const TValue) -> c_int {
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

unsafe fn lessthanothers(L: *mut lua_State, l: *const TValue, r: *const TValue) -> c_int {
    if ttisstring(l) && ttisstring(r) {
        c_int::from(l_strcmp(tsvalue(l), tsvalue(r)) < 0)
    } else {
        luaT_callorderTM(L, l, r, TM_LT)
    }
}

unsafe fn lessequalothers(L: *mut lua_State, l: *const TValue, r: *const TValue) -> c_int {
    if ttisstring(l) && ttisstring(r) {
        c_int::from(l_strcmp(tsvalue(l), tsvalue(r)) <= 0)
    } else {
        luaT_callorderTM(L, l, r, TM_LE)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_lessthan(L: *mut lua_State, l: *const TValue, r: *const TValue) -> c_int {
    if ttisnumber(l) && ttisnumber(r) {
        LTnum(l, r)
    } else {
        lessthanothers(L, l, r)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_lessequal(L: *mut lua_State, l: *const TValue, r: *const TValue) -> c_int {
    if ttisnumber(l) && ttisnumber(r) {
        LEnum(l, r)
    } else {
        lessequalothers(L, l, r)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_equalobj(L: *mut lua_State, t1: *const TValue, t2: *const TValue) -> c_int {
    let tm;
    if ttype(t1) != ttype(t2) {
        return 0;
    } else if ttypetag(t1) != ttypetag(t2) {
        match ttypetag(t1) {
            LUA_VNUMINT => {
                let mut i2 = 0;
                return c_int::from(luaV_flttointeger(fltvalue(t2), &mut i2, F2Ieq) != 0 && ivalue(t1) == i2);
            }
            LUA_VNUMFLT => {
                let mut i1 = 0;
                return c_int::from(luaV_flttointeger(fltvalue(t1), &mut i1, F2Ieq) != 0 && i1 == ivalue(t2));
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
                let tm = if tm.is_null() { fasttm(L, (*uvalue(t2)).metatable, TM_EQ) } else { tm };
                if tm.is_null() {
                    return 0;
                }
                let tag = luaT_callTMres(L, tm, t1, t2, (*L).top.p);
                return c_int::from(!l_isfalse(&TValue { value_: Value { ub: tag }, tt_: tag }));
            }
            LUA_VTABLE => {
                if hvalue(t1) == hvalue(t2) {
                    return 1;
                } else if L.is_null() {
                    return 0;
                }
                tm = fasttm(L, (*hvalue(t1)).metatable, TM_EQ);
                let tm = if tm.is_null() { fasttm(L, (*hvalue(t2)).metatable, TM_EQ) } else { tm };
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

unsafe fn tostring(L: *mut lua_State, o: *mut TValue) -> bool {
    ttisstring(o) || (cvt2str(o) && { luaO_tostring(L, o); true })
}

#[inline]
unsafe fn isemptystr(o: *const TValue) -> bool {
    ttisshrstring(o) && (*tsvalue(o)).shrlen == 0
}

unsafe fn copy2buff(top: StkId, mut n: c_int, buff: *mut c_char) {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_concat(L: *mut lua_State, mut total: c_int) {
    if total == 1 {
        return;
    }
    loop {
        let top = (*L).top.p;
        let mut n = 2;
        if !(ttisstring(s2v(top.sub(2))) || cvt2str(s2v(top.sub(2)))) || !tostring(L, s2v(top.sub(1))) {
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
                    luaG_runerror(L, c"string length overflow".as_ptr());
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_objlen(L: *mut lua_State, ra: StkId, rb: *const TValue) {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_idiv(L: *mut lua_State, m: lua_Integer, n: lua_Integer) -> lua_Integer {
    if (n as lua_Unsigned).wrapping_add(1) <= 1 {
        if n == 0 {
            luaG_runerror(L, c"attempt to divide by zero".as_ptr());
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_mod(L: *mut lua_State, m: lua_Integer, n: lua_Integer) -> lua_Integer {
    if (n as lua_Unsigned).wrapping_add(1) <= 1 {
        if n == 0 {
            luaG_runerror(L, c"attempt to perform 'n%%0'".as_ptr());
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_modf(_L: *mut lua_State, m: lua_Number, n: lua_Number) -> lua_Number {
    let mut r = m % n;
    if if r > 0.0 { n < 0.0 } else { r < 0.0 && n > 0.0 } {
        r += n;
    }
    r
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_shiftl(x: lua_Integer, y: lua_Integer) -> lua_Integer {
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

unsafe fn pushclosure(L: *mut lua_State, p: *mut Proto, encup: *mut *mut UpVal, base: StkId, ra: StkId) {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_finishOp(L: *mut lua_State) {
    let ci = (*L).ci;
    let base = (*ci).func.p.add(1);
    let inst = *(*ci).u.l.savedpc.sub(1);
    let op = GET_OPCODE(inst);
    match op {
        OP_MMBIN | OP_MMBINI | OP_MMBINK => {
            setobjs2s(L, base.add(GETARG_A(*(*ci).u.l.savedpc.sub(2)) as usize), { (*L).top.p = (*L).top.p.sub(1); (*L).top.p });
        }
        OP_UNM | OP_BNOT | OP_LEN | OP_GETTABUP | OP_GETTABLE | OP_GETI | OP_GETFIELD | OP_SELF => {
            setobjs2s(L, base.add(GETARG_A(inst) as usize), { (*L).top.p = (*L).top.p.sub(1); (*L).top.p });
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaV_execute(L: *mut lua_State, mut ci: *mut CallInfo) {
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
            debug_assert!(luaP_isIT(i) != 0 || { (*L).top.p = base; true });
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
                        if b == 0 { break; }
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
                    let upval = (*(*(*cl).upvals.as_mut_ptr().add(GETARG_B(i) as usize))).v.p;
                    let rc = KC(k, i);
                    let mut tag = if !ttistable(upval) { LUA_VNOTABLE } else { luaH_getshortstr(hvalue(upval), tsvalue(rc), s2v(ra)) };
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
                        if !ttistable(rb) { LUA_VNOTABLE } else { luaH_getint(hvalue(rb), ivalue(rc), s2v(ra)) }
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
                    let mut tag = if !ttistable(rb) { LUA_VNOTABLE } else { luaH_getint(hvalue(rb), c, s2v(ra)) };
                    if tagisempty(tag) {
                        let mut key = TValue { value_: Value { i: 0 }, tt_: LUA_VNIL };
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
                    let mut tag = if !ttistable(rb) { LUA_VNOTABLE } else { luaH_getshortstr(hvalue(rb), tsvalue(rc), s2v(ra)) };
                    if tagisempty(tag) {
                        savestate(L, ci, pc);
                        tag = luaV_finishget(L, rb, rc, ra, tag);
                        updatetrap(ci, &mut trap);
                        let _ = tag;
                    }
                }
                OP_SETTABUP => {
                    let upval = (*(*(*cl).upvals.as_mut_ptr().add(GETARG_A(i) as usize))).v.p;
                    let rb = KB(k, i);
                    let rc = RKC(base, k, i);
                    let hres = if !ttistable(upval) { HNOTATABLE } else { luaH_psetshortstr(hvalue(upval), tsvalue(rb), rc) };
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
                        let mut key = TValue { value_: Value { i: 0 }, tt_: LUA_VNIL };
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
                    let hres = if !ttistable(s2v(ra)) { HNOTATABLE } else { luaH_psetshortstr(hvalue(s2v(ra)), tsvalue(rb), rc) };
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
                    let mut tag = if !ttistable(rb) { LUA_VNOTABLE } else { luaH_getshortstr(hvalue(rb), tsvalue(rc), s2v(ra)) };
                    if tagisempty(tag) {
                        savestate(L, ci, pc);
                        tag = luaV_finishget(L, rb, rc, ra, tag);
                        updatetrap(ci, &mut trap);
                        let _ = tag;
                    }
                }
                OP_ADDI | OP_ADDK | OP_SUBK | OP_MULK | OP_MODK | OP_POWK | OP_DIVK | OP_IDIVK
                | OP_BANDK | OP_BORK | OP_BXORK | OP_SHLI | OP_SHRI | OP_ADD | OP_SUB | OP_MUL
                | OP_MOD | OP_POW | OP_DIV | OP_IDIV | OP_BAND | OP_BOR | OP_BXOR | OP_SHL | OP_SHR => {
                    let ra = RA(base, i);
                    let rb = s2v(RB(base, i));
                    let rc = if matches!(GET_OPCODE(i), OP_ADDK | OP_SUBK | OP_MULK | OP_MODK | OP_POWK | OP_DIVK | OP_IDIVK | OP_BANDK | OP_BORK | OP_BXORK) {
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
                        OP_ADDK | OP_SUBK | OP_MULK | OP_MODK | OP_IDIVK | OP_ADD | OP_SUB | OP_MUL | OP_MOD | OP_IDIV => {
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
                            } else if tonumberns(rb, &mut n1) != 0 && tonumberns(rc, &mut n2) != 0 {
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
                                setfltvalue(s2v(ra), if matches!(opcode, OP_POWK | OP_POW) { if n2 == 2.0 { n1 * n1 } else { n1.powf(n2) } } else { n1 / n2 });
                                pc = pc.add(1);
                            }
                        }
                        OP_BANDK | OP_BORK | OP_BXORK | OP_BAND | OP_BOR | OP_BXOR | OP_SHL | OP_SHR => {
                            if tointegerns(rb, &mut i1) != 0 && tointegerns(rc, &mut i2) != 0 {
                                let res = match opcode {
                                    OP_BANDK | OP_BAND => ((i1 as lua_Unsigned) & (i2 as lua_Unsigned)) as lua_Integer,
                                    OP_BORK | OP_BOR => ((i1 as lua_Unsigned) | (i2 as lua_Unsigned)) as lua_Integer,
                                    OP_BXORK | OP_BXOR => ((i1 as lua_Unsigned) ^ (i2 as lua_Unsigned)) as lua_Integer,
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
                    luaT_trybiniTM(L, s2v(ra), GETARG_SB(i) as lua_Integer, GETARG_K(i), RA(base, pi), GETARG_C(i));
                    updatetrap(ci, &mut trap);
                }
                OP_MMBINK => {
                    let ra = RA(base, i);
                    let pi = *pc.sub(2);
                    savestate(L, ci, pc);
                    luaT_trybinassocTM(L, s2v(ra), KB(k, i), GETARG_K(i), RA(base, pi), GETARG_C(i));
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
                        setivalue(s2v(ra), (!(0 as lua_Unsigned) ^ ib as lua_Unsigned) as lua_Integer);
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
                        let cond = luaT_callorderiTM(L, ra, im, if matches!(GET_OPCODE(i), OP_GTI | OP_GEI) { 1 } else { 0 }, GETARG_C(i), if matches!(GET_OPCODE(i), OP_LTI | OP_GTI) { TM_LT } else { TM_LE });
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
                    let delta = if nparams1 != 0 { (*ci).u.l.nextraargs + nparams1 } else { 0 };
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
                        (*ci).func.p = (*ci).func.p.sub(((*ci).u.l.nextraargs + nparams1) as usize);
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
                    let mut temp = TValue { value_: Value { i: 0 }, tt_: LUA_VNIL };
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
