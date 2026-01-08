project('lua', 'c')
cmake = import('cmake')
raylib_proj = cmake.subproject('raylib')
raylib_dep = raylib_proj.dependency('raylib')
add_project_arguments(['-DLUA_USE_DLOPEN','-DLUA_USE_POSIX'], language: ['c', 'cpp'])
lua_lib = static_library(
  'lua',
  sources: [
    'src/lapi.c',
    'src/lauxlib.c',
    'src/lbaselib.c',
    'src/lcode.c',
    'src/lcorolib.c',
    'src/lctype.c',
    'src/ldblib.c',
    'src/ldebug.c',
    'src/ldo.c',
    'src/ldump.c',
    'src/lfunc.c',
    'src/lgc.c',
    'src/linit.c',
    'src/liolib.c',
    'src/llex.c',
    'src/lmathlib.c',
    'src/lmem.c',
    'src/loadlib.c',
    'src/lobject.c',
    'src/lopcodes.c',
    'src/loslib.c',
    'src/lparser.c',
    'src/lstate.c',
    'src/lstring.c',
    'src/lstrlib.c',
    'src/ltable.c',
    'src/ltablib.c',
    'src/ltm.c',
    'src/lundump.c',
    'src/lutf8lib.c',
    'src/lvm.c',
    'src/lzio.c',
  ],
)

lua_exe = executable('lua', sources: ['src/lua.c'], link_with: lua_lib)
luac_exe = executable('luac', sources: ['src/luac.c'], link_with: lua_lib)

lua_raylib_lib = library(
  'raylib',
  sources: ['raylib/lraylib.c'],
  link_with: lua_lib,
  dependencies: [raylib_dep],
  include_directories: include_directories('src/'),
)
