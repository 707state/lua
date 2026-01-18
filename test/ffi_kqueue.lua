local thread = coroutine.create(function()
    for i = 0, 10 do
        print("i is: ", i)
        coroutine.yield(i)
    end
end)

while coroutine.status(thread) ~= "dead" do
    local _, x = coroutine.resume(thread)
    print("x is: ", x)
end
