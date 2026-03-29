local utf8lib = utf8

local s = "A\xe4\xb8\xad\xf0\x9f\x98\x80"
assert(utf8lib.len(s) == 3)

local strict_invalid = string.char(0xED, 0xA0, 0x80)
local n, pos = utf8lib.len(strict_invalid)
assert(n == nil and pos == 1)
assert(utf8lib.len(strict_invalid, 1, -1, true) == 1)

local a, b, c = utf8lib.codepoint(s, 1, -1)
assert(a == 0x41 and b == 0x4E2D and c == 0x1F600)

assert(utf8lib.char(0x41, 0x4E2D, 0x1F600) == s)

local start_pos, end_pos = utf8lib.offset(s, 2)
assert(start_pos == 2 and end_pos == 4)
start_pos, end_pos = utf8lib.offset(s, 0, 3)
assert(start_pos == 2 and end_pos == 4)
assert(utf8lib.offset(s, 10) == nil)

local positions = {}
local codes = {}
for p, cp in utf8lib.codes(s) do
  positions[#positions + 1] = p
  codes[#codes + 1] = cp
end
assert(table.concat(positions, ",") == "1,2,5")
assert(codes[1] == 0x41 and codes[2] == 0x4E2D and codes[3] == 0x1F600)

local ok, err = pcall(function()
  for _ in utf8lib.codes(strict_invalid) do
  end
end)
assert(not ok and err:match("invalid UTF%-8 code"))

local lax_positions = {}
local lax_codes = {}
for p, cp in utf8lib.codes(strict_invalid, true) do
  lax_positions[#lax_positions + 1] = p
  lax_codes[#lax_codes + 1] = cp
end
assert(#lax_positions == 1 and lax_positions[1] == 1)
assert(#lax_codes == 1 and lax_codes[1] == 0xD800)

assert(("中"):match(utf8lib.charpattern) == "中")
assert(("A"):match(utf8lib.charpattern) == "A")
