#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

use crate::runtime::*;
use core::mem::size_of;
use core::ptr;

const MAXVARS: c_int = 200;

const FIRST_RESERVED: c_int = u8::MAX as c_int + 1;
const TK_AND: c_int = FIRST_RESERVED;
const TK_BREAK: c_int = TK_AND + 1;
const TK_DO: c_int = TK_BREAK + 1;
const TK_ELSE: c_int = TK_DO + 1;
const TK_ELSEIF: c_int = TK_ELSE + 1;
const TK_END: c_int = TK_ELSEIF + 1;
const TK_FALSE: c_int = TK_END + 1;
const TK_FOR: c_int = TK_FALSE + 1;
const TK_FUNCTION: c_int = TK_FOR + 1;
const TK_GLOBAL: c_int = TK_FUNCTION + 1;
const TK_GOTO: c_int = TK_GLOBAL + 1;
const TK_IF: c_int = TK_GOTO + 1;
const TK_IN: c_int = TK_IF + 1;
const TK_LOCAL: c_int = TK_IN + 1;
const TK_NIL: c_int = TK_LOCAL + 1;
const TK_NOT: c_int = TK_NIL + 1;
const TK_OR: c_int = TK_NOT + 1;
const TK_REPEAT: c_int = TK_OR + 1;
const TK_RETURN: c_int = TK_REPEAT + 1;
const TK_THEN: c_int = TK_RETURN + 1;
const TK_TRUE: c_int = TK_THEN + 1;
const TK_UNTIL: c_int = TK_TRUE + 1;
const TK_WHILE: c_int = TK_UNTIL + 1;
const TK_IDIV: c_int = TK_WHILE + 1;
const TK_CONCAT: c_int = TK_IDIV + 1;
const TK_DOTS: c_int = TK_CONCAT + 1;
const TK_EQ: c_int = TK_DOTS + 1;
const TK_GE: c_int = TK_EQ + 1;
const TK_LE: c_int = TK_GE + 1;
const TK_NE: c_int = TK_LE + 1;
const TK_SHL: c_int = TK_NE + 1;
const TK_SHR: c_int = TK_SHL + 1;
const TK_DBCOLON: c_int = TK_SHR + 1;
const TK_EOS: c_int = TK_DBCOLON + 1;
const TK_FLT: c_int = TK_EOS + 1;
const TK_INT: c_int = TK_FLT + 1;
const TK_NAME: c_int = TK_INT + 1;
const TK_STRING: c_int = TK_NAME + 1;

const PF_VAHID: u8 = 1;
const PF_VATAB: u8 = 2;

const MAXARG_Bx: c_int = 131071;
const MAXARG_Ax: c_int = 33554431;
const MAXARG_vC: c_int = 1023;
const MAX_FSTACK: c_int = 255;

const POS_OP: u32 = 0;
const POS_A: u32 = 7;
const POS_K: u32 = 15;
const POS_B: u32 = 16;
const POS_VB: u32 = 16;
const POS_C: u32 = 24;
const POS_VC: u32 = 22;
const POS_BX: u32 = 15;

const SIZE_OP: u32 = 7;
const SIZE_A: u32 = 8;
const SIZE_B: u32 = 8;
const SIZE_C: u32 = 8;
const SIZE_BX: u32 = 17;

const OP_MOVE: c_int = 0;
const OP_GETUPVAL: c_int = 9;
const OP_NEWTABLE: c_int = 19;
const OP_SELF: c_int = 20;
const OP_CONCAT: c_int = 53;
const OP_CLOSE: c_int = 54;
const OP_TBC: c_int = 55;
const OP_CALL: c_int = 68;
const OP_TAILCALL: c_int = 69;
const OP_RETURN: c_int = 70;
const OP_FORLOOP: c_int = 73;
const OP_FORPREP: c_int = 74;
const OP_TFORPREP: c_int = 75;
const OP_TFORCALL: c_int = 76;
const OP_TFORLOOP: c_int = 77;
const OP_SETLIST: c_int = 78;
const OP_CLOSURE: c_int = 79;
const OP_VARARG: c_int = 80;
const OP_ERRNNIL: c_int = 82;
const OP_VARARGPREP: c_int = 83;

const NO_JUMP: c_int = -1;

const VVOID: c_int = 0;
const VNIL: c_int = 1;
const VTRUE: c_int = 2;
const VFALSE: c_int = 3;
const VK: c_int = 4;
const VKFLT: c_int = 5;
const VKINT: c_int = 6;
const VKSTR: c_int = 7;
const VNONRELOC: c_int = 8;
const VLOCAL: c_int = 9;
const VVARGVAR: c_int = 10;
const VGLOBAL: c_int = 11;
const VUPVAL: c_int = 12;
const VCONST: c_int = 13;
const VINDEXED: c_int = 14;
const VVARGIND: c_int = 15;
const VINDEXUP: c_int = 16;
const VINDEXI: c_int = 17;
const VINDEXSTR: c_int = 18;
const VJMP: c_int = 19;
const VRELOC: c_int = 20;
const VCALL: c_int = 21;
const VVARARG: c_int = 22;

const VDKREG: u8 = 0;
const RDKCONST: u8 = 1;
const RDKVAVAR: u8 = 2;
const RDKTOCLOSE: u8 = 3;
const RDKCTC: u8 = 4;
const GDKREG: u8 = 5;
const GDKCONST: u8 = 6;

const OPR_ADD: c_int = 0;
const OPR_SUB: c_int = 1;
const OPR_MUL: c_int = 2;
const OPR_MOD: c_int = 3;
const OPR_POW: c_int = 4;
const OPR_DIV: c_int = 5;
const OPR_IDIV: c_int = 6;
const OPR_BAND: c_int = 7;
const OPR_BOR: c_int = 8;
const OPR_BXOR: c_int = 9;
const OPR_SHL: c_int = 10;
const OPR_SHR: c_int = 11;
const OPR_CONCAT: c_int = 12;
const OPR_EQ: c_int = 13;
const OPR_LT: c_int = 14;
const OPR_LE: c_int = 15;
const OPR_NE: c_int = 16;
const OPR_GT: c_int = 17;
const OPR_GE: c_int = 18;
const OPR_AND: c_int = 19;
const OPR_OR: c_int = 20;
const OPR_NOBINOPR: c_int = 21;

const OPR_MINUS: c_int = 0;
const OPR_BNOT: c_int = 1;
const OPR_NOT: c_int = 2;
const OPR_LEN: c_int = 3;
const OPR_NOUNOPR: c_int = 4;

const UNARY_PRIORITY: c_int = 12;

static PRIORITY: [(u8, u8); OPR_NOBINOPR as usize] = [
    (10, 10),
    (10, 10),
    (11, 11),
    (11, 11),
    (14, 13),
    (11, 11),
    (11, 11),
    (6, 6),
    (4, 4),
    (5, 5),
    (7, 7),
    (7, 7),
    (9, 8),
    (3, 3),
    (3, 3),
    (3, 3),
    (3, 3),
    (3, 3),
    (3, 3),
    (2, 2),
    (1, 1),
];

#[derive(Copy, Clone)]
#[repr(C)]
union SemInfo {
    r: lua_Number,
    i: lua_Integer,
    ts: *mut TString,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Token {
    token: c_int,
    seminfo: SemInfo,
}

#[repr(C)]
struct Mbuffer {
    buffer: *mut c_char,
    n: usize,
    buffsize: usize,
}

#[repr(C)]
struct LexState {
    current: c_int,
    linenumber: c_int,
    lastline: c_int,
    t: Token,
    lookahead: Token,
    fs: *mut FuncState,
    L: *mut lua_State,
    z: *mut ZIO,
    buff: *mut Mbuffer,
    h: *mut Table,
    dyd: *mut Dyndata,
    source: *mut TString,
    envn: *mut TString,
    brkn: *mut TString,
    glbn: *mut TString,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct ExpdescInd {
    idx: i16,
    t: lu_byte,
    ro: lu_byte,
    keystr: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct ExpdescVar {
    ridx: lu_byte,
    vidx: i16,
}

#[derive(Copy, Clone)]
#[repr(C)]
union ExpdescUnion {
    ival: lua_Integer,
    nval: lua_Number,
    strval: *mut TString,
    info: c_int,
    ind: ExpdescInd,
    var: ExpdescVar,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct expdesc {
    k: c_int,
    u: ExpdescUnion,
    t: c_int,
    f: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct VardescFields {
    value_: Value,
    tt_: u8,
    kind: lu_byte,
    ridx: lu_byte,
    pidx: i16,
    name: *mut TString,
}

#[derive(Copy, Clone)]
#[repr(C)]
union Vardesc {
    vd: VardescFields,
    k: TValue,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Labeldesc {
    name: *mut TString,
    pc: c_int,
    line: c_int,
    nactvar: i16,
    close: lu_byte,
}

#[repr(C)]
struct Labellist {
    arr: *mut Labeldesc,
    n: c_int,
    size: c_int,
}

#[repr(C)]
struct VardescList {
    arr: *mut Vardesc,
    n: c_int,
    size: c_int,
}

#[repr(C)]
struct Dyndata {
    actvar: VardescList,
    gt: Labellist,
    label: Labellist,
}

#[repr(C)]
struct FuncState {
    f: *mut Proto,
    prev: *mut FuncState,
    ls: *mut LexState,
    bl: *mut BlockCnt,
    kcache: *mut Table,
    pc: c_int,
    lasttarget: c_int,
    previousline: c_int,
    nk: c_int,
    np: c_int,
    nabslineinfo: c_int,
    firstlocal: c_int,
    firstlabel: c_int,
    ndebugvars: i16,
    nactvar: i16,
    nups: lu_byte,
    freereg: lu_byte,
    iwthabs: lu_byte,
    needclose: lu_byte,
}

#[repr(C)]
struct BlockCnt {
    previous: *mut BlockCnt,
    firstlabel: c_int,
    firstgoto: c_int,
    nactvar: i16,
    upval: lu_byte,
    isloop: lu_byte,
    insidetbc: lu_byte,
}

#[repr(C)]
struct LocVar {
    varname: *mut TString,
    startpc: c_int,
    endpc: c_int,
}

#[repr(C)]
struct AbsLineInfo {
    pc: c_int,
    line: c_int,
}

#[repr(C)]
struct LHS_assign {
    prev: *mut LHS_assign,
    v: expdesc,
}

#[repr(C)]
struct ConsControl {
    v: expdesc,
    t: *mut expdesc,
    nh: c_int,
    na: c_int,
    tostore: c_int,
    maxtostore: c_int,
}

unsafe extern "C-unwind" {
    fn luaE_incCstack(L: *mut lua_State);
    fn luaD_inctop(L: *mut lua_State);

    fn luaF_newproto(L: *mut lua_State) -> *mut Proto;
    fn luaF_newLclosure(L: *mut lua_State, nupvals: c_int) -> *mut LClosure;

    fn luaX_setinput(
        L: *mut lua_State,
        ls: *mut LexState,
        z: *mut ZIO,
        source: *mut TString,
        firstchar: c_int,
    );
    fn luaX_newstring(ls: *mut LexState, s: *const c_char, l: usize) -> *mut TString;
    fn luaX_next(ls: *mut LexState);
    fn luaX_lookahead(ls: *mut LexState) -> c_int;
    fn luaX_syntaxerror(ls: *mut LexState, s: *const c_char) -> !;
    fn luaX_token2str(ls: *mut LexState, token: c_int) -> *const c_char;

    fn luaK_code(fs: *mut FuncState, i: Instruction) -> c_int;
    fn luaK_codeABx(fs: *mut FuncState, o: c_int, a: c_int, bx: c_int) -> c_int;
    fn luaK_codeABCk(fs: *mut FuncState, o: c_int, a: c_int, b: c_int, c: c_int, k: c_int)
        -> c_int;
    fn luaK_codevABCk(fs: *mut FuncState, o: c_int, a: c_int, b: c_int, c: c_int, k: c_int)
        -> c_int;
    fn luaK_exp2const(fs: *mut FuncState, e: *const expdesc, v: *mut TValue) -> c_int;
    fn luaK_fixline(fs: *mut FuncState, line: c_int);
    fn luaK_nil(fs: *mut FuncState, from: c_int, n: c_int);
    fn luaK_codecheckglobal(fs: *mut FuncState, var: *mut expdesc, k: c_int, line: c_int);
    fn luaK_reserveregs(fs: *mut FuncState, n: c_int);
    fn luaK_checkstack(fs: *mut FuncState, n: c_int);
    fn luaK_int(fs: *mut FuncState, reg: c_int, n: lua_Integer);
    fn luaK_vapar2local(fs: *mut FuncState, var: *mut expdesc);
    fn luaK_dischargevars(fs: *mut FuncState, e: *mut expdesc);
    fn luaK_exp2anyreg(fs: *mut FuncState, e: *mut expdesc) -> c_int;
    fn luaK_exp2anyregup(fs: *mut FuncState, e: *mut expdesc);
    fn luaK_exp2nextreg(fs: *mut FuncState, e: *mut expdesc);
    fn luaK_exp2val(fs: *mut FuncState, e: *mut expdesc);
    fn luaK_self(fs: *mut FuncState, e: *mut expdesc, key: *mut expdesc);
    fn luaK_indexed(fs: *mut FuncState, t: *mut expdesc, k: *mut expdesc);
    fn luaK_goiftrue(fs: *mut FuncState, e: *mut expdesc);
    fn luaK_storevar(fs: *mut FuncState, var: *mut expdesc, e: *mut expdesc);
    fn luaK_setreturns(fs: *mut FuncState, e: *mut expdesc, nresults: c_int);
    fn luaK_setoneret(fs: *mut FuncState, e: *mut expdesc);
    fn luaK_jump(fs: *mut FuncState) -> c_int;
    fn luaK_ret(fs: *mut FuncState, first: c_int, nret: c_int);
    fn luaK_patchlist(fs: *mut FuncState, list: c_int, target: c_int);
    fn luaK_patchtohere(fs: *mut FuncState, list: c_int);
    fn luaK_concat(fs: *mut FuncState, l1: *mut c_int, l2: c_int);
    fn luaK_getlabel(fs: *mut FuncState) -> c_int;
    fn luaK_prefix(fs: *mut FuncState, op: c_int, v: *mut expdesc, line: c_int);
    fn luaK_infix(fs: *mut FuncState, op: c_int, v: *mut expdesc);
    fn luaK_posfix(fs: *mut FuncState, op: c_int, v1: *mut expdesc, v2: *mut expdesc, line: c_int);
    fn luaK_settablesize(fs: *mut FuncState, pc: c_int, ra: c_int, asize: c_int, hsize: c_int);
    fn luaK_setlist(fs: *mut FuncState, base: c_int, nelems: c_int, tostore: c_int);
    fn luaK_finish(fs: *mut FuncState);
    fn luaK_semerror(ls: *mut LexState, fmt: *const c_char, ...) -> !;

    fn luaM_growaux_(
        L: *mut lua_State,
        block: *mut c_void,
        nelems: c_int,
        size: *mut c_int,
        size_elem: c_uint,
        limit: c_int,
        what: *const c_char,
    ) -> *mut c_void;
    fn luaM_shrinkvector_(
        L: *mut lua_State,
        block: *mut c_void,
        nelem: *mut c_int,
        final_n: c_int,
        size_elem: c_uint,
    ) -> *mut c_void;

    fn luaO_pushfstring(L: *mut lua_State, fmt: *const c_char, ...) -> *const c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

#[inline]
unsafe fn mask1(n: u32, p: u32) -> Instruction {
    ((!((!0u32) << n)) << p) as Instruction
}

#[inline]
unsafe fn setarg(i: &mut Instruction, v: c_int, pos: u32, size: u32) {
    *i = (*i & !((mask1(size, pos)) as Instruction)) | (((v as u32) << pos) & mask1(size, pos));
}

#[inline]
unsafe fn GETARG_A(i: Instruction) -> c_int {
    ((i >> POS_A) & mask1(SIZE_A, 0)) as c_int
}

#[inline]
unsafe fn SETARG_Bx(i: &mut Instruction, v: c_int) {
    setarg(i, v, POS_BX, SIZE_BX);
}

#[inline]
unsafe fn SETARG_C(i: &mut Instruction, v: c_int) {
    setarg(i, v, POS_C, SIZE_C);
}

#[inline]
unsafe fn SET_OPCODE(i: &mut Instruction, o: c_int) {
    setarg(i, o, POS_OP, SIZE_OP);
}

#[inline]
unsafe fn CREATE_ABCk(o: c_int, a: c_int, b: c_int, c: c_int, k: c_int) -> Instruction {
    ((o as u32) << POS_OP)
        | ((a as u32) << POS_A)
        | ((b as u32) << POS_B)
        | ((c as u32) << POS_C)
        | ((k as u32) << POS_K)
}

#[inline]
unsafe fn hasmultret(k: c_int) -> bool {
    k == VCALL || k == VVARARG
}

#[inline]
unsafe fn eqstr(a: *mut TString, b: *mut TString) -> bool {
    ptr::eq(a, b)
}

#[inline]
unsafe fn vkisvar(k: c_int) -> bool {
    (VLOCAL..=VINDEXSTR).contains(&k)
}

#[inline]
unsafe fn vkisindexed(k: c_int) -> bool {
    (VINDEXED..=VINDEXSTR).contains(&k)
}

#[inline]
unsafe fn varinreg(v: *const Vardesc) -> bool {
    (*v).vd.kind <= RDKTOCLOSE
}

#[inline]
unsafe fn varglobal(v: *const Vardesc) -> bool {
    (*v).vd.kind >= GDKREG
}

#[inline]
unsafe fn strisshr(ts: *mut TString) -> bool {
    (*ts).shrlen >= 0
}

#[inline]
unsafe fn isvararg(p: *mut Proto) -> bool {
    (*p).flag & (PF_VAHID | PF_VATAB) != 0
}

#[inline]
unsafe fn needvatab(p: *mut Proto) {
    (*p).flag |= PF_VATAB;
}

#[inline]
unsafe fn setclLvalue2s(_L: *mut lua_State, o: StkId, cl: *mut LClosure) {
    (*s2v(o)).value_.gc = cl.cast();
    settt_(s2v(o), LUA_VLCL | BIT_ISCOLLECTABLE);
}

#[inline]
unsafe fn cast_byte(v: c_int) -> lu_byte {
    v as lu_byte
}

#[inline]
unsafe fn cast_short(v: c_int) -> i16 {
    v as i16
}

#[inline]
unsafe fn grow_vector<T>(
    L: *mut lua_State,
    block: *mut T,
    nelems: c_int,
    size: &mut c_int,
    limit: c_int,
    what: *const c_char,
) -> *mut T {
    luaM_growaux_(
        L,
        block.cast(),
        nelems,
        size,
        size_of::<T>() as c_uint,
        limit,
        what,
    )
    .cast()
}

#[inline]
unsafe fn shrink_vector<T>(
    L: *mut lua_State,
    block: *mut T,
    size: &mut c_int,
    final_n: c_int,
) -> *mut T {
    luaM_shrinkvector_(L, block.cast(), size, final_n, size_of::<T>() as c_uint).cast()
}

#[inline]
unsafe fn getlocvars(p: *mut Proto) -> *mut LocVar {
    (*p).locvars.cast()
}

#[inline]
unsafe fn getabslineinfo(p: *mut Proto) -> *mut AbsLineInfo {
    (*p).abslineinfo.cast()
}

#[inline]
unsafe fn getinstruction(fs: *mut FuncState, e: *mut expdesc) -> *mut Instruction {
    (*(*fs).f).code.add((*e).u.info as usize)
}

unsafe fn error_expected(ls: *mut LexState, token: c_int) -> ! {
    luaX_syntaxerror(
        ls,
        luaO_pushfstring((*ls).L, c"%s expected".as_ptr(), luaX_token2str(ls, token)),
    )
}

unsafe fn errorlimit(fs: *mut FuncState, limit: c_int, what: *const c_char) -> ! {
    let L = (*(*fs).ls).L;
    let line = (*(*fs).f).linedefined;
    let where_ = if line == 0 {
        c"main function".as_ptr()
    } else {
        luaO_pushfstring(L, c"function at line %d".as_ptr(), line)
    };
    let msg = luaO_pushfstring(
        L,
        c"too many %s (limit is %d) in %s".as_ptr(),
        what,
        limit,
        where_,
    );
    luaX_syntaxerror((*fs).ls, msg)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaY_checklimit(
    fs: *mut FuncState,
    v: c_int,
    l: c_int,
    what: *const c_char,
) {
    if v > l {
        errorlimit(fs, l, what);
    }
}

unsafe fn testnext(ls: *mut LexState, c: c_int) -> c_int {
    if (*ls).t.token == c {
        luaX_next(ls);
        1
    } else {
        0
    }
}

unsafe fn check(ls: *mut LexState, c: c_int) {
    if (*ls).t.token != c {
        error_expected(ls, c);
    }
}

unsafe fn checknext(ls: *mut LexState, c: c_int) {
    check(ls, c);
    luaX_next(ls);
}

unsafe fn check_condition(ls: *mut LexState, cond: bool, msg: *const c_char) {
    if !cond {
        luaX_syntaxerror(ls, msg);
    }
}

unsafe fn check_match(ls: *mut LexState, what: c_int, who: c_int, where_: c_int) {
    if testnext(ls, what) == 0 {
        if where_ == (*ls).linenumber {
            error_expected(ls, what);
        } else {
            luaX_syntaxerror(
                ls,
                luaO_pushfstring(
                    (*ls).L,
                    c"%s expected (to close %s at line %d)".as_ptr(),
                    luaX_token2str(ls, what),
                    luaX_token2str(ls, who),
                    where_,
                ),
            );
        }
    }
}

unsafe fn str_checkname(ls: *mut LexState) -> *mut TString {
    check(ls, TK_NAME);
    let ts = (*ls).t.seminfo.ts;
    luaX_next(ls);
    ts
}

unsafe fn init_exp(e: *mut expdesc, k: c_int, i: c_int) {
    (*e).f = NO_JUMP;
    (*e).t = NO_JUMP;
    (*e).k = k;
    (*e).u.info = i;
}

unsafe fn codestring(e: *mut expdesc, s: *mut TString) {
    (*e).f = NO_JUMP;
    (*e).t = NO_JUMP;
    (*e).k = VKSTR;
    (*e).u.strval = s;
}

unsafe fn codename(ls: *mut LexState, e: *mut expdesc) {
    codestring(e, str_checkname(ls));
}

unsafe fn registerlocalvar(ls: *mut LexState, fs: *mut FuncState, varname: *mut TString) -> i16 {
    let f = (*fs).f;
    let mut oldsize = (*f).sizelocvars;
    (*f).locvars = grow_vector::<LocVar>(
        (*ls).L,
        getlocvars(f),
        (*fs).ndebugvars as c_int,
        &mut (*f).sizelocvars,
        SHRT_MAX,
        c"local variables".as_ptr(),
    )
    .cast();
    while oldsize < (*f).sizelocvars {
        (*getlocvars(f).add(oldsize as usize)).varname = ptr::null_mut();
        oldsize += 1;
    }
    (*getlocvars(f).add((*fs).ndebugvars as usize)).varname = varname;
    (*getlocvars(f).add((*fs).ndebugvars as usize)).startpc = (*fs).pc;
    luaC_objbarrier((*ls).L, obj2gco(f), obj2gco(varname));
    let res = (*fs).ndebugvars;
    (*fs).ndebugvars += 1;
    res
}

unsafe fn new_varkind(ls: *mut LexState, name: *mut TString, kind: lu_byte) -> c_int {
    let L = (*ls).L;
    let fs = (*ls).fs;
    let dyd = (*ls).dyd;
    (*dyd).actvar.arr = grow_vector::<Vardesc>(
        L,
        (*dyd).actvar.arr,
        (*dyd).actvar.n + 1,
        &mut (*dyd).actvar.size,
        SHRT_MAX,
        c"variable declarations".as_ptr(),
    );
    let var = (*dyd).actvar.arr.add((*dyd).actvar.n as usize);
    (*var).vd.kind = kind;
    (*var).vd.name = name;
    (*dyd).actvar.n += 1;
    (*dyd).actvar.n - 1 - (*fs).firstlocal
}

unsafe fn new_localvar(ls: *mut LexState, name: *mut TString) -> c_int {
    new_varkind(ls, name, VDKREG)
}

unsafe fn new_localvarliteral(ls: *mut LexState, v: &'static [u8]) -> c_int {
    new_localvar(ls, luaX_newstring(ls, v.as_ptr().cast(), v.len() - 1))
}

unsafe fn getlocalvardesc(fs: *mut FuncState, vidx: c_int) -> *mut Vardesc {
    (*(*(*fs).ls).dyd).actvar.arr.add(((*fs).firstlocal + vidx) as usize)
}

unsafe fn reglevel(fs: *mut FuncState, mut nvar: c_int) -> lu_byte {
    while nvar > 0 {
        nvar -= 1;
        let vd = getlocalvardesc(fs, nvar);
        if varinreg(vd) {
            return cast_byte((*vd).vd.ridx as c_int + 1);
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaY_nvarstack(fs: *mut FuncState) -> lu_byte {
    reglevel(fs, (*fs).nactvar as c_int)
}

unsafe fn localdebuginfo(fs: *mut FuncState, vidx: c_int) -> *mut LocVar {
    let vd = getlocalvardesc(fs, vidx);
    if !varinreg(vd) {
        ptr::null_mut()
    } else {
        let idx = (*vd).vd.pidx;
        debug_assert!(idx < (*fs).ndebugvars);
        getlocvars((*fs).f).add(idx as usize)
    }
}

unsafe fn init_var(fs: *mut FuncState, e: *mut expdesc, vidx: c_int) {
    (*e).f = NO_JUMP;
    (*e).t = NO_JUMP;
    (*e).k = VLOCAL;
    (*e).u.var.vidx = cast_short(vidx);
    (*e).u.var.ridx = (*getlocalvardesc(fs, vidx)).vd.ridx;
}

unsafe fn check_readonly(ls: *mut LexState, e: *mut expdesc) {
    let fs = (*ls).fs;
    let mut varname: *mut TString = ptr::null_mut();
    match (*e).k {
        VCONST => {
            varname = (*(*ls).dyd).actvar.arr.add((*e).u.info as usize).as_ref().unwrap().vd.name;
        }
        VLOCAL | VVARGVAR => {
            let vardesc = getlocalvardesc(fs, (*e).u.var.vidx as c_int);
            if (*vardesc).vd.kind != VDKREG {
                varname = (*vardesc).vd.name;
            }
        }
        VUPVAL => {
            let up = (*(*fs).f).upvalues.add((*e).u.info as usize);
            if (*up).kind != VDKREG {
                varname = (*up).name;
            }
        }
        VVARGIND => {
            needvatab((*fs).f);
            (*e).k = VINDEXED;
            if (*e).u.ind.ro != 0 {
                varname = tsvalue((*(*fs).f).k.add((*e).u.ind.keystr as usize));
            }
        }
        VINDEXUP | VINDEXSTR | VINDEXED => {
            if (*e).u.ind.ro != 0 {
                varname = tsvalue((*(*fs).f).k.add((*e).u.ind.keystr as usize));
            }
        }
        _ => {
            debug_assert!((*e).k == VINDEXI);
            return;
        }
    }
    if !varname.is_null() {
        luaK_semerror(
            ls,
            c"attempt to assign to const variable '%s'".as_ptr(),
            getstr(varname),
        );
    }
}

unsafe fn adjustlocalvars(ls: *mut LexState, nvars: c_int) {
    let fs = (*ls).fs;
    let mut reglevel_ = luaY_nvarstack(fs) as c_int;
    for _ in 0..nvars {
        let vidx = (*fs).nactvar as c_int;
        (*fs).nactvar += 1;
        let var = getlocalvardesc(fs, vidx);
        (*var).vd.ridx = cast_byte(reglevel_);
        reglevel_ += 1;
        (*var).vd.pidx = registerlocalvar(ls, fs, (*var).vd.name);
        luaY_checklimit(fs, reglevel_, MAXVARS, c"local variables".as_ptr());
    }
}

unsafe fn removevars(fs: *mut FuncState, tolevel: c_int) {
    (*(*(*fs).ls).dyd).actvar.n -= (*fs).nactvar as c_int - tolevel;
    while (*fs).nactvar as c_int > tolevel {
        (*fs).nactvar -= 1;
        let var = localdebuginfo(fs, (*fs).nactvar as c_int);
        if !var.is_null() {
            (*var).endpc = (*fs).pc;
        }
    }
}

unsafe fn searchupvalue(fs: *mut FuncState, name: *mut TString) -> c_int {
    let up = (*(*fs).f).upvalues;
    for i in 0..(*fs).nups as c_int {
        if eqstr((*up.add(i as usize)).name, name) {
            return i;
        }
    }
    -1
}

unsafe fn allocupvalue(fs: *mut FuncState) -> *mut Upvaldesc {
    let f = (*fs).f;
    let mut oldsize = (*f).sizeupvalues;
    luaY_checklimit(fs, (*fs).nups as c_int + 1, MAXUPVAL, c"upvalues".as_ptr());
    (*f).upvalues = grow_vector::<Upvaldesc>(
        (*(*fs).ls).L,
        (*f).upvalues,
        (*fs).nups as c_int,
        &mut (*f).sizeupvalues,
        MAXUPVAL,
        c"upvalues".as_ptr(),
    );
    while oldsize < (*f).sizeupvalues {
        (*(*f).upvalues.add(oldsize as usize)).name = ptr::null_mut();
        oldsize += 1;
    }
    let up = (*f).upvalues.add((*fs).nups as usize);
    (*fs).nups = (*fs).nups.wrapping_add(1);
    up
}

unsafe fn newupvalue(fs: *mut FuncState, name: *mut TString, v: *mut expdesc) -> c_int {
    let up = allocupvalue(fs);
    let prev = (*fs).prev;
    if (*v).k == VLOCAL {
        (*up).instack = 1;
        (*up).idx = (*v).u.var.ridx;
        (*up).kind = (*getlocalvardesc(prev, (*v).u.var.vidx as c_int)).vd.kind;
    } else {
        (*up).instack = 0;
        (*up).idx = cast_byte((*v).u.info);
        (*up).kind = (*(*prev).f).upvalues.add((*v).u.info as usize).as_ref().unwrap().kind;
    }
    (*up).name = name;
    luaC_objbarrier((*(*fs).ls).L, obj2gco((*fs).f), obj2gco(name));
    (*fs).nups as c_int - 1
}

unsafe fn searchvar(fs: *mut FuncState, n: *mut TString, var: *mut expdesc) -> c_int {
    let mut i = (*fs).nactvar as c_int - 1;
    while i >= 0 {
        let vd = getlocalvardesc(fs, i);
        if varglobal(vd) {
            if (*vd).vd.name.is_null() {
                if (*var).u.info < 0 {
                    (*var).u.info = (*fs).firstlocal + i;
                }
            } else if eqstr(n, (*vd).vd.name) {
                init_exp(var, VGLOBAL, (*fs).firstlocal + i);
                return VGLOBAL;
            } else if (*var).u.info == -1 {
                (*var).u.info = -2;
            }
        } else if eqstr(n, (*vd).vd.name) {
            if (*vd).vd.kind == RDKCTC {
                init_exp(var, VCONST, (*fs).firstlocal + i);
            } else {
                init_var(fs, var, i);
                if (*vd).vd.kind == RDKVAVAR {
                    (*var).k = VVARGVAR;
                }
            }
            return (*var).k;
        }
        i -= 1;
    }
    -1
}

unsafe fn markupval(fs: *mut FuncState, level: c_int) {
    let mut bl = (*fs).bl;
    while (*bl).nactvar as c_int > level {
        bl = (*bl).previous;
    }
    (*bl).upval = 1;
    (*fs).needclose = 1;
}

unsafe fn marktobeclosed(fs: *mut FuncState) {
    let bl = (*fs).bl;
    (*bl).upval = 1;
    (*bl).insidetbc = 1;
    (*fs).needclose = 1;
}

unsafe fn singlevaraux(fs: *mut FuncState, n: *mut TString, var: *mut expdesc, base: c_int) {
    let v = searchvar(fs, n, var);
    if v >= 0 {
        if base == 0 {
            if (*var).k == VVARGVAR {
                luaK_vapar2local(fs, var);
            }
            if (*var).k == VLOCAL {
                markupval(fs, (*var).u.var.vidx as c_int);
            }
        }
    } else {
        let mut idx = searchupvalue(fs, n);
        if idx < 0 {
            if !(*fs).prev.is_null() {
                singlevaraux((*fs).prev, n, var, 0);
            }
            if (*var).k == VLOCAL || (*var).k == VUPVAL {
                idx = newupvalue(fs, n, var);
            } else {
                return;
            }
        }
        init_exp(var, VUPVAL, idx);
    }
}

unsafe fn buildglobal(ls: *mut LexState, varname: *mut TString, var: *mut expdesc) {
    let fs = (*ls).fs;
    let mut key = expdesc {
        k: 0,
        u: ExpdescUnion { info: 0 },
        t: 0,
        f: 0,
    };
    init_exp(var, VGLOBAL, -1);
    singlevaraux(fs, (*ls).envn, var, 1);
    if (*var).k == VGLOBAL {
        luaK_semerror(
            ls,
            c"%s is global when accessing variable '%s'".as_ptr(),
            c"_ENV".as_ptr(),
            getstr(varname),
        );
    }
    luaK_exp2anyregup(fs, var);
    codestring(&mut key, varname);
    luaK_indexed(fs, var, &mut key);
}

unsafe fn buildvar(ls: *mut LexState, varname: *mut TString, var: *mut expdesc) {
    let fs = (*ls).fs;
    init_exp(var, VGLOBAL, -1);
    singlevaraux(fs, varname, var, 1);
    if (*var).k == VGLOBAL {
        let info = (*var).u.info;
        if info == -2 {
            luaK_semerror(ls, c"variable '%s' not declared".as_ptr(), getstr(varname));
        }
        buildglobal(ls, varname, var);
        if info != -1 && (*(*(*ls).dyd).actvar.arr.add(info as usize)).vd.kind == GDKCONST {
            (*var).u.ind.ro = 1;
        }
    }
}

unsafe fn singlevar(ls: *mut LexState, var: *mut expdesc) {
    buildvar(ls, str_checkname(ls), var);
}

unsafe fn adjust_assign(ls: *mut LexState, nvars: c_int, nexps: c_int, e: *mut expdesc) {
    let fs = (*ls).fs;
    let needed = nvars - nexps;
    luaK_checkstack(fs, needed);
    if hasmultret((*e).k) {
        let mut extra = needed + 1;
        if extra < 0 {
            extra = 0;
        }
        luaK_setreturns(fs, e, extra);
    } else {
        if (*e).k != VVOID {
            luaK_exp2nextreg(fs, e);
        }
        if needed > 0 {
            luaK_nil(fs, (*fs).freereg as c_int, needed);
        }
    }
    if needed > 0 {
        luaK_reserveregs(fs, needed);
    } else {
        (*fs).freereg = ((*fs).freereg as c_int + needed) as u8;
    }
}

#[inline]
unsafe fn enterlevel(ls: *mut LexState) {
    luaE_incCstack((*ls).L);
}

#[inline]
unsafe fn leavelevel(ls: *mut LexState) {
    (*(*ls).L).nCcalls -= 1;
}

unsafe fn jumpscopeerror(ls: *mut LexState, gt: *mut Labeldesc) -> ! {
    let tsname = (*getlocalvardesc((*ls).fs, (*gt).nactvar as c_int)).vd.name;
    let varname = if tsname.is_null() { c"*".as_ptr() } else { getstr(tsname) };
    luaK_semerror(
        ls,
        c"<goto %s> at line %d jumps into the scope of '%s'".as_ptr(),
        getstr((*gt).name),
        (*gt).line,
        varname,
    )
}

unsafe fn closegoto(ls: *mut LexState, g: c_int, label: *mut Labeldesc, bup: c_int) {
    let fs = (*ls).fs;
    let gl = ptr::addr_of_mut!((*(*ls).dyd).gt);
    let gt = (*gl).arr.add(g as usize);
    debug_assert!(eqstr((*gt).name, (*label).name));
    if (*gt).nactvar < (*label).nactvar {
        jumpscopeerror(ls, gt);
    }
    if (*gt).close != 0 || ((*label).nactvar < (*gt).nactvar && bup != 0) {
        let stklevel = reglevel(fs, (*label).nactvar as c_int);
        *(*(*fs).f).code.add(((*gt).pc + 1) as usize) = *(*(*fs).f).code.add((*gt).pc as usize);
        *(*(*fs).f).code.add((*gt).pc as usize) = CREATE_ABCk(OP_CLOSE, stklevel as c_int, 0, 0, 0);
        (*gt).pc += 1;
    }
    luaK_patchlist((*ls).fs, (*gt).pc, (*label).pc);
    for i in g..((*gl).n - 1) {
        *(*gl).arr.add(i as usize) = *(*gl).arr.add((i + 1) as usize);
    }
    (*gl).n -= 1;
}

unsafe fn findlabel(ls: *mut LexState, name: *mut TString, mut ilb: c_int) -> *mut Labeldesc {
    let dyd = (*ls).dyd;
    while ilb < (*dyd).label.n {
        let lb = (*dyd).label.arr.add(ilb as usize);
        if eqstr((*lb).name, name) {
            return lb;
        }
        ilb += 1;
    }
    ptr::null_mut()
}

unsafe fn newlabelentry(
    ls: *mut LexState,
    l: *mut Labellist,
    name: *mut TString,
    line: c_int,
    pc: c_int,
) -> c_int {
    let n = (*l).n;
    (*l).arr = grow_vector::<Labeldesc>(
        (*ls).L,
        (*l).arr,
        n,
        &mut (*l).size,
        SHRT_MAX,
        c"labels/gotos".as_ptr(),
    );
    let entry = (*l).arr.add(n as usize);
    (*entry).name = name;
    (*entry).line = line;
    (*entry).nactvar = (*(*ls).fs).nactvar;
    (*entry).close = 0;
    (*entry).pc = pc;
    (*l).n = n + 1;
    n
}

unsafe fn newgotoentry(ls: *mut LexState, name: *mut TString, line: c_int) -> c_int {
    let fs = (*ls).fs;
    let pc = luaK_jump(fs);
    luaK_codeABCk(fs, OP_CLOSE, 0, 1, 0, 0);
    newlabelentry(ls, ptr::addr_of_mut!((*(*ls).dyd).gt), name, line, pc)
}

unsafe fn createlabel(ls: *mut LexState, name: *mut TString, line: c_int, last: c_int) {
    let fs = (*ls).fs;
    let ll = ptr::addr_of_mut!((*(*ls).dyd).label);
    let l = newlabelentry(ls, ll, name, line, luaK_getlabel(fs));
    if last != 0 {
        (*(*ll).arr.add(l as usize)).nactvar = (*(*fs).bl).nactvar;
    }
}

unsafe fn solvegotos(fs: *mut FuncState, bl: *mut BlockCnt) {
    let ls = (*fs).ls;
    let gl = ptr::addr_of_mut!((*(*ls).dyd).gt);
    let outlevel = reglevel(fs, (*bl).nactvar as c_int);
    let mut igt = (*bl).firstgoto;
    while igt < (*gl).n {
        let gt = (*gl).arr.add(igt as usize);
        let lb = findlabel(ls, (*gt).name, (*bl).firstlabel);
        if !lb.is_null() {
            closegoto(ls, igt, lb, (*bl).upval as c_int);
        } else {
            if (*bl).upval != 0 && reglevel(fs, (*gt).nactvar as c_int) > outlevel {
                (*gt).close = 1;
            }
            (*gt).nactvar = (*bl).nactvar;
            igt += 1;
        }
    }
    (*(*ls).dyd).label.n = (*bl).firstlabel;
}

unsafe fn enterblock(fs: *mut FuncState, bl: *mut BlockCnt, isloop: lu_byte) {
    (*bl).isloop = isloop;
    (*bl).nactvar = (*fs).nactvar;
    (*bl).firstlabel = (*(*(*fs).ls).dyd).label.n;
    (*bl).firstgoto = (*(*(*fs).ls).dyd).gt.n;
    (*bl).upval = 0;
    (*bl).insidetbc = if !(*fs).bl.is_null() { (*(*fs).bl).insidetbc } else { 0 };
    (*bl).previous = (*fs).bl;
    (*fs).bl = bl;
    debug_assert!((*fs).freereg == luaY_nvarstack(fs));
}

unsafe fn undefgoto(ls: *mut LexState, gt: *mut Labeldesc) -> ! {
    debug_assert!(!eqstr((*gt).name, (*ls).brkn));
    luaK_semerror(
        ls,
        c"no visible label '%s' for <goto> at line %d".as_ptr(),
        getstr((*gt).name),
        (*gt).line,
    )
}

unsafe fn leaveblock(fs: *mut FuncState) {
    let bl = (*fs).bl;
    let ls = (*fs).ls;
    let stklevel = reglevel(fs, (*bl).nactvar as c_int);
    if !(*bl).previous.is_null() && (*bl).upval != 0 {
        luaK_codeABCk(fs, OP_CLOSE, stklevel as c_int, 0, 0, 0);
    }
    (*fs).freereg = stklevel;
    removevars(fs, (*bl).nactvar as c_int);
    debug_assert!((*bl).nactvar == (*fs).nactvar);
    if (*bl).isloop == 2 {
        createlabel(ls, (*ls).brkn, 0, 0);
    }
    solvegotos(fs, bl);
    if (*bl).previous.is_null() {
        if (*bl).firstgoto < (*(*ls).dyd).gt.n {
            undefgoto(ls, (*(*ls).dyd).gt.arr.add((*bl).firstgoto as usize));
        }
    }
    (*fs).bl = (*bl).previous;
}

unsafe fn addprototype(ls: *mut LexState) -> *mut Proto {
    let L = (*ls).L;
    let fs = (*ls).fs;
    let f = (*fs).f;
    if (*fs).np >= (*f).sizep {
        (*f).p = grow_vector::<*mut Proto>(
            L,
            (*f).p,
            (*fs).np,
            &mut (*f).sizep,
            MAXARG_Bx,
            c"functions".as_ptr(),
        );
        let mut oldsize = (*fs).np;
        while oldsize < (*f).sizep {
            *(*f).p.add(oldsize as usize) = ptr::null_mut();
            oldsize += 1;
        }
    }
    let clp = luaF_newproto(L);
    *(*f).p.add((*fs).np as usize) = clp;
    (*fs).np += 1;
    luaC_objbarrier(L, obj2gco(f), obj2gco(clp));
    clp
}

unsafe fn codeclosure(ls: *mut LexState, v: *mut expdesc) {
    let fs = (*(*ls).fs).prev;
    init_exp(v, VRELOC, luaK_codeABx(fs, OP_CLOSURE, 0, (*fs).np - 1));
    luaK_exp2nextreg(fs, v);
}

unsafe fn open_func(ls: *mut LexState, fs: *mut FuncState, bl: *mut BlockCnt) {
    let L = (*ls).L;
    let f = (*fs).f;
    (*fs).prev = (*ls).fs;
    (*fs).ls = ls;
    (*ls).fs = fs;
    (*fs).pc = 0;
    (*fs).previousline = (*f).linedefined;
    (*fs).iwthabs = 0;
    (*fs).lasttarget = 0;
    (*fs).freereg = 0;
    (*fs).nk = 0;
    (*fs).nabslineinfo = 0;
    (*fs).np = 0;
    (*fs).nups = 0;
    (*fs).ndebugvars = 0;
    (*fs).nactvar = 0;
    (*fs).needclose = 0;
    (*fs).firstlocal = (*(*ls).dyd).actvar.n;
    (*fs).firstlabel = (*(*ls).dyd).label.n;
    (*fs).bl = ptr::null_mut();
    (*f).source = (*ls).source;
    luaC_objbarrier(L, obj2gco(f), obj2gco((*f).source));
    (*f).maxstacksize = 2;
    (*fs).kcache = luaH_new(L);
    sethvalue2s(L, (*L).top.p, (*fs).kcache);
    luaD_inctop(L);
    enterblock(fs, bl, 0);
}

unsafe fn close_func(ls: *mut LexState) {
    let L = (*ls).L;
    let fs = (*ls).fs;
    let f = (*fs).f;
    luaK_ret(fs, luaY_nvarstack(fs) as c_int, 0);
    leaveblock(fs);
    debug_assert!((*fs).bl.is_null());
    luaK_finish(fs);
    (*f).code = shrink_vector::<Instruction>(L, (*f).code, &mut (*f).sizecode, (*fs).pc);
    (*f).lineinfo =
        shrink_vector::<i8>(L, (*f).lineinfo, &mut (*f).sizelineinfo, (*fs).pc);
    (*f).abslineinfo = shrink_vector::<AbsLineInfo>(
        L,
        getabslineinfo(f),
        &mut (*f).sizeabslineinfo,
        (*fs).nabslineinfo,
    )
    .cast();
    (*f).k = shrink_vector::<TValue>(L, (*f).k, &mut (*f).sizek, (*fs).nk);
    (*f).p = shrink_vector::<*mut Proto>(L, (*f).p, &mut (*f).sizep, (*fs).np);
    (*f).locvars = shrink_vector::<LocVar>(
        L,
        getlocvars(f),
        &mut (*f).sizelocvars,
        (*fs).ndebugvars as c_int,
    )
    .cast();
    (*f).upvalues = shrink_vector::<Upvaldesc>(
        L,
        (*f).upvalues,
        &mut (*f).sizeupvalues,
        (*fs).nups as c_int,
    );
    (*ls).fs = (*fs).prev;
    (*L).top.p = (*L).top.p.sub(1);
    luaC_checkGC(L);
}

unsafe fn block_follow(ls: *mut LexState, withuntil: c_int) -> c_int {
    match (*ls).t.token {
        TK_ELSE | TK_ELSEIF | TK_END | TK_EOS => 1,
        TK_UNTIL => withuntil,
        _ => 0,
    }
}

unsafe fn statlist(ls: *mut LexState) {
    while block_follow(ls, 1) == 0 {
        if (*ls).t.token == TK_RETURN {
            statement(ls);
            return;
        }
        statement(ls);
    }
}

unsafe fn fieldsel(ls: *mut LexState, v: *mut expdesc) {
    let fs = (*ls).fs;
    let mut key = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    luaK_exp2anyregup(fs, v);
    luaX_next(ls);
    codename(ls, &mut key);
    luaK_indexed(fs, v, &mut key);
}

unsafe fn yindex(ls: *mut LexState, v: *mut expdesc) {
    luaX_next(ls);
    expr(ls, v);
    luaK_exp2val((*ls).fs, v);
    checknext(ls, ']' as c_int);
}

unsafe fn recfield(ls: *mut LexState, cc: *mut ConsControl) {
    let fs = (*ls).fs;
    let reg = (*(*ls).fs).freereg;
    let mut tab = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let mut key = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let mut val = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    if (*ls).t.token == TK_NAME {
        codename(ls, &mut key);
    } else {
        yindex(ls, &mut key);
    }
    (*cc).nh += 1;
    checknext(ls, '=' as c_int);
    tab = *(*cc).t;
    luaK_indexed(fs, &mut tab, &mut key);
    expr(ls, &mut val);
    luaK_storevar(fs, &mut tab, &mut val);
    (*fs).freereg = reg;
}

unsafe fn closelistfield(fs: *mut FuncState, cc: *mut ConsControl) {
    debug_assert!((*cc).tostore > 0);
    luaK_exp2nextreg(fs, ptr::addr_of_mut!((*cc).v));
    (*cc).v.k = VVOID;
    if (*cc).tostore >= (*cc).maxtostore {
        luaK_setlist(fs, (*(*cc).t).u.info, (*cc).na, (*cc).tostore);
        (*cc).na += (*cc).tostore;
        (*cc).tostore = 0;
    }
}

unsafe fn lastlistfield(fs: *mut FuncState, cc: *mut ConsControl) {
    if (*cc).tostore == 0 {
        return;
    }
    if hasmultret((*cc).v.k) {
        luaK_setreturns(fs, ptr::addr_of_mut!((*cc).v), LUA_MULTRET);
        luaK_setlist(fs, (*(*cc).t).u.info, (*cc).na, LUA_MULTRET);
        (*cc).na -= 1;
    } else {
        if (*cc).v.k != VVOID {
            luaK_exp2nextreg(fs, ptr::addr_of_mut!((*cc).v));
        }
        luaK_setlist(fs, (*(*cc).t).u.info, (*cc).na, (*cc).tostore);
    }
    (*cc).na += (*cc).tostore;
}

unsafe fn listfield(ls: *mut LexState, cc: *mut ConsControl) {
    expr(ls, ptr::addr_of_mut!((*cc).v));
    (*cc).tostore += 1;
}

unsafe fn field(ls: *mut LexState, cc: *mut ConsControl) {
    match (*ls).t.token {
        TK_NAME => {
            if luaX_lookahead(ls) != '=' as c_int {
                listfield(ls, cc);
            } else {
                recfield(ls, cc);
            }
        }
        x if x == '[' as c_int => recfield(ls, cc),
        _ => listfield(ls, cc),
    }
}

unsafe fn maxtostore(fs: *mut FuncState) -> c_int {
    let numfreeregs = MAX_FSTACK - (*fs).freereg as c_int;
    if numfreeregs >= 160 {
        numfreeregs / 5
    } else if numfreeregs >= 80 {
        10
    } else {
        1
    }
}

unsafe fn constructor(ls: *mut LexState, t: *mut expdesc) {
    let fs = (*ls).fs;
    let line = (*ls).linenumber;
    let pc = luaK_codevABCk(fs, OP_NEWTABLE, 0, 0, 0, 0);
    let mut cc = ConsControl {
        v: expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 },
        t,
        nh: 0,
        na: 0,
        tostore: 0,
        maxtostore: 0,
    };
    luaK_code(fs, 0);
    init_exp(t, VNONRELOC, (*fs).freereg as c_int);
    luaK_reserveregs(fs, 1);
    init_exp(ptr::addr_of_mut!(cc.v), VVOID, 0);
    checknext(ls, '{' as c_int);
    cc.maxtostore = maxtostore(fs);
    loop {
        if (*ls).t.token == '}' as c_int {
            break;
        }
        if cc.v.k != VVOID {
            closelistfield(fs, &mut cc);
        }
        field(ls, &mut cc);
        luaY_checklimit(
            fs,
            cc.tostore + cc.na + cc.nh,
            i32::MAX / 2,
            c"items in a constructor".as_ptr(),
        );
        if testnext(ls, ',' as c_int) == 0 && testnext(ls, ';' as c_int) == 0 {
            break;
        }
    }
    check_match(ls, '}' as c_int, '{' as c_int, line);
    lastlistfield(fs, &mut cc);
    luaK_settablesize(fs, pc, (*t).u.info, cc.na, cc.nh);
}

unsafe fn setvararg(fs: *mut FuncState) {
    (*(*fs).f).flag |= PF_VAHID;
    luaK_codeABCk(fs, OP_VARARGPREP, 0, 0, 0, 0);
}

unsafe fn parlist(ls: *mut LexState) {
    let fs = (*ls).fs;
    let f = (*fs).f;
    let mut nparams = 0;
    let mut varargk = 0;
    if (*ls).t.token != ')' as c_int {
        loop {
            match (*ls).t.token {
                TK_NAME => {
                    new_localvar(ls, str_checkname(ls));
                    nparams += 1;
                }
                TK_DOTS => {
                    varargk = 1;
                    luaX_next(ls);
                    if (*ls).t.token == TK_NAME {
                        new_varkind(ls, str_checkname(ls), RDKVAVAR);
                    } else {
                        new_localvarliteral(ls, b"(vararg table)\0");
                    }
                }
                _ => luaX_syntaxerror(ls, c"<name> or '...' expected".as_ptr()),
            }
            if varargk != 0 || testnext(ls, ',' as c_int) == 0 {
                break;
            }
        }
    }
    adjustlocalvars(ls, nparams);
    (*f).numparams = cast_byte((*fs).nactvar as c_int);
    if varargk != 0 {
        setvararg(fs);
        adjustlocalvars(ls, 1);
    }
    luaK_reserveregs(fs, (*fs).nactvar as c_int);
}

unsafe fn body(ls: *mut LexState, e: *mut expdesc, ismethod: c_int, line: c_int) {
    let mut new_fs = FuncState {
        f: addprototype(ls),
        prev: ptr::null_mut(),
        ls: ptr::null_mut(),
        bl: ptr::null_mut(),
        kcache: ptr::null_mut(),
        pc: 0,
        lasttarget: 0,
        previousline: 0,
        nk: 0,
        np: 0,
        nabslineinfo: 0,
        firstlocal: 0,
        firstlabel: 0,
        ndebugvars: 0,
        nactvar: 0,
        nups: 0,
        freereg: 0,
        iwthabs: 0,
        needclose: 0,
    };
    let mut bl = BlockCnt {
        previous: ptr::null_mut(),
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    (*new_fs.f).linedefined = line;
    open_func(ls, &mut new_fs, &mut bl);
    checknext(ls, '(' as c_int);
    if ismethod != 0 {
        new_localvarliteral(ls, b"self\0");
        adjustlocalvars(ls, 1);
    }
    parlist(ls);
    checknext(ls, ')' as c_int);
    statlist(ls);
    (*new_fs.f).lastlinedefined = (*ls).linenumber;
    check_match(ls, TK_END, TK_FUNCTION, line);
    codeclosure(ls, e);
    close_func(ls);
}

unsafe fn explist(ls: *mut LexState, v: *mut expdesc) -> c_int {
    let mut n = 1;
    expr(ls, v);
    while testnext(ls, ',' as c_int) != 0 {
        luaK_exp2nextreg((*ls).fs, v);
        expr(ls, v);
        n += 1;
    }
    n
}

unsafe fn funcargs(ls: *mut LexState, f: *mut expdesc) {
    let fs = (*ls).fs;
    let mut args = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let line = (*ls).linenumber;
    match (*ls).t.token {
        x if x == '(' as c_int => {
            luaX_next(ls);
            if (*ls).t.token == ')' as c_int {
                args.k = VVOID;
            } else {
                explist(ls, &mut args);
                if hasmultret(args.k) {
                    luaK_setreturns(fs, &mut args, LUA_MULTRET);
                }
            }
            check_match(ls, ')' as c_int, '(' as c_int, line);
        }
        x if x == '{' as c_int => constructor(ls, &mut args),
        TK_STRING => {
            codestring(&mut args, (*ls).t.seminfo.ts);
            luaX_next(ls);
        }
        _ => luaX_syntaxerror(ls, c"function arguments expected".as_ptr()),
    }
    debug_assert!((*f).k == VNONRELOC);
    let base = (*f).u.info;
    let nparams = if hasmultret(args.k) {
        LUA_MULTRET
    } else {
        if args.k != VVOID {
            luaK_exp2nextreg(fs, &mut args);
        }
        (*fs).freereg as c_int - (base + 1)
    };
    init_exp(f, VCALL, luaK_codeABCk(fs, OP_CALL, base, nparams + 1, 2, 0));
    luaK_fixline(fs, line);
    (*fs).freereg = cast_byte(base + 1);
}

unsafe fn primaryexp(ls: *mut LexState, v: *mut expdesc) {
    match (*ls).t.token {
        x if x == '(' as c_int => {
            let line = (*ls).linenumber;
            luaX_next(ls);
            expr(ls, v);
            check_match(ls, ')' as c_int, '(' as c_int, line);
            luaK_dischargevars((*ls).fs, v);
        }
        TK_NAME => singlevar(ls, v),
        _ => luaX_syntaxerror(ls, c"unexpected symbol".as_ptr()),
    }
}

unsafe fn suffixedexp(ls: *mut LexState, v: *mut expdesc) {
    let fs = (*ls).fs;
    primaryexp(ls, v);
    loop {
        match (*ls).t.token {
            x if x == '.' as c_int => fieldsel(ls, v),
            x if x == '[' as c_int => {
                let mut key = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
                luaK_exp2anyregup(fs, v);
                yindex(ls, &mut key);
                luaK_indexed(fs, v, &mut key);
            }
            x if x == ':' as c_int => {
                let mut key = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
                luaX_next(ls);
                codename(ls, &mut key);
                luaK_self(fs, v, &mut key);
                funcargs(ls, v);
            }
            x if x == '(' as c_int || x == TK_STRING || x == '{' as c_int => {
                luaK_exp2nextreg(fs, v);
                funcargs(ls, v);
            }
            _ => return,
        }
    }
}

unsafe fn simpleexp(ls: *mut LexState, v: *mut expdesc) {
    match (*ls).t.token {
        TK_FLT => {
            init_exp(v, VKFLT, 0);
            (*v).u.nval = (*ls).t.seminfo.r;
        }
        TK_INT => {
            init_exp(v, VKINT, 0);
            (*v).u.ival = (*ls).t.seminfo.i;
        }
        TK_STRING => codestring(v, (*ls).t.seminfo.ts),
        TK_NIL => init_exp(v, VNIL, 0),
        TK_TRUE => init_exp(v, VTRUE, 0),
        TK_FALSE => init_exp(v, VFALSE, 0),
        TK_DOTS => {
            let fs = (*ls).fs;
            check_condition(ls, isvararg((*fs).f), c"cannot use '...' outside a vararg function".as_ptr());
            init_exp(v, VVARARG, luaK_codeABCk(fs, OP_VARARG, 0, (*(*fs).f).numparams as c_int, 1, 0));
        }
        x if x == '{' as c_int => {
            constructor(ls, v);
            return;
        }
        TK_FUNCTION => {
            luaX_next(ls);
            body(ls, v, 0, (*ls).linenumber);
            return;
        }
        _ => {
            suffixedexp(ls, v);
            return;
        }
    }
    luaX_next(ls);
}

unsafe fn getunopr(op: c_int) -> c_int {
    match op {
        TK_NOT => OPR_NOT,
        x if x == '-' as c_int => OPR_MINUS,
        x if x == '~' as c_int => OPR_BNOT,
        x if x == '#' as c_int => OPR_LEN,
        _ => OPR_NOUNOPR,
    }
}

unsafe fn getbinopr(op: c_int) -> c_int {
    match op {
        x if x == '+' as c_int => OPR_ADD,
        x if x == '-' as c_int => OPR_SUB,
        x if x == '*' as c_int => OPR_MUL,
        x if x == '%' as c_int => OPR_MOD,
        x if x == '^' as c_int => OPR_POW,
        x if x == '/' as c_int => OPR_DIV,
        TK_IDIV => OPR_IDIV,
        x if x == '&' as c_int => OPR_BAND,
        x if x == '|' as c_int => OPR_BOR,
        x if x == '~' as c_int => OPR_BXOR,
        TK_SHL => OPR_SHL,
        TK_SHR => OPR_SHR,
        TK_CONCAT => OPR_CONCAT,
        TK_NE => OPR_NE,
        TK_EQ => OPR_EQ,
        x if x == '<' as c_int => OPR_LT,
        TK_LE => OPR_LE,
        x if x == '>' as c_int => OPR_GT,
        TK_GE => OPR_GE,
        TK_AND => OPR_AND,
        TK_OR => OPR_OR,
        _ => OPR_NOBINOPR,
    }
}

unsafe fn subexpr(ls: *mut LexState, v: *mut expdesc, limit: c_int) -> c_int {
    enterlevel(ls);
    let uop = getunopr((*ls).t.token);
    if uop != OPR_NOUNOPR {
        let line = (*ls).linenumber;
        luaX_next(ls);
        subexpr(ls, v, UNARY_PRIORITY);
        luaK_prefix((*ls).fs, uop, v, line);
    } else {
        simpleexp(ls, v);
    }
    let mut op = getbinopr((*ls).t.token);
    while op != OPR_NOBINOPR && PRIORITY[op as usize].0 as c_int > limit {
        let mut v2 = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
        let line = (*ls).linenumber;
        luaX_next(ls);
        luaK_infix((*ls).fs, op, v);
        let nextop = subexpr(ls, &mut v2, PRIORITY[op as usize].1 as c_int);
        luaK_posfix((*ls).fs, op, v, &mut v2, line);
        op = nextop;
    }
    leavelevel(ls);
    op
}

unsafe fn expr(ls: *mut LexState, v: *mut expdesc) {
    subexpr(ls, v, 0);
}

unsafe fn block(ls: *mut LexState) {
    let fs = (*ls).fs;
    let mut bl = BlockCnt {
        previous: ptr::null_mut(),
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    enterblock(fs, &mut bl, 0);
    statlist(ls);
    leaveblock(fs);
}

unsafe fn check_conflict(ls: *mut LexState, mut lh: *mut LHS_assign, v: *mut expdesc) {
    let fs = (*ls).fs;
    let extra = (*fs).freereg;
    let mut conflict = 0;
    while !lh.is_null() {
        if vkisindexed((*lh).v.k) {
            if (*lh).v.k == VINDEXUP {
                if (*v).k == VUPVAL && (*lh).v.u.ind.t as c_int == (*v).u.info {
                    conflict = 1;
                    (*lh).v.k = VINDEXSTR;
                    (*lh).v.u.ind.t = extra;
                }
            } else {
                if (*v).k == VLOCAL && (*lh).v.u.ind.t == (*v).u.var.ridx {
                    conflict = 1;
                    (*lh).v.u.ind.t = extra;
                }
                if (*lh).v.k == VINDEXED && (*v).k == VLOCAL && (*lh).v.u.ind.idx as c_int == (*v).u.var.ridx as c_int {
                    conflict = 1;
                    (*lh).v.u.ind.idx = extra as i16;
                }
            }
        }
        lh = (*lh).prev;
    }
    if conflict != 0 {
        if (*v).k == VLOCAL {
            luaK_codeABCk(fs, OP_MOVE, extra as c_int, (*v).u.var.ridx as c_int, 0, 0);
        } else {
            luaK_codeABCk(fs, OP_GETUPVAL, extra as c_int, (*v).u.info, 0, 0);
        }
        luaK_reserveregs(fs, 1);
    }
}

unsafe fn storevartop(fs: *mut FuncState, var: *mut expdesc) {
    let mut e = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    init_exp(&mut e, VNONRELOC, (*fs).freereg as c_int - 1);
    luaK_storevar(fs, var, &mut e);
}

unsafe fn restassign(ls: *mut LexState, lh: *mut LHS_assign, nvars: c_int) {
    let mut e = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    check_condition(ls, vkisvar((*lh).v.k), c"syntax error".as_ptr());
    check_readonly(ls, ptr::addr_of_mut!((*lh).v));
    if testnext(ls, ',' as c_int) != 0 {
        let mut nv = LHS_assign {
            prev: lh,
            v: expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 },
        };
        suffixedexp(ls, ptr::addr_of_mut!(nv.v));
        if !vkisindexed(nv.v.k) {
            check_conflict(ls, lh, ptr::addr_of_mut!(nv.v));
        }
        enterlevel(ls);
        restassign(ls, &mut nv, nvars + 1);
        leavelevel(ls);
    } else {
        checknext(ls, '=' as c_int);
        let nexps = explist(ls, &mut e);
        if nexps != nvars {
            adjust_assign(ls, nvars, nexps, &mut e);
        } else {
            luaK_setoneret((*ls).fs, &mut e);
            luaK_storevar((*ls).fs, ptr::addr_of_mut!((*lh).v), &mut e);
            return;
        }
    }
    storevartop((*ls).fs, ptr::addr_of_mut!((*lh).v));
}

unsafe fn cond(ls: *mut LexState) -> c_int {
    let mut v = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    expr(ls, &mut v);
    if v.k == VNIL {
        v.k = VFALSE;
    }
    luaK_goiftrue((*ls).fs, &mut v);
    v.f
}

unsafe fn gotostat(ls: *mut LexState, line: c_int) {
    let name = str_checkname(ls);
    newgotoentry(ls, name, line);
}

unsafe fn breakstat(ls: *mut LexState, line: c_int) {
    let mut bl = (*(*ls).fs).bl;
    while !bl.is_null() {
        if (*bl).isloop != 0 {
            (*bl).isloop = 2;
            luaX_next(ls);
            newgotoentry(ls, (*ls).brkn, line);
            return;
        }
        bl = (*bl).previous;
    }
    luaX_syntaxerror(ls, c"break outside loop".as_ptr());
}

unsafe fn checkrepeated(ls: *mut LexState, name: *mut TString) {
    let lb = findlabel(ls, name, (*(*ls).fs).firstlabel);
    if !lb.is_null() {
        luaK_semerror(
            ls,
            c"label '%s' already defined on line %d".as_ptr(),
            getstr(name),
            (*lb).line,
        );
    }
}

unsafe fn labelstat(ls: *mut LexState, name: *mut TString, line: c_int) {
    checknext(ls, TK_DBCOLON);
    while (*ls).t.token == ';' as c_int || (*ls).t.token == TK_DBCOLON {
        statement(ls);
    }
    checkrepeated(ls, name);
    createlabel(ls, name, line, block_follow(ls, 0));
}

unsafe fn whilestat(ls: *mut LexState, line: c_int) {
    let fs = (*ls).fs;
    let mut bl = BlockCnt {
        previous: ptr::null_mut(),
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    luaX_next(ls);
    let whileinit = luaK_getlabel(fs);
    let condexit = cond(ls);
    enterblock(fs, &mut bl, 1);
    checknext(ls, TK_DO);
    block(ls);
    luaK_patchlist(fs, luaK_jump(fs), whileinit);
    check_match(ls, TK_END, TK_WHILE, line);
    leaveblock(fs);
    luaK_patchtohere(fs, condexit);
}

unsafe fn repeatstat(ls: *mut LexState, line: c_int) {
    let fs = (*ls).fs;
    let repeat_init = luaK_getlabel(fs);
    let mut bl1 = BlockCnt {
        previous: ptr::null_mut(),
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    let mut bl2 = BlockCnt {
        previous: ptr::null_mut(),
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    enterblock(fs, &mut bl1, 1);
    enterblock(fs, &mut bl2, 0);
    luaX_next(ls);
    statlist(ls);
    check_match(ls, TK_UNTIL, TK_REPEAT, line);
    let mut condexit = cond(ls);
    leaveblock(fs);
    if bl2.upval != 0 {
        let exit = luaK_jump(fs);
        luaK_patchtohere(fs, condexit);
        luaK_codeABCk(fs, OP_CLOSE, reglevel(fs, bl2.nactvar as c_int) as c_int, 0, 0, 0);
        condexit = luaK_jump(fs);
        luaK_patchtohere(fs, exit);
    }
    luaK_patchlist(fs, condexit, repeat_init);
    leaveblock(fs);
}

unsafe fn exp1(ls: *mut LexState) {
    let mut e = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    expr(ls, &mut e);
    luaK_exp2nextreg((*ls).fs, &mut e);
}

unsafe fn fixforjump(fs: *mut FuncState, pc: c_int, dest: c_int, back: c_int) {
    let jmp = (*(*fs).f).code.add(pc as usize);
    let mut offset = dest - (pc + 1);
    if back != 0 {
        offset = -offset;
    }
    if offset > MAXARG_Bx {
        luaX_syntaxerror((*fs).ls, c"control structure too long".as_ptr());
    }
    SETARG_Bx(&mut *jmp, offset);
}

unsafe fn forbody(ls: *mut LexState, base: c_int, line: c_int, nvars: c_int, isgen: c_int) {
    let fs = (*ls).fs;
    let mut bl = BlockCnt {
        previous: ptr::null_mut(),
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    checknext(ls, TK_DO);
    let prep = luaK_codeABx(
        fs,
        if isgen != 0 { OP_TFORPREP } else { OP_FORPREP },
        base,
        0,
    );
    (*fs).freereg -= 1;
    enterblock(fs, &mut bl, 0);
    adjustlocalvars(ls, nvars);
    luaK_reserveregs(fs, nvars);
    block(ls);
    leaveblock(fs);
    fixforjump(fs, prep, luaK_getlabel(fs), 0);
    if isgen != 0 {
        luaK_codeABCk(fs, OP_TFORCALL, base, 0, nvars, 0);
        luaK_fixline(fs, line);
    }
    let endfor = luaK_codeABx(
        fs,
        if isgen != 0 { OP_TFORLOOP } else { OP_FORLOOP },
        base,
        0,
    );
    fixforjump(fs, endfor, prep + 1, 1);
    luaK_fixline(fs, line);
}

unsafe fn fornum(ls: *mut LexState, varname: *mut TString, line: c_int) {
    let fs = (*ls).fs;
    let base = (*fs).freereg as c_int;
    new_localvarliteral(ls, b"(for state)\0");
    new_localvarliteral(ls, b"(for state)\0");
    new_varkind(ls, varname, RDKCONST);
    checknext(ls, '=' as c_int);
    exp1(ls);
    checknext(ls, ',' as c_int);
    exp1(ls);
    if testnext(ls, ',' as c_int) != 0 {
        exp1(ls);
    } else {
        luaK_int(fs, (*fs).freereg as c_int, 1);
        luaK_reserveregs(fs, 1);
    }
    adjustlocalvars(ls, 2);
    forbody(ls, base, line, 1, 0);
}

unsafe fn forlist(ls: *mut LexState, indexname: *mut TString) {
    let fs = (*ls).fs;
    let mut e = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let mut nvars = 4;
    let base = (*fs).freereg as c_int;
    new_localvarliteral(ls, b"(for state)\0");
    new_localvarliteral(ls, b"(for state)\0");
    new_localvarliteral(ls, b"(for state)\0");
    new_varkind(ls, indexname, RDKCONST);
    while testnext(ls, ',' as c_int) != 0 {
        new_localvar(ls, str_checkname(ls));
        nvars += 1;
    }
    checknext(ls, TK_IN);
    let line = (*ls).linenumber;
    adjust_assign(ls, 4, explist(ls, &mut e), &mut e);
    adjustlocalvars(ls, 3);
    marktobeclosed(fs);
    luaK_checkstack(fs, 2);
    forbody(ls, base, line, nvars - 3, 1);
}

unsafe fn forstat(ls: *mut LexState, line: c_int) {
    let fs = (*ls).fs;
    let mut bl = BlockCnt {
        previous: ptr::null_mut(),
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    enterblock(fs, &mut bl, 1);
    luaX_next(ls);
    let varname = str_checkname(ls);
    match (*ls).t.token {
        x if x == '=' as c_int => fornum(ls, varname, line),
        x if x == ',' as c_int || x == TK_IN => forlist(ls, varname),
        _ => luaX_syntaxerror(ls, c"'=' or 'in' expected".as_ptr()),
    }
    check_match(ls, TK_END, TK_FOR, line);
    leaveblock(fs);
}

unsafe fn test_then_block(ls: *mut LexState, escapelist: *mut c_int) {
    let fs = (*ls).fs;
    luaX_next(ls);
    let condtrue = cond(ls);
    checknext(ls, TK_THEN);
    block(ls);
    if (*ls).t.token == TK_ELSE || (*ls).t.token == TK_ELSEIF {
        luaK_concat(fs, escapelist, luaK_jump(fs));
    }
    luaK_patchtohere(fs, condtrue);
}

unsafe fn ifstat(ls: *mut LexState, line: c_int) {
    let fs = (*ls).fs;
    let mut escapelist = NO_JUMP;
    test_then_block(ls, &mut escapelist);
    while (*ls).t.token == TK_ELSEIF {
        test_then_block(ls, &mut escapelist);
    }
    if testnext(ls, TK_ELSE) != 0 {
        block(ls);
    }
    check_match(ls, TK_END, TK_IF, line);
    luaK_patchtohere(fs, escapelist);
}

unsafe fn localfunc(ls: *mut LexState) {
    let fs = (*ls).fs;
    let fvar = (*fs).nactvar as c_int;
    let mut b = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    new_localvar(ls, str_checkname(ls));
    adjustlocalvars(ls, 1);
    body(ls, &mut b, 0, (*ls).linenumber);
    (*localdebuginfo(fs, fvar)).startpc = (*fs).pc;
}

unsafe fn getvarattribute(ls: *mut LexState, df: lu_byte) -> lu_byte {
    if testnext(ls, '<' as c_int) != 0 {
        let ts = str_checkname(ls);
        let attr = getstr(ts);
        checknext(ls, '>' as c_int);
        if strcmp(attr, c"const".as_ptr()) == 0 {
            RDKCONST
        } else if strcmp(attr, c"close".as_ptr()) == 0 {
            RDKTOCLOSE
        } else {
            luaK_semerror(ls, c"unknown attribute '%s'".as_ptr(), attr);
        }
    } else {
        df
    }
}

unsafe fn checktoclose(fs: *mut FuncState, level: c_int) {
    if level != -1 {
        marktobeclosed(fs);
        luaK_codeABCk(fs, OP_TBC, reglevel(fs, level) as c_int, 0, 0, 0);
    }
}

unsafe fn localstat(ls: *mut LexState) {
    let fs = (*ls).fs;
    let mut toclose = -1;
    let mut vidx = 0;
    let mut nvars = 0;
    let mut e = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let defkind = getvarattribute(ls, VDKREG);
    loop {
        let vname = str_checkname(ls);
        let kind = getvarattribute(ls, defkind);
        vidx = new_varkind(ls, vname, kind);
        if kind == RDKTOCLOSE {
            if toclose != -1 {
                luaK_semerror(ls, c"multiple to-be-closed variables in local list".as_ptr());
            }
            toclose = (*fs).nactvar as c_int + nvars;
        }
        nvars += 1;
        if testnext(ls, ',' as c_int) == 0 {
            break;
        }
    }
    let nexps = if testnext(ls, '=' as c_int) != 0 {
        explist(ls, &mut e)
    } else {
        e.k = VVOID;
        0
    };
    let var = getlocalvardesc(fs, vidx);
    if nvars == nexps && (*var).vd.kind == RDKCONST && luaK_exp2const(fs, &e, ptr::addr_of_mut!((*var).k)) != 0
    {
        (*var).vd.kind = RDKCTC;
        adjustlocalvars(ls, nvars - 1);
        (*fs).nactvar += 1;
    } else {
        adjust_assign(ls, nvars, nexps, &mut e);
        adjustlocalvars(ls, nvars);
    }
    checktoclose(fs, toclose);
}

unsafe fn getglobalattribute(ls: *mut LexState, df: lu_byte) -> lu_byte {
    let kind = getvarattribute(ls, df);
    match kind {
        RDKTOCLOSE => luaK_semerror(ls, c"global variables cannot be to-be-closed".as_ptr()),
        RDKCONST => GDKCONST,
        _ => kind,
    }
}

unsafe fn checkglobal(ls: *mut LexState, varname: *mut TString, line: c_int) {
    let fs = (*ls).fs;
    let mut var = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    buildglobal(ls, varname, &mut var);
    let k = var.u.ind.keystr;
    luaK_codecheckglobal(fs, &mut var, k, line);
}

unsafe fn initglobal(ls: *mut LexState, nvars: c_int, firstidx: c_int, n: c_int, line: c_int) {
    if n == nvars {
        let mut e = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
        let nexps = explist(ls, &mut e);
        adjust_assign(ls, nvars, nexps, &mut e);
    } else {
        let fs = (*ls).fs;
        let mut var = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
        let varname = (*getlocalvardesc(fs, firstidx + n)).vd.name;
        buildglobal(ls, varname, &mut var);
        enterlevel(ls);
        initglobal(ls, nvars, firstidx, n + 1, line);
        leavelevel(ls);
        checkglobal(ls, varname, line);
        storevartop(fs, &mut var);
    }
}

unsafe fn globalnames(ls: *mut LexState, defkind: lu_byte) {
    let fs = (*ls).fs;
    let mut nvars = 0;
    let mut lastidx = 0;
    loop {
        let vname = str_checkname(ls);
        let kind = getglobalattribute(ls, defkind);
        lastidx = new_varkind(ls, vname, kind);
        nvars += 1;
        if testnext(ls, ',' as c_int) == 0 {
            break;
        }
    }
    if testnext(ls, '=' as c_int) != 0 {
        initglobal(ls, nvars, lastidx - nvars + 1, 0, (*ls).linenumber);
    }
    (*fs).nactvar += nvars as i16;
}

unsafe fn globalstat(ls: *mut LexState) {
    let fs = (*ls).fs;
    let defkind = getglobalattribute(ls, GDKREG);
    if testnext(ls, '*' as c_int) == 0 {
        globalnames(ls, defkind);
    } else {
        new_varkind(ls, ptr::null_mut(), defkind);
        (*fs).nactvar += 1;
    }
}

unsafe fn globalfunc(ls: *mut LexState, line: c_int) {
    let fs = (*ls).fs;
    let mut var = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let mut b = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let fname = str_checkname(ls);
    new_varkind(ls, fname, GDKREG);
    (*fs).nactvar += 1;
    buildglobal(ls, fname, &mut var);
    body(ls, &mut b, 0, (*ls).linenumber);
    checkglobal(ls, fname, line);
    luaK_storevar(fs, &mut var, &mut b);
    luaK_fixline(fs, line);
}

unsafe fn globalstatfunc(ls: *mut LexState, line: c_int) {
    luaX_next(ls);
    if testnext(ls, TK_FUNCTION) != 0 {
        globalfunc(ls, line);
    } else {
        globalstat(ls);
    }
}

unsafe fn funcname(ls: *mut LexState, v: *mut expdesc) -> c_int {
    let mut ismethod = 0;
    singlevar(ls, v);
    while (*ls).t.token == '.' as c_int {
        fieldsel(ls, v);
    }
    if (*ls).t.token == ':' as c_int {
        ismethod = 1;
        fieldsel(ls, v);
    }
    ismethod
}

unsafe fn funcstat(ls: *mut LexState, line: c_int) {
    let mut v = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let mut b = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    luaX_next(ls);
    let ismethod = funcname(ls, &mut v);
    check_readonly(ls, &mut v);
    body(ls, &mut b, ismethod, line);
    luaK_storevar((*ls).fs, &mut v, &mut b);
    luaK_fixline((*ls).fs, line);
}

unsafe fn exprstat(ls: *mut LexState) {
    let fs = (*ls).fs;
    let mut v = LHS_assign {
        prev: ptr::null_mut(),
        v: expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 },
    };
    suffixedexp(ls, ptr::addr_of_mut!(v.v));
    if (*ls).t.token == '=' as c_int || (*ls).t.token == ',' as c_int {
        restassign(ls, &mut v, 1);
    } else {
        check_condition(ls, v.v.k == VCALL, c"syntax error".as_ptr());
        let inst = getinstruction(fs, ptr::addr_of_mut!(v.v));
        SETARG_C(&mut *inst, 1);
    }
}

unsafe fn retstat(ls: *mut LexState) {
    let fs = (*ls).fs;
    let mut e = expdesc { k: 0, u: ExpdescUnion { info: 0 }, t: 0, f: 0 };
    let mut first = luaY_nvarstack(fs) as c_int;
    let nret;
    if block_follow(ls, 1) != 0 || (*ls).t.token == ';' as c_int {
        nret = 0;
    } else {
        let mut nr = explist(ls, &mut e);
        if hasmultret(e.k) {
            luaK_setreturns(fs, &mut e, LUA_MULTRET);
            if e.k == VCALL && nr == 1 && (*(*fs).bl).insidetbc == 0 {
                let inst = getinstruction(fs, &mut e);
                SET_OPCODE(&mut *inst, OP_TAILCALL);
                debug_assert!(GETARG_A(*inst) == luaY_nvarstack(fs) as c_int);
            }
            nr = LUA_MULTRET;
        } else if nr == 1 {
            first = luaK_exp2anyreg(fs, &mut e);
        } else {
            luaK_exp2nextreg(fs, &mut e);
            debug_assert!(nr == (*fs).freereg as c_int - first);
        }
        luaK_ret(fs, first, nr);
        testnext(ls, ';' as c_int);
        return;
    }
    luaK_ret(fs, first, nret);
    testnext(ls, ';' as c_int);
}

unsafe fn statement(ls: *mut LexState) {
    let line = (*ls).linenumber;
    enterlevel(ls);
    match (*ls).t.token {
        x if x == ';' as c_int => luaX_next(ls),
        TK_IF => ifstat(ls, line),
        TK_WHILE => whilestat(ls, line),
        TK_DO => {
            luaX_next(ls);
            block(ls);
            check_match(ls, TK_END, TK_DO, line);
        }
        TK_FOR => forstat(ls, line),
        TK_REPEAT => repeatstat(ls, line),
        TK_FUNCTION => funcstat(ls, line),
        TK_LOCAL => {
            luaX_next(ls);
            if testnext(ls, TK_FUNCTION) != 0 {
                localfunc(ls);
            } else {
                localstat(ls);
            }
        }
        TK_GLOBAL => globalstatfunc(ls, line),
        TK_DBCOLON => {
            luaX_next(ls);
            labelstat(ls, str_checkname(ls), line);
        }
        TK_RETURN => {
            luaX_next(ls);
            retstat(ls);
        }
        TK_BREAK => breakstat(ls, line),
        TK_GOTO => {
            luaX_next(ls);
            gotostat(ls, line);
        }
        TK_NAME => {
            if eqstr((*ls).t.seminfo.ts, (*ls).glbn) {
                let lk = luaX_lookahead(ls);
                if lk == '<' as c_int || lk == TK_NAME || lk == '*' as c_int || lk == TK_FUNCTION {
                    globalstatfunc(ls, line);
                    (*(*ls).fs).freereg = luaY_nvarstack((*ls).fs);
                    leavelevel(ls);
                    return;
                }
            }
            exprstat(ls);
        }
        _ => exprstat(ls),
    }
    debug_assert!((*(*(*ls).fs).f).maxstacksize >= (*(*ls).fs).freereg);
    debug_assert!((*(*ls).fs).freereg >= luaY_nvarstack((*ls).fs));
    (*(*ls).fs).freereg = luaY_nvarstack((*ls).fs);
    leavelevel(ls);
}

unsafe fn mainfunc(ls: *mut LexState, fs: *mut FuncState) {
    let mut bl = BlockCnt {
        previous: ptr::null_mut(),
        firstlabel: 0,
        firstgoto: 0,
        nactvar: 0,
        upval: 0,
        isloop: 0,
        insidetbc: 0,
    };
    open_func(ls, fs, &mut bl);
    setvararg(fs);
    let env = allocupvalue(fs);
    (*env).instack = 1;
    (*env).idx = 0;
    (*env).kind = VDKREG;
    (*env).name = (*ls).envn;
    luaC_objbarrier((*ls).L, obj2gco((*fs).f), obj2gco((*env).name));
    luaX_next(ls);
    statlist(ls);
    check(ls, TK_EOS);
    close_func(ls);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn luaY_parser(
    L: *mut lua_State,
    z: *mut ZIO,
    buff: *mut Mbuffer,
    dyd: *mut Dyndata,
    name: *const c_char,
    firstchar: c_int,
) -> *mut LClosure {
    let mut lexstate = LexState {
        current: 0,
        linenumber: 0,
        lastline: 0,
        t: Token { token: 0, seminfo: SemInfo { i: 0 } },
        lookahead: Token { token: 0, seminfo: SemInfo { i: 0 } },
        fs: ptr::null_mut(),
        L,
        z: ptr::null_mut(),
        buff,
        h: ptr::null_mut(),
        dyd,
        source: ptr::null_mut(),
        envn: ptr::null_mut(),
        brkn: ptr::null_mut(),
        glbn: ptr::null_mut(),
    };
    let mut funcstate = FuncState {
        f: ptr::null_mut(),
        prev: ptr::null_mut(),
        ls: ptr::null_mut(),
        bl: ptr::null_mut(),
        kcache: ptr::null_mut(),
        pc: 0,
        lasttarget: 0,
        previousline: 0,
        nk: 0,
        np: 0,
        nabslineinfo: 0,
        firstlocal: 0,
        firstlabel: 0,
        ndebugvars: 0,
        nactvar: 0,
        nups: 0,
        freereg: 0,
        iwthabs: 0,
        needclose: 0,
    };
    let cl = luaF_newLclosure(L, 1);
    setclLvalue2s(L, (*L).top.p, cl);
    luaD_inctop(L);
    lexstate.h = luaH_new(L);
    sethvalue2s(L, (*L).top.p, lexstate.h);
    luaD_inctop(L);
    funcstate.f = luaF_newproto(L);
    (*cl).p = funcstate.f;
    luaC_objbarrier(L, obj2gco(cl), obj2gco((*cl).p));
    (*funcstate.f).source = luaS_new(L, name);
    luaC_objbarrier(L, obj2gco(funcstate.f), obj2gco((*funcstate.f).source));
    (*dyd).actvar.n = 0;
    (*dyd).gt.n = 0;
    (*dyd).label.n = 0;
    luaX_setinput(L, &mut lexstate, z, (*funcstate.f).source, firstchar);
    mainfunc(&mut lexstate, &mut funcstate);
    debug_assert!(funcstate.prev.is_null() && funcstate.nups == 1 && lexstate.fs.is_null());
    debug_assert!((*dyd).actvar.n == 0 && (*dyd).gt.n == 0 && (*dyd).label.n == 0);
    (*L).top.p = (*L).top.p.sub(1);
    cl
}
