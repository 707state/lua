project('lua', ['c', 'cpp'])
cmake = import('cmake')
raylib_var = cmake.subproject_options()
raylib_var.add_cmake_defines(
  {'PLATFORM': 'Desktop'},
)
raylib_var.add_cmake_defines({'OPENGL_VERSION': get_option('OPENGL')})
raylib_var.add_cmake_defines({'WITH_PIC': 'ON'})
raylib_var.add_cmake_defines({'BUILD_SHARED_LIBS': 'ON'})
raylib_var.add_cmake_defines({'CUSTOMIZE_BUILD': 'ON'})
raylib_var.add_cmake_defines({'SUPPORT_MODULE_RAUDIO': 'ON'})

raylib_proj = cmake.subproject('raylib', options: raylib_var)

raylib_dep = raylib_proj.dependency('raylib')
ffi_proj = subproject('libffi')
ffi_dep = ffi_proj.get_variable('ffi_lib')
add_project_link_arguments('-lm', language: 'c')
add_project_arguments(['-DLUA_USE_DLOPEN', '-DLUA_USE_POSIX'], language: ['c', 'cpp'])

if host_machine.system() == 'darwin'
  add_project_arguments(
    ['-framework', 'CoreFoundation', '-framework', 'Cocoa'],
    language: ['c', 'cpp'],
  )
endif

dependencies = []

endian_flag = host_machine.endian() == 'little' ? '-DFFI_LITTLE_ENDIAN' : '-DFFI_BIG_ENDIAN'

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
    # ffi
    'ffi/ast.cc',
    'ffi/ffi.cc',
    'ffi/ffilib.cc',
    'ffi/lib.cc',
    'ffi/lua.cc',
    'ffi/main.cc',
    'ffi/parser.cc',
    'ffi/util.cc',
  ],
  include_directories: include_directories('src/', 'ffi/'),
  dependencies: dependencies + raylib_dep,
  cpp_args: [
    endian_flag,
  ],
  link_with: ffi_dep,
)
dependencies += raylib_dep

lua_exe = executable(
  'lua',
  sources: ['src/lua.c'],
  link_with: [lua_lib],
  dependencies: [raylib_dep],
)
luac_exe = executable('luac', sources: ['src/luac.c'], link_with: lua_lib)

executable(
  'simple_game',
  sources: ['test/game.cc'],
  dependencies: dependencies,
)
