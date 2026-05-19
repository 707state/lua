project('luajit', 'c',
  version : '2.1',
  meson_version : '>=1.1.0',
  default_options : ['warning_level=1'])

cc = meson.get_compiler('c')
host_sys = host_machine.system()
host_cpu = host_machine.cpu_family()

majver = '2'
minver = '1'
abiver = '5.1'
mmversion = majver + '.' + minver
nodotabiver = '51'

prefix = get_option('prefix')
multilib = get_option('libdir').split('/')[-1]
ljlibdir = prefix / get_option('datadir') / ('luajit-' + mmversion)
pkgconf_libdir = get_option('libdir') / 'pkgconfig'

if host_cpu == 'x86_64'
  ljarch = 'x64'
elif host_cpu == 'x86'
  ljarch = 'x86'
elif host_cpu == 'aarch64'
  ljarch = 'arm64'
elif host_cpu == 'arm'
  ljarch = 'arm'
elif host_cpu.startswith('ppc')
  ljarch = 'ppc'
elif host_cpu == 'mips64'
  ljarch = 'mips64'
elif host_cpu == 'mips'
  ljarch = 'mips'
else
  error('Unsupported LuaJIT target architecture: ' + host_cpu)
endif

target_sys = host_sys
if host_sys == 'darwin'
  ljvm_mode = 'machasm'
elif host_sys == 'windows'
  ljvm_mode = 'peobj'
else
  ljvm_mode = 'elfasm'
endif

target_c_args = [
  '-D_FILE_OFFSET_BITS=64',
  '-D_LARGEFILE_SOURCE',
  '-U_FORTIFY_SOURCE',
  '-DLUAJIT_TARGET=LUAJIT_ARCH_' + ljarch,
  '-DLUA_MULTILIB="@0@"'.format(multilib),
  '-DLUA_LJDIR="@0@"'.format(ljlibdir),
]

if ljarch == 'x86'
  target_c_args += ['-march=i686', '-msse', '-msse2', '-mfpmath=sse']
endif

if host_sys == 'darwin'
  target_c_args += ['-DLUAJIT_UNWIND_EXTERNAL']
endif

if cc.has_argument('-fno-stack-protector')
  target_c_args += ['-fno-stack-protector']
endif

host_c_args = ['-I' + meson.current_source_dir() / 'src']
if host_sys != target_sys
  if target_sys == 'darwin'
    host_c_args += ['-DLUAJIT_OS=LUAJIT_OS_OSX']
  elif target_sys == 'linux'
    host_c_args += ['-DLUAJIT_OS=LUAJIT_OS_LINUX']
  elif target_sys == 'windows'
    host_c_args += ['-DLUAJIT_OS=LUAJIT_OS_WINDOWS']
  else
    host_c_args += ['-DLUAJIT_OS=LUAJIT_OS_OTHER']
  endif
endif

src_inc = include_directories('src')
build_inc = include_directories('.')
host_inc = include_directories('src', 'src/host', '.')

m_dep = cc.find_library('m', required : false)
dl_dep = dependency('', required : false)
if host_sys in ['linux', 'gnu', 'kfreebsd']
  dl_dep = cc.find_library('dl', required : false)
endif
target_deps = [m_dep, dl_dep]

minilua = executable('minilua',
  'src/host/minilua.c',
  c_args : host_c_args,
  dependencies : [m_dep],
  native : true)

git = find_program('git', required : false)
relver = ''
if git.found()
  git_relver = run_command(git, 'show', '-s', '--format=%ct', check : false)
  if git_relver.returncode() == 0
    relver = git_relver.stdout().strip()
  endif
endif
if relver == ''
  relver_fallback = run_command('cat', '.relver', check : false)
  relver = relver_fallback.returncode() == 0 ? relver_fallback.stdout().strip() : 'ROLLING'
endif
project_version = mmversion + '.' + relver

relver_conf = configuration_data()
relver_conf.set('RELVER', relver)
relver_txt = configure_file(
  input : 'meson/luajit_relver.txt.in',
  output : 'luajit_relver.txt',
  configuration : relver_conf)

luajit_h = custom_target('luajit.h',
  input : ['src/luajit_rolling.h', relver_txt],
  output : 'luajit.h',
  command : [minilua, files('src/host/genversion.lua'), '@INPUT0@', '@INPUT1@', '@OUTPUT@'],
  install : true,
  install_dir : get_option('includedir') / ('luajit-' + mmversion))

lj_arch_prefix = '#include "lj_arch.h"'
foreach define_arg : target_c_args
  lj_arch_prefix = '#define ' + define_arg.substring(2).replace('=', ' ') + '\n' + lj_arch_prefix
endforeach

dasm_flags = []
if host_machine.endian() == 'little'
  dasm_flags += ['-D', 'ENDIAN_LE']
else
  dasm_flags += ['-D', 'ENDIAN_BE']
endif

foreach item : [
  ['LJ_ARCH_BITS', '64', ['-D', 'P64']],
  ['LJ_HASJIT', '1', ['-D', 'JIT']],
  ['LJ_HASFFI', '1', ['-D', 'FFI']],
  ['LJ_DUALNUM', '1', ['-D', 'DUALNUM']],
  ['LJ_ARCH_HASFPU', '1', ['-D', 'FPU']],
  ['LJ_NO_UNWIND', '1', ['-D', 'NO_UNWIND']],
  ['LJ_ABI_PAUTH', '1', ['-D', 'PAUTH']],
  ['LJ_ABI_BRANCH_TRACK', '1', ['-D', 'BRANCH_TRACK']],
  ['LJ_ABI_SHADOW_STACK', '1', ['-D', 'SHADOW_STACK']],
  ['LJ_TARGET_MIPSR6', '1', ['-D', 'MIPSR6']],
  ['LJ_ARCH_SQRT', '1', ['-D', 'SQRT']],
  ['LJ_ARCH_ROUND', '1', ['-D', 'ROUND']],
  ['LJ_ARCH_PPC32ON64', '1', ['-D', 'GPR64']],
]
  if cc.get_define(item[0], prefix : lj_arch_prefix, args : target_c_args, include_directories : src_inc) == item[1]
    dasm_flags += item[2]
  endif
endforeach

if cc.get_define('LJ_ABI_SOFTFP', prefix : lj_arch_prefix, args : target_c_args, include_directories : src_inc) != '1'
  dasm_flags += ['-D', 'HFABI']
endif

arch_version = cc.get_define('LJ_ARCH_VERSION', prefix : lj_arch_prefix, args : target_c_args, include_directories : src_inc)
if arch_version == ''
  arch_version = '0'
endif
dasm_flags += ['-D', 'VER=' + arch_version]

if host_sys == 'windows'
  dasm_flags += ['-D', 'WIN']
endif

dasm_arch = ljarch
if ljarch == 'x64' and cc.get_define('LJ_FR2', prefix : lj_arch_prefix, args : target_c_args, include_directories : src_inc) != '1'
  dasm_arch = 'x86'
endif

buildvm_arch_h = custom_target('buildvm_arch.h',
  input : 'src/vm_' + dasm_arch + '.dasc',
  output : 'buildvm_arch.h',
  command : [minilua, files('dynasm/dynasm.lua'), dasm_flags, '-o', '@OUTPUT@', '@INPUT@'])

buildvm = executable('buildvm',
  [
    'src/host/buildvm.c',
    'src/host/buildvm_asm.c',
    'src/host/buildvm_peobj.c',
    'src/host/buildvm_lib.c',
    'src/host/buildvm_fold.c',
    buildvm_arch_h,
    luajit_h,
  ],
  include_directories : [host_inc],
  c_args : host_c_args,
  dependencies : target_deps,
  native : true)

ljlib_c = [
  'src/lib_base.c',
  'src/lib_math.c',
  'src/lib_bit.c',
  'src/lib_string.c',
  'src/lib_table.c',
  'src/lib_io.c',
  'src/lib_os.c',
  'src/lib_package.c',
  'src/lib_debug.c',
  'src/lib_jit.c',
  'src/lib_ffi.c',
  'src/lib_buffer.c',
]

lj_vm_s = custom_target('lj_vm.S',
  output : 'lj_vm.S',
  command : [buildvm, '-m', ljvm_mode, '-o', '@OUTPUT@'])

generated_headers = [
  custom_target('lj_bcdef.h', input : ljlib_c, output : 'lj_bcdef.h',
    command : [buildvm, '-m', 'bcdef', '-o', '@OUTPUT@', '@INPUT@']),
  custom_target('lj_ffdef.h', input : ljlib_c, output : 'lj_ffdef.h',
    command : [buildvm, '-m', 'ffdef', '-o', '@OUTPUT@', '@INPUT@']),
  custom_target('lj_libdef.h', input : ljlib_c, output : 'lj_libdef.h',
    command : [buildvm, '-m', 'libdef', '-o', '@OUTPUT@', '@INPUT@']),
  custom_target('lj_recdef.h', input : ljlib_c, output : 'lj_recdef.h',
    command : [buildvm, '-m', 'recdef', '-o', '@OUTPUT@', '@INPUT@']),
  custom_target('lj_folddef.h', input : 'src/lj_opt_fold.c', output : 'lj_folddef.h',
    command : [buildvm, '-m', 'folddef', '-o', '@OUTPUT@', '@INPUT@']),
]

vmdef_lua = custom_target('vmdef.lua',
  input : ljlib_c,
  output : 'vmdef.lua',
  command : [buildvm, '-m', 'vmdef', '-o', '@OUTPUT@', '@INPUT@'],
  install : true,
  install_dir : ljlibdir / 'jit')

ljcore_c = [
  'src/lj_assert.c',
  'src/lj_gc.c',
  'src/lj_err.c',
  'src/lj_char.c',
  'src/lj_bc.c',
  'src/lj_obj.c',
  'src/lj_buf.c',
  'src/lj_str.c',
  'src/lj_tab.c',
  'src/lj_func.c',
  'src/lj_udata.c',
  'src/lj_meta.c',
  'src/lj_debug.c',
  'src/lj_prng.c',
  'src/lj_state.c',
  'src/lj_dispatch.c',
  'src/lj_vmevent.c',
  'src/lj_vmmath.c',
  'src/lj_strscan.c',
  'src/lj_strfmt.c',
  'src/lj_strfmt_num.c',
  'src/lj_serialize.c',
  'src/lj_api.c',
  'src/lj_profile.c',
  'src/lj_lex.c',
  'src/lj_parse.c',
  'src/lj_bcread.c',
  'src/lj_bcwrite.c',
  'src/lj_load.c',
  'src/lj_ir.c',
  'src/lj_opt_mem.c',
  'src/lj_opt_fold.c',
  'src/lj_opt_narrow.c',
  'src/lj_opt_dce.c',
  'src/lj_opt_loop.c',
  'src/lj_opt_split.c',
  'src/lj_opt_sink.c',
  'src/lj_mcode.c',
  'src/lj_snap.c',
  'src/lj_record.c',
  'src/lj_crecord.c',
  'src/lj_ffrecord.c',
  'src/lj_asm.c',
  'src/lj_trace.c',
  'src/lj_gdbjit.c',
  'src/lj_ctype.c',
  'src/lj_cdata.c',
  'src/lj_cconv.c',
  'src/lj_ccall.c',
  'src/lj_ccallback.c',
  'src/lj_carith.c',
  'src/lj_clib.c',
  'src/lj_cparse.c',
  'src/lj_lib.c',
  'src/lj_alloc.c',
  'src/lib_aux.c',
  ljlib_c,
  'src/lib_init.c',
]

ljcore_sources = [lj_vm_s, generated_headers, luajit_h, vmdef_lua] + ljcore_c
ljcore_args = target_c_args + ['-fomit-frame-pointer']

luajit_static = static_library('luajit-' + abiver,
  ljcore_sources,
  include_directories : [build_inc, src_inc],
  c_args : ljcore_args,
  dependencies : target_deps,
  install : true)

luajit_shared = shared_library('luajit-' + abiver,
  ljcore_sources,
  include_directories : [build_inc, src_inc],
  c_args : ljcore_args,
  dependencies : target_deps,
  soversion : majver,
  darwin_versions : [majver + '.' + minver + '.255', majver + '.' + minver],
  install : true)

luajit_exe = executable('luajit',
  ['src/luajit.c', luajit_h, vmdef_lua],
  include_directories : [build_inc, src_inc],
  c_args : target_c_args,
  link_with : luajit_static,
  dependencies : target_deps,
  install : true)

install_headers(
  'src/lua.h',
  'src/lualib.h',
  'src/lauxlib.h',
  'src/luaconf.h',
  'src/lua.hpp',
  subdir : 'luajit-' + mmversion)

install_data(
  'src/jit/bc.lua',
  'src/jit/bcsave.lua',
  'src/jit/dump.lua',
  'src/jit/p.lua',
  'src/jit/v.lua',
  'src/jit/zone.lua',
  'src/jit/dis_x86.lua',
  'src/jit/dis_x64.lua',
  'src/jit/dis_arm.lua',
  'src/jit/dis_arm64.lua',
  'src/jit/dis_arm64be.lua',
  'src/jit/dis_ppc.lua',
  'src/jit/dis_mips.lua',
  'src/jit/dis_mipsel.lua',
  'src/jit/dis_mips64.lua',
  'src/jit/dis_mips64el.lua',
  'src/jit/dis_mips64r6.lua',
  'src/jit/dis_mips64r6el.lua',
  install_dir : ljlibdir / 'jit')

install_man('etc/luajit.1')

pkg = import('pkgconfig')
pkg.generate(luajit_shared,
  name : 'LuaJIT',
  description : 'Just-in-time compiler for Lua',
  url : 'https://luajit.org',
  version : project_version,
  filebase : 'luajit',
  subdirs : 'luajit-' + mmversion,
  libraries_private : target_deps,
  extra_cflags : [],
  install_dir : pkgconf_libdir)

test('basic', luajit_exe, args : ['-e', 'assert(jit.version:match("^LuaJIT 2%.1"))'])
