use crate::func::{raw_luaF_newLclosure, raw_luaF_newproto};
use crate::runtime::*;
use crate::string::raw_luaS_newlstr;
use crate::table::{raw_luaH_getint, raw_luaH_new, raw_luaH_setint};
use crate::zio::{EOZ, luaZ_fill, luaZ_getaddr, luaZ_read};
use core::ffi::{c_char, c_int, c_void};
use core::mem::{MaybeUninit, size_of};
use core::ptr;

struct LoadState {
    l: *mut lua_State,
    z: *mut ZIO,
    name: *const c_char,
    h: *mut Table,
    offset: usize,
    nstr: lua_Unsigned,
    fixed: bool,
}

// luaO_pushfstring: 变参，直接用 crate::object::luaO_pushfstring
#[inline]
unsafe fn luaD_throw(s: *mut lua_State, e: u8) -> ! {
    unsafe { crate::do_rs::luaD_throw(s, e) }
}
#[inline]
unsafe fn luaD_inctop(s: *mut lua_State) {
    unsafe { crate::do_rs::luaD_inctop(s) }
}
#[inline]
unsafe fn luaS_newextlstr(
    s: *mut lua_State,
    c: *const c_char,
    l: usize,
    fa: lua_Alloc,
    u: *mut c_void,
) -> *mut TString {
    unsafe { crate::string::luaS_newextlstr(s, c, l, fa, u) }
}
#[inline]
unsafe fn luaS_createlngstrobj(s: *mut lua_State, l: usize) -> *mut TString {
    unsafe { crate::string::luaS_createlngstrobj(s, l) }
}
#[inline]
unsafe fn luaC_barrier_(s: *mut lua_State, o: *mut GCObject, v: *mut GCObject) {
    unsafe { crate::gc::luaC_barrier_(s, o, v) }
}
#[inline]
unsafe fn luaC_barrierback_(s: *mut lua_State, o: *mut GCObject) {
    unsafe { crate::gc::luaC_barrierback_(s, o) }
}
#[inline]
unsafe fn luaM_malloc_(s: *mut lua_State, sz: usize, t: c_int) -> *mut c_void {
    unsafe { crate::mem::luaM_malloc_(s, sz, t) }
}
#[inline]
unsafe fn luaM_toobig(s: *mut lua_State) -> ! {
    unsafe { crate::mem::luaM_toobig(s) }
}

#[inline]
fn ctb(tag: u8) -> u8 {
    tag | BIT_ISCOLLECTABLE
}

#[inline]
unsafe fn setcllvalue(value: *mut TValue, closure: *mut LClosure) {
    unsafe { (*value).value_.gc = obj2gco(closure) };
    unsafe { settt(value, ctb(LUA_VLCL)) };
}

#[inline]
unsafe fn pop_stack(state: *mut lua_State, count: usize) {
    unsafe {
        (*state).top.p = (*state).top.p.sub(count);
    }
}

#[inline]
unsafe fn push_lclosure(state: *mut lua_State, closure: *mut LClosure) {
    let slot = unsafe { s2v((*state).top.p) };
    unsafe { setcllvalue(slot, closure) };
    unsafe { luaD_inctop(state) };
}

#[inline]
unsafe fn push_table(state: *mut lua_State, table: *mut Table) {
    let slot = unsafe { s2v((*state).top.p) };
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
    let name_s = unsafe { std::ffi::CStr::from_ptr(state.name) }.to_string_lossy();
    let why_s = unsafe { std::ffi::CStr::from_ptr(why) }.to_string_lossy();
    unsafe {
        crate::object::luaO_pushstr(state.l, &format!("{name_s}: bad binary format ({why_s})"))
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
    unsafe {
        let size = load_size(state);
        if size == 0 {
            let index = load_varint(state, lua_Unsigned::MAX);
            if index == 0 {
                return;
            }

            let mut saved = MaybeUninit::<TValue>::uninit();
            if novariant(raw_luaH_getint(
                state.h.cast(),
                index as lua_Integer,
                saved.as_mut_ptr().cast(),
            )) != LUA_TSTRING
            {
                error(state, c"invalid string index".as_ptr());
            }
            let string = tsvalue(saved.as_ptr());
            *slot = string;
            objbarrier(state.l, proto, string);
            return;
        }

        let size = size - 1;
        if size <= LUAI_MAXSHORTLEN {
            let mut buffer = [0u8; LUAI_MAXSHORTLEN + 1];
            load_block(state, buffer.as_mut_ptr().cast(), size + 1);
            let string =
                raw_luaS_newlstr(state.l.cast(), buffer.as_ptr().cast(), size).cast::<TString>();
            *slot = string;
            objbarrier(state.l, proto, string);
        } else if state.fixed {
            let contents = getaddr_(state, size + 1).cast::<c_char>();
            let string = luaS_newextlstr(state.l, contents, size, None, ptr::null_mut());
            *slot = string;
            objbarrier(state.l, proto, string);
        } else {
            let string = luaS_createlngstrobj(state.l, size);
            *slot = string;
            objbarrier(state.l, proto, string);
            load_block(state, (*string).contents.cast(), size + 1);
        }

        state.nstr += 1;
        let mut saved = MaybeUninit::<TValue>::uninit();
        setsvalue(saved.as_mut_ptr(), *slot);
        raw_luaH_setint(
            state.l.cast(),
            state.h.cast(),
            state.nstr as lua_Integer,
            saved.as_mut_ptr().cast(),
        );
        objbarrierback(state.l, state.h, *slot);
    }
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
        let child = unsafe { raw_luaF_newproto(state.l.cast()).cast::<Proto>() };
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
    let tname_s = unsafe { std::ffi::CStr::from_ptr(tname) }.to_string_lossy();
    let what_s = unsafe { std::ffi::CStr::from_ptr(what) }.to_string_lossy();
    let name_s = unsafe { std::ffi::CStr::from_ptr(state.name) }.to_string_lossy();
    unsafe {
        crate::object::luaO_pushstr(
            state.l,
            &format!("{name_s}: bad binary format ({tname_s} {what_s} mismatch)"),
        )
    };
    unsafe { luaD_throw(state.l, LUA_ERRSYNTAX as u8) }
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

pub unsafe fn luaU_undump(
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
    let closure = unsafe {
        raw_luaF_newLclosure(state.cast(), load_byte(&mut load_state) as c_int).cast::<LClosure>()
    };
    unsafe { push_lclosure(state, closure) };

    load_state.h = unsafe { raw_luaH_new(state.cast()).cast::<Table>() };
    unsafe { push_table(state, load_state.h) };

    unsafe {
        (*closure).p = raw_luaF_newproto(state.cast()).cast::<Proto>();
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
