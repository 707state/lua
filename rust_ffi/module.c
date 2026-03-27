#include "lua.h"

int rust_ffi_open(lua_State *L);

int luaopen_rust_ffi(lua_State *L) {
  return rust_ffi_open(L);
}
