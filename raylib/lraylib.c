#include "lauxlib.h"
#include "lua.h"
#include <stdio.h>

// 一个导出给 Lua 的函数：rl.hello()
static int l_hello(lua_State *L) {
  (void)L;
  printf("[rl] hello from C!\n");
  return 0; // Lua 返回值个数
}

// 函数表
static const luaL_Reg rl_funcs[] = {{"hello", l_hello}, {NULL, NULL}};

// Lua 在 require("rl") 时会找 luaopen_rl
int luaopen_libraylib(lua_State *L) {
  luaL_newlib(L, rl_funcs); // push 一个 table
  return 1;                 // 返回这个 table
}
