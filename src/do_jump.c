#include <setjmp.h>
#include <stddef.h>

#include "lprefix.h"
#include "lua.h"
#include "ldo.h"
#include "lstate.h"

struct lua_longjmp {
  struct lua_longjmp *previous;
  jmp_buf b;
  volatile TStatus status;
};

size_t luaRS_longjmp_size(void) {
  return sizeof(struct lua_longjmp);
}

size_t luaRS_longjmp_align(void) {
  return _Alignof(struct lua_longjmp);
}

void luaRS_longjmp_set_previous(struct lua_longjmp *lj, struct lua_longjmp *previous) {
  lj->previous = previous;
}

struct lua_longjmp *luaRS_longjmp_get_previous(struct lua_longjmp *lj) {
  return lj->previous;
}

void luaRS_longjmp_set_status(struct lua_longjmp *lj, TStatus status) {
  lj->status = status;
}

TStatus luaRS_longjmp_get_status(struct lua_longjmp *lj) {
  return lj->status;
}

int luaRS_longjmp_try(lua_State *L, struct lua_longjmp *lj, Pfunc f, void *ud) {
  if (setjmp(lj->b) == 0) {
    f(L, ud);
    return 0;
  }
  return 1;
}

void luaRS_longjmp_throw(struct lua_longjmp *lj) {
  longjmp(lj->b, 1);
}
