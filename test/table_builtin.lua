assert(type(table) == "table")

local t = table.create(3, 0)
assert(#t == 0)

table.insert(t, "a")
table.insert(t, "c")
table.insert(t, 2, "b")
assert(table.concat(t, ",") == "a,b,c")

local removed = table.remove(t, 2)
assert(removed == "b")
assert(table.concat(t, ",") == "a,c")

local moved = table.move({10, 20, 30}, 1, 3, 2, {0, 0, 0, 0, 0})
assert(moved[1] == 0 and moved[2] == 10 and moved[3] == 20 and moved[4] == 30)

local packed = table.pack("x", nil, "z")
assert(packed.n == 3)
local a, b, c = table.unpack(packed, 1, packed.n)
assert(a == "x" and b == nil and c == "z")

local sortable = {4, 1, 3, 2}
table.sort(sortable)
assert(table.concat(sortable, ",") == "1,2,3,4")

table.sort(sortable, function(x, y) return x > y end)
assert(table.concat(sortable, ",") == "4,3,2,1")

assert(table.getn({11, 22, 33}) == 3)

print("builtin table ok")
