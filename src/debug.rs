#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

use crate::runtime::*;
use core::ffi::c_uchar;
use core::mem::ManuallyDrop;

const ABSLINEINFO: i8 = -0x80;
const MAXIWTHABS: c_int = 128;
const PF_VAHID: u8 = 1;

const CIST_CCMT: u32 = 8;
const MAX_CCMT: u32 = 0xfu32 << CIST_CCMT;
const CIST_HOOKED: u32 = 1 << 20;
const CIST_TAIL: u32 = 1 << 22;
const CIST_HOOKYIELD: u32 = 1 << 23;
const CIST_FIN: u32 = 1 << 24;

const LUA_HOOKCOUNT: c_int = 3;
const LUA_HOOKLINE: c_int = 2;
const LUA_MASKLINE: c_int = 1 << LUA_HOOKLINE;
const LUA_MASKCOUNT: c_int = 1 << LUA_HOOKCOUNT;
const LUA_ERRRUN: TStatus = 2;
const LUA_YIELD: TStatus = 1;
const LUA_FLOORN2I: c_int = 1;

const TM_INDEX: usize = 0;
const TM_LEN: usize = 4;
const TM_UNM: usize = 18;
const TM_BNOT: usize = 19;
const TM_LT: usize = 20;
const TM_LE: usize = 21;
const TM_CONCAT: usize = 22;
const TM_CLOSE: usize = 24;

const OP_MOVE: usize = 0;
const OP_LOADK: usize = 3;
const OP_LOADKX: usize = 4;
const OP_LOADNIL: usize = 8;
const OP_GETUPVAL: usize = 9;
const OP_GETTABUP: usize = 11;
const OP_GETTABLE: usize = 12;
const OP_GETI: usize = 13;
const OP_GETFIELD: usize = 14;
const OP_SETTABUP: usize = 15;
const OP_SETTABLE: usize = 16;
const OP_SETI: usize = 17;
const OP_SETFIELD: usize = 18;
const OP_SELF: usize = 20;
const OP_UNM: usize = 49;
const OP_BNOT: usize = 50;
const OP_LEN: usize = 52;
const OP_CONCAT: usize = 53;
const OP_CLOSE: usize = 54;
const OP_JMP: usize = 56;
const OP_EQ: usize = 57;
const OP_LT: usize = 58;
const OP_LE: usize = 59;
const OP_LTI: usize = 62;
const OP_LEI: usize = 63;
const OP_GTI: usize = 64;
const OP_GEI: usize = 65;
const OP_CALL: usize = 68;
const OP_TAILCALL: usize = 69;
const OP_RETURN: usize = 70;
const OP_TFORCALL: usize = 76;
const OP_MMBIN: usize = 46;
const OP_MMBINI: usize = 47;
const OP_MMBINK: usize = 48;
const OP_VARARGPREP: usize = 83;

const LUA_ENV: &[u8] = b"_ENV\0";
const STR_LOCAL: &[u8] = b"local\0";
const STR_UPVALUE: &[u8] = b"upvalue\0";
const STR_VARARG: &[u8] = b"(vararg)\0";
const STR_TEMP: &[u8] = b"(temporary)\0";
const STR_CTEMP: &[u8] = b"(C temporary)\0";
const STR_QUESTION: &[u8] = b"?\0";
const STR_C_SOURCE: &[u8] = b"=[C]\0";
const STR_UNKNOWN_SOURCE: &[u8] = b"=?\0";
const STR_C_WHAT: &[u8] = b"C\0";
const STR_MAIN: &[u8] = b"main\0";
const STR_LUA: &[u8] = b"Lua\0";
const STR_EMPTY: &[u8] = b"\0";
const STR_HOOK: &[u8] = b"hook\0";
const STR_GC: &[u8] = b"__gc\0";
const STR_META: &[u8] = b"metamethod\0";
const STR_FOR_ITER: &[u8] = b"for iterator\0";
const STR_CONSTANT: &[u8] = b"constant\0";
const STR_GLOBAL: &[u8] = b"global\0";
const STR_FIELD: &[u8] = b"field\0";
const STR_METHOD: &[u8] = b"method\0";
const STR_INTEGER_INDEX: &[u8] = b"integer index\0";
const NO_ERROR_OBJECT: &[u8] = b"<no error object>\0";

#[repr(C)]
struct LuaDebug {
    event: c_int,
    name: *const c_char,
    namewhat: *const c_char,
    what: *const c_char,
    source: *const c_char,
    srclen: usize,
    currentline: c_int,
    linedefined: c_int,
    lastlinedefined: c_int,
    nups: c_uchar,
    nparams: c_uchar,
    isvararg: c_char,
    extraargs: c_uchar,
    istailcall: c_char,
    ftransfer: c_int,
    ntransfer: c_int,
    short_src: [c_char; 60],
    i_ci: *mut CallInfo,
}

#[repr(C)]
struct AbsLineInfo {
    pc: c_int,
    line: c_int,
}

#[repr(C)]
struct LocVar {
    varname: *mut TString,
    startpc: c_int,
    endpc: c_int,
}

#[repr(C)]
struct ProtoDebug {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    numparams: u8,
    flag: u8,
    maxstacksize: u8,
    sizeupvalues: c_int,
    sizek: c_int,
    sizecode: c_int,
    sizelineinfo: c_int,
    sizep: c_int,
    sizelocvars: c_int,
    sizeabslineinfo: c_int,
    linedefined: c_int,
    lastlinedefined: c_int,
    k: *mut TValue,
    code: *mut Instruction,
    p: *mut *mut Proto,
    upvalues: *mut Upvaldesc,
    lineinfo: *mut i8,
    abslineinfo: *mut AbsLineInfo,
    locvars: *mut LocVar,
    source: *mut TString,
    gclist: *mut GCObject,
}

#[repr(C)]
union Closure {
    c: ManuallyDrop<CClosure>,
    l: ManuallyDrop<LClosure>,
}

unsafe extern "C" {
    fn luaD_hook(L: *mut lua_State, event: c_int, line: c_int, ftransfer: c_int, ntransfer: c_int);
    fn luaD_hookcall(L: *mut lua_State, ci: *mut CallInfo);
    fn luaF_getlocalname(p: *const Proto, local_number: c_int, pc: c_int) -> *const c_char;
    fn luaV_tointegerns(obj: *const TValue, p: *mut lua_Integer, mode: c_int) -> c_int;
    fn luaO_chunkid(out: *mut c_char, source: *const c_char, srclen: usize);
    fn luaO_pushfstring(L: *mut lua_State, fmt: *const c_char, ...) -> *const c_char;
    static luaP_opmodes: [u8; 85];
    fn luaP_isIT(i: Instruction) -> c_int;
    fn strcmp(lhs: *const c_char, rhs: *const c_char) -> c_int;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
}

#[inline]
unsafe fn ar_mut<'a>(ar: *mut lua_Debug) -> &'a mut LuaDebug {
    unsafe { &mut *ar.cast::<LuaDebug>() }
}

#[inline]
unsafe fn ar_ref<'a>(ar: *const lua_Debug) -> &'a LuaDebug {
    unsafe { &*ar.cast::<LuaDebug>() }
}

#[inline]
unsafe fn pdebug(p: *const Proto) -> *const ProtoDebug {
    p.cast()
}

#[inline]
unsafe fn pdebug_mut(p: *mut Proto) -> *mut ProtoDebug {
    p.cast()
}

#[inline]
unsafe fn ci_func(ci: *mut CallInfo) -> *mut LClosure {
    unsafe { clLvalue(s2v((*ci).func.p)) }
}

#[inline]
unsafe fn pc_rel(pc: *const Instruction, p: *const ProtoDebug) -> c_int {
    unsafe { pc.offset_from((*p).code) as c_int - 1 }
}

#[inline]
unsafe fn resethookcount(L: *mut lua_State) {
    unsafe { (*L).hookcount = (*L).basehookcount };
}

#[inline]
unsafe fn test_amode(op: usize) -> bool {
    unsafe { luaP_opmodes[op] & (1 << 3) != 0 }
}

#[inline]
unsafe fn test_mmmode(op: usize) -> bool {
    unsafe { luaP_opmodes[op] & (1 << 7) != 0 }
}

#[inline]
unsafe fn get_opcode(i: Instruction) -> usize {
    (i & 0x7f) as usize
}

#[inline]
unsafe fn getarg_a(i: Instruction) -> c_int {
    ((i >> 7) & 0xff) as c_int
}

#[inline]
unsafe fn getarg_b(i: Instruction) -> c_int {
    ((i >> 16) & 0xff) as c_int
}

#[inline]
unsafe fn getarg_c(i: Instruction) -> c_int {
    ((i >> 24) & 0xff) as c_int
}

#[inline]
unsafe fn getarg_bx(i: Instruction) -> c_int {
    ((i >> 15) & 0x1ffff) as c_int
}

#[inline]
unsafe fn getarg_ax(i: Instruction) -> c_int {
    (i >> 7) as c_int
}

#[inline]
unsafe fn getarg_sj(i: Instruction) -> c_int {
    ((i >> 7) & 0x1ffffff) as c_int - ((0x1ffffff as c_int) >> 1)
}

unsafe fn currentpc(ci: *mut CallInfo) -> c_int {
    unsafe { pc_rel((*ci).u.l.savedpc, pdebug((*ci_func(ci)).p)) }
}

unsafe fn getbaseline(f: *const ProtoDebug, pc: c_int, basepc: *mut c_int) -> c_int {
    if unsafe { (*f).sizeabslineinfo == 0 || pc < (*(*f).abslineinfo).pc } {
        unsafe { *basepc = -1 };
        unsafe { (*f).linedefined }
    } else {
        let mut i = pc / MAXIWTHABS - 1;
        while unsafe {
            i + 1 < (*f).sizeabslineinfo && pc >= (*(*f).abslineinfo.add((i + 1) as usize)).pc
        } {
            i += 1;
        }
        unsafe { *basepc = (*(*f).abslineinfo.add(i as usize)).pc };
        unsafe { (*(*f).abslineinfo.add(i as usize)).line }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_getfuncline(f: *const Proto, pc: c_int) -> c_int {
    let f = unsafe { pdebug(f) };
    if unsafe { (*f).lineinfo.is_null() } {
        -1
    } else {
        let mut basepc = 0;
        let mut baseline = unsafe { getbaseline(f, pc, &mut basepc) };
        while basepc < pc {
            basepc += 1;
            baseline += unsafe { *(*f).lineinfo.add(basepc as usize) as c_int };
        }
        baseline
    }
}

unsafe fn getcurrentline(ci: *mut CallInfo) -> c_int {
    unsafe { luaG_getfuncline((*ci_func(ci)).p, currentpc(ci)) }
}

unsafe fn settraps(mut ci: *mut CallInfo) {
    while !ci.is_null() {
        if unsafe { isLua(ci) } {
            unsafe { (*ci).u.l.trap = 1 };
        }
        ci = unsafe { (*ci).previous };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_sethook(
    L: *mut lua_State,
    mut func: lua_Hook,
    mut mask: c_int,
    count: c_int,
) {
    if func.is_none() || mask == 0 {
        mask = 0;
        func = None;
    }
    unsafe {
        (*L).hook = func;
        (*L).basehookcount = count;
        resethookcount(L);
        (*L).hookmask = mask;
    }
    if mask != 0 {
        unsafe { settraps((*L).ci) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_gethook(L: *mut lua_State) -> lua_Hook {
    unsafe { (*L).hook }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_gethookmask(L: *mut lua_State) -> c_int {
    unsafe { (*L).hookmask }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_gethookcount(L: *mut lua_State) -> c_int {
    unsafe { (*L).basehookcount }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_getstack(
    L: *mut lua_State,
    mut level: c_int,
    ar: *mut lua_Debug,
) -> c_int {
    unsafe {
        if level < 0 {
            return 0;
        }
        let mut ci = unsafe { (*L).ci };
        while level > 0 && !ptr::eq(ci, ptr::addr_of_mut!((*L).base_ci)) {
            ci = unsafe { (*ci).previous };
            level -= 1;
        }
        if level == 0 && !ptr::eq(ci, ptr::addr_of_mut!((*L).base_ci)) {
            unsafe { ar_mut(ar).i_ci = ci };
            1
        } else {
            0
        }
    }
}

unsafe fn upvalname(p: *const ProtoDebug, uv: c_int) -> *const c_char {
    let s = unsafe { (*(*p).upvalues.add(uv as usize)).name };
    if s.is_null() {
        STR_QUESTION.as_ptr().cast()
    } else {
        unsafe { getstr(s) }
    }
}

unsafe fn findvararg(ci: *mut CallInfo, n: c_int, pos: *mut StkId) -> *const c_char {
    let p = unsafe { pdebug((*ci_func(ci)).p) };
    if unsafe { (*p).flag & PF_VAHID != 0 } {
        let nextra = unsafe { (*ci).u.l.nextraargs };
        if n >= -nextra {
            unsafe { *pos = (*ci).func.p.sub(nextra as usize).sub((n + 1) as usize) };
            return STR_VARARG.as_ptr().cast();
        }
    }
    ptr::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_findlocal(
    L: *mut lua_State,
    ci: *mut CallInfo,
    n: c_int,
    pos: *mut StkId,
) -> *const c_char {
    let base = unsafe { (*ci).func.p.add(1) };
    let mut name = ptr::null();
    if unsafe { isLua(ci) } {
        if n < 0 {
            return unsafe { findvararg(ci, n, pos) };
        }
        name = unsafe { luaF_getlocalname((*ci_func(ci)).p, n, currentpc(ci)) };
    }
    if name.is_null() {
        let limit = if ptr::eq(ci, unsafe { (*L).ci }) {
            unsafe { (*L).top.p }
        } else {
            unsafe { (*(*ci).next).func.p }
        };
        if n > 0 && unsafe { limit.offset_from(base) } >= n as isize {
            name = if unsafe { isLua(ci) } {
                STR_TEMP.as_ptr().cast()
            } else {
                STR_CTEMP.as_ptr().cast()
            };
        } else {
            return ptr::null();
        }
    }
    if !pos.is_null() {
        unsafe { *pos = base.add((n - 1) as usize) };
    }
    name
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_getlocal(
    L: *mut lua_State,
    ar: *const lua_Debug,
    n: c_int,
) -> *const c_char {
    let name;
    if ar.is_null() {
        if !unsafe { isLfunction(s2v((*L).top.p.sub(1))) } {
            name = ptr::null();
        } else {
            name = unsafe { luaF_getlocalname((*clLvalue(s2v((*L).top.p.sub(1)))).p, n, 0) };
        }
    } else {
        let mut pos = ptr::null_mut();
        name = unsafe { luaG_findlocal(L, ar_ref(ar).i_ci, n, &mut pos) };
        if !name.is_null() {
            unsafe {
                setobjs2s(L, (*L).top.p, pos);
                api_incr_top(L);
            }
        }
    }
    name
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_setlocal(
    L: *mut lua_State,
    ar: *const lua_Debug,
    n: c_int,
) -> *const c_char {
    let mut pos = ptr::null_mut();
    let name = unsafe { luaG_findlocal(L, ar_ref(ar).i_ci, n, &mut pos) };
    if !name.is_null() {
        unsafe {
            api_checkpop(L, 1);
            setobjs2s(L, pos, (*L).top.p.sub(1));
            (*L).top.p = (*L).top.p.sub(1);
        }
    }
    name
}

unsafe fn funcinfo(ar: &mut LuaDebug, cl: *mut Closure) {
    if cl.is_null() || unsafe { (*cl.cast::<CClosure>()).tt != LUA_VLCL } {
        ar.source = STR_C_SOURCE.as_ptr().cast();
        ar.srclen = STR_C_SOURCE.len() - 1;
        ar.linedefined = -1;
        ar.lastlinedefined = -1;
        ar.what = STR_C_WHAT.as_ptr().cast();
    } else {
        let p = unsafe { pdebug((*cl.cast::<LClosure>()).p) };
        if unsafe { !(*p).source.is_null() } {
            ar.source = unsafe { getlstr((*p).source, &mut ar.srclen) };
        } else {
            ar.source = STR_UNKNOWN_SOURCE.as_ptr().cast();
            ar.srclen = STR_UNKNOWN_SOURCE.len() - 1;
        }
        ar.linedefined = unsafe { (*p).linedefined };
        ar.lastlinedefined = unsafe { (*p).lastlinedefined };
        ar.what = if ar.linedefined == 0 {
            STR_MAIN.as_ptr().cast()
        } else {
            STR_LUA.as_ptr().cast()
        };
    }
    unsafe { luaO_chunkid(ar.short_src.as_mut_ptr(), ar.source, ar.srclen) };
}

unsafe fn nextline(p: *const ProtoDebug, currentline: c_int, pc: c_int) -> c_int {
    if unsafe { *(*p).lineinfo.add(pc as usize) != ABSLINEINFO } {
        currentline + unsafe { *(*p).lineinfo.add(pc as usize) as c_int }
    } else {
        unsafe { luaG_getfuncline(p.cast(), pc) }
    }
}

unsafe fn collectvalidlines(L: *mut lua_State, f: *mut Closure) {
    if f.is_null() || unsafe { (*f.cast::<CClosure>()).tt != LUA_VLCL } {
        unsafe {
            setnilvalue(s2v((*L).top.p));
            api_incr_top(L);
        }
    } else {
        let p = unsafe { pdebug((*f.cast::<LClosure>()).p) };
        let mut currentline = unsafe { (*p).linedefined };
        let t = unsafe { luaH_new(L) };
        unsafe {
            sethvalue2s(L, (*L).top.p, t);
            api_incr_top(L);
        }
        if unsafe { !(*p).lineinfo.is_null() } {
            let mut i;
            let mut v = TValue {
                value_: Value { ub: 0 },
                tt_: 0,
            };
            unsafe { setbtvalue(&mut v) };
            if unsafe { (*p).flag & PF_VAHID == 0 } {
                i = 0;
            } else {
                currentline = unsafe { nextline(p, currentline, 0) };
                i = 1;
            }
            while i < unsafe { (*p).sizelineinfo } {
                currentline = unsafe { nextline(p, currentline, i) };
                unsafe { luaH_setint(L, t, currentline as lua_Integer, &mut v) };
                i += 1;
            }
        }
    }
}

unsafe fn getfuncname(
    L: *mut lua_State,
    ci: *mut CallInfo,
    name: *mut *const c_char,
) -> *const c_char {
    if !ci.is_null() && unsafe { (*ci).callstatus & CIST_TAIL == 0 } {
        unsafe { funcnamefromcall(L, (*ci).previous, name) }
    } else {
        ptr::null()
    }
}

unsafe fn auxgetinfo(
    L: *mut lua_State,
    mut what: *const c_char,
    ar: &mut LuaDebug,
    f: *mut Closure,
    ci: *mut CallInfo,
) -> c_int {
    let mut status = 1;
    while !what.is_null() && unsafe { *what } != 0 {
        match unsafe { *what as u8 } {
            b'S' => unsafe { funcinfo(ar, f) },
            b'l' => {
                ar.currentline = if !ci.is_null() && unsafe { isLua(ci) } {
                    unsafe { getcurrentline(ci) }
                } else {
                    -1
                }
            }
            b'u' => {
                ar.nups = if f.is_null() {
                    0
                } else {
                    unsafe { (*f.cast::<CClosure>()).nupvalues }
                };
                if f.is_null() || unsafe { (*f.cast::<CClosure>()).tt != LUA_VLCL } {
                    ar.isvararg = 1;
                    ar.nparams = 0;
                } else {
                    let p = unsafe { pdebug((*f.cast::<LClosure>()).p) };
                    ar.isvararg = c_char::from((unsafe { (*p).flag & PF_VAHID }) != 0);
                    ar.nparams = unsafe { (*p).numparams };
                }
            }
            b't' => {
                if !ci.is_null() {
                    ar.istailcall = c_char::from((unsafe { (*ci).callstatus & CIST_TAIL }) != 0);
                    ar.extraargs =
                        unsafe { (((*ci).callstatus & MAX_CCMT) >> CIST_CCMT) as c_uchar };
                } else {
                    ar.istailcall = 0;
                    ar.extraargs = 0;
                }
            }
            b'n' => {
                ar.namewhat = unsafe { getfuncname(L, ci, &mut ar.name) };
                if ar.namewhat.is_null() {
                    ar.namewhat = STR_EMPTY.as_ptr().cast();
                    ar.name = ptr::null();
                }
            }
            b'r' => {
                if ci.is_null() || unsafe { (*ci).callstatus & CIST_HOOKED == 0 } {
                    ar.ftransfer = 0;
                    ar.ntransfer = 0;
                } else {
                    ar.ftransfer = unsafe { (*L).transferinfo.ftransfer };
                    ar.ntransfer = unsafe { (*L).transferinfo.ntransfer };
                }
            }
            b'L' | b'f' => {}
            _ => status = 0,
        }
        what = unsafe { what.add(1) };
    }
    status
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lua_getinfo(
    L: *mut lua_State,
    mut what: *const c_char,
    ar: *mut lua_Debug,
) -> c_int {
    let ar = unsafe { ar_mut(ar) };
    let (ci, func) = if unsafe { *what } == b'>' as c_char {
        let func = unsafe { s2v((*L).top.p.sub(1)) };
        unsafe {
            api_check(ttisfunction(func), "function expected");
            (*L).top.p = (*L).top.p.sub(1);
        }
        what = unsafe { what.add(1) };
        (ptr::null_mut(), func)
    } else {
        let ci = ar.i_ci;
        (ci, unsafe { s2v((*ci).func.p) })
    };
    let cl = if unsafe { ttisclosure(func) } {
        unsafe { clvalue(func) }
    } else {
        ptr::null_mut()
    };
    let status = unsafe { auxgetinfo(L, what, ar, cl, ci) };
    if unsafe { !strchr(what, b'f' as c_int).is_null() } {
        unsafe {
            setobj2s(L, (*L).top.p, func);
            api_incr_top(L);
        }
    }
    if unsafe { !strchr(what, b'L' as c_int).is_null() } {
        unsafe { collectvalidlines(L, cl) };
    }
    status
}

unsafe fn filterpc(pc: c_int, jmptarget: c_int) -> c_int {
    if pc < jmptarget { -1 } else { pc }
}

unsafe fn findsetreg(p: *const ProtoDebug, mut lastpc: c_int, reg: c_int) -> c_int {
    let mut setreg = -1;
    let mut jmptarget = 0;
    if unsafe { test_mmmode(get_opcode(*(*p).code.add(lastpc as usize))) } {
        lastpc -= 1;
    }
    let mut pc = 0;
    while pc < lastpc {
        let i = unsafe { *(*p).code.add(pc as usize) };
        let op = unsafe { get_opcode(i) };
        let a = unsafe { getarg_a(i) };
        let change = match op {
            OP_LOADNIL => {
                let b = unsafe { getarg_b(i) };
                a <= reg && reg <= a + b
            }
            OP_TFORCALL => reg >= a + 2,
            OP_CALL | OP_TAILCALL => reg >= a,
            OP_JMP => {
                let dest = pc + 1 + unsafe { getarg_sj(i) };
                if dest <= lastpc && dest > jmptarget {
                    jmptarget = dest;
                }
                false
            }
            _ => unsafe { test_amode(op) && reg == a },
        };
        if change {
            setreg = unsafe { filterpc(pc, jmptarget) };
        }
        pc += 1;
    }
    setreg
}

unsafe fn kname(p: *const ProtoDebug, index: c_int, name: *mut *const c_char) -> *const c_char {
    let kvalue = unsafe { (*p).k.add(index as usize) };
    if unsafe { ttisstring(kvalue) } {
        unsafe { *name = getstr(tsvalue(kvalue)) };
        STR_CONSTANT.as_ptr().cast()
    } else {
        unsafe { *name = STR_QUESTION.as_ptr().cast() };
        ptr::null()
    }
}

unsafe fn basicgetobjname(
    p: *const ProtoDebug,
    ppc: *mut c_int,
    reg: c_int,
    name: *mut *const c_char,
) -> *const c_char {
    let mut pc = unsafe { *ppc };
    unsafe { *name = luaF_getlocalname(p.cast(), reg + 1, pc) };
    if unsafe { !(*name).is_null() } {
        return STR_LOCAL.as_ptr().cast();
    }
    pc = unsafe { findsetreg(p, pc, reg) };
    unsafe { *ppc = pc };
    if pc != -1 {
        let i = unsafe { *(*p).code.add(pc as usize) };
        match unsafe { get_opcode(i) } {
            OP_MOVE => {
                let b = unsafe { getarg_b(i) };
                if b < unsafe { getarg_a(i) } {
                    return unsafe { basicgetobjname(p, ppc, b, name) };
                }
            }
            OP_GETUPVAL => {
                unsafe { *name = upvalname(p, getarg_b(i)) };
                return STR_UPVALUE.as_ptr().cast();
            }
            OP_LOADK => return unsafe { kname(p, getarg_bx(i), name) },
            OP_LOADKX => {
                let extra = unsafe { *(*p).code.add(pc as usize + 1) };
                return unsafe { kname(p, getarg_ax(extra), name) };
            }
            _ => {}
        }
    }
    ptr::null()
}

unsafe fn rname(p: *const ProtoDebug, pc: c_int, c: c_int, name: *mut *const c_char) {
    let mut pc1 = pc;
    let what = unsafe { basicgetobjname(p, &mut pc1, c, name) };
    if what.is_null() || unsafe { *what } != b'c' as c_char {
        unsafe { *name = STR_QUESTION.as_ptr().cast() };
    }
}

unsafe fn is_env(p: *const ProtoDebug, pc: c_int, i: Instruction, isup: bool) -> *const c_char {
    let t = unsafe { getarg_b(i) };
    let name = if isup {
        unsafe { upvalname(p, t) }
    } else {
        let mut name = ptr::null();
        let mut pc1 = pc;
        let what = unsafe { basicgetobjname(p, &mut pc1, t, &mut name) };
        if what != STR_LOCAL.as_ptr().cast::<c_char>()
            && what != STR_UPVALUE.as_ptr().cast::<c_char>()
        {
            ptr::null()
        } else {
            name
        }
    };
    if !name.is_null() && unsafe { strcmp(name, LUA_ENV.as_ptr().cast()) == 0 } {
        STR_GLOBAL.as_ptr().cast()
    } else {
        STR_FIELD.as_ptr().cast()
    }
}

unsafe fn getobjname(
    p: *const ProtoDebug,
    lastpc: c_int,
    reg: c_int,
    name: *mut *const c_char,
) -> *const c_char {
    let mut pc = lastpc;
    let kind = unsafe { basicgetobjname(p, &mut pc, reg, name) };
    if !kind.is_null() {
        return kind;
    }
    if pc != -1 {
        let i = unsafe { *(*p).code.add(pc as usize) };
        match unsafe { get_opcode(i) } {
            OP_GETTABUP => {
                unsafe { kname(p, getarg_c(i), name) };
                return unsafe { is_env(p, pc, i, true) };
            }
            OP_GETTABLE => {
                unsafe { rname(p, pc, getarg_c(i), name) };
                return unsafe { is_env(p, pc, i, false) };
            }
            OP_GETI => {
                unsafe { *name = STR_INTEGER_INDEX.as_ptr().cast() };
                return STR_FIELD.as_ptr().cast();
            }
            OP_GETFIELD => {
                unsafe { kname(p, getarg_c(i), name) };
                return unsafe { is_env(p, pc, i, false) };
            }
            OP_SELF => {
                unsafe { kname(p, getarg_c(i), name) };
                return STR_METHOD.as_ptr().cast();
            }
            _ => {}
        }
    }
    ptr::null()
}

unsafe fn funcnamefromcode(
    L: *mut lua_State,
    p: *const ProtoDebug,
    pc: c_int,
    name: *mut *const c_char,
) -> *const c_char {
    let i = unsafe { *(*p).code.add(pc as usize) };
    let tm = match unsafe { get_opcode(i) } {
        OP_CALL | OP_TAILCALL => return unsafe { getobjname(p, pc, getarg_a(i), name) },
        OP_TFORCALL => {
            unsafe { *name = STR_FOR_ITER.as_ptr().cast() };
            return STR_FOR_ITER.as_ptr().cast();
        }
        OP_SELF | OP_GETTABUP | OP_GETTABLE | OP_GETI | OP_GETFIELD => TM_INDEX,
        OP_SETTABUP | OP_SETTABLE | OP_SETI | OP_SETFIELD => TM_NEWINDEX,
        OP_MMBIN | OP_MMBINI | OP_MMBINK => unsafe { getarg_c(i) as usize },
        OP_UNM => TM_UNM,
        OP_BNOT => TM_BNOT,
        OP_LEN => TM_LEN,
        OP_CONCAT => TM_CONCAT,
        OP_EQ => TM_EQ,
        OP_LT | OP_LTI | OP_GTI => TM_LT,
        OP_LE | OP_LEI | OP_GEI => TM_LE,
        OP_CLOSE | OP_RETURN => TM_CLOSE,
        _ => return ptr::null(),
    };
    unsafe { *name = getstr((*G(L)).tmname[tm]).add(2) };
    STR_META.as_ptr().cast()
}

unsafe fn funcnamefromcall(
    L: *mut lua_State,
    ci: *mut CallInfo,
    name: *mut *const c_char,
) -> *const c_char {
    if unsafe { (*ci).callstatus & CIST_HOOKED != 0 } {
        unsafe { *name = STR_QUESTION.as_ptr().cast() };
        STR_HOOK.as_ptr().cast()
    } else if unsafe { (*ci).callstatus & CIST_FIN != 0 } {
        unsafe { *name = STR_GC.as_ptr().cast() };
        STR_META.as_ptr().cast()
    } else if unsafe { isLua(ci) } {
        unsafe { funcnamefromcode(L, pdebug((*ci_func(ci)).p), currentpc(ci), name) }
    } else {
        ptr::null()
    }
}

#[inline]
unsafe fn ttisfunction(o: *const TValue) -> bool {
    unsafe { ttype(o) == LUA_TFUNCTION }
}

#[inline]
unsafe fn isLfunction(o: *const TValue) -> bool {
    unsafe { ttisLclosure(o) }
}

#[inline]
unsafe fn ttisclosure(o: *const TValue) -> bool {
    unsafe { ttisLclosure(o) || ttisCclosure(o) }
}

#[inline]
unsafe fn clvalue(o: *const TValue) -> *mut Closure {
    unsafe { gcvalue(o).cast() }
}

unsafe fn instack(ci: *mut CallInfo, o: *const TValue) -> c_int {
    let base = unsafe { (*ci).func.p.add(1) };
    let mut pos = 0;
    while unsafe { base.add(pos as usize) < (*ci).top.p } {
        if ptr::eq(o, unsafe { s2v(base.add(pos as usize)) }) {
            return pos;
        }
        pos += 1;
    }
    -1
}

unsafe fn getupvalname(
    ci: *mut CallInfo,
    o: *const TValue,
    name: *mut *const c_char,
) -> *const c_char {
    let c = unsafe { ci_func(ci) };
    let mut i = 0;
    while i < unsafe { (*c).nupvalues as c_int } {
        if ptr::eq(
            unsafe { (*(*(*c).upvals.as_ptr().add(i as usize))).v.p },
            o.cast_mut(),
        ) {
            unsafe { *name = upvalname(pdebug((*c).p), i) };
            return STR_UPVALUE.as_ptr().cast();
        }
        i += 1;
    }
    ptr::null()
}

unsafe fn formatvarinfo(
    L: *mut lua_State,
    kind: *const c_char,
    name: *const c_char,
) -> *const c_char {
    if kind.is_null() {
        STR_EMPTY.as_ptr().cast()
    } else {
        unsafe { luaO_pushfstring(L, c" (%s '%s')".as_ptr(), kind, name) }
    }
}

unsafe fn varinfo(L: *mut lua_State, o: *const TValue) -> *const c_char {
    let ci = unsafe { (*L).ci };
    let mut name = ptr::null();
    let mut kind = ptr::null();
    if unsafe { isLua(ci) } {
        kind = unsafe { getupvalname(ci, o, &mut name) };
        if kind.is_null() {
            let reg = unsafe { instack(ci, o) };
            if reg >= 0 {
                kind =
                    unsafe { getobjname(pdebug((*ci_func(ci)).p), currentpc(ci), reg, &mut name) };
            }
        }
    }
    unsafe { formatvarinfo(L, kind, name) }
}

unsafe fn typeerror(
    L: *mut lua_State,
    o: *const TValue,
    op: *const c_char,
    extra: *const c_char,
) -> ! {
    let t = unsafe { luaT_objtypename(L, o) };
    unsafe { luaG_runerror(L, c"attempt to %s a %s value%s".as_ptr(), op, t, extra) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_typeerror(
    L: *mut lua_State,
    o: *const TValue,
    op: *const c_char,
) -> ! {
    unsafe { typeerror(L, o, op, varinfo(L, o)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_callerror(L: *mut lua_State, o: *const TValue) -> ! {
    let ci = unsafe { (*L).ci };
    let mut name = ptr::null();
    let kind = unsafe { funcnamefromcall(L, ci, &mut name) };
    let extra = if kind.is_null() {
        unsafe { varinfo(L, o) }
    } else {
        unsafe { formatvarinfo(L, kind, name) }
    };
    unsafe { typeerror(L, o, c"call".as_ptr(), extra) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_forerror(
    L: *mut lua_State,
    o: *const TValue,
    what: *const c_char,
) -> ! {
    unsafe {
        luaG_runerror(
            L,
            c"bad 'for' %s (number expected, got %s)".as_ptr(),
            what,
            luaT_objtypename(L, o),
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_concaterror(
    L: *mut lua_State,
    mut p1: *const TValue,
    p2: *const TValue,
) -> ! {
    if unsafe { ttisstring(p1) || cvt2str(p1) } {
        p1 = p2;
    }
    unsafe { luaG_typeerror(L, p1, c"concatenate".as_ptr()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_opinterror(
    L: *mut lua_State,
    p1: *const TValue,
    mut p2: *const TValue,
    msg: *const c_char,
) -> ! {
    if !unsafe { ttisnumber(p1) } {
        p2 = p1;
    }
    unsafe { luaG_typeerror(L, p2, msg) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_tointerror(
    L: *mut lua_State,
    p1: *const TValue,
    mut p2: *const TValue,
) -> ! {
    let mut temp = 0;
    if unsafe { luaV_tointegerns(p1, &mut temp, LUA_FLOORN2I) } == 0 {
        p2 = p1;
    }
    unsafe {
        luaG_runerror(
            L,
            c"number%s has no integer representation".as_ptr(),
            varinfo(L, p2),
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_ordererror(
    L: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
) -> ! {
    let t1 = unsafe { luaT_objtypename(L, p1) };
    let t2 = unsafe { luaT_objtypename(L, p2) };
    if unsafe { strcmp(t1, t2) == 0 } {
        unsafe { luaG_runerror(L, c"attempt to compare two %s values".as_ptr(), t1) }
    } else {
        unsafe { luaG_runerror(L, c"attempt to compare %s with %s".as_ptr(), t1, t2) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_errnnil(L: *mut lua_State, cl: *mut LClosure, k: c_int) -> ! {
    let mut globalname = STR_QUESTION.as_ptr().cast();
    if k > 0 {
        let _ = unsafe { kname(pdebug((*cl).p), k - 1, &mut globalname) };
    }
    unsafe { luaG_runerror(L, c"global '%s' already defined".as_ptr(), globalname) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_addinfo(
    L: *mut lua_State,
    msg: *const c_char,
    src: *mut TString,
    line: c_int,
) -> *const c_char {
    if src.is_null() {
        unsafe { luaO_pushfstring(L, c"?:?: %s".as_ptr(), msg) }
    } else {
        let mut buff = [0 as c_char; 60];
        let mut idlen = 0;
        let id = unsafe { getlstr(src, &mut idlen) };
        unsafe { luaO_chunkid(buff.as_mut_ptr(), id, idlen) };
        unsafe { luaO_pushfstring(L, c"%s:%d: %s".as_ptr(), buff.as_ptr(), line, msg) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_errormsg(L: *mut lua_State) -> ! {
    if unsafe { (*L).errfunc != 0 } {
        let errfunc = unsafe { restorestack(L, (*L).errfunc) };
        unsafe {
            setobjs2s(L, (*L).top.p, (*L).top.p.sub(1));
            setobjs2s(L, (*L).top.p.sub(1), errfunc);
            (*L).top.p = (*L).top.p.add(1);
            luaD_callnoyield(L, (*L).top.p.sub(2), 1);
        }
    }
    if unsafe { ttisnil(s2v((*L).top.p.sub(1))) } {
        let s = unsafe { luaS_new(L, NO_ERROR_OBJECT.as_ptr().cast()) };
        unsafe { setsvalue2s(L, (*L).top.p.sub(1), s) };
    }
    unsafe { luaD_throw(L, LUA_ERRRUN) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_runerror(L: *mut lua_State, fmt: *const c_char, argp: ...) -> ! {
    let ci = unsafe { (*L).ci };
    unsafe { luaC_checkGC(L) };
    let msg = unsafe { luaO_pushvfstring(L, fmt, argp) };
    if unsafe { isLua(ci) } {
        unsafe {
            luaG_addinfo(
                L,
                msg,
                (*pdebug((*ci_func(ci)).p)).source,
                getcurrentline(ci),
            );
            setobjs2s(L, (*L).top.p.sub(2), (*L).top.p.sub(1));
            (*L).top.p = (*L).top.p.sub(1);
        }
    }
    unsafe { luaG_errormsg(L) }
}

unsafe fn changedline(p: *const ProtoDebug, oldpc: c_int, newpc: c_int) -> c_int {
    if unsafe { (*p).lineinfo.is_null() } {
        return 0;
    }
    if newpc - oldpc < MAXIWTHABS / 2 {
        let mut delta = 0;
        let mut pc = oldpc;
        loop {
            pc += 1;
            let lineinfo = unsafe { *(*p).lineinfo.add(pc as usize) };
            if lineinfo == ABSLINEINFO {
                break;
            }
            delta += lineinfo as c_int;
            if pc == newpc {
                return c_int::from(delta != 0);
            }
        }
    }
    c_int::from(unsafe { luaG_getfuncline(p.cast(), oldpc) != luaG_getfuncline(p.cast(), newpc) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_tracecall(L: *mut lua_State) -> c_int {
    let ci = unsafe { (*L).ci };
    let p = unsafe { pdebug((*ci_func(ci)).p) };
    unsafe { (*ci).u.l.trap = 1 };
    if unsafe { (*ci).u.l.savedpc == (*p).code } {
        if unsafe { (*p).flag & PF_VAHID != 0 } {
            return 0;
        }
        if unsafe { (*ci).callstatus & CIST_HOOKYIELD == 0 } {
            unsafe { luaD_hookcall(L, ci) };
        }
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaG_traceexec(L: *mut lua_State, pc: *const Instruction) -> c_int {
    let ci = unsafe { (*L).ci };
    let mask = unsafe { (*L).hookmask as u8 };
    let p = unsafe { pdebug((*ci_func(ci)).p) };
    if mask & ((LUA_MASKLINE | LUA_MASKCOUNT) as u8) == 0 {
        unsafe { (*ci).u.l.trap = 0 };
        return 0;
    }
    let pc = unsafe { pc.add(1) };
    unsafe { (*ci).u.l.savedpc = pc };
    let counthook = (mask & (LUA_MASKCOUNT as u8) != 0) && {
        unsafe { (*L).hookcount -= 1 };
        unsafe { (*L).hookcount == 0 }
    };
    if counthook {
        unsafe { resethookcount(L) };
    } else if mask & (LUA_MASKLINE as u8) == 0 {
        return 1;
    }
    if unsafe { (*ci).callstatus & CIST_HOOKYIELD != 0 } {
        unsafe { (*ci).callstatus &= !CIST_HOOKYIELD };
        return 1;
    }
    if unsafe { luaP_isIT(*((*ci).u.l.savedpc).sub(1)) } == 0 {
        unsafe { (*L).top.p = (*ci).top.p };
    }
    if counthook {
        unsafe { luaD_hook(L, LUA_HOOKCOUNT, -1, 0, 0) };
    }
    if mask & (LUA_MASKLINE as u8) != 0 {
        let oldpc = if unsafe { (*L).oldpc < (*p).sizecode } {
            unsafe { (*L).oldpc }
        } else {
            0
        };
        let npci = unsafe { pc_rel(pc, p) };
        if npci <= oldpc || unsafe { changedline(p, oldpc, npci) } != 0 {
            let newline = unsafe { luaG_getfuncline(p.cast(), npci) };
            unsafe { luaD_hook(L, LUA_HOOKLINE, newline, 0, 0) };
        }
        unsafe { (*L).oldpc = npci };
    }
    if unsafe { (*L).status == LUA_YIELD } {
        if counthook {
            unsafe { (*L).hookcount = 1 };
        }
        unsafe { (*ci).callstatus |= CIST_HOOKYIELD };
        unsafe { luaD_throw(L, LUA_YIELD) };
    }
    1
}
