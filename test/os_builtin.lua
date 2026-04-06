assert(type(os.clock()) == "number" and os.clock() >= 0)

local has_shell = os.execute()
assert(type(has_shell) == "boolean")

local ok, what, code = os.execute("true")
assert(ok == true and what == "exit" and code == 0)

local fail, fail_what, fail_code = os.execute("exit 7")
assert(fail == nil and fail_what == "exit" and fail_code == 7)

assert(type(os.getenv("PATH")) == "string")
assert(os.getenv("__LUA_OS_RS_SHOULD_NOT_EXIST__") == nil)

local tmp = assert(os.tmpname())
assert(type(tmp) == "string" and #tmp > 0)

local f = assert(io.open(tmp, "w"))
f:write("hello")
f:close()

local renamed = tmp .. ".renamed"
assert(os.rename(tmp, renamed) == true)

local fh = assert(io.open(renamed, "r"))
assert(fh:read("*a") == "hello")
fh:close()

assert(os.remove(renamed) == true)

local now = os.time()
assert(type(now) == "number" or type(now) == "integer")

local utc_epoch = os.date("!%Y-%m-%d %H:%M:%S", 0)
print(utc_epoch)
assert(utc_epoch == "1970-01-01 00:00:00")

local parts = os.date("*t", now)
assert(type(parts) == "table")
assert(type(parts.year) == "number")
assert(type(parts.month) == "number")
assert(type(parts.day) == "number")

local roundtrip = os.time({
    year = parts.year,
    month = parts.month,
    day = parts.day,
    hour = parts.hour,
    min = parts.min,
    sec = parts.sec,
    isdst = parts.isdst,
})
assert(math.abs(os.difftime(roundtrip, now)) <= 1)
assert(os.difftime(10, 3) == 7)

local ok_invalid, err_invalid = pcall(function()
    print(os.date("%Q", 0))
end)
assert(not ok_invalid and err_invalid:match("invalid conversion specifier"))
