 local function inner()
    error("oops from inner")
  end

  local function outer()
    print("before inner")
    inner()
    print("after inner") -- 不会执行
  end

  local ok, err = pcall(outer)
  print("ok =", ok)
  print("err =", err)

