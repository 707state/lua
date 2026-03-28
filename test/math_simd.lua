local simd = math.simd
local f64x4 = simd.f64x4
local i32x4 = simd.i32x4

local function assert_array_eq(actual, expected)
  assert(#actual == #expected, "array length mismatch")
  for i = 1, #expected do
    assert(actual[i] == expected[i], ("lane %d mismatch: %s ~= %s"):format(i, tostring(actual[i]), tostring(expected[i])))
  end
end

local function assert_bool_array_eq(actual, expected)
  assert(#actual == #expected, "array length mismatch")
  for i = 1, #expected do
    assert(actual[i] == expected[i], ("lane %d mismatch: %s ~= %s"):format(i, tostring(actual[i]), tostring(expected[i])))
  end
end

assert(simd.lanes == 4)
assert(f64x4.lanes == 4)
assert(i32x4.lanes == 4)

assert_array_eq(f64x4.splat(2.5), {2.5, 2.5, 2.5, 2.5})
assert_array_eq(f64x4.add({1.5, 2.5, 3.5, 4.5}, {10.0, 20.0, 30.0, 40.0}), {11.5, 22.5, 33.5, 44.5})
assert_array_eq(f64x4.sub({10.0, 20.0, 30.0, 40.0}, {1.5, 2.5, 3.5, 4.5}), {8.5, 17.5, 26.5, 35.5})
assert_array_eq(f64x4.mul({1.0, 2.0, 3.0, 4.0}, {0.5, 1.5, 2.5, 3.5}), {0.5, 3.0, 7.5, 14.0})
assert_array_eq(f64x4.div({8.0, 27.0, 64.0, 125.0}, {2.0, 3.0, 4.0, 5.0}), {4.0, 9.0, 16.0, 25.0})
assert(f64x4.dot({1.0, 2.0, 3.0, 4.0}, {5.0, 6.0, 7.0, 8.0}) == 70.0)
assert(f64x4.sum({1.0, 2.0, 3.0, 4.0}) == 10.0)
assert(f64x4.product({1.0, 2.0, 3.0, 4.0}) == 24.0)
assert_array_eq(f64x4.min({1.0, 9.0, 3.0, 7.0}, {2.0, 4.0, 8.0, 6.0}), {1.0, 4.0, 3.0, 6.0})
assert_array_eq(f64x4.max({1.0, 9.0, 3.0, 7.0}, {2.0, 4.0, 8.0, 6.0}), {2.0, 9.0, 8.0, 7.0})
assert_array_eq(f64x4.sqrt({1.0, 4.0, 9.0, 16.0}), {1.0, 2.0, 3.0, 4.0})
assert_array_eq(f64x4.abs({-1.5, 2.5, -3.5, 4.5}), {1.5, 2.5, 3.5, 4.5})
assert_array_eq(f64x4.neg({-1.5, 2.5, -3.5, 4.5}), {1.5, -2.5, 3.5, -4.5})
assert_array_eq(f64x4.floor({1.2, 2.8, -3.2, -4.8}), {1.0, 2.0, -4.0, -5.0})
assert_array_eq(f64x4.ceil({1.2, 2.8, -3.2, -4.8}), {2.0, 3.0, -3.0, -4.0})
assert_array_eq(f64x4.round({1.2, 2.8, -3.2, -4.8}), {1.0, 3.0, -3.0, -5.0})
assert_array_eq(f64x4.trunc({1.2, 2.8, -3.2, -4.8}), {1.0, 2.0, -3.0, -4.0})
assert_array_eq(f64x4.recip({2.0, 4.0, 8.0, 16.0}), {0.5, 0.25, 0.125, 0.0625})
assert_bool_array_eq(f64x4.eq({1.0, 2.0, 3.0, 4.0}, {1.0, 0.0, 3.5, 4.0}), {true, false, false, true})
assert_bool_array_eq(f64x4.ne({1.0, 2.0, 3.0, 4.0}, {1.0, 0.0, 3.5, 4.0}), {false, true, true, false})
assert_bool_array_eq(f64x4.lt({1.0, 2.0, 3.0, 4.0}, {2.0, 2.0, 2.0, 5.0}), {true, false, false, true})
assert_bool_array_eq(f64x4.le({1.0, 2.0, 3.0, 4.0}, {2.0, 2.0, 2.0, 5.0}), {true, true, false, true})
assert_bool_array_eq(f64x4.gt({1.0, 2.0, 3.0, 4.0}, {2.0, 2.0, 2.0, 5.0}), {false, false, true, false})
assert_bool_array_eq(f64x4.ge({1.0, 2.0, 3.0, 4.0}, {2.0, 2.0, 2.0, 5.0}), {false, true, true, false})

assert_array_eq(i32x4.splat(7), {7, 7, 7, 7})
assert_array_eq(i32x4.add({1, 2, 3, 4}, {10, 20, 30, 40}), {11, 22, 33, 44})
assert_array_eq(i32x4.sub({10, 20, 30, 40}, {1, 2, 3, 4}), {9, 18, 27, 36})
assert_array_eq(i32x4.mul({1, 2, 3, 4}, {5, 6, 7, 8}), {5, 12, 21, 32})
assert(i32x4.sum({1, 2, 3, 4}) == 10)
assert(i32x4.product({1, 2, 3, 4}) == 24)
assert_array_eq(i32x4.min({1, 9, 3, 7}, {2, 4, 8, 6}), {1, 4, 3, 6})
assert_array_eq(i32x4.max({1, 9, 3, 7}, {2, 4, 8, 6}), {2, 9, 8, 7})
assert_array_eq(i32x4.abs({-1, 2, -3, 4}), {1, 2, 3, 4})
assert_array_eq(i32x4.neg({-1, 2, -3, 4}), {1, -2, 3, -4})
assert_array_eq(i32x4.bitand({1, 3, 7, 15}, {1, 1, 3, 7}), {1, 1, 3, 7})
assert_array_eq(i32x4.bitor({1, 2, 4, 8}, {16, 32, 64, 128}), {17, 34, 68, 136})
assert_array_eq(i32x4.bitxor({1, 3, 7, 15}, {1, 1, 3, 7}), {0, 2, 4, 8})
assert_array_eq(i32x4.shl({1, 2, 3, 4}, 2), {4, 8, 12, 16})
assert_array_eq(i32x4.shr({8, 16, 24, 32}, 2), {2, 4, 6, 8})
assert_bool_array_eq(i32x4.eq({1, 2, 3, 4}, {1, 0, 3, 5}), {true, false, true, false})
assert_bool_array_eq(i32x4.ne({1, 2, 3, 4}, {1, 0, 3, 5}), {false, true, false, true})
assert_bool_array_eq(i32x4.lt({1, 2, 3, 4}, {2, 2, 2, 5}), {true, false, false, true})
assert_bool_array_eq(i32x4.le({1, 2, 3, 4}, {2, 2, 2, 5}), {true, true, false, true})
assert_bool_array_eq(i32x4.gt({1, 2, 3, 4}, {2, 2, 2, 5}), {false, false, true, false})
assert_bool_array_eq(i32x4.ge({1, 2, 3, 4}, {2, 2, 2, 5}), {false, true, true, false})

local ok, err = pcall(f64x4.add, {1.0, 2.0, 3.0}, {4.0, 5.0, 6.0, 7.0})
assert(not ok)
assert(err:match("expected exactly 4 lanes"))

local ok2, err2 = pcall(i32x4.add, {1, 2, 3, 2147483648}, {0, 0, 0, 0})
assert(not ok2)
assert(err2:match("out of i32 range"))

print("math simd ok")
