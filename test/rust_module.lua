package.cpath = "./target/debug/lib?.so;./target/debug/?.so;" .. package.cpath

local open_rust = assert(package.loadlib("./target/debug/librust_ffi.so", "luaopen_rust_ffi"))
local rust_from_loadlib = open_rust()

local rust = require("rust_ffi")

assert(rust_from_loadlib.add(20, 22) == 42)
assert(rust_from_loadlib.version() == "0.1.0")
assert(rust.add(20, 22) == 42)
assert(rust.factorial(5) == 120)
assert(rust.version() == "0.1.0")
assert(type(rust.simd) == "table")

print("rust_ffi module ok")
