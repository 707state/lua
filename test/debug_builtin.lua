assert(type(debug) == "table")
assert(type(debug.getregistry()) == "table")

local mt = {}
local obj = {}
assert(debug.setmetatable(obj, mt) == obj)
assert(debug.getmetatable(obj) == mt)

local f = assert(io.tmpfile())
local ok_uv = select(1, debug.getuservalue(f))
assert(ok_uv == nil)
assert(debug.setuservalue(f, { tag = "uv" }) == nil)
local ok_setuv, err_setuv = pcall(function()
  debug.setuservalue("not-ud", {})
end)
assert(ok_setuv == false and err_setuv:match("userdata"))
f:close()

local info = debug.getinfo(debug.getinfo, "Sn")
assert(type(info) == "table")
assert(info.what == "C")

local function local_probe()
  local value = 10
  local name, current = debug.getlocal(1, 1)
  assert(name == "value" and current == 10)
  assert(debug.setlocal(1, 1, 42) == "value")
  return value
end
assert(local_probe() == 42)

local function make_up()
  local captured = 5
  return function()
    return captured
  end
end

local up = make_up()
local up_name, up_value = debug.getupvalue(up, 1)
assert(up_name == "captured" and up_value == 5)
assert(debug.setupvalue(up, 1, 9) == "captured")
assert(up() == 9)

local a = make_up()
local b = make_up()
local id1 = debug.upvalueid(a, 1)
local id2 = debug.upvalueid(b, 1)
assert(id1 ~= id2)
debug.upvaluejoin(a, 1, b, 1)
assert(debug.upvalueid(a, 1) == debug.upvalueid(b, 1))

local hook_events = {}
local function hook(event, line)
  hook_events[#hook_events + 1] = event
end
debug.sethook(hook, "l", 0)
local x = 0
x = x + 1
x = x + 1
local hook_fn, mask, count = debug.gethook()
assert(hook_fn == hook and mask:match("l") and count == 0)
debug.sethook()
assert(debug.gethook() == nil)
assert(#hook_events > 0)

local tb = debug.traceback("dbg", 1)
assert(type(tb) == "string" and tb:match("stack traceback:"))

assert(type(debug.debug) == "function")

print("builtin debug ok")
