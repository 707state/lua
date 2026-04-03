-- ==========================================================================
-- tostring() number formatting tests
-- Covers: integer formatting, float formatting, round-trip fidelity,
--         special values (inf, nan, -0), decimal point guarantee
-- ==========================================================================

-- ── integer tostring ─────────────────────────────────────────────────────

assert(tostring(0) == "0")
assert(tostring(1) == "1")
assert(tostring(-1) == "-1")
assert(tostring(42) == "42")
assert(tostring(-42) == "-42")
assert(tostring(2147483647) == "2147483647")
assert(tostring(-2147483648) == "-2147483648")

-- large integers
assert(tostring(9007199254740992) == "9007199254740992")
assert(tostring(-9007199254740992) == "-9007199254740992")

-- min/max Lua integer
local maxint = math.maxinteger
local minint = math.mininteger
assert(type(tostring(maxint)) == "string")
assert(type(tostring(minint)) == "string")
assert(tonumber(tostring(maxint)) == maxint)
assert(tonumber(tostring(minint)) == minint)

-- ── float tostring ───────────────────────────────────────────────────────

-- simple floats
assert(tostring(0.0) == "0.0")
assert(tostring(1.0) == "1.0")
assert(tostring(-1.0) == "-1.0")
assert(tostring(3.14) == "3.14")
assert(tostring(0.5) == "0.5")
assert(tostring(0.1) == "0.1")

-- floats that look like integers must still have a decimal point
do
  local s = tostring(100.0)
  assert(s:find("%."), "float 100.0 should contain decimal point, got: " .. s)
end

do
  local s = tostring(1e10)
  -- either scientific notation or has a decimal point
  assert(s:find("[%.eE]"), "float 1e10 should contain '.' or 'e', got: " .. s)
end

-- ── round-trip fidelity ──────────────────────────────────────────────────
-- tostring(n) should produce a string that tonumber() converts back to the
-- exact same float value

local function check_roundtrip(n)
  local s = tostring(n)
  local back = tonumber(s)
  assert(back == n or (n ~= n and back ~= back),
    string.format("round-trip failed: tostring(%s) = %q, tonumber => %s",
      string.format("%.17g", n), s, tostring(back)))
end

check_roundtrip(0.0)
check_roundtrip(1.0)
check_roundtrip(-1.0)
check_roundtrip(0.1)
check_roundtrip(0.2)
check_roundtrip(0.3)
check_roundtrip(1/3)
check_roundtrip(math.pi)
check_roundtrip(1e-10)
check_roundtrip(1e10)
check_roundtrip(1e100)
check_roundtrip(1e-100)
check_roundtrip(1.7976931348623157e+308)  -- near max double
check_roundtrip(5e-324)                    -- min subnormal
check_roundtrip(2.2250738585072014e-308)  -- min normal

-- ── special float values ─────────────────────────────────────────────────

-- infinity
do
  local s = tostring(1/0)
  assert(s == "inf" or s == "Inf" or s:match("[Ii]nf"),
    "expected inf, got: " .. s)
end

do
  local s = tostring(-1/0)
  assert(s == "-inf" or s == "-Inf" or s:match("%-[Ii]nf"),
    "expected -inf, got: " .. s)
end

-- NaN
do
  local s = tostring(0/0)
  assert(s == "nan" or s == "NaN" or s == "-nan" or s:match("[Nn][Aa][Nn]"),
    "expected nan, got: " .. s)
end

-- negative zero
do
  local s = tostring(-0.0)
  -- Lua 5.5 may or may not show "-0.0"; just check it's valid
  local n = tonumber(s)
  assert(n == 0.0, "tostring(-0.0) should parse back to 0, got: " .. s)
end

-- ── concatenation uses tostring ──────────────────────────────────────────

assert(42 .. "" == "42")
assert(3.14 .. "" == "3.14")
assert(0.0 .. "" == "0.0")

-- ── number to string in table keys ───────────────────────────────────────

do
  local t = {}
  t[tostring(42)] = true
  assert(t["42"])
end

-- ── very small and very large ────────────────────────────────────────────

do
  local tiny = 1e-300
  local s = tostring(tiny)
  assert(tonumber(s) == tiny, "tiny float round-trip failed: " .. s)
end

do
  local huge = 1e300
  local s = tostring(huge)
  assert(tonumber(s) == huge, "huge float round-trip failed: " .. s)
end

print("tostring number tests ok")
