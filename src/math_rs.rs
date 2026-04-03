use crate::aux_rs::*;
use crate::lua_module::*;
use crate::lua_module::luaL_Reg;
use crate::runtime::*;
use core::ffi::c_int;
use core::mem::size_of;
use core::ptr;
use std::simd::Simd;
use std::simd::StdFloat;
use std::simd::cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd};
use std::simd::num::{SimdFloat, SimdInt};

#[repr(C)]
struct RanState {
    s: [u64; 4],
}
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

static SIMD_F64X4_REGS: [luaL_Reg; 25] = [
    luaL_Reg {
        name: NAME_SPLAT.as_ptr().cast(),
        func: Some(simd_f64x4_splat),
    },
    luaL_Reg {
        name: NAME_ADD.as_ptr().cast(),
        func: Some(simd_f64x4_add),
    },
    luaL_Reg {
        name: NAME_SUB.as_ptr().cast(),
        func: Some(simd_f64x4_sub),
    },
    luaL_Reg {
        name: NAME_MUL.as_ptr().cast(),
        func: Some(simd_f64x4_mul),
    },
    luaL_Reg {
        name: NAME_DIV.as_ptr().cast(),
        func: Some(simd_f64x4_div),
    },
    luaL_Reg {
        name: NAME_DOT.as_ptr().cast(),
        func: Some(simd_f64x4_dot),
    },
    luaL_Reg {
        name: NAME_SUM.as_ptr().cast(),
        func: Some(simd_f64x4_sum),
    },
    luaL_Reg {
        name: NAME_PRODUCT.as_ptr().cast(),
        func: Some(simd_f64x4_product),
    },
    luaL_Reg {
        name: NAME_SIMD_MIN.as_ptr().cast(),
        func: Some(simd_f64x4_min),
    },
    luaL_Reg {
        name: NAME_SIMD_MAX.as_ptr().cast(),
        func: Some(simd_f64x4_max),
    },
    luaL_Reg {
        name: NAME_SIMD_SQRT.as_ptr().cast(),
        func: Some(simd_f64x4_sqrt),
    },
    luaL_Reg {
        name: NAME_ABS_VEC.as_ptr().cast(),
        func: Some(simd_f64x4_abs),
    },
    luaL_Reg {
        name: NAME_NEG.as_ptr().cast(),
        func: Some(simd_f64x4_neg),
    },
    luaL_Reg {
        name: NAME_FLOOR_VEC.as_ptr().cast(),
        func: Some(simd_f64x4_floor),
    },
    luaL_Reg {
        name: NAME_CEIL_VEC.as_ptr().cast(),
        func: Some(simd_f64x4_ceil),
    },
    luaL_Reg {
        name: NAME_ROUND_VEC.as_ptr().cast(),
        func: Some(simd_f64x4_round),
    },
    luaL_Reg {
        name: NAME_TRUNC_VEC.as_ptr().cast(),
        func: Some(simd_f64x4_trunc),
    },
    luaL_Reg {
        name: NAME_RECIP.as_ptr().cast(),
        func: Some(simd_f64x4_recip),
    },
    luaL_Reg {
        name: NAME_EQ.as_ptr().cast(),
        func: Some(simd_f64x4_eq),
    },
    luaL_Reg {
        name: NAME_NE.as_ptr().cast(),
        func: Some(simd_f64x4_ne),
    },
    luaL_Reg {
        name: NAME_LT.as_ptr().cast(),
        func: Some(simd_f64x4_lt),
    },
    luaL_Reg {
        name: NAME_LE.as_ptr().cast(),
        func: Some(simd_f64x4_le),
    },
    luaL_Reg {
        name: NAME_GT.as_ptr().cast(),
        func: Some(simd_f64x4_gt),
    },
    luaL_Reg {
        name: NAME_GE.as_ptr().cast(),
        func: Some(simd_f64x4_ge),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

static SIMD_I32X4_REGS: [luaL_Reg; 22] = [
    luaL_Reg {
        name: NAME_SPLAT.as_ptr().cast(),
        func: Some(simd_i32x4_splat),
    },
    luaL_Reg {
        name: NAME_ADD.as_ptr().cast(),
        func: Some(simd_i32x4_add),
    },
    luaL_Reg {
        name: NAME_SUB.as_ptr().cast(),
        func: Some(simd_i32x4_sub),
    },
    luaL_Reg {
        name: NAME_MUL.as_ptr().cast(),
        func: Some(simd_i32x4_mul),
    },
    luaL_Reg {
        name: NAME_SUM.as_ptr().cast(),
        func: Some(simd_i32x4_sum),
    },
    luaL_Reg {
        name: NAME_PRODUCT.as_ptr().cast(),
        func: Some(simd_i32x4_product),
    },
    luaL_Reg {
        name: NAME_SIMD_MIN.as_ptr().cast(),
        func: Some(simd_i32x4_min),
    },
    luaL_Reg {
        name: NAME_SIMD_MAX.as_ptr().cast(),
        func: Some(simd_i32x4_max),
    },
    luaL_Reg {
        name: NAME_ABS_VEC.as_ptr().cast(),
        func: Some(simd_i32x4_abs),
    },
    luaL_Reg {
        name: NAME_NEG.as_ptr().cast(),
        func: Some(simd_i32x4_neg),
    },
    luaL_Reg {
        name: NAME_BITAND.as_ptr().cast(),
        func: Some(simd_i32x4_bitand),
    },
    luaL_Reg {
        name: NAME_BITOR.as_ptr().cast(),
        func: Some(simd_i32x4_bitor),
    },
    luaL_Reg {
        name: NAME_BITXOR.as_ptr().cast(),
        func: Some(simd_i32x4_bitxor),
    },
    luaL_Reg {
        name: NAME_SHL.as_ptr().cast(),
        func: Some(simd_i32x4_shl),
    },
    luaL_Reg {
        name: NAME_SHR.as_ptr().cast(),
        func: Some(simd_i32x4_shr),
    },
    luaL_Reg {
        name: NAME_EQ.as_ptr().cast(),
        func: Some(simd_i32x4_eq),
    },
    luaL_Reg {
        name: NAME_NE.as_ptr().cast(),
        func: Some(simd_i32x4_ne),
    },
    luaL_Reg {
        name: NAME_LT.as_ptr().cast(),
        func: Some(simd_i32x4_lt),
    },
    luaL_Reg {
        name: NAME_LE.as_ptr().cast(),
        func: Some(simd_i32x4_le),
    },
    luaL_Reg {
        name: NAME_GT.as_ptr().cast(),
        func: Some(simd_i32x4_gt),
    },
    luaL_Reg {
        name: NAME_GE.as_ptr().cast(),
        func: Some(simd_i32x4_ge),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

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

#[inline]
fn fmod(lhs: lua_Number, rhs: lua_Number) -> lua_Number {
    lhs % rhs
}

#[inline]
fn frexp(value: lua_Number, exponent: &mut c_int) -> lua_Number {
    if value == 0.0 || !value.is_finite() {
        *exponent = 0;
        return value;
    }

    let bits = value.to_bits();
    let sign = bits & F64_SIGN_MASK;
    let fraction = bits & F64_FRAC_MASK;
    let exp_bits = ((bits & F64_EXP_MASK) >> F64_MANTISSA_BITS) as i32;

    if exp_bits == 0 {
        let scaled = value * ((1_u64 << 54) as lua_Number);
        let scaled_bits = scaled.to_bits();
        let scaled_exp = ((scaled_bits & F64_EXP_MASK) >> F64_MANTISSA_BITS) as i32;
        let scaled_frac = scaled_bits & F64_FRAC_MASK;
        *exponent = (scaled_exp - (F64_EXP_BIAS - 1) - 54) as c_int;
        return lua_Number::from_bits(
            sign | (((F64_EXP_BIAS - 1) as u64) << F64_MANTISSA_BITS) | scaled_frac,
        );
    }

    *exponent = (exp_bits - (F64_EXP_BIAS - 1)) as c_int;
    lua_Number::from_bits(sign | (((F64_EXP_BIAS - 1) as u64) << F64_MANTISSA_BITS) | fraction)
}

#[inline]
fn pow2(exp: i32) -> lua_Number {
    if exp < -1074 {
        0.0
    } else if exp < -1022 {
        lua_Number::from_bits(1_u64 << (exp + 1074))
    } else if exp <= 1023 {
        lua_Number::from_bits(((exp + F64_EXP_BIAS) as u64) << F64_MANTISSA_BITS)
    } else {
        lua_Number::INFINITY
    }
}

#[inline]
fn ldexp(value: lua_Number, exponent: c_int) -> lua_Number {
    if value == 0.0 || !value.is_finite() {
        return value;
    }

    let mut base_exp = 0;
    let mantissa = frexp(value, &mut base_exp);
    let total_exp = base_exp.saturating_add(exponent);

    if total_exp > 1024 {
        return if value.is_sign_negative() {
            lua_Number::NEG_INFINITY
        } else {
            lua_Number::INFINITY
        };
    }

    if total_exp < -1073 {
        return value.copysign(0.0);
    }

    if total_exp == 1024 {
        return (mantissa * pow2(1023)) * 2.0;
    }

    mantissa * pow2(total_exp)
}

unsafe fn check_vector_table(state: *mut lua_State, arg: c_int, lanes: usize) {
    if unsafe { lua_type(state, arg) } != 5 {
        let _ = luaL_argerror(state, arg, ERR_EXPECTED_VECTOR_TABLE.as_ptr().cast());
        unsafe { core::hint::unreachable_unchecked() }
    }

    if unsafe { lua_rawlen(state, arg) } != lanes.try_into().unwrap() {
        let _ = luaL_argerror(state, arg, ERR_EXPECTED_4_LANES.as_ptr().cast());
        unsafe { core::hint::unreachable_unchecked() }
    }
}

unsafe fn read_f64x4(state: *mut lua_State, arg: c_int) -> Simd<f64, 4> {
    unsafe { check_vector_table(state, arg, 4) };

    let mut values = [0.0; 4];
    for lane in 0..4 {
        unsafe { lua_geti(state, arg, (lane + 1) as lua_Integer) };
        values[lane] = luaL_checknumber(state, -1);
        unsafe { lua_pop(state, 1) };
    }

    Simd::from_array(values)
}

unsafe fn read_i32x4(state: *mut lua_State, arg: c_int) -> Simd<i32, 4> {
    unsafe { check_vector_table(state, arg, 4) };

    let mut values = [0_i32; 4];
    for lane in 0..4 {
        unsafe { lua_geti(state, arg, (lane + 1) as lua_Integer) };
        let value = { luaL_checkinteger(state, -1) };
        unsafe { lua_pop(state, 1) };

        if !(i32::MIN as lua_Integer..=i32::MAX as lua_Integer).contains(&value) {
            let _ = { luaL_argerror(state, arg, ERR_I32_RANGE.as_ptr().cast()) };
            unsafe { core::hint::unreachable_unchecked() }
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

unsafe fn push_boolx4(state: *mut lua_State, value: [bool; 4]) {
    unsafe { lua_createtable(state, value.len() as c_int, 0) };
    for (index, lane) in value.iter().copied().enumerate() {
        unsafe { lua_pushboolean(state, lane as c_int) };
        unsafe { lua_seti(state, -2, (index + 1) as lua_Integer) };
    }
}

unsafe fn create_simd_library(state: *mut lua_State) {
    unsafe { lua_createtable(state, 0, 3) };

    unsafe { create_library(state, &SIMD_F64X4_REGS) };
    unsafe { lua_pushinteger(state, 4) };
    unsafe { lua_setfield(state, -2, FIELD_LANES.as_ptr().cast()) };
    unsafe { lua_setfield(state, -2, FIELD_F64X4.as_ptr().cast()) };

    unsafe { create_library(state, &SIMD_I32X4_REGS) };
    unsafe { lua_pushinteger(state, 4) };
    unsafe { lua_setfield(state, -2, FIELD_LANES.as_ptr().cast()) };
    unsafe { lua_setfield(state, -2, FIELD_I32X4.as_ptr().cast()) };

    unsafe { lua_pushinteger(state, 4) };
    unsafe { lua_setfield(state, -2, FIELD_LANES.as_ptr().cast()) };
}

unsafe  fn simd_f64x4_add(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_f64x4(state, lhs + rhs) };
    1
}

unsafe  fn simd_f64x4_splat(state: *mut lua_State) -> c_int {
    let value = { luaL_checknumber(state, 1) };
    unsafe { push_f64x4(state, Simd::splat(value)) };
    1
}

unsafe  fn simd_f64x4_sub(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_f64x4(state, lhs - rhs) };
    1
}

unsafe  fn simd_f64x4_mul(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_f64x4(state, lhs * rhs) };
    1
}

unsafe  fn simd_f64x4_div(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_f64x4(state, lhs / rhs) };
    1
}

unsafe  fn simd_f64x4_dot(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { lua_pushnumber(state, (lhs * rhs).reduce_sum()) };
    1
}

unsafe  fn simd_f64x4_sum(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { lua_pushnumber(state, value.reduce_sum()) };
    1
}

unsafe  fn simd_f64x4_product(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { lua_pushnumber(state, value.reduce_product()) };
    1
}

unsafe  fn simd_f64x4_min(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_f64x4(state, lhs.simd_min(rhs)) };
    1
}

unsafe  fn simd_f64x4_max(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_f64x4(state, lhs.simd_max(rhs)) };
    1
}

unsafe  fn simd_f64x4_sqrt(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { push_f64x4(state, value.sqrt()) };
    1
}

unsafe  fn simd_f64x4_abs(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { push_f64x4(state, value.abs()) };
    1
}

unsafe  fn simd_f64x4_neg(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { push_f64x4(state, -value) };
    1
}

unsafe  fn simd_f64x4_floor(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { push_f64x4(state, value.floor()) };
    1
}

unsafe  fn simd_f64x4_ceil(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { push_f64x4(state, value.ceil()) };
    1
}

unsafe  fn simd_f64x4_round(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { push_f64x4(state, value.round()) };
    1
}

unsafe  fn simd_f64x4_trunc(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { push_f64x4(state, value.trunc()) };
    1
}

unsafe  fn simd_f64x4_recip(state: *mut lua_State) -> c_int {
    let value = unsafe { read_f64x4(state, 1) };
    unsafe { push_f64x4(state, value.recip()) };
    1
}

unsafe  fn simd_f64x4_eq(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_eq(rhs).to_array()) };
    1
}

unsafe  fn simd_f64x4_ne(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_ne(rhs).to_array()) };
    1
}

unsafe  fn simd_f64x4_lt(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_lt(rhs).to_array()) };
    1
}

unsafe  fn simd_f64x4_le(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_le(rhs).to_array()) };
    1
}

unsafe  fn simd_f64x4_gt(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_gt(rhs).to_array()) };
    1
}

unsafe  fn simd_f64x4_ge(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_f64x4(state, 1) };
    let rhs = unsafe { read_f64x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_ge(rhs).to_array()) };
    1
}

unsafe  fn simd_i32x4_splat(state: *mut lua_State) -> c_int {
    let value = { luaL_checkinteger(state, 1) };
    if !(i32::MIN as lua_Integer..=i32::MAX as lua_Integer).contains(&value) {
        return luaL_argerror(state, 1, ERR_I32_RANGE.as_ptr().cast());
    }
    unsafe { push_i32x4(state, Simd::splat(value as i32)) };
    1
}

unsafe  fn simd_i32x4_add(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs + rhs) };
    1
}

unsafe  fn simd_i32x4_sub(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs - rhs) };
    1
}

unsafe  fn simd_i32x4_mul(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs * rhs) };
    1
}

unsafe  fn simd_i32x4_sum(state: *mut lua_State) -> c_int {
    let value = unsafe { read_i32x4(state, 1) };
    unsafe { lua_pushinteger(state, value.reduce_sum() as lua_Integer) };
    1
}

unsafe  fn simd_i32x4_product(state: *mut lua_State) -> c_int {
    let value = unsafe { read_i32x4(state, 1) };
    unsafe { lua_pushinteger(state, value.reduce_product() as lua_Integer) };
    1
}

unsafe  fn simd_i32x4_min(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs.simd_min(rhs)) };
    1
}

unsafe  fn simd_i32x4_max(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs.simd_max(rhs)) };
    1
}

unsafe  fn simd_i32x4_abs(state: *mut lua_State) -> c_int {
    let value = unsafe { read_i32x4(state, 1) };
    unsafe { push_i32x4(state, value.abs()) };
    1
}

unsafe  fn simd_i32x4_neg(state: *mut lua_State) -> c_int {
    let value = unsafe { read_i32x4(state, 1) };
    unsafe { push_i32x4(state, -value) };
    1
}

unsafe  fn simd_i32x4_bitand(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs & rhs) };
    1
}

unsafe  fn simd_i32x4_bitor(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs | rhs) };
    1
}

unsafe  fn simd_i32x4_bitxor(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_i32x4(state, lhs ^ rhs) };
    1
}

unsafe  fn simd_i32x4_shl(state: *mut lua_State) -> c_int {
    let value = unsafe { read_i32x4(state, 1) };
    let amount = { luaL_checkinteger(state, 2) };
    unsafe { push_i32x4(state, value << Simd::splat(amount as i32)) };
    1
}

unsafe  fn simd_i32x4_shr(state: *mut lua_State) -> c_int {
    let value = unsafe { read_i32x4(state, 1) };
    let amount = { luaL_checkinteger(state, 2) };
    unsafe { push_i32x4(state, value >> Simd::splat(amount as i32)) };
    1
}

unsafe  fn simd_i32x4_eq(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_eq(rhs).to_array()) };
    1
}

unsafe  fn simd_i32x4_ne(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_ne(rhs).to_array()) };
    1
}

unsafe  fn simd_i32x4_lt(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_lt(rhs).to_array()) };
    1
}

unsafe  fn simd_i32x4_le(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_le(rhs).to_array()) };
    1
}

unsafe  fn simd_i32x4_gt(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_gt(rhs).to_array()) };
    1
}

unsafe  fn simd_i32x4_ge(state: *mut lua_State) -> c_int {
    let lhs = unsafe { read_i32x4(state, 1) };
    let rhs = unsafe { read_i32x4(state, 2) };
    unsafe { push_boolx4(state, lhs.simd_ge(rhs).to_array()) };
    1
}

unsafe  fn math_abs(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 {
        let value = { luaL_checkinteger(state, 1) };
        unsafe { lua_pushinteger(state, value.wrapping_abs()) };
    } else {
        let value = { luaL_checknumber(state, 1) };
        unsafe { lua_pushnumber(state, value.abs()) };
    }
    1
}

unsafe  fn math_sin(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).sin()) };
    1
}

unsafe  fn math_cos(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).cos()) };
    1
}

unsafe  fn math_tan(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).tan()) };
    1
}

unsafe  fn math_asin(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).asin()) };
    1
}

unsafe  fn math_acos(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).acos()) };
    1
}

unsafe  fn math_atan(state: *mut lua_State) -> c_int {
    let y = { luaL_checknumber(state, 1) };
    let x = { luaL_optnumber(state, 2, 1.0) };
    unsafe { lua_pushnumber(state, y.atan2(x)) };
    1
}

unsafe  fn math_toint(state: *mut lua_State) -> c_int {
    let mut valid = 0;
    let value = unsafe { lua_tointegerx(state, 1, &mut valid) };
    if valid != 0 {
        unsafe { lua_pushinteger(state, value) };
    } else {
        {
            luaL_checkany(state, 1)
        };
        unsafe { push_fail(state) };
    }
    1
}

unsafe  fn math_floor(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 {
        unsafe { lua_settop(state, 1) };
    } else {
        let value = { luaL_checknumber(state, 1) }.floor();
        unsafe { pushnumint(state, value) };
    }
    1
}

unsafe  fn math_ceil(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 {
        unsafe { lua_settop(state, 1) };
    } else {
        let value = { luaL_checknumber(state, 1) }.ceil();
        unsafe { pushnumint(state, value) };
    }
    1
}

unsafe  fn math_fmod(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 && unsafe { lua_isinteger(state, 2) } != 0 {
        let divisor = { luaL_checkinteger(state, 2) };
        if divisor == 0 {
            return luaL_argerror(state, 2, ERR_ZERO.as_ptr().cast());
        }
        if divisor == -1 {
            unsafe { lua_pushinteger(state, 0) };
        } else {
            let value = { luaL_checkinteger(state, 1) };
            unsafe { lua_pushinteger(state, value % divisor) };
        }
    } else {
        let lhs = { luaL_checknumber(state, 1) };
        let rhs = { luaL_checknumber(state, 2) };
        unsafe { lua_pushnumber(state, fmod(lhs, rhs)) };
    }
    1
}

unsafe  fn math_modf(state: *mut lua_State) -> c_int {
    if unsafe { lua_isinteger(state, 1) } != 0 {
        unsafe { lua_settop(state, 1) };
        unsafe { lua_pushnumber(state, 0.0) };
    } else {
        let value = { luaL_checknumber(state, 1) };
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

unsafe  fn math_sqrt(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).sqrt()) };
    1
}

unsafe  fn math_ult(state: *mut lua_State) -> c_int {
    let lhs = { luaL_checkinteger(state, 1) } as lua_Unsigned;
    let rhs = { luaL_checkinteger(state, 2) } as lua_Unsigned;
    unsafe { lua_pushboolean(state, (lhs < rhs) as c_int) };
    1
}

unsafe  fn math_log(state: *mut lua_State) -> c_int {
    let value = { luaL_checknumber(state, 1) };
    let result = if unsafe { lua_type(state, 2) } <= 0 {
        value.ln()
    } else {
        let base = { luaL_checknumber(state, 2) };
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

unsafe  fn math_exp(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1).exp()) };
    1
}

unsafe  fn math_deg(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1) * (180.0 / PI)) };
    1
}

unsafe  fn math_rad(state: *mut lua_State) -> c_int {
    unsafe { lua_pushnumber(state, luaL_checknumber(state, 1) * (PI / 180.0)) };
    1
}

unsafe  fn math_frexp(state: *mut lua_State) -> c_int {
    let value = { luaL_checknumber(state, 1) };
    let mut exponent = 0;
    let mantissa = frexp(value, &mut exponent);
    unsafe { lua_pushnumber(state, mantissa) };
    unsafe { lua_pushinteger(state, exponent as lua_Integer) };
    2
}

unsafe  fn math_ldexp(state: *mut lua_State) -> c_int {
    let value = { luaL_checknumber(state, 1) };
    let exponent = { luaL_checkinteger(state, 2) } as c_int;
    unsafe { lua_pushnumber(state, ldexp(value, exponent)) };
    1
}

unsafe  fn math_min(state: *mut lua_State) -> c_int {
    let count = unsafe { lua_gettop(state) };
    if count < 1 {
        return luaL_argerror(state, 1, b"value expected\0".as_ptr().cast());
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

unsafe  fn math_max(state: *mut lua_State) -> c_int {
    let count = unsafe { lua_gettop(state) };
    if count < 1 {
        return luaL_argerror(state, 1, b"value expected\0".as_ptr().cast());
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

unsafe  fn math_type(state: *mut lua_State) -> c_int {
    if unsafe { lua_type(state, 1) } == LUA_TNUMBER as c_int {
        let result = if unsafe { lua_isinteger(state, 1) } != 0 {
            STR_INTEGER
        } else {
            STR_FLOAT
        };
        unsafe { lua_pushstring(state, result.as_ptr().cast()) };
    } else {
        {
            luaL_checkany(state, 1)
        };
        unsafe { push_fail(state) };
    }
    1
}

unsafe  fn math_random(state: *mut lua_State) -> c_int {
    let ran_state = unsafe { &mut *(lua_touserdata(state, lua_upvalueindex(1)) as *mut RanState) };
    let random = next_random_value(&mut ran_state.s);

    match unsafe { lua_gettop(state) } {
        0 => {
            unsafe { lua_pushnumber(state, i2d(random)) };
            1
        }
        1 => {
            let low = 1_i64;
            let up = luaL_checkinteger(state, 1);
            if up == 0 {
                unsafe { lua_pushinteger(state, random as lua_Integer) };
                return 1;
            }
            if low > up {
                return luaL_argerror(state, 1, ERR_INTERVAL_EMPTY.as_ptr().cast());
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
            let low = { luaL_checkinteger(state, 1) };
            let up = { luaL_checkinteger(state, 2) };
            if low > up {
                return luaL_argerror(state, 1, ERR_INTERVAL_EMPTY.as_ptr().cast());
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

unsafe  fn math_randomseed(state: *mut lua_State) -> c_int {
    let ran_state = unsafe { &mut *(lua_touserdata(state, lua_upvalueindex(1)) as *mut RanState) };
    let (seed1, seed2) = if unsafe { lua_type(state, 1) } == -1 {
        (
            { luaL_makeseed(state) as lua_Unsigned },
            next_random_value(&mut ran_state.s),
        )
    } else {
        ({ luaL_checkinteger(state, 1) as lua_Unsigned }, {
            luaL_optinteger(state, 2, 0) as lua_Unsigned
        })
    };

    unsafe { setseed(state, ran_state, seed1, seed2) };
    2
}

unsafe fn setrandfunc(state: *mut lua_State) {
    let userdata = unsafe { lua_newuserdatauv(state, size_of::<RanState>(), 0) as *mut RanState };
    let ran_state = unsafe { &mut *userdata };
    unsafe { setseed(state, ran_state, luaL_makeseed(state) as lua_Unsigned, 0) };
    unsafe { lua_pop(state, 2) };
    {
        luaL_setfuncs(state, RANDFUNCS.as_ptr(), 1)
    };
}

pub(crate) unsafe  fn luaopen_math(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &MATHLIB_REGS) };
    unsafe { create_simd_library(state) };
    unsafe { lua_setfield(state, -2, FIELD_SIMD.as_ptr().cast()) };
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

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn math_builtin_script() {
        run_lua_test(
            "test/math_builtin.lua",
            include_str!("../test/math_builtin.lua"),
        );
    }

    #[test]
    fn math_simd_script() {
        run_lua_test("test/math_simd.lua", include_str!("../test/math_simd.lua"));
    }
}
