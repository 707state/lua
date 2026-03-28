use std::ffi::{c_char, c_int, c_uint};
use std::mem::size_of;
use std::ptr;
use std::simd::Simd;
use std::simd::num::SimdFloat;
#[allow(non_camel_case_types)]
type lua_Integer = i64;
#[allow(non_camel_case_types)]
type lua_Number = f64;
#[allow(non_camel_case_types)]
type lua_Unsigned = u64;

#[repr(C)]
pub struct lua_State {
    _private: [u8; 0],
}

#[repr(C)]
struct luaL_Reg {
    name: *const c_char,
    func: LuaCFunction,
}

unsafe impl Sync for luaL_Reg {}

#[repr(C)]
struct RanState {
    s: [u64; 4],
}

type LuaCFunction = Option<unsafe extern "C" fn(*mut lua_State) -> c_int>;

const LUA_VERSION_NUM: lua_Number = 505.0;
const LUAL_NUMSIZES: usize = size_of::<lua_Integer>() * 16 + size_of::<lua_Number>();
const LUA_TNUMBER: c_int = 3;
const LUA_TTABLE: c_int = 5;
const LUA_OPLT: c_int = 1;
const LUA_REGISTRYINDEX: c_int = -(i32::MAX / 2 + 1000);
const LUA_MININTEGER: lua_Integer = i64::MIN;
const LUA_MAXINTEGER: lua_Integer = i64::MAX;
const PI: lua_Number = std::f64::consts::PI;
const FIGS: u32 = 53;
const I2D_SHIFT: u32 = 64 - FIGS;
const I2D_SCALE: lua_Number = 1.0 / ((1_u64 << FIGS) as lua_Number);

const FIELD_ADD: &[u8] = b"add\0";
const FIELD_FACTORIAL: &[u8] = b"factorial\0";
const FIELD_SIMD: &[u8] = b"simd\0";
const FIELD_VERSION: &[u8] = b"version\0";
const VERSION_TEXT: &[u8] = b"0.1.0\0";

const NAME_ABS: &[u8] = b"abs\0";
const NAME_ACOS: &[u8] = b"acos\0";
const NAME_ASIN: &[u8] = b"asin\0";
const NAME_ATAN: &[u8] = b"atan\0";
const NAME_CEIL: &[u8] = b"ceil\0";
const NAME_COS: &[u8] = b"cos\0";
const NAME_DEG: &[u8] = b"deg\0";
const NAME_EXP: &[u8] = b"exp\0";
const NAME_TOINTEGER: &[u8] = b"tointeger\0";
const NAME_FLOOR: &[u8] = b"floor\0";
const NAME_FMOD: &[u8] = b"fmod\0";
const NAME_FREXP: &[u8] = b"frexp\0";
const NAME_ULT: &[u8] = b"ult\0";
const NAME_LDEXP: &[u8] = b"ldexp\0";
const NAME_LOG: &[u8] = b"log\0";
const NAME_MAX: &[u8] = b"max\0";
const NAME_MIN: &[u8] = b"min\0";
const NAME_MODF: &[u8] = b"modf\0";
const NAME_RAD: &[u8] = b"rad\0";
const NAME_SIN: &[u8] = b"sin\0";
const NAME_SQRT: &[u8] = b"sqrt\0";
const NAME_TAN: &[u8] = b"tan\0";
const NAME_TYPE: &[u8] = b"type\0";
const NAME_RANDOM: &[u8] = b"random\0";
const NAME_RANDOMSEED: &[u8] = b"randomseed\0";
const FIELD_PI: &[u8] = b"pi\0";
const FIELD_HUGE: &[u8] = b"huge\0";
const FIELD_MAXINTEGER: &[u8] = b"maxinteger\0";
const FIELD_MININTEGER: &[u8] = b"mininteger\0";
const NAME_F64X4_ADD: &[u8] = b"f64x4_add\0";
const NAME_F64X4_MUL: &[u8] = b"f64x4_mul\0";
const NAME_F64X4_DOT: &[u8] = b"f64x4_dot\0";
const NAME_I32X4_ADD: &[u8] = b"i32x4_add\0";
const STR_INTEGER: &[u8] = b"integer\0";
const STR_FLOAT: &[u8] = b"float\0";
const ERR_WRONG_NUMBER_OF_ARGUMENTS: &[u8] = b"wrong number of arguments\0";
const ERR_INTERVAL_EMPTY: &[u8] = b"interval is empty\0";
const ERR_ZERO: &[u8] = b"zero\0";
const ERR_EXPECTED_VECTOR_TABLE: &[u8] = b"expected a Lua array table\0";
const ERR_EXPECTED_4_LANES: &[u8] = b"expected exactly 4 lanes\0";
const ERR_I32_RANGE: &[u8] = b"lane value is out of i32 range\0";

static MATH_RS_REGS: [luaL_Reg; 4] = [
    luaL_Reg {
        name: FIELD_ADD.as_ptr().cast(),
        func: Some(rust_add),
    },
    luaL_Reg {
        name: FIELD_FACTORIAL.as_ptr().cast(),
        func: Some(rust_factorial),
    },
    luaL_Reg {
        name: FIELD_VERSION.as_ptr().cast(),
        func: Some(rust_version),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

static MATHLIB_REGS: [luaL_Reg; 24] = [
    luaL_Reg {
        name: NAME_ABS.as_ptr().cast(),
        func: Some(math_abs),
    },
    luaL_Reg {
        name: NAME_ACOS.as_ptr().cast(),
        func: Some(math_acos),
    },
    luaL_Reg {
        name: NAME_ASIN.as_ptr().cast(),
        func: Some(math_asin),
    },
    luaL_Reg {
        name: NAME_ATAN.as_ptr().cast(),
        func: Some(math_atan),
    },
    luaL_Reg {
        name: NAME_CEIL.as_ptr().cast(),
        func: Some(math_ceil),
    },
    luaL_Reg {
        name: NAME_COS.as_ptr().cast(),
        func: Some(math_cos),
    },
    luaL_Reg {
        name: NAME_DEG.as_ptr().cast(),
        func: Some(math_deg),
    },
    luaL_Reg {
        name: NAME_EXP.as_ptr().cast(),
        func: Some(math_exp),
    },
    luaL_Reg {
        name: NAME_TOINTEGER.as_ptr().cast(),
        func: Some(math_toint),
    },
    luaL_Reg {
        name: NAME_FLOOR.as_ptr().cast(),
        func: Some(math_floor),
    },
    luaL_Reg {
        name: NAME_FMOD.as_ptr().cast(),
        func: Some(math_fmod),
    },
    luaL_Reg {
        name: NAME_FREXP.as_ptr().cast(),
        func: Some(math_frexp),
    },
    luaL_Reg {
        name: NAME_ULT.as_ptr().cast(),
        func: Some(math_ult),
    },
    luaL_Reg {
        name: NAME_LDEXP.as_ptr().cast(),
        func: Some(math_ldexp),
    },
    luaL_Reg {
        name: NAME_LOG.as_ptr().cast(),
        func: Some(math_log),
    },
    luaL_Reg {
        name: NAME_MAX.as_ptr().cast(),
        func: Some(math_max),
    },
    luaL_Reg {
        name: NAME_MIN.as_ptr().cast(),
        func: Some(math_min),
    },
    luaL_Reg {
        name: NAME_MODF.as_ptr().cast(),
        func: Some(math_modf),
    },
    luaL_Reg {
        name: NAME_RAD.as_ptr().cast(),
        func: Some(math_rad),
    },
    luaL_Reg {
        name: NAME_SIN.as_ptr().cast(),
        func: Some(math_sin),
    },
    luaL_Reg {
        name: NAME_SQRT.as_ptr().cast(),
        func: Some(math_sqrt),
    },
    luaL_Reg {
        name: NAME_TAN.as_ptr().cast(),
        func: Some(math_tan),
    },
    luaL_Reg {
        name: NAME_TYPE.as_ptr().cast(),
        func: Some(math_type),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

static RANDFUNCS: [luaL_Reg; 3] = [
    luaL_Reg {
        name: NAME_RANDOM.as_ptr().cast(),
        func: Some(math_random),
    },
    luaL_Reg {
        name: NAME_RANDOMSEED.as_ptr().cast(),
        func: Some(math_randomseed),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

static SIMD_REGS: [luaL_Reg; 5] = [
    luaL_Reg {
        name: NAME_F64X4_ADD.as_ptr().cast(),
        func: Some(simd_f64x4_add),
    },
    luaL_Reg {
        name: NAME_F64X4_MUL.as_ptr().cast(),
        func: Some(simd_f64x4_mul),
    },
    luaL_Reg {
        name: NAME_F64X4_DOT.as_ptr().cast(),
        func: Some(simd_f64x4_dot),
    },
    luaL_Reg {
        name: NAME_I32X4_ADD.as_ptr().cast(),
        func: Some(simd_i32x4_add),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

pub fn link_anchor() {}

unsafe extern "C" {
    fn luaL_checkversion_(state: *mut lua_State, version: lua_Number, sizes: usize);
    fn luaL_checknumber(state: *mut lua_State, arg: c_int) -> lua_Number;
    fn luaL_optnumber(state: *mut lua_State, arg: c_int, def: lua_Number) -> lua_Number;
    fn luaL_checkinteger(state: *mut lua_State, arg: c_int) -> lua_Integer;
    fn luaL_optinteger(state: *mut lua_State, arg: c_int, def: lua_Integer) -> lua_Integer;
    fn luaL_checkany(state: *mut lua_State, arg: c_int);
    fn luaL_argerror(state: *mut lua_State, arg: c_int, extra: *const c_char) -> c_int;
    fn luaL_makeseed(state: *mut lua_State) -> c_uint;
    fn luaL_setfuncs(state: *mut lua_State, regs: *const luaL_Reg, nup: c_int);

    fn lua_gettop(state: *mut lua_State) -> c_int;
    fn lua_settop(state: *mut lua_State, index: c_int);
    fn lua_pushvalue(state: *mut lua_State, index: c_int);
    fn lua_isinteger(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_type(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_tointegerx(state: *mut lua_State, index: c_int, isnum: *mut c_int) -> lua_Integer;
    fn lua_touserdata(state: *mut lua_State, index: c_int) -> *mut std::ffi::c_void;
    fn lua_compare(state: *mut lua_State, left: c_int, right: c_int, op: c_int) -> c_int;
    fn lua_pushnil(state: *mut lua_State);
    fn lua_pushnumber(state: *mut lua_State, n: lua_Number);
    fn lua_pushinteger(state: *mut lua_State, n: lua_Integer);
    fn lua_pushstring(state: *mut lua_State, s: *const c_char) -> *const c_char;
    fn lua_pushboolean(state: *mut lua_State, b: c_int);
    fn lua_createtable(state: *mut lua_State, narr: c_int, nrec: c_int);
    fn lua_newuserdatauv(
        state: *mut lua_State,
        size: usize,
        nuvalue: c_int,
    ) -> *mut std::ffi::c_void;
    fn lua_setfield(state: *mut lua_State, index: c_int, key: *const c_char);
    fn lua_geti(state: *mut lua_State, index: c_int, n: lua_Integer) -> c_int;
    fn lua_seti(state: *mut lua_State, index: c_int, n: lua_Integer);
    fn lua_rawlen(state: *mut lua_State, index: c_int) -> usize;
    fn lua_error(state: *mut lua_State) -> c_int;
}

#[link(name = "m")]
unsafe extern "C" {
    fn fmod(x: lua_Number, y: lua_Number) -> lua_Number;
    fn frexp(x: lua_Number, exp: *mut c_int) -> lua_Number;
    fn ldexp(x: lua_Number, exp: c_int) -> lua_Number;
}

#[inline]
fn lua_upvalueindex(index: c_int) -> c_int {
    LUA_REGISTRYINDEX - index
}

#[inline]
unsafe fn lua_pop(state: *mut lua_State, count: c_int) {
    unsafe { lua_settop(state, -count - 1) };
}

#[inline]
unsafe fn push_fail(state: *mut lua_State) {
    unsafe { lua_pushnil(state) };
}

#[inline]
unsafe fn checkversion(state: *mut lua_State) {
    unsafe { luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES) };
}

#[inline]
unsafe fn create_library(state: *mut lua_State, regs: &[luaL_Reg]) {
    unsafe { checkversion(state) };
    unsafe { lua_createtable(state, 0, (regs.len() - 1) as c_int) };
    unsafe { luaL_setfuncs(state, regs.as_ptr(), 0) };
}

#[inline]
fn number_to_integer(value: lua_Number) -> Option<lua_Integer> {
    if value.is_finite()
        && value >= LUA_MININTEGER as lua_Number
        && value < -(LUA_MININTEGER as lua_Number)
        && value.trunc() == value
    {
        Some(value as lua_Integer)
    } else {
        None
    }
}

#[inline]
unsafe fn pushnumint(state: *mut lua_State, value: lua_Number) {
    if let Some(integer) = number_to_integer(value) {
        unsafe { lua_pushinteger(state, integer) };
    } else {
        unsafe { lua_pushnumber(state, value) };
    }
}

#[inline]
fn i2d(value: u64) -> lua_Number {
    ((value >> I2D_SHIFT) as lua_Number) * I2D_SCALE
}

#[inline]
fn next_random_value(state: &mut [u64; 4]) -> u64 {
    let value = state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
    let t = state[1] << 17;

    state[2] ^= state[0];
    state[3] ^= state[1];
    state[1] ^= state[2];
    state[0] ^= state[3];
    state[2] ^= t;
    state[3] = state[3].rotate_left(45);

    value
}

fn project(mut random: lua_Unsigned, n: lua_Unsigned, state: &mut RanState) -> lua_Unsigned {
    let mut limit = n;
    let mut shift = 1_u32;

    while (limit & limit.wrapping_add(1)) != 0 {
        limit |= limit >> shift;
        shift *= 2;
    }

    loop {
        random &= limit;
        if random <= n {
            return random;
        }
        random = next_random_value(&mut state.s);
    }
}

unsafe fn setseed(
    state: *mut lua_State,
    ran_state: &mut RanState,
    n1: lua_Unsigned,
    n2: lua_Unsigned,
) {
    ran_state.s = [n1, 0xff, n2, 0];
    for _ in 0..16 {
        let _ = next_random_value(&mut ran_state.s);
    }
    unsafe { lua_pushinteger(state, n1 as lua_Integer) };
    unsafe { lua_pushinteger(state, n2 as lua_Integer) };
}

unsafe fn raise_argerror(state: *mut lua_State, arg: c_int, message: &[u8]) -> ! {
    let _ = unsafe { luaL_argerror(state, arg, message.as_ptr().cast()) };
    unsafe { std::hint::unreachable_unchecked() }
}

unsafe fn check_vector_table(state: *mut lua_State, arg: c_int, lanes: usize) {
    if unsafe { lua_type(state, arg) } != LUA_TTABLE {
        unsafe { raise_argerror(state, arg, ERR_EXPECTED_VECTOR_TABLE) };
    }

    if unsafe { lua_rawlen(state, arg) } != lanes {
        unsafe { raise_argerror(state, arg, ERR_EXPECTED_4_LANES) };
    }
}

unsafe fn read_f64x4(state: *mut lua_State, arg: c_int) -> Simd<f64, 4> {
    unsafe { check_vector_table(state, arg, 4) };

    let mut values = [0.0; 4];
    for lane in 0..4 {
        unsafe { lua_geti(state, arg, (lane + 1) as lua_Integer) };
        values[lane] = unsafe { luaL_checknumber(state, -1) };
        unsafe { lua_pop(state, 1) };
    }

    Simd::from_array(values)
}

unsafe fn read_i32x4(state: *mut lua_State, arg: c_int) -> Simd<i32, 4> {
    unsafe { check_vector_table(state, arg, 4) };

    let mut values = [0_i32; 4];
    for lane in 0..4 {
        unsafe { lua_geti(state, arg, (lane + 1) as lua_Integer) };
        let value = unsafe { luaL_checkinteger(state, -1) };
        unsafe { lua_pop(state, 1) };

        if !(i32::MIN as lua_Integer..=i32::MAX as lua_Integer).contains(&value) {
            unsafe { raise_argerror(state, arg, ERR_I32_RANGE) };
        }

        values[lane] = value as i32;
    }

    Simd::from_array(values)
}

unsafe fn push_f64x4(state: *mut lua_State, value: Simd<f64, 4>) {
    let lanes = value.to_array();
    unsafe { lua_createtable(state, lanes.len() as c_int, 0) };
    for (index, lane) in lanes.iter().copied().enumerate() {
        unsafe { lua_pushnumber(state, lane) };
        unsafe { lua_seti(state, -2, (index + 1) as lua_Integer) };
    }
}

unsafe fn push_i32x4(state: *mut lua_State, value: Simd<i32, 4>) {
    let lanes = value.to_array();
    unsafe { lua_createtable(state, lanes.len() as c_int, 0) };
    for (index, lane) in lanes.iter().copied().enumerate() {
        unsafe { lua_pushinteger(state, lane as lua_Integer) };
        unsafe { lua_seti(state, -2, (index + 1) as lua_Integer) };
    }
}

unsafe fn create_simd_library(state: *mut lua_State) {
    unsafe { create_library(state, &SIMD_REGS) };
}

unsafe extern "C" fn rust_add(state: *mut lua_State) -> c_int {
    let lhs = unsafe { luaL_checknumber(state, 1) };
    let rhs = unsafe { luaL_checknumber(state, 2) };
    unsafe { lua_pushnumber(state, lhs + rhs) };
    1
}

unsafe extern "C" fn rust_factorial(state: *mut lua_State) -> c_int {
    let value = unsafe { luaL_checkinteger(state, 1) };
    let mut result = 1_i64;

    for item in 2..=value {
        result = result.saturating_mul(item);
    }

    unsafe { lua_pushinteger(state, result) };
    1
}

unsafe extern "C" fn rust_version(state: *mut lua_State) -> c_int {
    unsafe { lua_pushstring(state, VERSION_TEXT.as_ptr().cast()) };
    1
}

unsafe extern "C" fn simd_f64x4_add(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_f64x4(state, lhs + rhs) };
    1
}

unsafe extern "C" fn simd_f64x4_mul(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_f64x4(state, lhs * rhs) };
    1
}

unsafe extern "C" fn simd_f64x4_dot(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { lua_pushnumber(state, (lhs * rhs).reduce_sum()) };
    1
}

unsafe extern "C" fn simd_i32x4_add(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs + rhs) };
    1
}

unsafe extern "C" fn math_abs(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 {
        let value = unsafe { luaL_checkinteger(state, 1) };
        unsafe { lua_pushinteger(state, value.wrapping_abs()) };
    } else {
        let value = unsafe { luaL_checknumber(state, 1) };
        unsafe { lua_pushnumber(state, value.abs()) };
    }
    1
}

unsafe extern "C" fn math_sin(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).sin()) };
    1
}

unsafe extern "C" fn math_cos(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).cos()) };
    1
}

unsafe extern "C" fn math_tan(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).tan()) };
    1
}

unsafe extern "C" fn math_asin(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).asin()) };
    1
}

unsafe extern "C" fn math_acos(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).acos()) };
    1
}

unsafe extern "C" fn math_atan(state: *mut lua_State) -> c_int {
    let y = unsafe { luaL_checknumber(state, 1) };
    let x = unsafe { luaL_optnumber(state, 2, 1.0) };
    unsafe { lua_pushnumber(state, y.atan2(x)) };
    1
}

unsafe extern "C" fn math_toint(state: *mut lua_State) -> c_int {
    let mut valid = 0;
    let value = unsafe { lua_tointegerx(state, 1, &mut valid) };
    if valid != 0 {
        unsafe { lua_pushinteger(state, value) };
    } else {
        unsafe { luaL_checkany(state, 1) };
        unsafe { push_fail(state) };
    }
    1
}

unsafe extern "C" fn math_floor(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 {
        unsafe { lua_settop(state, 1) };
    } else {
        let value = unsafe { luaL_checknumber(state, 1) }.floor();
        unsafe { pushnumint(state, value) };
    }
    1
}

unsafe extern "C" fn math_ceil(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 {
        unsafe { lua_settop(state, 1) };
    } else {
        let value = unsafe { luaL_checknumber(state, 1) }.ceil();
        unsafe { pushnumint(state, value) };
    }
    1
}

unsafe extern "C" fn math_fmod(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 && unsafe { lua_isinteger(state, 2) } != 0 {
        let divisor = unsafe { luaL_checkinteger(state, 2) };
        if divisor == 0 {
            return unsafe { luaL_argerror(state, 2, ERR_ZERO.as_ptr().cast()) };
        }
        if divisor == -1 {
            unsafe { lua_pushinteger(state, 0) };
        } else {
            let value = unsafe { luaL_checkinteger(state, 1) };
            unsafe { lua_pushinteger(state, value % divisor) };
        }
    } else {
        let lhs = unsafe { luaL_checknumber(state, 1) };
        let rhs = unsafe { luaL_checknumber(state, 2) };
        unsafe { lua_pushnumber(state, fmod(lhs, rhs)) };
    }
    1
}

unsafe extern "C" fn math_modf(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 {
        unsafe { lua_settop(state, 1) };
        unsafe { lua_pushnumber(state, 0.0) };
    } else {
        let value = unsafe { luaL_checknumber(state, 1) };
        let integer_part = if value < 0.0 {
            value.ceil()
        } else {
            value.floor()
        };
        unsafe { pushnumint(state, integer_part) };
        let fraction = if value == integer_part {
            0.0
        } else {
            value - integer_part
        };
        unsafe { lua_pushnumber(state, fraction) };
    }
    2
}

unsafe extern "C" fn math_sqrt(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).sqrt()) };
    1
}

unsafe extern "C" fn math_ult(state: *mut lua_State) -> c_int {
    let lhs = unsafe { luaL_checkinteger(state, 1) } as lua_Unsigned;
    let rhs = unsafe { luaL_checkinteger(state, 2) } as lua_Unsigned;
    unsafe { lua_pushboolean(state, (lhs < rhs) as c_int) };
    1
}

unsafe extern "C" fn math_log(state: *mut lua_State) -> c_int {
    let value = unsafe { luaL_checknumber(state, 1) };
    let result = if unsafe { lua_type(state, 2) } <= 0 {
        value.ln()
    } else {
        let base = unsafe { luaL_checknumber(state, 2) };
        if base == 2.0 {
            value.log2()
        } else if base == 10.0 {
            value.log10()
        } else {
            value.ln() / base.ln()
        }
    };
    unsafe { lua_pushnumber(state, result) };
    1
}

unsafe extern "C" fn math_exp(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).exp()) };
    1
}

unsafe extern "C" fn math_deg(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1) * (180.0 / PI)) };
    1
}

unsafe extern "C" fn math_rad(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1) * (PI / 180.0)) };
    1
}

unsafe extern "C" fn math_frexp(state: *mut lua_State) -> c_int {
    let value = unsafe { luaL_checknumber(state, 1) };
    let mut exponent = 0;
    let mantissa = unsafe { frexp(value, &mut exponent) };
    unsafe { lua_pushnumber(state, mantissa) };
    unsafe { lua_pushinteger(state, exponent as lua_Integer) };
    2
}

unsafe extern "C" fn math_ldexp(state: *mut lua_State) -> c_int {
    let value = unsafe { luaL_checknumber(state, 1) };
    let exponent = unsafe { luaL_checkinteger(state, 2) } as c_int;
    unsafe { lua_pushnumber(state, ldexp(value, exponent)) };
    1
}

unsafe extern "C" fn math_min(state: *mut lua_State) -> c_int {
    let count = unsafe { lua_gettop(state) };
    if count < 1 {
        return unsafe { luaL_argerror(state, 1, b"value expected\0".as_ptr().cast()) };
    }

    let mut min_index = 1;
    for index in 2..=count {
        if unsafe { lua_compare(state, index, min_index, LUA_OPLT) } != 0 {
            min_index = index;
        }
    }

    unsafe { lua_pushvalue(state, min_index) };
    1
}

unsafe extern "C" fn math_max(state: *mut lua_State) -> c_int {
    let count = unsafe { lua_gettop(state) };
    if count < 1 {
        return unsafe { luaL_argerror(state, 1, b"value expected\0".as_ptr().cast()) };
    }

    let mut max_index = 1;
    for index in 2..=count {
        if unsafe { lua_compare(state, max_index, index, LUA_OPLT) } != 0 {
            max_index = index;
        }
    }

    unsafe { lua_pushvalue(state, max_index) };
    1
}

unsafe extern "C" fn math_type(state: *mut lua_State) -> c_int {
    if unsafe { lua_type(state, 1) } == LUA_TNUMBER {
        let result = if unsafe { lua_isinteger(state, 1) } != 0 {
            STR_INTEGER
        } else {
            STR_FLOAT
        };
        unsafe { lua_pushstring(state, result.as_ptr().cast()) };
    } else {
        unsafe { luaL_checkany(state, 1) };
        unsafe { push_fail(state) };
    }
    1
}

unsafe extern "C" fn math_random(state: *mut lua_State) -> c_int {
    let ran_state = unsafe { &mut *(lua_touserdata(state, lua_upvalueindex(1)) as *mut RanState) };
    let random = next_random_value(&mut ran_state.s);

    match unsafe { lua_gettop(state) } {
        0 => {
            unsafe { lua_pushnumber(state, i2d(random)) };
            1
        }
        1 => {
            let low = 1_i64;
            let up = unsafe { luaL_checkinteger(state, 1) };
            if up == 0 {
                unsafe { lua_pushinteger(state, random as lua_Integer) };
                return 1;
            }
            if low > up {
                return unsafe { luaL_argerror(state, 1, ERR_INTERVAL_EMPTY.as_ptr().cast()) };
            }
            let projected = project(
                random,
                (up as lua_Unsigned).wrapping_sub(low as lua_Unsigned),
                ran_state,
            );
            unsafe {
                lua_pushinteger(
                    state,
                    projected.wrapping_add(low as lua_Unsigned) as lua_Integer,
                )
            };
            1
        }
        2 => {
            let low = unsafe { luaL_checkinteger(state, 1) };
            let up = unsafe { luaL_checkinteger(state, 2) };
            if low > up {
                return unsafe { luaL_argerror(state, 1, ERR_INTERVAL_EMPTY.as_ptr().cast()) };
            }
            let projected = project(
                random,
                (up as lua_Unsigned).wrapping_sub(low as lua_Unsigned),
                ran_state,
            );
            unsafe {
                lua_pushinteger(
                    state,
                    projected.wrapping_add(low as lua_Unsigned) as lua_Integer,
                )
            };
            1
        }
        _ => {
            unsafe { lua_pushstring(state, ERR_WRONG_NUMBER_OF_ARGUMENTS.as_ptr().cast()) };
            unsafe { lua_error(state) }
        }
    }
}

unsafe extern "C" fn math_randomseed(state: *mut lua_State) -> c_int {
    let ran_state = unsafe { &mut *(lua_touserdata(state, lua_upvalueindex(1)) as *mut RanState) };
    let (seed1, seed2) = if unsafe { lua_type(state, 1) } == -1 {
        (
            unsafe { luaL_makeseed(state) as lua_Unsigned },
            next_random_value(&mut ran_state.s),
        )
    } else {
        (
            unsafe { luaL_checkinteger(state, 1) as lua_Unsigned },
            unsafe { luaL_optinteger(state, 2, 0) as lua_Unsigned },
        )
    };

    unsafe { setseed(state, ran_state, seed1, seed2) };
    2
}

unsafe fn setrandfunc(state: *mut lua_State) {
    let userdata = unsafe { lua_newuserdatauv(state, size_of::<RanState>(), 0) as *mut RanState };
    let ran_state = unsafe { &mut *userdata };
    unsafe { setseed(state, ran_state, luaL_makeseed(state) as lua_Unsigned, 0) };
    unsafe { lua_pop(state, 2) };
    unsafe { luaL_setfuncs(state, RANDFUNCS.as_ptr(), 1) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn math_rs_open(state: *mut lua_State) -> c_int {
    unsafe { luaopen_math_rs(state) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_math_rs(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &MATH_RS_REGS) };
    unsafe { create_simd_library(state) };
    unsafe { lua_setfield(state, -2, FIELD_SIMD.as_ptr().cast()) };
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_math(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &MATHLIB_REGS) };
    unsafe { lua_pushnumber(state, PI) };
    unsafe { lua_setfield(state, -2, FIELD_PI.as_ptr().cast()) };
    unsafe { lua_pushnumber(state, lua_Number::INFINITY) };
    unsafe { lua_setfield(state, -2, FIELD_HUGE.as_ptr().cast()) };
    unsafe { lua_pushinteger(state, LUA_MAXINTEGER) };
    unsafe { lua_setfield(state, -2, FIELD_MAXINTEGER.as_ptr().cast()) };
    unsafe { lua_pushinteger(state, LUA_MININTEGER) };
    unsafe { lua_setfield(state, -2, FIELD_MININTEGER.as_ptr().cast()) };
    unsafe { setrandfunc(state) };
    1
}
