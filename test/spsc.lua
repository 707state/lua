---@param prod thread
---@return any
function receive(prod)
    local status, val = coroutine.resume(prod)
    return val
end

function send(x)
    coroutine.yield(x)
end

function producer()
    return coroutine.create(function()
        while true do
            local x = io.read() -- produce new value
            send(x)
        end
    end)
end

function filter(prod)
    return coroutine.create(function()
        local line = 1
        while true do
            local x = receive(prod) -- get new value
            x = string.format("%5d %s", line, x)
            send(x)                 -- send it to consumer
            line = line + 1
        end
    end)
end

function consumer(prod)
    while true do
        local x = receive(prod) -- get new value
        io.write(x, "\n")       -- consume new value
    end
end

-- consumer(filter(producer()))

function printResult(a)
    for i, v in ipairs(a) do
        io.write(v, " ")
    end
    io.write("\n")
end

function permgen(a, n)
    if n == 0 then
        coroutine.yield(a)
    else
        for i = 1, n do
            -- put i-th element as the last one
            a[n], a[i] = a[i], a[n]
            -- generate all permutations of the other elements
            permgen(a, n - 1)
            -- restore i-th element
            a[n], a[i] = a[i], a[n]
        end
    end
end
---@param a table
---@return function
function perm(a)
    local n = table.getn(a)
    local co = coroutine.create(function() permgen(a, n) end)
    return function() -- iterator
        local code, res = coroutine.resume(co)
        return res
    end
end

for p in perm({ "a", "b", "c" }) do
    printResult(p)
end
