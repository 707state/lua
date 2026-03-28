#define LUA_CORE

#include "lua.h"
#include "lapi.h"
#include "ldebug.h"
#include "lobject.h"
#include "lstate.h"

const Proto *rust_luavm_top_proto(lua_State *L) {
  return getproto(s2v(L->top.p - 1));
}

int rust_luavm_getfuncline(const Proto *f, int pc) {
  return luaG_getfuncline(f, pc);
}

const TString *rust_luavm_eventname(lua_State *L, int idx) {
  return G(L)->tmname[idx];
}
