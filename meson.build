project('lua', ['c', 'cpp'])

lua_src = [
    'src/lapi.c',
    'src/lauxlib.c',
    'src/lbaselib.c',
    'src/lcode.c',
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
    'src/lvm.c',
    'src/lzio.c',
    'src/print.c',
]
lua_header = include_directories('src/')
lua_shared_target = library('lua_static', sources: lua_src, include_directories: lua_header)
lua_exe = executable(
    'lua',
    sources: 'src/lua.c',
    link_with: lua_shared_target,
    include_directories: lua_header,
)
luac_exe = executable(
    'luac',
    sources: 'src/luac.c',
    link_with: lua_shared_target,
    include_directories: lua_header,
)
