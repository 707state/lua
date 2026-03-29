use crate::lua_module::{
    argcheck, create_library, lua_Integer, lua_State, lua_Unsigned, lua_gettop, lua_pop,
    lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushstring, lua_pushvalue, lua_setfield,
    lua_settop, luaL_Reg,
};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

const LUA_TNIL: c_int = 0;
const LUA_TTABLE: c_int = 5;
const LUA_TFUNCTION: c_int = 6;
const LUA_OPEQ: c_int = 0;
const LUA_OPLT: c_int = 1;

const TAB_R: c_int = 1;
const TAB_W: c_int = 2;
const TAB_L: c_int = 4;
const TAB_RW: c_int = TAB_R | TAB_W;

const RANLIMIT: u32 = 100;

const NAME_CONCAT: &[u8] = b"concat\0";
const NAME_CREATE: &[u8] = b"create\0";
const NAME_GETN: &[u8] = b"getn\0";
const NAME_INSERT: &[u8] = b"insert\0";
const NAME_MOVE: &[u8] = b"move\0";
const NAME_N: &[u8] = b"n\0";
const NAME_PACK: &[u8] = b"pack\0";
const NAME_REMOVE: &[u8] = b"remove\0";
const NAME_SORT: &[u8] = b"sort\0";
const NAME_UNPACK: &[u8] = b"unpack\0";

const FIELD_INDEX: &[u8] = b"__index\0";
const FIELD_NEWINDEX: &[u8] = b"__newindex\0";
const FIELD_LEN: &[u8] = b"__len\0";

const ERR_OUT_OF_RANGE: &[u8] = b"out of range\0";
const ERR_POSITION_OUT_OF_BOUNDS: &[u8] = b"position out of bounds\0";
const ERR_WRONG_INSERT_ARGS: &[u8] = b"wrong number of arguments to 'insert'\0";
const ERR_TOO_MANY_ELEMENTS_TO_MOVE: &[u8] = b"too many elements to move\0";
const ERR_DEST_WRAP_AROUND: &[u8] = b"destination wrap around\0";
const ERR_TOO_MANY_RESULTS_TO_UNPACK: &[u8] = b"too many results to unpack\0";
const ERR_INVALID_ORDER_FUNCTION: &[u8] = b"invalid order function for sorting\0";
const ERR_ARRAY_TOO_BIG: &[u8] = b"array too big\0";
const ERR_INVALID_CONCAT_VALUE: &[u8] = b"invalid value in table for 'concat'\0";

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

unsafe extern "C" {
    fn luaL_checkinteger(state: *mut lua_State, arg: c_int) -> lua_Integer;
    fn luaL_optinteger(state: *mut lua_State, arg: c_int, def: lua_Integer) -> lua_Integer;
    fn luaL_optlstring(
        state: *mut lua_State,
        arg: c_int,
        default: *const c_char,
        len: *mut usize,
    ) -> *const c_char;
    fn luaL_checktype(state: *mut lua_State, arg: c_int, tag: c_int);
    fn luaL_len(state: *mut lua_State, index: c_int) -> lua_Integer;
    fn luaL_makeseed(state: *mut lua_State) -> c_uint;

    fn lua_callk(
        state: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        context: isize,
        k: Option<unsafe extern "C" fn(*mut lua_State, c_int, isize) -> c_int>,
    );

    fn lua_checkstack(state: *mut lua_State, n: c_int) -> c_int;
    fn lua_compare(state: *mut lua_State, idx1: c_int, idx2: c_int, op: c_int) -> c_int;
    fn lua_geti(state: *mut lua_State, index: c_int, n: lua_Integer) -> c_int;
    fn lua_getmetatable(state: *mut lua_State, objindex: c_int) -> c_int;
    fn lua_isstring(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_rawget(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_seti(state: *mut lua_State, index: c_int, n: lua_Integer);
    fn lua_toboolean(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_tolstring(state: *mut lua_State, index: c_int, len: *mut usize) -> *const c_char;
    fn lua_type(state: *mut lua_State, index: c_int) -> c_int;
}

#[inline]
unsafe fn lua_call(state: *mut lua_State, nargs: c_int, nresults: c_int) {
    unsafe { lua_callk(state, nargs, nresults, 0, None) };
}

#[inline]
fn is_none_or_nil(state: *mut lua_State, index: c_int) -> bool {
    unsafe { lua_type(state, index) <= LUA_TNIL }
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
    unsafe { lua_rawget(state, -n) != LUA_TNIL }
}

unsafe fn checktab(state: *mut lua_State, arg: c_int, what: c_int) {
    if unsafe { lua_type(state, arg) } != LUA_TTABLE {
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
            unsafe { luaL_checktype(state, arg, LUA_TTABLE) };
        }
    }
}

#[inline]
unsafe fn aux_getn(state: *mut lua_State, n: c_int, what: c_int) -> lua_Integer {
    unsafe { checktab(state, n, what | TAB_L) };
    unsafe { luaL_len(state, n) }
}

unsafe extern "C" fn tcreate(state: *mut lua_State) -> c_int {
    let sizeseq = unsafe { luaL_checkinteger(state, 1) } as lua_Unsigned;
    let sizerest = unsafe { luaL_optinteger(state, 2, 0) } as lua_Unsigned;
    unsafe { argcheck(state, sizeseq <= i32::MAX as lua_Unsigned, 1, ERR_OUT_OF_RANGE) };
    unsafe { argcheck(state, sizerest <= i32::MAX as lua_Unsigned, 2, ERR_OUT_OF_RANGE) };
    unsafe { crate::lua_module::lua_createtable(state, sizeseq as c_int, sizerest as c_int) };
    1
}

unsafe extern "C" fn tinsert(state: *mut lua_State) -> c_int {
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
            pos = unsafe { luaL_checkinteger(state, 2) };
            unsafe {
                argcheck(
                    state,
                    pos >= 1 && pos <= e,
                    2,
                    ERR_POSITION_OUT_OF_BOUNDS,
                )
            };
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

unsafe extern "C" fn tremove(state: *mut lua_State) -> c_int {
    let size = unsafe { aux_getn(state, 1, TAB_RW) };
    let mut pos = unsafe { luaL_optinteger(state, 2, size) };
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

unsafe extern "C" fn tmove(state: *mut lua_State) -> c_int {
    let f = unsafe { luaL_checkinteger(state, 2) };
    let e = unsafe { luaL_checkinteger(state, 3) };
    let t = unsafe { luaL_checkinteger(state, 4) };
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

unsafe extern "C" fn tconcat(state: *mut lua_State) -> c_int {
    let mut last = unsafe { aux_getn(state, 1, TAB_R) };
    let mut lsep = 0usize;
    let sep = unsafe { luaL_optlstring(state, 2, b"\0".as_ptr().cast(), &mut lsep) };
    let mut i = unsafe { luaL_optinteger(state, 3, 1) };
    last = unsafe { luaL_optinteger(state, 4, last) };
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

unsafe extern "C" fn tpack(state: *mut lua_State) -> c_int {
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

unsafe extern "C" fn tunpack(state: *mut lua_State) -> c_int {
    let i = unsafe { luaL_optinteger(state, 2, 1) };
    let e = if is_none_or_nil(state, 3) {
        unsafe { luaL_len(state, 1) }
    } else {
        unsafe { luaL_checkinteger(state, 3) }
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

unsafe fn auxsort(state: *mut lua_State, mut lo: IdxT, mut up: IdxT, mut rnd: u32) -> Result<(), c_int> {
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
            rnd = unsafe { luaL_makeseed(state) };
        }
    }
    Ok(())
}

unsafe extern "C" fn sort(state: *mut lua_State) -> c_int {
    let n = unsafe { aux_getn(state, 1, TAB_RW) };
    if n > 1 {
        unsafe { argcheck(state, n < i32::MAX as lua_Integer, 1, ERR_ARRAY_TOO_BIG) };
        if !is_none_or_nil(state, 2) {
            unsafe { luaL_checktype(state, 2, LUA_TFUNCTION) };
        }
        unsafe { lua_settop(state, 2) };
        if let Err(code) = unsafe { auxsort(state, 1, n as IdxT, 0) } {
            return code;
        }
    }
    0
}

unsafe extern "C" fn getn(state: *mut lua_State) -> c_int {
    let n = unsafe { aux_getn(state, 1, TAB_R) };
    unsafe { lua_pushinteger(state, n) };
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_table(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &TAB_FUNCS) };
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::luaffi::{
        LUA_OK, LUAL_NUMSIZES, LUA_VERSION_NUM, luaL_checkversion_, luaL_loadbufferx,
        luaL_newstate, luaL_openselectedlibs, lua_close, lua_pcall, lua_tolstring,
    };
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
            if status != LUA_OK {
                return Err(lua_error_string(state));
            }
            let status = lua_pcall(state, 0, 0, 0);
            if status != LUA_OK {
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
        let state = unsafe { luaL_newstate() };
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
}
