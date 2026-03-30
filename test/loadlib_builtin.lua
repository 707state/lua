local module_path = "test/loadlib_tmp_module.lua"

do
  local f = assert(io.open(module_path, "w"))
  f:write([[
local M = {}
M.value = 42
return M
]])
  f:close()
end

local original_path = package.path
package.path = "./?.lua;" .. original_path

local found = assert(package.searchpath("test/loadlib_tmp_module", package.path))
assert(found == "./test/loadlib_tmp_module.lua" or found == "test/loadlib_tmp_module.lua")

package.preload["builtin_preload_mod"] = function(name, where)
  return {
    name = name,
    where = where,
  }
end

local preload_mod, preload_where = require("builtin_preload_mod")
assert(preload_mod.name == "builtin_preload_mod")
assert(preload_where == ":preload:")

local mod, loader_data = require("test/loadlib_tmp_module")
assert(mod.value == 42)
assert(loader_data == "./test/loadlib_tmp_module.lua" or loader_data == "test/loadlib_tmp_module.lua")

local mod2, loader_data2 = require("test/loadlib_tmp_module")
assert(mod2 == mod)
assert(loader_data2 == nil)

assert(package.loaded["test/loadlib_tmp_module"] == mod)
assert(type(package.searchers) == "table" and #package.searchers >= 2)
assert(type(package.config) == "string")

local ok_missing, err_missing = pcall(function()
  require("module_that_should_not_exist_xyz")
end)
assert(ok_missing == false)
assert(type(err_missing) == "string" and err_missing:match("module 'module_that_should_not_exist_xyz' not found"))

local fn, err, where = package.loadlib("/definitely/not/found/library", "luaopen_nowhere")
assert(fn == nil)
assert(type(err) == "string")
assert(where == "open" or where == "absent")

package.path = original_path
assert(os.remove(module_path))

print("builtin loadlib ok")
