-- ==========================================================================
-- string.format comprehensive tests
-- Covers: %d %i %u %o %x %X %c %s %f %e %E %g %G %a %A %p %q %%
--         width, precision, flags (- + 0 # space), edge cases
-- ==========================================================================

-- ── basic specifiers ─────────────────────────────────────────────────────

-- %d / %i  signed decimal integer
assert(string.format("%d", 42) == "42")
assert(string.format("%i", 42) == "42")
assert(string.format("%d", -42) == "-42")
assert(string.format("%d", 0) == "0")
assert(string.format("%d", 2147483647) == "2147483647")
assert(string.format("%d", -2147483648) == "-2147483648")

-- %u  unsigned decimal
assert(string.format("%u", 0) == "0")
assert(string.format("%u", 42) == "42")
assert(string.format("%u", 4294967295) == "4294967295")

-- %o  octal
assert(string.format("%o", 8) == "10")
assert(string.format("%o", 255) == "377")
assert(string.format("%o", 0) == "0")

-- %x / %X  hexadecimal
assert(string.format("%x", 255) == "ff")
assert(string.format("%X", 255) == "FF")
assert(string.format("%x", 0) == "0")
assert(string.format("%x", 16) == "10")
assert(string.format("%X", 4095) == "FFF")

-- %c  character
assert(string.format("%c", 65) == "A")
assert(string.format("%c", 97) == "a")
assert(string.format("%c", 48) == "0")

-- %s  string
assert(string.format("%s", "hello") == "hello")
assert(string.format("%s", "") == "")
assert(string.format("%s", "a\0b") == "a\0b")  -- embedded NUL in simple %s

-- %%  literal percent
assert(string.format("%%") == "%")
assert(string.format("100%%") == "100%")
assert(string.format("%%%%") == "%%")

-- ── width and alignment ──────────────────────────────────────────────────

-- right-aligned (default)
assert(string.format("%5d", 42) == "   42")
assert(string.format("%5d", -42) == "  -42")
assert(string.format("%10s", "hi") == "        hi")

-- left-aligned with '-'
assert(string.format("%-5d", 42) == "42   ")
assert(string.format("%-5d", -42) == "-42  ")
assert(string.format("%-10s", "hi") == "hi        ")

-- zero-padded
assert(string.format("%05d", 42) == "00042")
assert(string.format("%05d", -42) == "-0042")
assert(string.format("%08x", 255) == "000000ff")

-- ── sign flags ───────────────────────────────────────────────────────────

-- '+' forces sign
assert(string.format("%+d", 42) == "+42")
assert(string.format("%+d", -42) == "-42")
assert(string.format("%+d", 0) == "+0")

-- ' ' space before positive
assert(string.format("% d", 42) == " 42")
assert(string.format("% d", -42) == "-42")

-- ── '#' alternate form ───────────────────────────────────────────────────

-- octal with '#'
assert(string.format("%#o", 8) == "010")
assert(string.format("%#o", 0) == "0")

-- hex with '#'
assert(string.format("%#x", 255) == "0xff")
assert(string.format("%#X", 255) == "0XFF")
assert(string.format("%#x", 0) == "0")

-- ── precision ────────────────────────────────────────────────────────────

-- string precision truncates
assert(string.format("%.3s", "hello") == "hel")
assert(string.format("%.10s", "hi") == "hi")
assert(string.format("%.0s", "hello") == "")

-- combined width + precision for strings
assert(string.format("%10.3s", "hello") == "       hel")
assert(string.format("%-10.3s", "hello") == "hel       ")

-- ── floating point %f ────────────────────────────────────────────────────

assert(string.format("%f", 3.14) == "3.140000")
assert(string.format("%.2f", 3.14159) == "3.14")
assert(string.format("%.0f", 3.7) == "4")
assert(string.format("%f", 0.0) == "0.000000")
assert(string.format("%f", -0.0) == "-0.000000")
assert(string.format("%.1f", 99.95) == "100.0")
assert(string.format("%+.1f", 3.14) == "+3.1")
assert(string.format("% .1f", 3.14) == " 3.1")

-- ── scientific notation %e / %E ──────────────────────────────────────────

assert(string.format("%e", 100000.0) == "1.000000e+05")
assert(string.format("%E", 100000.0) == "1.000000E+05")
assert(string.format("%.2e", 0.00123) == "1.23e-03")
assert(string.format("%.0e", 100.0) == "1e+02")
assert(string.format("%e", 0.0) == "0.000000e+00")

-- ── general float %g / %G ────────────────────────────────────────────────

assert(string.format("%g", 100000.0) == "100000")
assert(string.format("%g", 1000000.0) == "1e+06")
assert(string.format("%g", 0.00001) == "1e-05")
assert(string.format("%g", 0.0001) == "0.0001")
assert(string.format("%g", 3.14) == "3.14")
assert(string.format("%g", 0.0) == "0")
assert(string.format("%.2g", 3.14159) == "3.1")
assert(string.format("%.1g", 3.14) == "3")
assert(string.format("%G", 1e10) == "1E+10")

-- ── hex float %a / %A ────────────────────────────────────────────────────

-- basic hex float
do
  local s = string.format("%a", 1.0)
  assert(s:match("^0x") or s:match("^0X"), "expected hex float prefix, got: " .. s)
  assert(s:match("[pP]"), "expected exponent marker 'p', got: " .. s)
end

do
  local s = string.format("%A", 1.0)
  assert(s:match("^0X"), "expected uppercase hex float prefix, got: " .. s)
  assert(s:match("P"), "expected uppercase exponent marker 'P', got: " .. s)
end

-- hex float zero
do
  local s = string.format("%a", 0.0)
  assert(s:match("0x0") or s:match("0x0p"), "expected hex zero, got: " .. s)
end

-- hex float with precision
do
  local s = string.format("%.4a", 1.0)
  assert(s:match("%.%x%x%x%x"), "expected 4 hex digits after dot, got: " .. s)
end

-- hex float negative
do
  local s = string.format("%a", -1.5)
  assert(s:sub(1, 1) == "-", "expected negative sign, got: " .. s)
end

-- ── pointer %p ───────────────────────────────────────────────────────────

-- %p on a table should produce a non-empty string (address)
do
  local t = {}
  local s = string.format("%p", t)
  assert(#s > 0, "expected non-empty pointer string")
  -- should look like a hex address (0x...) or at least contain hex digits
  assert(s:match("0x") or s:match("%x+"), "expected hex address, got: " .. s)
end

-- %p on nil pointer
do
  local s = string.format("%p", nil)
  assert(s == "(null)" or s:match("nil") or s:match("0x0"),
    "expected null representation, got: " .. s)
end

-- ── quoted string %q ─────────────────────────────────────────────────────

assert(string.format("%q", "hello") == '"hello"')
assert(string.format("%q", 'say "hi"') == '"say \\"hi\\""')
assert(string.format("%q", "a\nb") == '"a\\\nb"')
assert(string.format("%q", "a\0b") == '"a\\0b"')
assert(string.format("%q", "\t\r") == '"\\\t\\\r"' or
       string.format("%q", "\t\r"):match('^".*"$'))

-- %q with numbers
assert(string.format("%q", 42) == "42")
assert(string.format("%q", true) == "true")
assert(string.format("%q", false) == "false")
assert(string.format("%q", nil) == "nil")

-- ── multiple arguments ───────────────────────────────────────────────────

assert(string.format("%s = %d", "x", 42) == "x = 42")
assert(string.format("%d + %d = %d", 1, 2, 3) == "1 + 2 = 3")
assert(string.format("[%05d|%-8s|%+.2f]", 7, "lua", 3.14) == "[00007|lua     |+3.14]")

-- ── edge cases ───────────────────────────────────────────────────────────

-- empty format string
assert(string.format("") == "")

-- format string with no specifiers
assert(string.format("hello world") == "hello world")

-- large integers
assert(string.format("%d", 9007199254740992) == "9007199254740992")
assert(string.format("%d", -9007199254740992) == "-9007199254740992")

-- special floats
assert(string.format("%f", 1/0) == "inf" or string.format("%f", 1/0) == "Inf"
       or string.format("%f", 1/0):match("[Ii]nf"))
assert(string.format("%f", -1/0) == "-inf" or string.format("%f", -1/0) == "-Inf"
       or string.format("%f", -1/0):match("%-[Ii]nf"))
assert(string.format("%f", 0/0) == "nan" or string.format("%f", 0/0) == "NaN"
       or string.format("%f", 0/0) == "-nan"
       or string.format("%f", 0/0):match("[Nn][Aa][Nn]"))

-- error: missing argument
do
  local ok, err = pcall(string.format, "%d%d", 1)
  assert(not ok, "expected error for missing argument")
end

-- error: invalid specifier
do
  local ok, err = pcall(string.format, "%y", 1)
  assert(not ok, "expected error for invalid specifier")
end

-- ── width + zero-pad + sign combinations ─────────────────────────────────

assert(string.format("%+05d", 42) == "+0042")
assert(string.format("%+05d", -42) == "-0042")
assert(string.format("% 05d", 42) == " 0042")
assert(string.format("%08.2f", 3.14) == "00003.14")

-- ── hex with zero-pad and width ──────────────────────────────────────────

assert(string.format("%#010x", 255) == "0x000000ff")
assert(string.format("%#010X", 255) == "0X000000FF")

print("string.format tests ok")
