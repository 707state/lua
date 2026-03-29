local path = "test/io_builtin_tmp.txt"

do
  local f = assert(io.open(path, "w"))
  assert(io.type(f) == "file")
  assert(f:write("alpha\n", 123, "\n"))
  assert(f:flush())
  assert(f:close())
  assert(io.type(f) == "closed file")
end

do
  local f = assert(io.open(path, "r"))
  local line = f:read("*l")
  assert(line == "alpha")
  local num = f:read("*n")
  assert(num == 123)
  assert(f:seek("set", 0) == 0)
  local all = f:read("*a")
  assert(all == "alpha\n123\n")
  assert(f:close())
end

do
  local f = assert(io.open(path, "r"))
  local iter = f:lines()
  assert(iter() == "alpha")
  assert(iter() == "123")
  assert(iter() == nil)
  assert(f:close())
end

do
  local seen = {}
  for line in io.lines(path) do
    seen[#seen + 1] = line
  end
  assert(#seen == 2 and seen[1] == "alpha" and seen[2] == "123")
end

do
  local f = assert(io.tmpfile())
  assert(f:write("xyz"))
  assert(f:seek("set", 0) == 0)
  assert(f:read(3) == "xyz")
  assert(f:close())
end

do
  local old_in = io.input()
  local old_out = io.output()
  local input_file = assert(io.open(path, "r"))
  local output_path = "test/io_builtin_tmp_out.txt"
  local output_file = assert(io.open(output_path, "w"))
  assert(io.input(input_file) == input_file)
  assert(io.output(output_file) == output_file)
  assert(io.read("*l") == "alpha")
  assert(io.write("beta\n"))
  assert(io.flush())
  assert(io.input(old_in) == old_in)
  assert(io.output(old_out) == old_out)
  assert(input_file:close())
  assert(output_file:close())
  local check = assert(io.open(output_path, "r"))
  assert(check:read("*a") == "beta\n")
  assert(check:close())
  assert(os.remove(output_path))
end

do
  local proc = assert(io.popen("printf 'popen-test'", "r"))
  assert(proc:read("*a") == "popen-test")
  local ok, exit_kind, code = proc:close()
  assert(ok == true and exit_kind == "exit" and code == 0)
end

assert(os.remove(path))
