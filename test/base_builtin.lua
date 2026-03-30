assert(_G == _G._G)
assert(_VERSION == "Lua 5.5")

local n = tonumber("123")
assert(n == 123)
assert(tonumber("ff", 16) == 255)
assert(tonumber("101", 2) == 5)
assert(tonumber("  -1f  ", 16) == -31)
assert(tonumber("not-a-number") == nil)

local ok_base, err_base = pcall(function()
  tonumber("10", 37)
end)
assert(ok_base == false and err_base:match("base out of range"))

assert(type(123) == "number")
assert(tostring(true) == "true")
assert(rawequal({}, {}) == false)

local t = setmetatable({ x = 1 }, { __metatable = "locked" })
assert(getmetatable(t) == "locked")
local ok_setmt, err_setmt = pcall(function()
  setmetatable(t, {})
end)
assert(ok_setmt == false and err_setmt:match("cannot change a protected metatable"))

local plain = {}
assert(setmetatable(plain, nil) == plain)
local mt = {}
assert(setmetatable(plain, mt) == plain)
assert(getmetatable(plain) == mt)

local raw = {}
rawset(raw, "k", 42)
assert(rawget(raw, "k") == 42)
assert(rawlen("hello") == 5)

local key, value = next({ a = 1 }, nil)
assert(key == "a" and value == 1)

local items = {}
for i, v in ipairs({ 10, 20, 30 }) do
  items[#items + 1] = i .. ":" .. v
end
assert(table.concat(items, ",") == "1:10,2:20,3:30")

local custom_pairs_obj = setmetatable({}, {
  __pairs = function(self)
    local done = false
    local function iter(_, last)
      if done then
        return nil
      end
      done = true
      return "only", 99
    end
    return iter, self, nil, "close-token"
  end,
})
local iter, state, init, close_token = pairs(custom_pairs_obj)
assert(type(iter) == "function" and state == custom_pairs_obj and init == nil and close_token == "close-token")
local pk, pv = iter(state, init)
assert(pk == "only" and pv == 99)

local f = assert(load("return ... + 1", "chunk", "t"))
assert(f(41) == 42)

local env = { value = 7 }
local f_env = assert(load("return value * 2", "envchunk", "t", env))
assert(f_env() == 14)

local reader_parts = { "return ", "6", " * 7" }
local reader_index = 0
local f_reader = assert(load(function()
  reader_index = reader_index + 1
  return reader_parts[reader_index]
end, "readerchunk", "t"))
assert(f_reader() == 42)

local bad_reader_fn, bad_reader_err = load(function()
  return {}
end)
assert(bad_reader_fn == nil and bad_reader_err:match("reader function must return a string"))

do
  local path = "test/base_tmp.lua"
  local h = assert(io.open(path, "w"))
  h:write("return function(x) return x + 5 end")
  h:close()

  local lf = assert(loadfile(path))
  local loaded_fn = lf()
  assert(loaded_fn(37) == 42)

  local res = dofile(path)
  assert(type(res) == "function" and res(1) == 6)
  assert(os.remove(path))
end

local ok_pcall, sum = pcall(function(a, b)
  return a + b
end, 19, 23)
assert(ok_pcall and sum == 42)

local ok_pcall2, err_pcall2 = pcall(function()
  error("boom", 0)
end)
assert(ok_pcall2 == false and err_pcall2:match("boom"))

local ok_xpcall, transformed = xpcall(function()
  error("kapow", 0)
end, function(err)
  return "wrapped:" .. err
end)
assert(ok_xpcall == false and transformed == "wrapped:kapow")

assert(select("#", "a", "b", "c") == 3)
assert(select(2, "a", "b", "c") == "b")
assert(select(-1, "a", "b", "c") == "c")

collectgarbage("collect")
assert(type(collectgarbage("count")) == "number")
assert(type(collectgarbage("isrunning")) == "boolean")
assert(collectgarbage("incremental") == "generational" or collectgarbage("incremental") == "incremental")

warn("base", " warning")
print("builtin base ok")
