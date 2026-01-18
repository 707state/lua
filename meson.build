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
if host_machine.system() == 'darwin'
  add_project_arguments(
    ['-framework', 'CoreFoundation', '-framework', 'Cocoa'],
    language: ['c', 'cpp'],
  )
endif

dependencies = []

dependencies += raylib_dep
executable(
  'simple_game',
  sources: ['src/game.cc'],
  dependencies: dependencies,
)
