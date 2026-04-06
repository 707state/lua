#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clashing_extern_declarations
)]

use crate::lex::luaX_syntaxerror;
use crate::object::luaO_ceillog2;
use crate::object::luaO_pushstr;
use crate::object::luaO_rawarith;
use crate::opcodes::*;
use crate::parser_rs::*;
use crate::runtime::*;
use crate::vm_rs::*;

#[derive(Copy, Clone)]
#[repr(C)]
struct ExpdescInd {
    idx: i16,
    t: lu_byte,
    ro: lu_byte,
    keystr: c_int,
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
unsafe fn setarg(i: &mut Instruction, v: c_int, pos: u32, size: u32) {
    unsafe {
        *i = (*i & !mask1(size, pos)) | (((v as u32) << pos) & mask1(size, pos));
    }
}

#[inline]
unsafe fn GET_OPCODE(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_OP, SIZE_OP) }
}

#[inline]
unsafe fn SET_OPCODE(i: &mut Instruction, o: c_int) {
    unsafe {
        setarg(i, o, POS_OP, SIZE_OP);
    }
}

#[inline]
unsafe fn GETARG_A(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_A, SIZE_A) }
}

#[inline]
unsafe fn SETARG_A(i: &mut Instruction, v: c_int) {
    unsafe {
        setarg(i, v, POS_A, SIZE_A);
    }
}

#[inline]
unsafe fn GETARG_B(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_B, SIZE_B) }
}

#[inline]
unsafe fn SETARG_B(i: &mut Instruction, v: c_int) {
    unsafe {
        setarg(i, v, POS_B, SIZE_B);
    }
}

#[inline]
unsafe fn GETARG_C(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_C, SIZE_C) }
}

#[inline]
unsafe fn SETARG_C(i: &mut Instruction, v: c_int) {
    unsafe {
        setarg(i, v, POS_C, SIZE_C);
    }
}

#[inline]
unsafe fn GETARG_k(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_k, 1) }
}

#[inline]
unsafe fn SETARG_k(i: &mut Instruction, v: c_int) {
    unsafe {
        setarg(i, v, POS_k, 1);
    }
}

#[inline]
unsafe fn GETARG_Bx(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_Bx, SIZE_Bx) }
}

#[inline]
unsafe fn SETARG_Bx(i: &mut Instruction, v: c_int) {
    unsafe {
        setarg(i, v, POS_Bx, SIZE_Bx);
    }
}

#[inline]
unsafe fn GETARG_sJ(i: Instruction) -> c_int {
    unsafe { getarg(i, POS_sJ, SIZE_sJ) - OFFSET_sJ }
}

#[inline]
unsafe fn SETARG_sJ(i: &mut Instruction, j: c_int) {
    unsafe {
        setarg(i, j + OFFSET_sJ, POS_sJ, SIZE_sJ);
    }
}

#[inline]
unsafe fn CREATE_ABCk(o: c_int, a: c_int, b: c_int, c: c_int, k: c_int) -> Instruction {
    ((o as u32) << POS_OP)
        | ((a as u32) << POS_A)
        | ((b as u32) << POS_B)
        | ((c as u32) << POS_C)
        | ((k as u32) << POS_k)
}

#[inline]
unsafe fn CREATE_vABCk(o: c_int, a: c_int, b: c_int, c: c_int, k: c_int) -> Instruction {
    ((o as u32) << POS_OP)
        | ((a as u32) << POS_A)
        | ((b as u32) << POS_vB)
        | ((c as u32) << POS_vC)
        | ((k as u32) << POS_k)
}

#[inline]
unsafe fn CREATE_ABx(o: c_int, a: c_int, bx: c_int) -> Instruction {
    ((o as u32) << POS_OP) | ((a as u32) << POS_A) | ((bx as u32) << POS_Bx)
}

#[inline]
unsafe fn CREATE_Ax(o: c_int, a: c_int) -> Instruction {
    ((o as u32) << POS_OP) | ((a as u32) << POS_Ax)
}

#[inline]
unsafe fn CREATE_sJ(o: c_int, j: c_int, k: c_int) -> Instruction {
    ((o as u32) << POS_OP) | ((j as u32) << POS_sJ) | ((k as u32) << POS_k)
}

#[inline]
unsafe fn getOpMode(op: c_int) -> u8 {
    luaP_opmodes[op as usize] & 7
}

#[inline]
unsafe fn testTMode(op: c_int) -> bool {
    (luaP_opmodes[op as usize] & (1 << 4)) != 0
}

#[inline]
unsafe fn hasjumps(e: *const expdesc) -> bool {
    unsafe { (*e).t != (*e).f }
}

#[inline]
unsafe fn foldbinop(op: c_int) -> bool {
    op <= OPR_SHR
}

#[inline]
unsafe fn int2sC(i: c_int) -> c_int {
    i + OFFSET_sC
}

#[inline]
unsafe fn fitsC(i: lua_Integer) -> bool {
    ((i as u64).wrapping_add(OFFSET_sC as u64)) <= MAXARG_C as u64
}

#[inline]
unsafe fn fitsBx(i: lua_Integer) -> bool {
    -OFFSET_sBx as lua_Integer <= i && i <= (MAXARG_Bx - OFFSET_sBx) as lua_Integer
}

#[inline]
unsafe fn nvalue(o: *const TValue) -> lua_Number {
    unsafe {
        if ttisinteger(o) {
            ivalue(o) as lua_Number
        } else {
            fltvalue(o)
        }
    }
}

#[inline]
unsafe fn needvatab(p: *mut Proto) {
    unsafe {
        (*p).flag |= PF_VATAB;
    }
}

#[inline]
unsafe fn getinstruction_ref(fs: *mut FuncState, e: *const expdesc) -> *mut Instruction {
    unsafe { (*(*fs).f).code.add((*e).u.info as usize) }
}

#[inline]
unsafe fn getinstruction(fs: *mut FuncState, e: *const expdesc) -> Instruction {
    unsafe { *getinstruction_ref(fs, e) }
}

#[inline]
unsafe fn tonumeral(e: *const expdesc, v: *mut TValue) -> c_int {
    unsafe {
        if hasjumps(e) {
            return 0;
        }
        match (*e).k {
            VKINT => {
                if !v.is_null() {
                    setivalue(v, (*e).u.ival);
                }
                1
            }
            VKFLT => {
                if !v.is_null() {
                    setfltvalue(v, (*e).u.nval);
                }
                1
            }
            _ => 0,
        }
    }
}

#[inline]
unsafe fn const2val(fs: *mut FuncState, e: *const expdesc) -> *mut TValue {
    unsafe { ptr::addr_of_mut!((*(*(*(*fs).ls).dyd).actvar.arr.add((*e).u.info as usize)).k) }
}

#[inline]
unsafe fn previousinstruction(fs: *mut FuncState) -> Instruction {
    unsafe {
        if (*fs).pc > (*fs).lasttarget {
            *(*(*fs).f).code.add(((*fs).pc - 1) as usize)
        } else {
            !0
        }
    }
}

#[inline]
unsafe fn freereg(fs: *mut FuncState, reg: c_int) {
    unsafe {
        if reg >= luaY_nvarstack(fs) as c_int {
            (*fs).freereg = (*fs).freereg.wrapping_sub(1);
            debug_assert_eq!(reg, (*fs).freereg as c_int);
        }
    }
}

#[inline]
unsafe fn freeregs(fs: *mut FuncState, r1: c_int, r2: c_int) {
    unsafe {
        if r1 > r2 {
            freereg(fs, r1);
            freereg(fs, r2);
        } else {
            freereg(fs, r2);
            freereg(fs, r1);
        }
    }
}

#[inline]
unsafe fn freeexp(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        if (*e).k == VNONRELOC {
            freereg(fs, (*e).u.info);
        }
    }
}

#[inline]
unsafe fn freeexps(fs: *mut FuncState, e1: *mut expdesc, e2: *mut expdesc) {
    unsafe {
        let r1 = if (*e1).k == VNONRELOC {
            (*e1).u.info
        } else {
            -1
        };
        let r2 = if (*e2).k == VNONRELOC {
            (*e2).u.info
        } else {
            -1
        };
        freeregs(fs, r1, r2);
    }
}

/// 语义错误（只接受预格式化的消息字符串，消除 extern C 变参）

pub unsafe fn luaK_semerror(ls: *mut LexState, msg: *const c_char) -> ! {
    unsafe {
        (*ls).t.token = 0;
        (*ls).linenumber = (*ls).lastline;
        luaX_syntaxerror(ls, msg)
    }
}

/// 带格式化的语义错误（内部用，接受一个 `&str` 消息）
pub(crate) unsafe fn luaK_semerror1(ls: *mut LexState, msg: &str) -> ! {
    let pushed = unsafe { luaO_pushstr((*ls).L, msg) };
    unsafe { luaK_semerror(ls, pushed) }
}

pub unsafe fn luaK_exp2const(fs: *mut FuncState, e: *const expdesc, v: *mut TValue) -> c_int {
    unsafe {
        if hasjumps(e) {
            return 0;
        }
        match (*e).k {
            VFALSE => {
                setbfvalue(v);
                1
            }
            VTRUE => {
                setbtvalue(v);
                1
            }
            VNIL => {
                setnilvalue(v);
                1
            }
            VKSTR => {
                setsvalue(v, (*e).u.strval);
                1
            }
            VCONST => {
                setobj(v, const2val(fs, e));
                1
            }
            _ => tonumeral(e, v),
        }
    }
}

pub unsafe fn luaK_nil(fs: *mut FuncState, mut from: c_int, n: c_int) {
    unsafe {
        let mut l = from + n - 1;
        let prev = previousinstruction(fs);
        if GET_OPCODE(prev) == OP_LOADNIL {
            let pfrom = GETARG_A(prev);
            let pl = pfrom + GETARG_B(prev);
            if (pfrom <= from && from <= pl + 1) || (from <= pfrom && pfrom <= l + 1) {
                if pfrom < from {
                    from = pfrom;
                }
                if pl > l {
                    l = pl;
                }
                let previous = (*(*fs).f).code.add(((*fs).pc - 1) as usize);
                SETARG_A(&mut *previous, from);
                SETARG_B(&mut *previous, l - from);
                return;
            }
        }
        luaK_codeABCk(fs, OP_LOADNIL, from, n - 1, 0, 0);
    }
}

#[inline]
unsafe fn getjump(fs: *mut FuncState, pc: c_int) -> c_int {
    unsafe {
        let offset = GETARG_sJ(*(*(*fs).f).code.add(pc as usize));
        if offset == NO_JUMP {
            NO_JUMP
        } else {
            pc + 1 + offset
        }
    }
}

#[inline]
unsafe fn fixjump(fs: *mut FuncState, pc: c_int, dest: c_int) {
    unsafe {
        let jmp = (*(*fs).f).code.add(pc as usize);
        let offset = dest - (pc + 1);
        if !(-OFFSET_sJ <= offset && offset <= MAXARG_sJ - OFFSET_sJ) {
            luaX_syntaxerror((*fs).ls, c"control structure too long".as_ptr());
        }
        debug_assert_eq!(GET_OPCODE(*jmp), OP_JMP);
        SETARG_sJ(&mut *jmp, offset);
    }
}

pub unsafe fn luaK_concat(fs: *mut FuncState, l1: *mut c_int, l2: c_int) {
    unsafe {
        if l2 == NO_JUMP {
            return;
        }
        if *l1 == NO_JUMP {
            *l1 = l2;
            return;
        }
        let mut list = *l1;
        loop {
            let next = getjump(fs, list);
            if next == NO_JUMP {
                break;
            }
            list = next;
        }
        fixjump(fs, list, l2);
    }
}

#[inline]
unsafe fn codesJ(fs: *mut FuncState, o: c_int, sj: c_int, k: c_int) -> c_int {
    unsafe {
        let j = sj + OFFSET_sJ;
        debug_assert_eq!(getOpMode(o), isJ);
        debug_assert!(j <= MAXARG_sJ && (k & !1) == 0);
        luaK_code(fs, CREATE_sJ(o, j, k))
    }
}

pub unsafe fn luaK_jump(fs: *mut FuncState) -> c_int {
    unsafe { codesJ(fs, OP_JMP, NO_JUMP, 0) }
}

pub unsafe fn luaK_ret(fs: *mut FuncState, first: c_int, nret: c_int) {
    unsafe {
        let op = match nret {
            0 => OP_RETURN0,
            1 => OP_RETURN1,
            _ => OP_RETURN,
        };
        luaY_checklimit(fs, nret + 1, MAXARG_B, c"returns".as_ptr());
        luaK_codeABCk(fs, op, first, nret + 1, 0, 0);
    }
}

#[inline]
unsafe fn condjump(fs: *mut FuncState, op: c_int, a: c_int, b: c_int, c: c_int, k: c_int) -> c_int {
    unsafe {
        luaK_codeABCk(fs, op, a, b, c, k);
        luaK_jump(fs)
    }
}

pub unsafe fn luaK_getlabel(fs: *mut FuncState) -> c_int {
    unsafe {
        (*fs).lasttarget = (*fs).pc;
        (*fs).pc
    }
}

#[inline]
unsafe fn getjumpcontrol(fs: *mut FuncState, pc: c_int) -> *mut Instruction {
    unsafe {
        let pi = (*(*fs).f).code.add(pc as usize);
        if pc >= 1 && testTMode(GET_OPCODE(*pi.sub(1))) {
            pi.sub(1)
        } else {
            pi
        }
    }
}

#[inline]
unsafe fn patchtestreg(fs: *mut FuncState, node: c_int, reg: c_int) -> c_int {
    unsafe {
        let i = getjumpcontrol(fs, node);
        if GET_OPCODE(*i) != OP_TESTSET {
            return 0;
        }
        if reg != NO_REG && reg != GETARG_B(*i) {
            SETARG_A(&mut *i, reg);
        } else {
            *i = CREATE_ABCk(OP_TEST, GETARG_B(*i), 0, 0, GETARG_k(*i));
        }
        1
    }
}

#[inline]
unsafe fn removevalues(fs: *mut FuncState, mut list: c_int) {
    unsafe {
        while list != NO_JUMP {
            patchtestreg(fs, list, NO_REG);
            list = getjump(fs, list);
        }
    }
}

#[inline]
unsafe fn patchlistaux(
    fs: *mut FuncState,
    mut list: c_int,
    vtarget: c_int,
    reg: c_int,
    dtarget: c_int,
) {
    unsafe {
        while list != NO_JUMP {
            let next = getjump(fs, list);
            if patchtestreg(fs, list, reg) != 0 {
                fixjump(fs, list, vtarget);
            } else {
                fixjump(fs, list, dtarget);
            }
            list = next;
        }
    }
}

pub unsafe fn luaK_patchlist(fs: *mut FuncState, list: c_int, target: c_int) {
    unsafe {
        debug_assert!(target <= (*fs).pc);
        patchlistaux(fs, list, target, NO_REG, target);
    }
}

pub unsafe fn luaK_patchtohere(fs: *mut FuncState, list: c_int) {
    unsafe {
        let hr = luaK_getlabel(fs);
        luaK_patchlist(fs, list, hr);
    }
}

#[inline]
unsafe fn savelineinfo(fs: *mut FuncState, f: *mut Proto, line: c_int) {
    unsafe {
        let mut linedif = line - (*fs).previousline;
        let pc = (*fs).pc - 1;
        if linedif.abs() >= LIMLINEDIFF || ((*fs).iwthabs as c_int) >= MAXIWTHABS {
            (*fs).iwthabs = (*fs).iwthabs.wrapping_add(1);
            let abslineinfo = getabslineinfo(f);
            (*f).abslineinfo = grow_vector(
                (*(*fs).ls).L,
                abslineinfo,
                (*fs).nabslineinfo,
                &mut (*f).sizeabslineinfo,
                c_int::MAX,
                c"lines".as_ptr(),
            )
            .cast();
            let abslineinfo = getabslineinfo(f);
            (*abslineinfo.add((*fs).nabslineinfo as usize)).pc = pc;
            (*abslineinfo.add((*fs).nabslineinfo as usize)).line = line;
            (*fs).nabslineinfo += 1;
            linedif = ABSLINEINFO as c_int;
            (*fs).iwthabs = 1;
        } else {
            (*fs).iwthabs = (*fs).iwthabs.wrapping_add(1);
        }
        (*f).lineinfo = grow_vector(
            (*(*fs).ls).L,
            (*f).lineinfo,
            pc,
            &mut (*f).sizelineinfo,
            c_int::MAX,
            c"opcodes".as_ptr(),
        );
        *(*f).lineinfo.add(pc as usize) = linedif as i8;
        (*fs).previousline = line;
    }
}

#[inline]
unsafe fn removelastlineinfo(fs: *mut FuncState) {
    unsafe {
        let f = (*fs).f;
        let pc = (*fs).pc - 1;
        let lineinfo = *(*f).lineinfo.add(pc as usize);
        if lineinfo != ABSLINEINFO {
            (*fs).previousline -= lineinfo as c_int;
            (*fs).iwthabs = (*fs).iwthabs.wrapping_sub(1);
        } else {
            debug_assert_eq!(
                (*getabslineinfo(f).add(((*fs).nabslineinfo - 1) as usize)).pc,
                pc
            );
            (*fs).nabslineinfo -= 1;
            (*fs).iwthabs = (MAXIWTHABS + 1) as u8;
        }
    }
}

#[inline]
unsafe fn removelastinstruction(fs: *mut FuncState) {
    unsafe {
        removelastlineinfo(fs);
        (*fs).pc -= 1;
    }
}

pub unsafe fn luaK_code(fs: *mut FuncState, i: Instruction) -> c_int {
    unsafe {
        let f = (*fs).f;
        (*f).code = grow_vector(
            (*(*fs).ls).L,
            (*f).code,
            (*fs).pc,
            &mut (*f).sizecode,
            c_int::MAX,
            c"opcodes".as_ptr(),
        );
        *(*f).code.add((*fs).pc as usize) = i;
        (*fs).pc += 1;
        savelineinfo(fs, f, (*(*fs).ls).lastline);
        (*fs).pc - 1
    }
}

pub unsafe fn luaK_codeABCk(
    fs: *mut FuncState,
    o: c_int,
    a: c_int,
    b: c_int,
    c: c_int,
    k: c_int,
) -> c_int {
    unsafe {
        debug_assert_eq!(getOpMode(o), iABC);
        luaK_code(fs, CREATE_ABCk(o, a, b, c, k))
    }
}

pub unsafe fn luaK_codevABCk(
    fs: *mut FuncState,
    o: c_int,
    a: c_int,
    b: c_int,
    c: c_int,
    k: c_int,
) -> c_int {
    unsafe {
        debug_assert_eq!(getOpMode(o), ivABC);
        luaK_code(fs, CREATE_vABCk(o, a, b, c, k))
    }
}

pub unsafe fn luaK_codeABx(fs: *mut FuncState, o: c_int, a: c_int, bx: c_int) -> c_int {
    unsafe {
        debug_assert_eq!(getOpMode(o), iABx);
        luaK_code(fs, CREATE_ABx(o, a, bx))
    }
}

#[inline]
unsafe fn codeAsBx(fs: *mut FuncState, o: c_int, a: c_int, bc: c_int) -> c_int {
    unsafe { luaK_code(fs, CREATE_ABx(o, a, bc + OFFSET_sBx)) }
}

#[inline]
unsafe fn codeextraarg(fs: *mut FuncState, a: c_int) -> c_int {
    unsafe { luaK_code(fs, CREATE_Ax(OP_EXTRAARG, a)) }
}

#[inline]
unsafe fn luaK_codek(fs: *mut FuncState, reg: c_int, k: c_int) -> c_int {
    unsafe {
        if k <= MAXARG_Bx {
            luaK_codeABx(fs, OP_LOADK, reg, k)
        } else {
            let p = luaK_codeABx(fs, OP_LOADKX, reg, 0);
            codeextraarg(fs, k);
            p
        }
    }
}

pub unsafe fn luaK_checkstack(fs: *mut FuncState, n: c_int) {
    unsafe {
        let newstack = (*fs).freereg as c_int + n;
        if newstack > (*(*fs).f).maxstacksize as c_int {
            luaY_checklimit(fs, newstack, MAX_FSTACK, c"registers".as_ptr());
            (*(*fs).f).maxstacksize = cast_byte(newstack);
        }
    }
}

pub unsafe fn luaK_reserveregs(fs: *mut FuncState, n: c_int) {
    unsafe {
        luaK_checkstack(fs, n);
        (*fs).freereg = cast_byte((*fs).freereg as c_int + n);
    }
}

#[inline]
unsafe fn addk(fs: *mut FuncState, f: *mut Proto, v: *mut TValue) -> c_int {
    unsafe {
        let L = (*(*fs).ls).L;
        let mut oldsize = (*f).sizek;
        let k = (*fs).nk;
        (*f).k = grow_vector(
            L,
            (*f).k,
            k,
            &mut (*f).sizek,
            MAXARG_Ax,
            c"constants".as_ptr(),
        );
        while oldsize < (*f).sizek {
            setnilvalue((*f).k.add(oldsize as usize));
            oldsize += 1;
        }
        setobj((*f).k.add(k as usize), v);
        (*fs).nk += 1;
        luaC_barrier(L, obj2gco(f), v);
        k
    }
}

#[inline]
unsafe fn k2proto(fs: *mut FuncState, key: *mut TValue, v: *mut TValue) -> c_int {
    unsafe {
        let mut val = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        let f = (*fs).f;
        let tag = luaH_get((*fs).kcache, key, ptr::addr_of_mut!(val));
        if !tagisempty(tag) {
            let k = ivalue(ptr::addr_of!(val)) as c_int;
            debug_assert!(ttisfloat(key) || luaV_rawequalobj((*f).k.add(k as usize), v) != 0);
            k
        } else {
            let k = addk(fs, f, v);
            setivalue(ptr::addr_of_mut!(val), k as lua_Integer);
            luaH_set((*(*fs).ls).L, (*fs).kcache, key, ptr::addr_of_mut!(val));
            k
        }
    }
}

#[inline]
unsafe fn stringK(fs: *mut FuncState, s: *mut TString) -> c_int {
    unsafe {
        let mut o = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        setsvalue(ptr::addr_of_mut!(o), s);
        k2proto(fs, ptr::addr_of_mut!(o), ptr::addr_of_mut!(o))
    }
}

#[inline]
unsafe fn luaK_intK(fs: *mut FuncState, n: lua_Integer) -> c_int {
    unsafe {
        let mut o = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        setivalue(ptr::addr_of_mut!(o), n);
        k2proto(fs, ptr::addr_of_mut!(o), ptr::addr_of_mut!(o))
    }
}

#[inline]
unsafe fn luaK_numberK(fs: *mut FuncState, r: lua_Number) -> c_int {
    unsafe {
        let mut o = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        let mut kv = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        setfltvalue(ptr::addr_of_mut!(o), r);
        if r == 0.0 {
            setpvalue(ptr::addr_of_mut!(kv), fs.cast());
            return k2proto(fs, ptr::addr_of_mut!(kv), ptr::addr_of_mut!(o));
        }
        let q = 2.0f64.powi(-(f64::MANTISSA_DIGITS as i32) + 1);
        let k = r * (1.0 + q);
        let mut ik = 0;
        setfltvalue(ptr::addr_of_mut!(kv), k);
        if luaV_flttointeger(k, ptr::addr_of_mut!(ik), F2Ieq) == 0 {
            let n = k2proto(fs, ptr::addr_of_mut!(kv), ptr::addr_of_mut!(o));
            if luaV_rawequalobj((*(*fs).f).k.add(n as usize), ptr::addr_of!(o)) != 0 {
                return n;
            }
        }
        addk(fs, (*fs).f, ptr::addr_of_mut!(o))
    }
}

#[inline]
unsafe fn boolF(fs: *mut FuncState) -> c_int {
    unsafe {
        let mut o = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        setbfvalue(ptr::addr_of_mut!(o));
        k2proto(fs, ptr::addr_of_mut!(o), ptr::addr_of_mut!(o))
    }
}

#[inline]
unsafe fn boolT(fs: *mut FuncState) -> c_int {
    unsafe {
        let mut o = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        setbtvalue(ptr::addr_of_mut!(o));
        k2proto(fs, ptr::addr_of_mut!(o), ptr::addr_of_mut!(o))
    }
}

#[inline]
unsafe fn nilK(fs: *mut FuncState) -> c_int {
    unsafe {
        let mut k = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        let mut v = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        setnilvalue(ptr::addr_of_mut!(v));
        sethvalue(ptr::addr_of_mut!(k), (*fs).kcache);
        k2proto(fs, ptr::addr_of_mut!(k), ptr::addr_of_mut!(v))
    }
}

pub unsafe fn luaK_int(fs: *mut FuncState, reg: c_int, i: lua_Integer) {
    unsafe {
        if fitsBx(i) {
            codeAsBx(fs, OP_LOADI, reg, i as c_int);
        } else {
            luaK_codek(fs, reg, luaK_intK(fs, i));
        }
    }
}

#[inline]
unsafe fn luaK_float(fs: *mut FuncState, reg: c_int, f: lua_Number) {
    unsafe {
        let mut fi = 0;
        if luaV_flttointeger(f, ptr::addr_of_mut!(fi), F2Ieq) != 0 && fitsBx(fi) {
            codeAsBx(fs, OP_LOADF, reg, fi as c_int);
        } else {
            luaK_codek(fs, reg, luaK_numberK(fs, f));
        }
    }
}

pub unsafe fn luaK_codecheckglobal(
    fs: *mut FuncState,
    var: *mut expdesc,
    mut k: c_int,
    line: c_int,
) {
    unsafe {
        luaK_exp2anyreg(fs, var);
        luaK_fixline(fs, line);
        k = if k >= MAXARG_Bx { 0 } else { k + 1 };
        luaK_codeABx(fs, OP_ERRNNIL, (*var).u.info, k);
        luaK_fixline(fs, line);
        freeexp(fs, var);
    }
}

#[inline]
unsafe fn const2exp(v: *mut TValue, e: *mut expdesc) {
    unsafe {
        match ttypetag(v) {
            LUA_VNUMINT => {
                (*e).k = VKINT;
                (*e).u.ival = ivalue(v);
            }
            LUA_VNUMFLT => {
                (*e).k = VKFLT;
                (*e).u.nval = fltvalue(v);
            }
            LUA_VFALSE => (*e).k = VFALSE,
            LUA_VTRUE => (*e).k = VTRUE,
            LUA_VNIL => (*e).k = VNIL,
            LUA_VSHRSTR | LUA_VLNGSTR => {
                (*e).k = VKSTR;
                (*e).u.strval = tsvalue(v);
            }
            _ => debug_assert!(false),
        }
    }
}

pub unsafe fn luaK_setreturns(fs: *mut FuncState, e: *mut expdesc, nresults: c_int) {
    unsafe {
        let pc = getinstruction_ref(fs, e);
        luaY_checklimit(fs, nresults + 1, MAXARG_C, c"multiple results".as_ptr());
        if (*e).k == VCALL {
            SETARG_C(&mut *pc, nresults + 1);
        } else {
            debug_assert_eq!((*e).k, VVARARG);
            SETARG_C(&mut *pc, nresults + 1);
            SETARG_A(&mut *pc, (*fs).freereg as c_int);
            luaK_reserveregs(fs, 1);
        }
    }
}

#[inline]
unsafe fn str2K(fs: *mut FuncState, e: *mut expdesc) -> c_int {
    unsafe {
        let info = stringK(fs, (*e).u.strval);
        (*e).u.info = info;
        (*e).k = VK;
        info
    }
}

pub unsafe fn luaK_setoneret(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        if (*e).k == VCALL {
            debug_assert_eq!(GETARG_C(getinstruction(fs, e)), 2);
            (*e).k = VNONRELOC;
            (*e).u.info = GETARG_A(getinstruction(fs, e));
        } else if (*e).k == VVARARG {
            let pc = getinstruction_ref(fs, e);
            SETARG_C(&mut *pc, 2);
            (*e).k = VRELOC;
        }
    }
}

pub unsafe fn luaK_vapar2local(fs: *mut FuncState, var: *mut expdesc) {
    unsafe {
        needvatab((*fs).f);
        (*var).k = VLOCAL;
    }
}

pub unsafe fn luaK_dischargevars(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        match (*e).k {
            VCONST => const2exp(const2val(fs, e), e),
            VVARGVAR => {
                luaK_vapar2local(fs, e);
                let temp = (*e).u.var.ridx;
                (*e).u.info = temp as c_int;
                (*e).k = VNONRELOC;
            }
            VLOCAL => {
                let temp = (*e).u.var.ridx;
                (*e).u.info = temp as c_int;
                (*e).k = VNONRELOC;
            }
            VUPVAL => {
                (*e).u.info = luaK_codeABCk(fs, OP_GETUPVAL, 0, (*e).u.info, 0, 0);
                (*e).k = VRELOC;
            }
            VINDEXUP => {
                (*e).u.info = luaK_codeABCk(
                    fs,
                    OP_GETTABUP,
                    0,
                    (*e).u.ind.t as c_int,
                    (*e).u.ind.idx as c_int,
                    0,
                );
                (*e).k = VRELOC;
            }
            VINDEXI => {
                freereg(fs, (*e).u.ind.t as c_int);
                (*e).u.info = luaK_codeABCk(
                    fs,
                    OP_GETI,
                    0,
                    (*e).u.ind.t as c_int,
                    (*e).u.ind.idx as c_int,
                    0,
                );
                (*e).k = VRELOC;
            }
            VINDEXSTR => {
                freereg(fs, (*e).u.ind.t as c_int);
                (*e).u.info = luaK_codeABCk(
                    fs,
                    OP_GETFIELD,
                    0,
                    (*e).u.ind.t as c_int,
                    (*e).u.ind.idx as c_int,
                    0,
                );
                (*e).k = VRELOC;
            }
            VINDEXED => {
                freeregs(fs, (*e).u.ind.t as c_int, (*e).u.ind.idx as c_int);
                (*e).u.info = luaK_codeABCk(
                    fs,
                    OP_GETTABLE,
                    0,
                    (*e).u.ind.t as c_int,
                    (*e).u.ind.idx as c_int,
                    0,
                );
                (*e).k = VRELOC;
            }
            VVARGIND => {
                freeregs(fs, (*e).u.ind.t as c_int, (*e).u.ind.idx as c_int);
                (*e).u.info = luaK_codeABCk(
                    fs,
                    OP_GETVARG,
                    0,
                    (*e).u.ind.t as c_int,
                    (*e).u.ind.idx as c_int,
                    0,
                );
                (*e).k = VRELOC;
            }
            VVARARG | VCALL => luaK_setoneret(fs, e),
            _ => {}
        }
    }
}

#[inline]
unsafe fn discharge2reg(fs: *mut FuncState, e: *mut expdesc, reg: c_int) {
    unsafe {
        luaK_dischargevars(fs, e);
        match (*e).k {
            VNIL => luaK_nil(fs, reg, 1),
            VFALSE => {
                luaK_codeABCk(fs, OP_LOADFALSE, reg, 0, 0, 0);
            }
            VTRUE => {
                luaK_codeABCk(fs, OP_LOADTRUE, reg, 0, 0, 0);
            }
            VKSTR => {
                str2K(fs, e);
                luaK_codek(fs, reg, (*e).u.info);
            }
            VK => {
                luaK_codek(fs, reg, (*e).u.info);
            }
            VKFLT => luaK_float(fs, reg, (*e).u.nval),
            VKINT => luaK_int(fs, reg, (*e).u.ival),
            VRELOC => {
                let pc = getinstruction_ref(fs, e);
                SETARG_A(&mut *pc, reg);
            }
            VNONRELOC => {
                if reg != (*e).u.info {
                    luaK_codeABCk(fs, OP_MOVE, reg, (*e).u.info, 0, 0);
                }
            }
            _ => {
                debug_assert_eq!((*e).k, VJMP);
                return;
            }
        }
        (*e).u.info = reg;
        (*e).k = VNONRELOC;
    }
}

#[inline]
unsafe fn discharge2anyreg(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        if (*e).k != VNONRELOC {
            luaK_reserveregs(fs, 1);
            discharge2reg(fs, e, (*fs).freereg as c_int - 1);
        }
    }
}

#[inline]
unsafe fn code_loadbool(fs: *mut FuncState, a: c_int, op: c_int) -> c_int {
    unsafe {
        luaK_getlabel(fs);
        luaK_codeABCk(fs, op, a, 0, 0, 0)
    }
}

#[inline]
unsafe fn need_value(fs: *mut FuncState, mut list: c_int) -> c_int {
    unsafe {
        while list != NO_JUMP {
            let i = *getjumpcontrol(fs, list);
            if GET_OPCODE(i) != OP_TESTSET {
                return 1;
            }
            list = getjump(fs, list);
        }
        0
    }
}

#[inline]
unsafe fn exp2reg(fs: *mut FuncState, e: *mut expdesc, reg: c_int) {
    unsafe {
        discharge2reg(fs, e, reg);
        if (*e).k == VJMP {
            luaK_concat(fs, ptr::addr_of_mut!((*e).t), (*e).u.info);
        }
        if hasjumps(e) {
            let mut p_f = NO_JUMP;
            let mut p_t = NO_JUMP;
            if need_value(fs, (*e).t) != 0 || need_value(fs, (*e).f) != 0 {
                let fj = if (*e).k == VJMP {
                    NO_JUMP
                } else {
                    luaK_jump(fs)
                };
                p_f = code_loadbool(fs, reg, OP_LFALSESKIP);
                p_t = code_loadbool(fs, reg, OP_LOADTRUE);
                luaK_patchtohere(fs, fj);
            }
            let final_ = luaK_getlabel(fs);
            patchlistaux(fs, (*e).f, final_, reg, p_f);
            patchlistaux(fs, (*e).t, final_, reg, p_t);
        }
        (*e).f = NO_JUMP;
        (*e).t = NO_JUMP;
        (*e).u.info = reg;
        (*e).k = VNONRELOC;
    }
}

pub unsafe fn luaK_exp2nextreg(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        luaK_dischargevars(fs, e);
        freeexp(fs, e);
        luaK_reserveregs(fs, 1);
        exp2reg(fs, e, (*fs).freereg as c_int - 1);
    }
}

pub unsafe fn luaK_exp2anyreg(fs: *mut FuncState, e: *mut expdesc) -> c_int {
    unsafe {
        luaK_dischargevars(fs, e);
        if (*e).k == VNONRELOC {
            if !hasjumps(e) {
                return (*e).u.info;
            }
            if (*e).u.info >= luaY_nvarstack(fs) as c_int {
                exp2reg(fs, e, (*e).u.info);
                return (*e).u.info;
            }
        }
        luaK_exp2nextreg(fs, e);
        (*e).u.info
    }
}

pub unsafe fn luaK_exp2anyregup(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        if (((*e).k != VUPVAL) && ((*e).k != VVARGVAR)) || hasjumps(e) {
            luaK_exp2anyreg(fs, e);
        }
    }
}

pub unsafe fn luaK_exp2val(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        if (*e).k == VJMP || hasjumps(e) {
            luaK_exp2anyreg(fs, e);
        } else {
            luaK_dischargevars(fs, e);
        }
    }
}

#[inline]
unsafe fn luaK_exp2K(fs: *mut FuncState, e: *mut expdesc) -> c_int {
    unsafe {
        if hasjumps(e) {
            return 0;
        }
        let info = match (*e).k {
            VTRUE => boolT(fs),
            VFALSE => boolF(fs),
            VNIL => nilK(fs),
            VKINT => luaK_intK(fs, (*e).u.ival),
            VKFLT => luaK_numberK(fs, (*e).u.nval),
            VKSTR => stringK(fs, (*e).u.strval),
            VK => (*e).u.info,
            _ => return 0,
        };
        if info <= MAXINDEXRK {
            (*e).k = VK;
            (*e).u.info = info;
            1
        } else {
            0
        }
    }
}

#[inline]
unsafe fn exp2RK(fs: *mut FuncState, e: *mut expdesc) -> c_int {
    unsafe {
        if luaK_exp2K(fs, e) != 0 {
            1
        } else {
            luaK_exp2anyreg(fs, e);
            0
        }
    }
}

#[inline]
unsafe fn codeABRK(fs: *mut FuncState, o: c_int, a: c_int, b: c_int, ec: *mut expdesc) {
    unsafe {
        let k = exp2RK(fs, ec);
        luaK_codeABCk(fs, o, a, b, (*ec).u.info, k);
    }
}

pub unsafe fn luaK_storevar(fs: *mut FuncState, var: *mut expdesc, ex: *mut expdesc) {
    unsafe {
        match (*var).k {
            VLOCAL => {
                freeexp(fs, ex);
                exp2reg(fs, ex, (*var).u.var.ridx as c_int);
                return;
            }
            VUPVAL => {
                let e = luaK_exp2anyreg(fs, ex);
                luaK_codeABCk(fs, OP_SETUPVAL, e, (*var).u.info, 0, 0);
            }
            VINDEXUP => codeABRK(
                fs,
                OP_SETTABUP,
                (*var).u.ind.t as c_int,
                (*var).u.ind.idx as c_int,
                ex,
            ),
            VINDEXI => codeABRK(
                fs,
                OP_SETI,
                (*var).u.ind.t as c_int,
                (*var).u.ind.idx as c_int,
                ex,
            ),
            VINDEXSTR => codeABRK(
                fs,
                OP_SETFIELD,
                (*var).u.ind.t as c_int,
                (*var).u.ind.idx as c_int,
                ex,
            ),
            VVARGIND => {
                needvatab((*fs).f);
                codeABRK(
                    fs,
                    OP_SETTABLE,
                    (*var).u.ind.t as c_int,
                    (*var).u.ind.idx as c_int,
                    ex,
                );
            }
            VINDEXED => codeABRK(
                fs,
                OP_SETTABLE,
                (*var).u.ind.t as c_int,
                (*var).u.ind.idx as c_int,
                ex,
            ),
            _ => debug_assert!(false),
        }
        freeexp(fs, ex);
    }
}

#[inline]
unsafe fn negatecondition(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        let pc = getjumpcontrol(fs, (*e).u.info);
        SETARG_k(&mut *pc, GETARG_k(*pc) ^ 1);
    }
}

#[inline]
unsafe fn jumponcond(fs: *mut FuncState, e: *mut expdesc, cond: c_int) -> c_int {
    unsafe {
        if (*e).k == VRELOC {
            let ie = getinstruction(fs, e);
            if GET_OPCODE(ie) == OP_NOT {
                removelastinstruction(fs);
                return condjump(
                    fs,
                    OP_TEST,
                    GETARG_B(ie),
                    0,
                    0,
                    if cond == 0 { 1 } else { 0 },
                );
            }
        }
        discharge2anyreg(fs, e);
        freeexp(fs, e);
        condjump(fs, OP_TESTSET, NO_REG, (*e).u.info, 0, cond)
    }
}

pub unsafe fn luaK_goiftrue(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        luaK_dischargevars(fs, e);
        let pc = match (*e).k {
            VJMP => {
                negatecondition(fs, e);
                (*e).u.info
            }
            VK | VKFLT | VKINT | VKSTR | VTRUE => NO_JUMP,
            _ => jumponcond(fs, e, 0),
        };
        luaK_concat(fs, ptr::addr_of_mut!((*e).f), pc);
        luaK_patchtohere(fs, (*e).t);
        (*e).t = NO_JUMP;
    }
}

#[inline]
unsafe fn luaK_goiffalse(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        luaK_dischargevars(fs, e);
        let pc = match (*e).k {
            VJMP => (*e).u.info,
            VNIL | VFALSE => NO_JUMP,
            _ => jumponcond(fs, e, 1),
        };
        luaK_concat(fs, ptr::addr_of_mut!((*e).t), pc);
        luaK_patchtohere(fs, (*e).f);
        (*e).f = NO_JUMP;
    }
}

#[inline]
unsafe fn codenot(fs: *mut FuncState, e: *mut expdesc) {
    unsafe {
        match (*e).k {
            VNIL | VFALSE => (*e).k = VTRUE,
            VK | VKFLT | VKINT | VKSTR | VTRUE => (*e).k = VFALSE,
            VJMP => negatecondition(fs, e),
            VRELOC | VNONRELOC => {
                discharge2anyreg(fs, e);
                freeexp(fs, e);
                (*e).u.info = luaK_codeABCk(fs, OP_NOT, 0, (*e).u.info, 0, 0);
                (*e).k = VRELOC;
            }
            _ => debug_assert!(false),
        }
        core::mem::swap(&mut (*e).f, &mut (*e).t);
        removevalues(fs, (*e).f);
        removevalues(fs, (*e).t);
    }
}

#[inline]
unsafe fn isKstr(fs: *mut FuncState, e: *mut expdesc) -> bool {
    unsafe {
        (*e).k == VK
            && !hasjumps(e)
            && (*e).u.info <= MAXINDEXRK
            && ttisshrstring((*(*fs).f).k.add((*e).u.info as usize))
    }
}

#[inline]
unsafe fn isKint(e: *mut expdesc) -> bool {
    unsafe { (*e).k == VKINT && !hasjumps(e) }
}

#[inline]
unsafe fn isCint(e: *mut expdesc) -> bool {
    unsafe { isKint(e) && ((*e).u.ival as u64) <= MAXARG_C as u64 }
}

#[inline]
unsafe fn isSCint(e: *mut expdesc) -> bool {
    unsafe { isKint(e) && fitsC((*e).u.ival) }
}

#[inline]
unsafe fn isSCnumber(e: *mut expdesc, pi: *mut c_int, isfloat: *mut c_int) -> c_int {
    unsafe {
        let mut i = 0;
        if (*e).k == VKINT {
            i = (*e).u.ival;
        } else if (*e).k == VKFLT
            && luaV_flttointeger((*e).u.nval, ptr::addr_of_mut!(i), F2Ieq) != 0
        {
            *isfloat = 1;
        } else {
            return 0;
        }
        if !hasjumps(e) && fitsC(i) {
            *pi = int2sC(i as c_int);
            1
        } else {
            0
        }
    }
}

pub unsafe fn luaK_self(fs: *mut FuncState, e: *mut expdesc, key: *mut expdesc) {
    unsafe {
        luaK_exp2anyreg(fs, e);
        let ereg = (*e).u.info;
        freeexp(fs, e);
        let base = (*fs).freereg as c_int;
        (*e).u.info = base;
        (*e).k = VNONRELOC;
        luaK_reserveregs(fs, 2);
        if strisshr((*key).u.strval) && luaK_exp2K(fs, key) != 0 {
            luaK_codeABCk(fs, OP_SELF, base, ereg, (*key).u.info, 0);
        } else {
            luaK_exp2anyreg(fs, key);
            luaK_codeABCk(fs, OP_MOVE, base + 1, ereg, 0, 0);
            luaK_codeABCk(fs, OP_GETTABLE, base, ereg, (*key).u.info, 0);
        }
        freeexp(fs, key);
    }
}

#[inline]
unsafe fn fillidxk(t: *mut expdesc, idx: c_int, k: c_int) {
    unsafe {
        (*t).u.ind.idx = idx as i16;
        (*t).k = k;
    }
}

pub unsafe fn luaK_indexed(fs: *mut FuncState, t: *mut expdesc, k: *mut expdesc) {
    unsafe {
        let mut keystr = -1;
        if (*k).k == VKSTR {
            keystr = str2K(fs, k);
        }
        if (*t).k == VUPVAL && !isKstr(fs, k) {
            luaK_exp2anyreg(fs, t);
        }
        if (*t).k == VUPVAL {
            let temp = (*t).u.info as lu_byte;
            (*t).u.ind.t = temp;
            fillidxk(t, (*k).u.info, VINDEXUP);
        } else if (*t).k == VVARGVAR {
            let kreg = luaK_exp2anyreg(fs, k);
            let vreg = (*t).u.var.ridx;
            (*t).u.ind.t = vreg;
            fillidxk(t, kreg, VVARGIND);
        } else {
            (*t).u.ind.t = if (*t).k == VLOCAL {
                (*t).u.var.ridx
            } else {
                (*t).u.info as lu_byte
            };
            if isKstr(fs, k) {
                fillidxk(t, (*k).u.info, VINDEXSTR);
            } else if isCint(k) {
                fillidxk(t, (*k).u.ival as c_int, VINDEXI);
            } else {
                fillidxk(t, luaK_exp2anyreg(fs, k), VINDEXED);
            }
        }
        (*t).u.ind.keystr = keystr;
        (*t).u.ind.ro = 0;
    }
}

#[inline]
unsafe fn validop(op: c_int, v1: *mut TValue, v2: *mut TValue) -> c_int {
    unsafe {
        match op {
            LUA_OPBAND | LUA_OPBOR | LUA_OPBXOR | LUA_OPSHL | LUA_OPSHR | LUA_OPBNOT => {
                let mut i = 0;
                ((luaV_tointegerns(v1, ptr::addr_of_mut!(i), LUA_FLOORN2I) != 0)
                    && (luaV_tointegerns(v2, ptr::addr_of_mut!(i), LUA_FLOORN2I) != 0))
                    as c_int
            }
            LUA_OPDIV | LUA_OPIDIV | LUA_OPMOD => (nvalue(v2) != 0.0) as c_int,
            _ => 1,
        }
    }
}

#[inline]
unsafe fn constfolding(
    fs: *mut FuncState,
    op: c_int,
    e1: *mut expdesc,
    e2: *const expdesc,
) -> c_int {
    unsafe {
        let mut v1 = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        let mut v2 = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        let mut res = TValue {
            value_: Value { i: 0 },
            tt_: 0,
        };
        if tonumeral(e1, ptr::addr_of_mut!(v1)) == 0
            || tonumeral(e2, ptr::addr_of_mut!(v2)) == 0
            || validop(op, ptr::addr_of_mut!(v1), ptr::addr_of_mut!(v2)) == 0
        {
            return 0;
        }
        luaO_rawarith(
            (*(*fs).ls).L,
            op,
            ptr::addr_of!(v1),
            ptr::addr_of!(v2),
            ptr::addr_of_mut!(res),
        );
        if ttisinteger(ptr::addr_of!(res)) {
            (*e1).k = VKINT;
            (*e1).u.ival = ivalue(ptr::addr_of!(res));
        } else {
            let n = fltvalue(ptr::addr_of!(res));
            if n.is_nan() || n == 0.0 {
                return 0;
            }
            (*e1).k = VKFLT;
            (*e1).u.nval = n;
        }
        1
    }
}

#[inline]
unsafe fn binopr2op(opr: c_int, baser: c_int, base: c_int) -> c_int {
    (opr - baser) + base
}

#[inline]
unsafe fn unopr2op(opr: c_int) -> c_int {
    (opr - OPR_MINUS) + OP_UNM
}

#[inline]
unsafe fn binopr2TM(opr: c_int) -> c_int {
    (opr - OPR_ADD) + TM_ADD
}

#[inline]
unsafe fn codeunexpval(fs: *mut FuncState, op: c_int, e: *mut expdesc, line: c_int) {
    unsafe {
        let r = luaK_exp2anyreg(fs, e);
        freeexp(fs, e);
        (*e).u.info = luaK_codeABCk(fs, op, 0, r, 0, 0);
        (*e).k = VRELOC;
        luaK_fixline(fs, line);
    }
}

#[inline]
unsafe fn finishbinexpval(
    fs: *mut FuncState,
    e1: *mut expdesc,
    e2: *mut expdesc,
    op: c_int,
    v2: c_int,
    flip: c_int,
    line: c_int,
    mmop: c_int,
    event: c_int,
) {
    unsafe {
        let v1 = luaK_exp2anyreg(fs, e1);
        let pc = luaK_codeABCk(fs, op, 0, v1, v2, 0);
        freeexps(fs, e1, e2);
        (*e1).u.info = pc;
        (*e1).k = VRELOC;
        luaK_fixline(fs, line);
        luaK_codeABCk(fs, mmop, v1, v2, event, flip);
        luaK_fixline(fs, line);
    }
}

#[inline]
unsafe fn codebinexpval(
    fs: *mut FuncState,
    opr: c_int,
    e1: *mut expdesc,
    e2: *mut expdesc,
    line: c_int,
) {
    unsafe {
        let op = binopr2op(opr, OPR_ADD, OP_ADD);
        let v2 = luaK_exp2anyreg(fs, e2);
        finishbinexpval(fs, e1, e2, op, v2, 0, line, OP_MMBIN, binopr2TM(opr));
    }
}

#[inline]
unsafe fn codebini(
    fs: *mut FuncState,
    op: c_int,
    e1: *mut expdesc,
    e2: *mut expdesc,
    flip: c_int,
    line: c_int,
    event: c_int,
) {
    unsafe {
        let v2 = int2sC((*e2).u.ival as c_int);
        finishbinexpval(fs, e1, e2, op, v2, flip, line, OP_MMBINI, event);
    }
}

#[inline]
unsafe fn codebinK(
    fs: *mut FuncState,
    opr: c_int,
    e1: *mut expdesc,
    e2: *mut expdesc,
    flip: c_int,
    line: c_int,
) {
    unsafe {
        let event = binopr2TM(opr);
        let v2 = (*e2).u.info;
        let op = binopr2op(opr, OPR_ADD, OP_ADDK);
        finishbinexpval(fs, e1, e2, op, v2, flip, line, OP_MMBINK, event);
    }
}

#[inline]
unsafe fn finishbinexpneg(
    fs: *mut FuncState,
    e1: *mut expdesc,
    e2: *mut expdesc,
    op: c_int,
    line: c_int,
    event: c_int,
) -> c_int {
    unsafe {
        if !isKint(e2) {
            return 0;
        }
        let i2 = (*e2).u.ival;
        if !(fitsC(i2) && fitsC(-i2)) {
            return 0;
        }
        let v2 = i2 as c_int;
        finishbinexpval(fs, e1, e2, op, int2sC(-v2), 0, line, OP_MMBINI, event);
        let last = (*(*fs).f).code.add(((*fs).pc - 1) as usize);
        SETARG_B(&mut *last, int2sC(v2));
        1
    }
}

#[inline]
unsafe fn swapexps(e1: *mut expdesc, e2: *mut expdesc) {
    unsafe {
        core::mem::swap(&mut *e1, &mut *e2);
    }
}

#[inline]
unsafe fn codebinNoK(
    fs: *mut FuncState,
    opr: c_int,
    e1: *mut expdesc,
    e2: *mut expdesc,
    flip: c_int,
    line: c_int,
) {
    unsafe {
        if flip != 0 {
            swapexps(e1, e2);
        }
        codebinexpval(fs, opr, e1, e2, line);
    }
}

#[inline]
unsafe fn codearith(
    fs: *mut FuncState,
    opr: c_int,
    e1: *mut expdesc,
    e2: *mut expdesc,
    flip: c_int,
    line: c_int,
) {
    unsafe {
        if tonumeral(e2, ptr::null_mut()) != 0 && luaK_exp2K(fs, e2) != 0 {
            codebinK(fs, opr, e1, e2, flip, line);
        } else {
            codebinNoK(fs, opr, e1, e2, flip, line);
        }
    }
}

#[inline]
unsafe fn codecommutative(
    fs: *mut FuncState,
    op: c_int,
    e1: *mut expdesc,
    e2: *mut expdesc,
    line: c_int,
) {
    unsafe {
        let mut flip = 0;
        if tonumeral(e1, ptr::null_mut()) != 0 {
            swapexps(e1, e2);
            flip = 1;
        }
        if op == OPR_ADD && isSCint(e2) {
            codebini(fs, OP_ADDI, e1, e2, flip, line, TM_ADD);
        } else {
            codearith(fs, op, e1, e2, flip, line);
        }
    }
}

#[inline]
unsafe fn codebitwise(
    fs: *mut FuncState,
    opr: c_int,
    e1: *mut expdesc,
    e2: *mut expdesc,
    line: c_int,
) {
    unsafe {
        let mut flip = 0;
        if (*e1).k == VKINT {
            swapexps(e1, e2);
            flip = 1;
        }
        if (*e2).k == VKINT && luaK_exp2K(fs, e2) != 0 {
            codebinK(fs, opr, e1, e2, flip, line);
        } else {
            codebinNoK(fs, opr, e1, e2, flip, line);
        }
    }
}

#[inline]
unsafe fn codeorder(fs: *mut FuncState, opr: c_int, e1: *mut expdesc, e2: *mut expdesc) {
    unsafe {
        let mut im = 0;
        let mut isfloat = 0;
        let (r1, r2, op) = if isSCnumber(e2, ptr::addr_of_mut!(im), ptr::addr_of_mut!(isfloat)) != 0
        {
            (luaK_exp2anyreg(fs, e1), im, binopr2op(opr, OPR_LT, OP_LTI))
        } else if isSCnumber(e1, ptr::addr_of_mut!(im), ptr::addr_of_mut!(isfloat)) != 0 {
            (luaK_exp2anyreg(fs, e2), im, binopr2op(opr, OPR_LT, OP_GTI))
        } else {
            (
                luaK_exp2anyreg(fs, e1),
                luaK_exp2anyreg(fs, e2),
                binopr2op(opr, OPR_LT, OP_LT),
            )
        };
        freeexps(fs, e1, e2);
        (*e1).u.info = condjump(fs, op, r1, r2, isfloat, 1);
        (*e1).k = VJMP;
    }
}

#[inline]
unsafe fn codeeq(fs: *mut FuncState, opr: c_int, e1: *mut expdesc, e2: *mut expdesc) {
    unsafe {
        if (*e1).k != VNONRELOC {
            swapexps(e1, e2);
        }
        let r1 = luaK_exp2anyreg(fs, e1);
        let mut im = 0;
        let mut isfloat = 0;
        let (op, r2) = if isSCnumber(e2, ptr::addr_of_mut!(im), ptr::addr_of_mut!(isfloat)) != 0 {
            (OP_EQI, im)
        } else if exp2RK(fs, e2) != 0 {
            (OP_EQK, (*e2).u.info)
        } else {
            (OP_EQ, luaK_exp2anyreg(fs, e2))
        };
        freeexps(fs, e1, e2);
        (*e1).u.info = condjump(fs, op, r1, r2, isfloat, (opr == OPR_EQ) as c_int);
        (*e1).k = VJMP;
    }
}

pub unsafe fn luaK_prefix(fs: *mut FuncState, opr: c_int, e: *mut expdesc, line: c_int) {
    unsafe {
        let ef = expdesc {
            k: VKINT,
            u: ExpdescUnion { ival: 0 },
            t: NO_JUMP,
            f: NO_JUMP,
        };
        luaK_dischargevars(fs, e);
        match opr {
            OPR_MINUS | OPR_BNOT => {
                if constfolding(fs, opr + LUA_OPUNM, e, ptr::addr_of!(ef)) == 0 {
                    codeunexpval(fs, unopr2op(opr), e, line);
                }
            }
            OPR_LEN => codeunexpval(fs, unopr2op(opr), e, line),
            OPR_NOT => codenot(fs, e),
            _ => debug_assert!(false),
        }
    }
}

pub unsafe fn luaK_infix(fs: *mut FuncState, op: c_int, v: *mut expdesc) {
    unsafe {
        luaK_dischargevars(fs, v);
        match op {
            OPR_AND => luaK_goiftrue(fs, v),
            OPR_OR => luaK_goiffalse(fs, v),
            OPR_CONCAT => luaK_exp2nextreg(fs, v),
            OPR_ADD | OPR_SUB | OPR_MUL | OPR_DIV | OPR_IDIV | OPR_MOD | OPR_POW | OPR_BAND
            | OPR_BOR | OPR_BXOR | OPR_SHL | OPR_SHR => {
                if tonumeral(v, ptr::null_mut()) == 0 {
                    luaK_exp2anyreg(fs, v);
                }
            }
            OPR_EQ | OPR_NE => {
                if tonumeral(v, ptr::null_mut()) == 0 {
                    exp2RK(fs, v);
                }
            }
            OPR_LT | OPR_LE | OPR_GT | OPR_GE => {
                let mut d1 = 0;
                let mut d2 = 0;
                if isSCnumber(v, ptr::addr_of_mut!(d1), ptr::addr_of_mut!(d2)) == 0 {
                    luaK_exp2anyreg(fs, v);
                }
            }
            _ => debug_assert!(false),
        }
    }
}

#[inline]
unsafe fn codeconcat(fs: *mut FuncState, e1: *mut expdesc, e2: *mut expdesc, line: c_int) {
    unsafe {
        let ie2 = previousinstruction(fs);
        if GET_OPCODE(ie2) == OP_CONCAT {
            let n = GETARG_B(ie2);
            freeexp(fs, e2);
            let previous = (*(*fs).f).code.add(((*fs).pc - 1) as usize);
            SETARG_A(&mut *previous, (*e1).u.info);
            SETARG_B(&mut *previous, n + 1);
        } else {
            luaK_codeABCk(fs, OP_CONCAT, (*e1).u.info, 2, 0, 0);
            freeexp(fs, e2);
            luaK_fixline(fs, line);
        }
    }
}

pub unsafe fn luaK_posfix(
    fs: *mut FuncState,
    mut opr: c_int,
    e1: *mut expdesc,
    e2: *mut expdesc,
    line: c_int,
) {
    unsafe {
        luaK_dischargevars(fs, e2);
        if foldbinop(opr) && constfolding(fs, opr + LUA_OPADD, e1, e2) != 0 {
            return;
        }
        match opr {
            OPR_AND => {
                luaK_concat(fs, ptr::addr_of_mut!((*e2).f), (*e1).f);
                *e1 = *e2;
            }
            OPR_OR => {
                luaK_concat(fs, ptr::addr_of_mut!((*e2).t), (*e1).t);
                *e1 = *e2;
            }
            OPR_CONCAT => {
                luaK_exp2nextreg(fs, e2);
                codeconcat(fs, e1, e2, line);
            }
            OPR_ADD | OPR_MUL => codecommutative(fs, opr, e1, e2, line),
            OPR_SUB => {
                if finishbinexpneg(fs, e1, e2, OP_ADDI, line, TM_SUB) == 0 {
                    codearith(fs, opr, e1, e2, 0, line);
                }
            }
            OPR_DIV | OPR_IDIV | OPR_MOD | OPR_POW => codearith(fs, opr, e1, e2, 0, line),
            OPR_BAND | OPR_BOR | OPR_BXOR => codebitwise(fs, opr, e1, e2, line),
            OPR_SHL => {
                if isSCint(e1) {
                    swapexps(e1, e2);
                    codebini(fs, OP_SHLI, e1, e2, 1, line, TM_SHL);
                } else if finishbinexpneg(fs, e1, e2, OP_SHRI, line, TM_SHL) == 0 {
                    codebinexpval(fs, opr, e1, e2, line);
                }
            }
            OPR_SHR => {
                if isSCint(e2) {
                    codebini(fs, OP_SHRI, e1, e2, 0, line, TM_SHR);
                } else {
                    codebinexpval(fs, opr, e1, e2, line);
                }
            }
            OPR_EQ | OPR_NE => codeeq(fs, opr, e1, e2),
            OPR_GT | OPR_GE => {
                swapexps(e1, e2);
                opr = (opr - OPR_GT) + OPR_LT;
                codeorder(fs, opr, e1, e2);
            }
            OPR_LT | OPR_LE => codeorder(fs, opr, e1, e2),
            _ => debug_assert!(false),
        }
    }
}

pub unsafe fn luaK_fixline(fs: *mut FuncState, line: c_int) {
    unsafe {
        removelastlineinfo(fs);
        savelineinfo(fs, (*fs).f, line);
    }
}

pub unsafe fn luaK_settablesize(
    fs: *mut FuncState,
    pc: c_int,
    ra: c_int,
    asize: c_int,
    mut hsize: c_int,
) {
    unsafe {
        let inst = (*(*fs).f).code.add(pc as usize);
        let extra = asize / (MAXARG_vC + 1);
        let rc = asize % (MAXARG_vC + 1);
        let k = (extra > 0) as c_int;
        hsize = if hsize != 0 {
            luaO_ceillog2(hsize as c_uint) as c_int + 1
        } else {
            0
        };
        *inst = CREATE_vABCk(OP_NEWTABLE, ra, hsize, rc, k);
        *inst.add(1) = CREATE_Ax(OP_EXTRAARG, extra);
    }
}

pub unsafe fn luaK_setlist(fs: *mut FuncState, base: c_int, mut nelems: c_int, mut tostore: c_int) {
    unsafe {
        if tostore == LUA_MULTRET {
            tostore = 0;
        }
        if nelems <= MAXARG_vC {
            luaK_codevABCk(fs, OP_SETLIST, base, tostore, nelems, 0);
        } else {
            let extra = nelems / (MAXARG_vC + 1);
            nelems %= MAXARG_vC + 1;
            luaK_codevABCk(fs, OP_SETLIST, base, tostore, nelems, 1);
            codeextraarg(fs, extra);
        }
        (*fs).freereg = cast_byte(base + 1);
    }
}

#[inline]
unsafe fn finaltarget(code: *mut Instruction, mut i: c_int) -> c_int {
    unsafe {
        let mut count = 0;
        while count < 100 {
            let pc = *code.add(i as usize);
            if GET_OPCODE(pc) != OP_JMP {
                break;
            }
            i += GETARG_sJ(pc) + 1;
            count += 1;
        }
        i
    }
}

pub unsafe fn luaK_finish(fs: *mut FuncState) {
    unsafe {
        let p = (*fs).f;
        if (*p).flag & PF_VATAB != 0 {
            (*p).flag &= !PF_VAHID;
        }
        let mut i = 0;
        while i < (*fs).pc {
            let pc = (*p).code.add(i as usize);
            debug_assert!(
                i == 0 || crate::opcodes::luaP_isOT(*pc.sub(1)) == crate::opcodes::luaP_isIT(*pc)
            );
            match GET_OPCODE(*pc) {
                OP_RETURN0 | OP_RETURN1 => {
                    if (*fs).needclose != 0 || ((*p).flag & PF_VAHID) != 0 {
                        SET_OPCODE(&mut *pc, OP_RETURN);
                        if (*fs).needclose != 0 {
                            SETARG_k(&mut *pc, 1);
                        }
                        if (*p).flag & PF_VAHID != 0 {
                            SETARG_C(&mut *pc, (*p).numparams as c_int + 1);
                        }
                    }
                }
                OP_RETURN | OP_TAILCALL => {
                    if (*fs).needclose != 0 {
                        SETARG_k(&mut *pc, 1);
                    }
                    if (*p).flag & PF_VAHID != 0 {
                        SETARG_C(&mut *pc, (*p).numparams as c_int + 1);
                    }
                }
                OP_GETVARG => {
                    if (*p).flag & PF_VATAB != 0 {
                        SET_OPCODE(&mut *pc, OP_GETTABLE);
                    }
                }
                OP_VARARG => {
                    if (*p).flag & PF_VATAB != 0 {
                        SETARG_k(&mut *pc, 1);
                    }
                }
                OP_JMP => {
                    let target = finaltarget((*p).code, i);
                    fixjump(fs, i, target);
                }
                _ => {}
            }
            i += 1;
        }
    }
}
