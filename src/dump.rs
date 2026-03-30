use crate::lua_module::{lua_Integer, lua_Number, lua_State, lua_Unsigned};
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

const LUA_SIGNATURE: &[u8] = b"\x1bLua";
const LUAC_DATA: &[u8] = b"\x19\x93\r\n\x1a\n";
const LUAC_VERSION: u8 = 0x55;
const LUAC_FORMAT: u8 = 0;
const LUAC_INT: c_int = -0x5678;
const LUAC_INST: Instruction = 0x1234_5678;
const LUAC_NUM: lua_Number = -370.5;

const BIT_ISCOLLECTABLE: u8 = 1 << 6;
const LUA_VNIL: u8 = 0;
const LUA_VFALSE: u8 = 1;
const LUA_VTRUE: u8 = 17;
const LUA_VNUMINT: u8 = 3;
const LUA_VNUMFLT: u8 = 19;
const LUA_VSHRSTR: u8 = 68;
const LUA_VLNGSTR: u8 = 84;
const LUA_VTABLE: u8 = 69;

type Instruction = u32;
type LuaWriter =
    Option<unsafe extern "C" fn(*mut lua_State, *const c_void, usize, *mut c_void) -> c_int>;

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
union TStringUnion {
    lnglen: usize,
    hnext: *mut TString,
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
    falloc: *mut c_void,
    ud: *mut c_void,
}

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

#[allow(non_camel_case_types)]
type ls_byte = i8;

#[repr(C)]
pub struct Proto {
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
    k: *const TValue,
    code: *const Instruction,
    p: *const *const Proto,
    upvalues: *const Upvaldesc,
    lineinfo: *const ls_byte,
    abslineinfo: *const AbsLineInfo,
    locvars: *const LocVar,
    source: *mut TString,
    gclist: *mut GCObject,
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

struct DumpState {
    l: *mut lua_State,
    writer: LuaWriter,
    data: *mut c_void,
    offset: usize,
    strip: bool,
    status: c_int,
    h: *mut Table,
    nstr: lua_Unsigned,
}

unsafe extern "C" {
    fn luaD_inctop(state: *mut lua_State);
    fn luaH_new(state: *mut lua_State) -> *mut Table;
    fn luaH_getstr(table: *mut Table, key: *mut TString, result: *mut TValue) -> u8;
    fn luaH_set(state: *mut lua_State, table: *mut Table, key: *mut TValue, value: *mut TValue);
}

#[inline]
fn ctb(tag: u8) -> u8 {
    tag | BIT_ISCOLLECTABLE
}

#[inline]
fn tagisempty(tag: u8) -> bool {
    (tag & 0x0f) == 0
}

#[inline]
unsafe fn ivalue(value: *const TValue) -> lua_Integer {
    unsafe { (*value).value_.i }
}

#[inline]
unsafe fn fltvalue(value: *const TValue) -> lua_Number {
    unsafe { (*value).value_.n }
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
unsafe fn setivalue(value: *mut TValue, integer: lua_Integer) {
    unsafe { (*value).value_.i = integer };
    unsafe { settt(value, LUA_VNUMINT) };
}

#[inline]
unsafe fn setsvalue(state: *mut lua_State, value: *mut TValue, string: *mut TString) {
    let _ = state;
    unsafe { (*value).value_.gc = string.cast() };
    unsafe { settt(value, ctb((*string).tt)) };
}

#[inline]
unsafe fn sethvalue(value: *mut TValue, table: *mut Table) {
    unsafe { (*value).value_.gc = table.cast() };
    unsafe { settt(value, ctb(LUA_VTABLE)) };
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
unsafe fn push_table(state: *mut lua_State, table: *mut Table) {
    let slot = unsafe { stack_top_value(state) };
    unsafe { sethvalue(slot, table) };
    unsafe { luaD_inctop(state) };
}

#[inline]
unsafe fn str_len(string: *mut TString) -> usize {
    if unsafe { (*string).shrlen } >= 0 {
        unsafe { (*string).shrlen as usize }
    } else {
        unsafe { (*string).u.lnglen }
    }
}

#[inline]
unsafe fn str_data(string: *mut TString) -> *const c_char {
    if unsafe { (*string).shrlen } >= 0 {
        unsafe { ptr::addr_of!((*string).contents).cast() }
    } else {
        unsafe { (*string).contents }
    }
}

fn dump_block(state: &mut DumpState, block: *const c_void, size: usize) {
    if state.status == 0 {
        let writer = state.writer.expect("luaU_dump writer must be present");
        state.status = unsafe { writer(state.l, block, size, state.data) };
        state.offset += size;
    }
}

fn dump_align(state: &mut DumpState, align: usize) {
    let padding = align - (state.offset % align);
    if padding < align {
        let zero: lua_Integer = 0;
        dump_block(state, (&zero as *const lua_Integer).cast(), padding);
    }
}

fn dump_var<T>(state: &mut DumpState, value: &T) {
    dump_block(state, (value as *const T).cast(), size_of::<T>());
}

fn dump_bytes(state: &mut DumpState, bytes: &[u8]) {
    dump_block(state, bytes.as_ptr().cast(), bytes.len());
}

fn dump_byte(state: &mut DumpState, value: c_int) {
    dump_var(state, &(value as u8));
}

fn dump_varint(state: &mut DumpState, mut value: lua_Unsigned) {
    let mut buffer = [0u8; 10];
    let mut n = 1usize;
    buffer[buffer.len() - 1] = (value & 0x7f) as u8;
    while {
        value >>= 7;
        value != 0
    } {
        n += 1;
        buffer[buffer.len() - n] = ((value & 0x7f) as u8) | 0x80;
    }
    dump_bytes(state, &buffer[buffer.len() - n..]);
}

fn dump_size(state: &mut DumpState, size: usize) {
    dump_varint(state, size as lua_Unsigned);
}

fn dump_int(state: &mut DumpState, value: c_int) {
    debug_assert!(value >= 0);
    dump_varint(state, value as u32 as lua_Unsigned);
}

fn dump_number(state: &mut DumpState, value: lua_Number) {
    dump_var(state, &value);
}

fn dump_integer(state: &mut DumpState, value: lua_Integer) {
    let encoded = if value >= 0 {
        2u64 * value as u64
    } else {
        (2u64 * !(value as u64)).wrapping_add(1)
    };
    dump_varint(state, encoded);
}

unsafe fn dump_string(state: &mut DumpState, string: *mut TString) {
    if string.is_null() {
        dump_varint(state, 0);
        dump_varint(state, 0);
        return;
    }

    let mut idx = TValue {
        value_: Value { ub: 0 },
        tt_: 0,
    };
    let tag = unsafe { luaH_getstr(state.h, string, &mut idx) };
    if !tagisempty(tag) {
        dump_varint(state, 0);
        dump_varint(state, unsafe { ivalue(&idx) as lua_Unsigned });
        return;
    }

    let size = unsafe { str_len(string) };
    let data = unsafe { str_data(string) };
    dump_size(state, size + 1);
    dump_block(state, data.cast(), size + 1);

    state.nstr += 1;
    let mut key = TValue {
        value_: Value { ub: 0 },
        tt_: 0,
    };
    let mut value = TValue {
        value_: Value { ub: 0 },
        tt_: 0,
    };
    unsafe {
        setsvalue(state.l, &mut key, string);
        setivalue(&mut value, state.nstr as lua_Integer);
        luaH_set(state.l, state.h, &mut key, &mut value);
    }
}

unsafe fn dump_code(state: &mut DumpState, proto: *const Proto) {
    let sizecode = unsafe { (*proto).sizecode };
    dump_int(state, sizecode);
    dump_align(state, size_of::<Instruction>());
    if sizecode > 0 {
        dump_block(
            state,
            unsafe { (*proto).code.cast() },
            sizecode as usize * size_of::<Instruction>(),
        );
    }
}

unsafe fn dump_constants(state: &mut DumpState, proto: *const Proto) {
    let count = unsafe { (*proto).sizek };
    dump_int(state, count);
    for index in 0..count as usize {
        let value = unsafe { (*proto).k.add(index) };
        let tag = unsafe { (*value).tt_ & 0x3f };
        dump_byte(state, tag as c_int);
        match tag {
            LUA_VNUMFLT => dump_number(state, unsafe { fltvalue(value) }),
            LUA_VNUMINT => dump_integer(state, unsafe { ivalue(value) }),
            LUA_VSHRSTR | LUA_VLNGSTR => unsafe { dump_string(state, tsvalue(value)) },
            _ => debug_assert!(matches!(tag, LUA_VNIL | LUA_VFALSE | LUA_VTRUE)),
        }
    }
}

unsafe fn dump_protos(state: &mut DumpState, proto: *const Proto) {
    let count = unsafe { (*proto).sizep };
    dump_int(state, count);
    for index in 0..count as usize {
        unsafe { dump_function(state, *(*proto).p.add(index)) };
    }
}

unsafe fn dump_upvalues(state: &mut DumpState, proto: *const Proto) {
    let count = unsafe { (*proto).sizeupvalues };
    dump_int(state, count);
    for index in 0..count as usize {
        let upvalue = unsafe { (*proto).upvalues.add(index) };
        dump_byte(state, unsafe { (*upvalue).instack as c_int });
        dump_byte(state, unsafe { (*upvalue).idx as c_int });
        dump_byte(state, unsafe { (*upvalue).kind as c_int });
    }
}

unsafe fn dump_debug(state: &mut DumpState, proto: *const Proto) {
    let lineinfo_count = if state.strip {
        0
    } else {
        unsafe { (*proto).sizelineinfo }
    };
    dump_int(state, lineinfo_count);
    if lineinfo_count > 0 && unsafe { !(*proto).lineinfo.is_null() } {
        dump_block(
            state,
            unsafe { (*proto).lineinfo.cast() },
            lineinfo_count as usize * size_of::<ls_byte>(),
        );
    }

    let absline_count = if state.strip {
        0
    } else {
        unsafe { (*proto).sizeabslineinfo }
    };
    dump_int(state, absline_count);
    if absline_count > 0 {
        dump_align(state, size_of::<c_int>());
        dump_block(
            state,
            unsafe { (*proto).abslineinfo.cast() },
            absline_count as usize * size_of::<AbsLineInfo>(),
        );
    }

    let locvar_count = if state.strip {
        0
    } else {
        unsafe { (*proto).sizelocvars }
    };
    dump_int(state, locvar_count);
    for index in 0..locvar_count as usize {
        let locvar = unsafe { (*proto).locvars.add(index) };
        unsafe { dump_string(state, (*locvar).varname) };
        dump_int(state, unsafe { (*locvar).startpc });
        dump_int(state, unsafe { (*locvar).endpc });
    }

    let upvalue_name_count = if state.strip {
        0
    } else {
        unsafe { (*proto).sizeupvalues }
    };
    dump_int(state, upvalue_name_count);
    for index in 0..upvalue_name_count as usize {
        unsafe { dump_string(state, (*(*proto).upvalues.add(index)).name) };
    }
}

unsafe fn dump_function(state: &mut DumpState, proto: *const Proto) {
    dump_int(state, unsafe { (*proto).linedefined });
    dump_int(state, unsafe { (*proto).lastlinedefined });
    dump_byte(state, unsafe { (*proto).numparams as c_int });
    dump_byte(state, unsafe { (*proto).flag as c_int });
    dump_byte(state, unsafe { (*proto).maxstacksize as c_int });
    unsafe { dump_code(state, proto) };
    unsafe { dump_constants(state, proto) };
    unsafe { dump_upvalues(state, proto) };
    unsafe { dump_protos(state, proto) };
    unsafe {
        dump_string(
            state,
            if state.strip {
                ptr::null_mut()
            } else {
                (*proto).source
            },
        )
    };
    unsafe { dump_debug(state, proto) };
}

fn dump_header(state: &mut DumpState) {
    dump_bytes(state, LUA_SIGNATURE);
    dump_byte(state, LUAC_VERSION as c_int);
    dump_byte(state, LUAC_FORMAT as c_int);
    dump_bytes(state, LUAC_DATA);

    dump_byte(state, size_of::<c_int>() as c_int);
    dump_var(state, &LUAC_INT);

    dump_byte(state, size_of::<Instruction>() as c_int);
    dump_var(state, &LUAC_INST);

    let luac_int = LUAC_INT as lua_Integer;
    dump_byte(state, size_of::<lua_Integer>() as c_int);
    dump_var(state, &luac_int);

    dump_byte(state, size_of::<lua_Number>() as c_int);
    dump_var(state, &LUAC_NUM);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaU_dump(
    state: *mut lua_State,
    proto: *const Proto,
    writer: LuaWriter,
    data: *mut c_void,
    strip: c_int,
) -> c_int {
    let mut dump = DumpState {
        l: state,
        writer,
        data,
        offset: 0,
        strip: strip != 0,
        status: 0,
        h: unsafe { luaH_new(state) },
        nstr: 0,
    };

    unsafe { push_table(state, dump.h) };
    dump_header(&mut dump);
    dump_byte(&mut dump, unsafe { (*proto).sizeupvalues });
    unsafe { dump_function(&mut dump, proto) };
    dump_block(&mut dump, ptr::null(), 0);
    dump.status
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn dump_round_trip_matches_builtin_paths() {
        run_lua_test(
            "test/string_builtin.lua",
            include_str!("../test/string_builtin.lua"),
        );
    }
}
