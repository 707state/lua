use crate::do_rs::luaD_inctop;
use crate::runtime::*;
use crate::table::{raw_luaH_getstr, raw_luaH_new, raw_luaH_set};
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

#[repr(C)]
struct AbsLineInfo {
    pc: c_int,
    line: c_int,
}

struct DumpState {
    l: *mut lua_State,
    writer: lua_Writer,
    data: *mut c_void,
    offset: usize,
    strip: bool,
    status: c_int,
    h: *mut Table,
    nstr: lua_Unsigned,
}


#[inline]
fn ctb(tag: u8) -> u8 {
    tag | BIT_ISCOLLECTABLE
}

#[inline]
unsafe fn push_table(state: *mut lua_State, table: *mut Table) {
    let slot = unsafe { s2v((*state).top.p) };
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
    let tag = unsafe {
        raw_luaH_getstr(
            state.h.cast(),
            string.cast(),
            (&mut idx as *mut TValue).cast(),
        )
    };
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
        setsvalue(&mut key, string);
        setivalue(&mut value, state.nstr as lua_Integer);
        raw_luaH_set(
            state.l.cast(),
            state.h.cast(),
            (&mut key as *mut TValue).cast(),
            (&mut value as *mut TValue).cast(),
        );
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

pub(crate) unsafe fn luaU_dump(
    state: *mut lua_State,
    proto: *const Proto,
    writer: lua_Writer,
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
        h: unsafe { raw_luaH_new(state.cast()).cast::<Table>() },
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
