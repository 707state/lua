assert(type(string) == "table")

assert(string.len("hello") == 5)
assert(("hello"):len() == 5)

assert(string.sub("abcdef", 2, 4) == "bcd")
assert(string.sub("abcdef", -3, -1) == "def")
assert(("abcdef"):sub(4, 2) == "")

assert(string.reverse("stressed") == "desserts")
assert(string.lower("AbC123") == "abc123")
assert(string.upper("AbC123") == "ABC123")

assert(string.rep("ha", 3) == "hahaha")
assert(string.rep("ha", 3, ",") == "ha,ha,ha")
assert(string.rep("ha", 0) == "")

local a, b, c = string.byte("ABC", 1, 3)
assert(a == 65 and b == 66 and c == 67)
assert(select("#", string.byte("ABC", 4, 3)) == 0)
assert(string.byte("ABC", -1) == 67)

assert(string.char(65, 66, 67) == "ABC")
local ok_char, err_char = pcall(function()
  string.char(256)
end)
assert(ok_char == false)
assert(type(err_char) == "string" and err_char:match("value out of range"))

assert("40" + "2" == 42)
assert("40" - "2" == 38)
assert("6" * "7" == 42)
assert("43" % "10" == 3)
assert("4" ^ "3" == 64.0)
assert("7" / "2" == 3.5)
assert("7" // "2" == 3)
assert(-"12" == -12)

local fallback = setmetatable({}, {
  __add = function(lhs, rhs)
    return "fallback-add"
  end,
})
assert("abc" + fallback == "fallback-add")

local ok_add, err_add = pcall(function()
  return "abc" + {}
end)
assert(ok_add == false)
assert(type(err_add) == "string" and err_add:match("attempt to add"))

local dumped = string.dump(function(x)
  return x + 1
end)
assert(type(dumped) == "string" and #dumped > 0)
local restored = assert(load(dumped))
assert(restored(41) == 42)

local i, j = string.find("hello world", "world")
assert(i == 7 and j == 11)

local pi, pj = string.find("a.c", ".", 1, true)
assert(pi == 2 and pj == 2)

local ci, cj, cap = string.find("foo123bar", "(%d+)")
assert(ci == 4 and cj == 6 and cap == "123")

assert(string.match("abc123def", "(%a+)(%d+)(%a+)") == "abc")
local letters, digits, tail = string.match("abc123def", "(%a+)(%d+)(%a+)")
assert(letters == "abc" and digits == "123" and tail == "def")
assert(string.match("abc", "^%a+$") == "abc")
assert(string.match("(abc)", "%b()") == "(abc)")
assert(string.match("hello, world", "%f[%a]world") == "world")
local bi, bj, bcap = string.find("banana", "(an)%1")
assert(bi == 2 and bj == 5 and bcap == "an")

local words = {}
for w in string.gmatch("one two three", "%a+") do
  words[#words + 1] = w
end
assert(table.concat(words, ",") == "one,two,three")

local replaced, count = string.gsub("hello 123 world 456", "%d+", "#")
assert(replaced == "hello # world #" and count == 2)

local mapped, mapped_count = string.gsub("ab12cd34", "(%d+)", function(d)
  return "[" .. d .. "]"
end)
assert(mapped == "ab[12]cd[34]" and mapped_count == 2)

local table_replaced, table_count = string.gsub("hello world", "%a+", {
  hello = "hi",
  world = "earth",
})
assert(table_replaced == "hi earth" and table_count == 2)

local pos_replaced = string.gsub("ab", ".", "<%0>")
assert(pos_replaced == "<a><b>")

assert(string.format("hello %s %d", "lua", 55) == "hello lua 55")
assert(string.format("%.2f", 3.14159) == "3.14")
assert(string.format("%x", 255) == "ff")
assert(string.format("%c", 65) == "A")
assert(string.format("%q", "a\nb") == "\"a\\\nb\"")

local packed = string.pack("<i2I2c3", -7, 500, "xy")
local a1, a2, a3, nextpos = string.unpack("<i2I2c3", packed)
assert(a1 == -7 and a2 == 500 and a3 == "xy\0" and nextpos == #packed + 1)

local packed2 = string.pack(">s2z", "lua", "rocks")
local s1, s2, nextpos2 = string.unpack(">s2z", packed2)
assert(s1 == "lua" and s2 == "rocks" and nextpos2 == #packed2 + 1)

assert(string.packsize("<i2I2c3") == 7)

print("builtin string ok")
