use crate::lua_module::{lua_Integer, lua_Number, lua_State, lua_Unsigned};
use crate::luaffi::LUA_ERRSYNTAX;
use crate::zio::{EOZ, ZIO, luaZ_fill, luaZ_getaddr, luaZ_read};
use core::ffi::{c_char, c_int, c_void};
use core::mem::{MaybeUninit, size_of};
use core::ptr;

const LUA_SIGNATURE: &[u8] = b"\x1bLua";
const LUAC_DATA: &[u8] = b"\x19\x93\r\n\x1a\n";
const LUAC_VERSION: u8 = 0x55;
const LUAC_FORMAT: u8 = 0;
const LUAC_INT: c_int = -0x5678;
const LUAC_INST: Instruction = 0x1234_5678;
const LUAC_NUM: lua_Number = -370.5;
const LUAI_MAXSHORTLEN: usize = 40;
const MAX_SIZE: usize = lua_Integer::MAX as usize;
const PF_FIXED: u8 = 4;

const LUA_TSTRING: u8 = 4;
const BIT_ISCOLLECTABLE: u8 = 1 << 6;
const LUA_VNIL: u8 = 0;
const LUA_VFALSE: u8 = 1;
const LUA_VTRUE: u8 = 17;
const LUA_VNUMINT: u8 = 3;
const LUA_VNUMFLT: u8 = 19;
const LUA_VSHRSTR: u8 = 4;
const LUA_VLNGSTR: u8 = 20;
const LUA_VLCL: u8 = 6;
const LUA_VTABLE: u8 = 5;

const WHITEBITS: u8 = 0b11;
const BLACKBIT: u8 = 5;

type LuaAlloc = Option<unsafe extern "C" fn(*mut c_void, *mut c_void, usize, usize) -> *mut c_void>;

#[derive(Copy, Clone)]
#[repr(C)]
union Value {
    gc: *mut GCObject,
    p: *mut c_void,
    f: Option<unsafe extern "C" fn(*mut lua_State) -> c_int>,
    i: lua_Integer,
    n: lua_Number,
    ub: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct TValue {
    value_: Value,
    tt_: u8,
}

#[repr(C)]
struct GCObject {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
}

#[repr(C)]
struct TStringUnion {
    lnglen: usize,
}

#[repr(C)]
struct TString {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    extra: u8,
    shrlen: i8,
    hash: u32,
    u: TStringUnion,
    contents: *mut c_char,
    falloc: LuaAlloc,
    ud: *mut c_void,
}

type Instruction = u32;
#[allow(non_camel_case_types)]
type ls_byte = i8;

#[repr(C)]
struct Upvaldesc {
    name: *mut TString,
    instack: u8,
    idx: u8,
    kind: u8,
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
struct Proto {
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
    lineinfo: *mut ls_byte,
    abslineinfo: *mut AbsLineInfo,
    locvars: *mut LocVar,
    source: *mut TString,
    gclist: *mut GCObject,
}

#[repr(C)]
struct UpVal {
    _private: [u8; 0],
}

#[repr(C)]
pub struct LClosure {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    nupvalues: u8,
    gclist: *mut GCObject,
    p: *mut Proto,
    upvals: [*mut UpVal; 1],
}

#[repr(C)]
struct Node {
    _private: [u8; 0],
}

#[repr(C)]
struct Table {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    flags: u8,
    lsizenode: u8,
    asize: u32,
    array: *mut Value,
    node: *mut Node,
    metatable: *mut Table,
    gclist: *mut GCObject,
}

#[derive(Copy, Clone)]
#[repr(C)]
union StkIdRel {
    p: *mut StackValue,
    offset: isize,
}

#[derive(Copy, Clone)]
#[repr(C)]
union StackValue {
    val: TValue,
    _tbclist: StackValueTbc,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct StackValueTbc {
    value_: Value,
    tt_: u8,
    delta: u16,
}

#[repr(C)]
struct LuaStatePrefix {
    next: *mut GCObject,
    tt: u8,
    marked: u8,
    allowhook: u8,
    status: u8,
    top: StkIdRel,
}

struct LoadState {
    l: *mut lua_State,
    z: *mut ZIO,
    name: *const c_char,
    h: *mut Table,
    offset: usize,
    nstr: lua_Unsigned,
    fixed: bool,
}

unsafe extern "C" {
    fn luaO_pushfstring(state: *mut lua_State, fmt: *const c_char, ...) -> *const c_char;
    fn luaD_throw(state: *mut lua_State, errcode: u8) -> !;
    fn luaD_inctop(state: *mut lua_State);
    fn luaF_newproto(state: *mut lua_State) -> *mut Proto;
    fn luaF_newLclosure(state: *mut lua_State, nupvals: c_int) -> *mut LClosure;
    fn luaH_new(state: *mut lua_State) -> *mut Table;
    fn luaH_getint(table: *mut Table, key: lua_Integer, result: *mut TValue) -> u8;
    fn luaH_setint(state: *mut lua_State, table: *mut Table, key: lua_Integer, value: *mut TValue);
    fn luaS_newlstr(state: *mut lua_State, s: *const c_char, len: usize) -> *mut TString;
    fn luaS_newextlstr(
        state: *mut lua_State,
        s: *const c_char,
        len: usize,
        falloc: LuaAlloc,
        ud: *mut c_void,
    ) -> *mut TString;
    fn luaS_createlngstrobj(state: *mut lua_State, len: usize) -> *mut TString;
    fn luaC_barrier_(state: *mut lua_State, object: *mut GCObject, value: *mut GCObject);
    fn luaC_barrierback_(state: *mut lua_State, object: *mut GCObject);
    fn luaM_malloc_(state: *mut lua_State, size: usize, tag: c_int) -> *mut c_void;
    fn luaM_toobig(state: *mut lua_State) -> !;
}

#[inline]
fn ctb(tag: u8) -> u8 {
    tag | BIT_ISCOLLECTABLE
}

#[inline]
fn novariant(tag: u8) -> u8 {
    tag & 0x0f
}

#[inline]
unsafe fn iswhite(object: *mut GCObject) -> bool {
    unsafe { (*object).marked & WHITEBITS != 0 }
}

#[inline]
unsafe fn isblack(object: *mut GCObject) -> bool {
    unsafe { (*object).marked & (1 << BLACKBIT) != 0 }
}

#[inline]
unsafe fn obj2gco<T>(object: *mut T) -> *mut GCObject {
    object.cast()
}

#[inline]
unsafe fn tsvalue(value: *const TValue) -> *mut TString {
    unsafe { (*value).value_.gc.cast() }
}

#[inline]
unsafe fn settt(value: *mut TValue, tag: u8) {
    unsafe { (*value).tt_ = tag };
}

#[inline]
unsafe fn setnilvalue(value: *mut TValue) {
    unsafe { settt(value, LUA_VNIL) };
}

#[inline]
unsafe fn setbfvalue(value: *mut TValue) {
    unsafe { settt(value, LUA_VFALSE) };
}

#[inline]
unsafe fn setbtvalue(value: *mut TValue) {
    unsafe { settt(value, LUA_VTRUE) };
}

#[inline]
unsafe fn setfltvalue(value: *mut TValue, number: lua_Number) {
    unsafe { (*value).value_.n = number };
    unsafe { settt(value, LUA_VNUMFLT) };
}

#[inline]
unsafe fn setivalue(value: *mut TValue, integer: lua_Integer) {
    unsafe { (*value).value_.i = integer };
    unsafe { settt(value, LUA_VNUMINT) };
}

#[inline]
unsafe fn setsvalue(value: *mut TValue, string: *mut TString) {
    unsafe { (*value).value_.gc = obj2gco(string) };
    unsafe { settt(value, ctb((*string).tt)) };
}

#[inline]
unsafe fn sethvalue(value: *mut TValue, table: *mut Table) {
    unsafe { (*value).value_.gc = obj2gco(table) };
    unsafe { settt(value, ctb(LUA_VTABLE)) };
}

#[inline]
unsafe fn setcllvalue(value: *mut TValue, closure: *mut LClosure) {
    unsafe { (*value).value_.gc = obj2gco(closure) };
    unsafe { settt(value, ctb(LUA_VLCL)) };
}

#[inline]
unsafe fn state_prefix(state: *mut lua_State) -> *mut LuaStatePrefix {
    state.cast()
}

#[inline]
unsafe fn stack_top_value(state: *mut lua_State) -> *mut TValue {
    unsafe { (*state_prefix(state)).top.p.cast::<TValue>() }
}

#[inline]
unsafe fn pop_stack(state: *mut lua_State, count: usize) {
    unsafe {
        (*state_prefix(state)).top.p = (*state_prefix(state)).top.p.sub(count);
    }
}

#[inline]
unsafe fn push_lclosure(state: *mut lua_State, closure: *mut LClosure) {
    let slot = unsafe { stack_top_value(state) };
    unsafe { setcllvalue(slot, closure) };
    unsafe { luaD_inctop(state) };
}

#[inline]
unsafe fn push_table(state: *mut lua_State, table: *mut Table) {
    let slot = unsafe { stack_top_value(state) };
    unsafe { sethvalue(slot, table) };
    unsafe { luaD_inctop(state) };
}

#[inline]
unsafe fn objbarrier<T, U>(state: *mut lua_State, parent: *mut T, child: *mut U) {
    let parent = unsafe { obj2gco(parent) };
    let child = unsafe { obj2gco(child) };
    if unsafe { isblack(parent) && iswhite(child) } {
        unsafe { luaC_barrier_(state, parent, child) };
    }
}

#[inline]
unsafe fn objbarrierback<T, U>(state: *mut lua_State, parent: *mut T, child: *mut U) {
    let parent = unsafe { obj2gco(parent) };
    let child = unsafe { obj2gco(child) };
    if unsafe { isblack(parent) && iswhite(child) } {
        unsafe { luaC_barrierback_(state, parent) };
    }
}

fn array_bytes<T>(state: *mut lua_State, count: usize) -> usize {
    count
        .checked_mul(size_of::<T>())
        .unwrap_or_else(|| unsafe { luaM_toobig(state) })
}

unsafe fn alloc_array<T>(state: *mut lua_State, count: usize) -> *mut T {
    if count == 0 {
        ptr::null_mut()
    } else {
        unsafe { luaM_malloc_(state, array_bytes::<T>(state, count), 0).cast() }
    }
}

fn error(state: &LoadState, why: *const c_char) -> ! {
    let _ = unsafe {
        luaO_pushfstring(
            state.l,
            c"%s: bad binary format (%s)".as_ptr(),
            state.name,
            why,
        )
    };
    unsafe { luaD_throw(state.l, LUA_ERRSYNTAX as u8) }
}

fn load_block(state: &mut LoadState, buffer: *mut c_void, size: usize) {
    if unsafe { luaZ_read(state.z, buffer, size) } != 0 {
        error(state, c"truncated chunk".as_ptr());
    }
    state.offset += size;
}

fn load_align(state: &mut LoadState, align: usize) {
    let padding = align - (state.offset % align);
    if padding < align {
        let mut scratch = [0u8; size_of::<lua_Integer>()];
        load_block(state, scratch.as_mut_ptr().cast(), padding);
    }
}

fn getaddr_(state: &mut LoadState, size: usize) -> *const c_void {
    let block = unsafe { luaZ_getaddr(state.z, size) };
    state.offset += size;
    if block.is_null() {
        error(state, c"truncated fixed buffer".as_ptr());
    }
    block
}

fn load_var<T: Copy>(state: &mut LoadState) -> T {
    let mut out = MaybeUninit::<T>::uninit();
    load_block(state, out.as_mut_ptr().cast(), size_of::<T>());
    unsafe { out.assume_init() }
}

fn load_byte(state: &mut LoadState) -> u8 {
    let z = unsafe { &mut *state.z };
    let byte = if z.n > 0 {
        z.n -= 1;
        let byte = unsafe { *z.p.cast::<u8>() };
        z.p = unsafe { z.p.add(1) };
        byte as c_int
    } else {
        unsafe { luaZ_fill(state.z) }
    };

    if byte == EOZ {
        error(state, c"truncated chunk".as_ptr());
    }
    state.offset += 1;
    byte as u8
}

fn load_varint(state: &mut LoadState, mut limit: lua_Unsigned) -> lua_Unsigned {
    let mut value = 0u64;
    limit >>= 7;
    loop {
        let byte = load_byte(state);
        if value > limit {
            error(state, c"integer overflow".as_ptr());
        }
        value = (value << 7) | u64::from(byte & 0x7f);
        if (byte & 0x80) == 0 {
            return value;
        }
    }
}

fn load_size(state: &mut LoadState) -> usize {
    load_varint(state, lua_Unsigned::MAX.min(MAX_SIZE as u64)) as usize
}

fn load_int(state: &mut LoadState) -> c_int {
    load_varint(state, i32::MAX as u64) as c_int
}

fn load_number(state: &mut LoadState) -> lua_Number {
    load_var(state)
}

fn load_integer(state: &mut LoadState) -> lua_Integer {
    let coded = load_varint(state, lua_Unsigned::MAX);
    if (coded & 1) != 0 {
        !(coded >> 1) as lua_Integer
    } else {
        (coded >> 1) as lua_Integer
    }
}

unsafe fn load_string(state: &mut LoadState, proto: *mut Proto, slot: *mut *mut TString) {
    let size = load_size(state);
    if size == 0 {
        let index = load_varint(state, lua_Unsigned::MAX);
        if index == 0 {
            return;
        }

        let mut saved = MaybeUninit::<TValue>::uninit();
        if novariant(unsafe { luaH_getint(state.h, index as lua_Integer, saved.as_mut_ptr()) })
            != LUA_TSTRING
        {
            error(state, c"invalid string index".as_ptr());
        }
        let string = unsafe { tsvalue(saved.as_ptr()) };
        unsafe { *slot = string };
        unsafe { objbarrier(state.l, proto, string) };
        return;
    }

    let size = size - 1;
    if size <= LUAI_MAXSHORTLEN {
        let mut buffer = [0u8; LUAI_MAXSHORTLEN + 1];
        load_block(state, buffer.as_mut_ptr().cast(), size + 1);
        let string = unsafe { luaS_newlstr(state.l, buffer.as_ptr().cast(), size) };
        unsafe { *slot = string };
        unsafe { objbarrier(state.l, proto, string) };
    } else if state.fixed {
        let contents = getaddr_(state, size + 1).cast::<c_char>();
        let string = unsafe { luaS_newextlstr(state.l, contents, size, None, ptr::null_mut()) };
        unsafe { *slot = string };
        unsafe { objbarrier(state.l, proto, string) };
    } else {
        let string = unsafe { luaS_createlngstrobj(state.l, size) };
        unsafe { *slot = string };
        unsafe { objbarrier(state.l, proto, string) };
        load_block(state, unsafe { (*string).contents.cast() }, size + 1);
    }

    state.nstr += 1;
    let mut saved = MaybeUninit::<TValue>::uninit();
    unsafe { setsvalue(saved.as_mut_ptr(), *slot) };
    unsafe {
        luaH_setint(
            state.l,
            state.h,
            state.nstr as lua_Integer,
            saved.as_mut_ptr(),
        )
    };
    unsafe { objbarrierback(state.l, state.h, *slot) };
}

unsafe fn load_code(state: &mut LoadState, proto: *mut Proto) {
    let count = load_int(state) as usize;
    load_align(state, size_of::<Instruction>());
    if state.fixed {
        unsafe {
            (*proto).code = getaddr_(state, array_bytes::<Instruction>(state.l, count))
                .cast_mut()
                .cast();
            (*proto).sizecode = count as c_int;
        }
    } else {
        let code = unsafe { alloc_array::<Instruction>(state.l, count) };
        unsafe {
            (*proto).code = code;
            (*proto).sizecode = count as c_int;
        }
        load_block(
            state,
            code.cast(),
            array_bytes::<Instruction>(state.l, count),
        );
    }
}

unsafe fn load_constants(state: &mut LoadState, proto: *mut Proto) {
    let count = load_int(state) as usize;
    let constants = unsafe { alloc_array::<TValue>(state.l, count) };
    unsafe {
        (*proto).k = constants;
        (*proto).sizek = count as c_int;
    }

    for index in 0..count {
        unsafe { setnilvalue(constants.add(index)) };
    }

    for index in 0..count {
        let value = unsafe { constants.add(index) };
        match load_byte(state) {
            LUA_VNIL => unsafe { setnilvalue(value) },
            LUA_VFALSE => unsafe { setbfvalue(value) },
            LUA_VTRUE => unsafe { setbtvalue(value) },
            LUA_VNUMFLT => unsafe { setfltvalue(value, load_number(state)) },
            LUA_VNUMINT => unsafe { setivalue(value, load_integer(state)) },
            LUA_VSHRSTR | LUA_VLNGSTR => unsafe {
                load_string(state, proto, &mut (*proto).source);
                if (*proto).source.is_null() {
                    error(state, c"bad format for constant string".as_ptr());
                }
                setsvalue(value, (*proto).source);
                (*proto).source = ptr::null_mut();
            },
            _ => error(state, c"invalid constant".as_ptr()),
        }
    }
}

unsafe fn load_protos(state: &mut LoadState, proto: *mut Proto) {
    let count = load_int(state) as usize;
    let protos = unsafe { alloc_array::<*mut Proto>(state.l, count) };
    unsafe {
        (*proto).p = protos;
        (*proto).sizep = count as c_int;
    }

    for index in 0..count {
        unsafe { *protos.add(index) = ptr::null_mut() };
    }

    for index in 0..count {
        let child = unsafe { luaF_newproto(state.l) };
        unsafe {
            *protos.add(index) = child;
            objbarrier(state.l, proto, child);
            load_function(state, child);
        }
    }
}

unsafe fn load_upvalues(state: &mut LoadState, proto: *mut Proto) {
    let count = load_int(state) as usize;
    let upvalues = unsafe { alloc_array::<Upvaldesc>(state.l, count) };
    unsafe {
        (*proto).upvalues = upvalues;
        (*proto).sizeupvalues = count as c_int;
    }

    for index in 0..count {
        unsafe { (*upvalues.add(index)).name = ptr::null_mut() };
    }

    for index in 0..count {
        unsafe {
            (*upvalues.add(index)).instack = load_byte(state);
            (*upvalues.add(index)).idx = load_byte(state);
            (*upvalues.add(index)).kind = load_byte(state);
        }
    }
}

unsafe fn load_debug(state: &mut LoadState, proto: *mut Proto) {
    let lineinfo_count = load_int(state) as usize;
    if state.fixed {
        unsafe {
            (*proto).lineinfo = getaddr_(state, array_bytes::<ls_byte>(state.l, lineinfo_count))
                .cast_mut()
                .cast();
            (*proto).sizelineinfo = lineinfo_count as c_int;
        }
    } else {
        let lineinfo = unsafe { alloc_array::<ls_byte>(state.l, lineinfo_count) };
        unsafe {
            (*proto).lineinfo = lineinfo;
            (*proto).sizelineinfo = lineinfo_count as c_int;
        }
        load_block(
            state,
            lineinfo.cast(),
            array_bytes::<ls_byte>(state.l, lineinfo_count),
        );
    }

    let absline_count = load_int(state) as usize;
    if absline_count > 0 {
        load_align(state, size_of::<c_int>());
        if state.fixed {
            unsafe {
                (*proto).abslineinfo =
                    getaddr_(state, array_bytes::<AbsLineInfo>(state.l, absline_count))
                        .cast_mut()
                        .cast();
                (*proto).sizeabslineinfo = absline_count as c_int;
            }
        } else {
            let abslineinfo = unsafe { alloc_array::<AbsLineInfo>(state.l, absline_count) };
            unsafe {
                (*proto).abslineinfo = abslineinfo;
                (*proto).sizeabslineinfo = absline_count as c_int;
            }
            load_block(
                state,
                abslineinfo.cast(),
                array_bytes::<AbsLineInfo>(state.l, absline_count),
            );
        }
    }

    let locvar_count = load_int(state) as usize;
    let locvars = unsafe { alloc_array::<LocVar>(state.l, locvar_count) };
    unsafe {
        (*proto).locvars = locvars;
        (*proto).sizelocvars = locvar_count as c_int;
    }
    for index in 0..locvar_count {
        unsafe { (*locvars.add(index)).varname = ptr::null_mut() };
    }
    for index in 0..locvar_count {
        let locvar = unsafe { locvars.add(index) };
        unsafe {
            load_string(state, proto, &mut (*locvar).varname);
            (*locvar).startpc = load_int(state);
            (*locvar).endpc = load_int(state);
        }
    }

    let mut upvalue_names = load_int(state) as usize;
    if upvalue_names != 0 {
        upvalue_names = unsafe { (*proto).sizeupvalues as usize };
    }
    for index in 0..upvalue_names {
        unsafe { load_string(state, proto, &mut (*(*proto).upvalues.add(index)).name) };
    }
}

unsafe fn load_function(state: &mut LoadState, proto: *mut Proto) {
    unsafe {
        (*proto).linedefined = load_int(state);
        (*proto).lastlinedefined = load_int(state);
        (*proto).numparams = load_byte(state);
        (*proto).flag = load_byte(state) & !PF_FIXED;
        if state.fixed {
            (*proto).flag |= PF_FIXED;
        }
        (*proto).maxstacksize = load_byte(state);
    }
    unsafe { load_code(state, proto) };
    unsafe { load_constants(state, proto) };
    unsafe { load_upvalues(state, proto) };
    unsafe { load_protos(state, proto) };
    unsafe { load_string(state, proto, &mut (*proto).source) };
    unsafe { load_debug(state, proto) };
}

fn checkliteral(state: &mut LoadState, expected: &[u8], message: *const c_char) {
    let mut actual = vec![0u8; expected.len()];
    load_block(state, actual.as_mut_ptr().cast(), actual.len());
    if actual != expected {
        error(state, message);
    }
}

fn numerror(state: &LoadState, what: *const c_char, tname: *const c_char) -> ! {
    let msg = unsafe { luaO_pushfstring(state.l, c"%s %s mismatch".as_ptr(), tname, what) };
    error(state, msg)
}

fn checknumsize(state: &mut LoadState, size: usize, tname: *const c_char) {
    if usize::from(load_byte(state)) != size {
        numerror(state, c"size".as_ptr(), tname);
    }
}

fn checknum<T: Copy + PartialEq>(state: &mut LoadState, expected: T, tname: *const c_char) {
    checknumsize(state, size_of::<T>(), tname);
    if load_var::<T>(state) != expected {
        numerror(state, c"format".as_ptr(), tname);
    }
}

fn check_header(state: &mut LoadState) {
    checkliteral(state, &LUA_SIGNATURE[1..], c"not a binary chunk".as_ptr());
    if load_byte(state) != LUAC_VERSION {
        error(state, c"version mismatch".as_ptr());
    }
    if load_byte(state) != LUAC_FORMAT {
        error(state, c"format mismatch".as_ptr());
    }
    checkliteral(state, LUAC_DATA, c"corrupted chunk".as_ptr());
    checknum(state, LUAC_INT, c"int".as_ptr());
    checknum(state, LUAC_INST, c"instruction".as_ptr());
    checknum(state, LUAC_INT as lua_Integer, c"Lua integer".as_ptr());
    checknum(state, LUAC_NUM, c"Lua number".as_ptr());
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaU_undump(
    state: *mut lua_State,
    z: *mut ZIO,
    mut name: *const c_char,
    fixed: c_int,
) -> *mut LClosure {
    let first = unsafe { *name.cast::<u8>() };
    if first == b'@' || first == b'=' {
        name = unsafe { name.add(1) };
    } else if first == LUA_SIGNATURE[0] {
        name = c"binary string".as_ptr();
    }

    let mut load_state = LoadState {
        l: state,
        z,
        name,
        h: ptr::null_mut(),
        offset: 1,
        nstr: 0,
        fixed: fixed != 0,
    };

    check_header(&mut load_state);
    let closure = unsafe { luaF_newLclosure(state, load_byte(&mut load_state) as c_int) };
    unsafe { push_lclosure(state, closure) };

    load_state.h = unsafe { luaH_new(state) };
    unsafe { push_table(state, load_state.h) };

    unsafe {
        (*closure).p = luaF_newproto(state);
        objbarrier(state, closure, (*closure).p);
        load_function(&mut load_state, (*closure).p);
    }

    if unsafe { usize::from((*closure).nupvalues) != (*(*closure).p).sizeupvalues as usize } {
        error(&load_state, c"corrupted chunk".as_ptr());
    }

    unsafe { pop_stack(state, 1) };
    closure
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn dumped_chunk_round_trip() {
        run_lua_test(
            "test/string_builtin.lua",
            include_str!("../test/string_builtin.lua"),
        );
    }
}
