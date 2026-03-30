local obj = setmetatable({}, {
  __tostring = function()
    return "aux-ok"
  end,
})
assert(tostring(obj) == "aux-ok")

local ok_tostring, err_tostring = pcall(function()
  return tostring(setmetatable({}, {
    __tostring = function()
      return {}
    end,
  }))
end)
assert(ok_tostring == false and err_tostring:match("__tostring"))

local tb = debug.traceback("hello", 1)
assert(type(tb) == "string")
assert(tb:match("hello"))
assert(tb:match("stack traceback:"))

local path = "test/auxlib_tmp.lua"
do
  local f = assert(io.open(path, "wb"))
  f:write("\239\187\191return 42")
  f:close()
end

local f = assert(loadfile(path))
assert(f() == 42)
assert(os.remove(path))

local mod = {}
package.loaded.auxlib_tmp_mod = mod
package.loaded.auxlib_tmp_mod2 = nil
package.preload.auxlib_tmp_mod2 = function(name)
  return { loaded_name = name }
end
local loaded = require("auxlib_tmp_mod")
assert(loaded == mod)
local loaded2 = require("auxlib_tmp_mod2")
assert(loaded2.loaded_name == "auxlib_tmp_mod2")

local ok_badarg, err_badarg = pcall(function()
  string.byte()
end)
assert(ok_badarg == false and err_badarg:match("bad argument"))

warn("aux", "lib")
print("builtin auxlib ok")
