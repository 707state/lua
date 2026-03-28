package.cpath = "./target/debug/lib?;" .. package.cpath

local open_math_rs = assert(package.loadlib("./target/debug/libmath_rs", "luaopen_math_rs"))
local math_rs_from_loadlib = open_math_rs()

local math_rs = require("math_rs")

assert(math_rs_from_loadlib.add(20, 22) == 42)
assert(math_rs_from_loadlib.version() == "0.1.0")
assert(math_rs.add(20, 22) == 42)
assert(math_rs.factorial(5) == 120)
assert(math_rs.version() == "0.1.0")
assert(type(math_rs.simd) == "table")

print("math_rs module ok")
