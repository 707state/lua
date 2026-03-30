assert(type(coroutine) == "table")

local current, ismain = coroutine.running()
assert(type(current) == "thread")
assert(ismain == true)
assert(coroutine.status(current) == "running")
assert(coroutine.isyieldable(current) == false)
assert(coroutine.isyieldable() == false)

local co = coroutine.create(function(a, b)
  assert(coroutine.running() ~= current)
  assert(coroutine.isyieldable())
  assert(coroutine.status(coroutine.running()) == "running")
  coroutine.yield("yielded", a + b)
  return a * b, "done"
end)

assert(coroutine.status(co) == "suspended")

local ok, tag, sum = coroutine.resume(co, 20, 22)
assert(ok == true and tag == "yielded" and sum == 42)
assert(coroutine.status(co) == "suspended")

local ok2, product, done = coroutine.resume(co)
assert(ok2 == true and product == 440 and done == "done")
assert(coroutine.status(co) == "dead")

local ok3, err3 = coroutine.resume(co)
assert(ok3 == false)
assert(type(err3) == "string" and err3:match("dead coroutine"))

local wrapped = coroutine.wrap(function(x)
  coroutine.yield(x + 1)
  return x + 2
end)

assert(wrapped(41) == 42)
assert(wrapped(41) == 43)

local wrapped_err = coroutine.wrap(function()
  error("wrapped boom")
end)

local ok_wrap, err_wrap = pcall(wrapped_err)
assert(ok_wrap == false)
assert(type(err_wrap) == "string")
assert(err_wrap:match("wrapped boom"))
assert(err_wrap:match("test/coroutine_builtin%.lua"))

local bad_arg_ok, bad_arg_err = pcall(function()
  coroutine.resume({})
end)
assert(bad_arg_ok == false)
assert(type(bad_arg_err) == "string" and bad_arg_err:match("thread"))

local close_dead = coroutine.create(function()
end)
assert(coroutine.resume(close_dead) == true)
assert(coroutine.status(close_dead) == "dead")
assert(coroutine.close(close_dead) == true)

local close_yielded = coroutine.create(function()
  coroutine.yield("pause")
end)
local ok4, pause = coroutine.resume(close_yielded)
assert(ok4 == true and pause == "pause")
assert(coroutine.status(close_yielded) == "suspended")
assert(coroutine.close(close_yielded) == true)
assert(coroutine.status(close_yielded) == "dead")

local parent = coroutine.create(function()
  local self = coroutine.running()
  local child = coroutine.create(function(target)
    local ok, err = pcall(function()
      coroutine.close(target)
    end)
    assert(ok == false)
    assert(type(err) == "string")
    assert(err:match("cannot close a normal coroutine"))
    coroutine.yield("child-yield")
  end)
  local ok, value = coroutine.resume(child, self)
  assert(ok == true and value == "child-yield")
end)

assert(coroutine.resume(parent) == true)
assert(coroutine.status(parent) == "dead")
assert(coroutine.close(parent) == true)

local close_main_ok, close_main_err = pcall(function()
  coroutine.close(current)
end)
assert(close_main_ok == false)
assert(type(close_main_err) == "string")
assert(close_main_err:match("cannot close main thread"))

print("builtin coroutine ok")
