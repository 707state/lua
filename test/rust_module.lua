local rust = require("rust_ffi")

assert(rust.add(20, 22) == 42)
assert(rust.factorial(5) == 120)
assert(rust.version() == "0.1.0")

print("rust_ffi module ok")
