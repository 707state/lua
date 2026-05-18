/*
** LLVM ORC JIT support for the Lua VM
** See Copyright Notice in lua.h
*/

#ifndef ljit_h
#define ljit_h

#include "llimits.h"
#include "lstate.h"

LUAI_FUNC int luaJIT_execute (lua_State *L);

#endif
