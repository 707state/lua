package.cpath = "./target/debug/lib?.so;./target/debug/?.so;" .. package.cpath

local rust = require("rust_ffi")
local simd = rust.simd

local function assert_array_eq(actual, expected)
  assert(#actual == #expected, "array length mismatch")
  for i = 1, #expected do
    assert(actual[i] == expected[i], ("lane %d mismatch: %s ~= %s"):format(i, tostring(actual[i]), tostring(expected[i])))
  end
end

assert_array_eq(simd.f64x4_add({1.5, 2.5, 3.5, 4.5}, {10.0, 20.0, 30.0, 40.0}), {11.5, 22.5, 33.5, 44.5})
assert_array_eq(simd.f64x4_mul({1.0, 2.0, 3.0, 4.0}, {0.5, 1.5, 2.5, 3.5}), {0.5, 3.0, 7.5, 14.0})
assert(simd.f64x4_dot({1.0, 2.0, 3.0, 4.0}, {5.0, 6.0, 7.0, 8.0}) == 70.0)
assert_array_eq(simd.i32x4_add({1, 2, 3, 4}, {10, 20, 30, 40}), {11, 22, 33, 44})

local ok, err = pcall(simd.f64x4_add, {1.0, 2.0, 3.0}, {4.0, 5.0, 6.0, 7.0})
assert(not ok)
assert(err:match("expected exactly 4 lanes"))

local ok2, err2 = pcall(simd.i32x4_add, {1, 2, 3, 2147483648}, {0, 0, 0, 0})
assert(not ok2)
assert(err2:match("out of i32 range"))

print("rust_ffi simd ok")
