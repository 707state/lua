assert(type(math) == "table")
assert(type(math.simd) == "table")

assert(math.abs(-42) == 42)
assert(math.sin(0) == 0.0)
assert(math.cos(0) == 1.0)
assert(math.sqrt(81) == 9.0)
assert(math.floor(3.75) == 3)
assert(math.ceil(3.25) == 4)
assert(math.tointeger(17.0) == 17)
assert(math.type(17) == "integer")
assert(math.type(17.5) == "float")
assert(math.max(1, 9, 3) == 9)
assert(math.min(1, 9, 3) == 1)

local m, e = math.frexp(12.0)
assert(m == 0.75 and e == 4)
assert(math.ldexp(m, e) == 12.0)

local ipart, fpart = math.modf(-3.25)
assert(ipart == -3 and fpart == -0.25)

local s1, s2 = math.randomseed(123, 456)
assert(s1 == 123 and s2 == 456)
local r = math.random()
assert(r >= 0.0 and r < 1.0)
assert(math.random(5, 5) == 5)

print("builtin math ok")
