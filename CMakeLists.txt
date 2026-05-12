cmake_minimum_required(VERSION 3.20)
project(lua_with_llvm LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(LLVM_TARGETS_TO_BUILD "Native" CACHE STRING "" FORCE)
set(LLVM_INCLUDE_TESTS OFF CACHE BOOL "" FORCE)
set(LLVM_INCLUDE_EXAMPLES OFF CACHE BOOL "" FORCE)
set(LLVM_INCLUDE_BENCHMARKS OFF CACHE BOOL "" FORCE)

add_subdirectory(subprojects/llvm-project-llvmorg-22.1.0/llvm llvm-build)

set(LLVM_SRC_INCLUDE_DIR
    ${CMAKE_CURRENT_SOURCE_DIR}/subprojects/llvm-project-llvmorg-22.1.0/llvm/include
)

set(LUA_SRC
    src/lapi.cc
    src/lauxlib.cc
    src/lbaselib.cc
    src/lcode.cc
    src/ldblib.cc
    src/ldebug.cc
    src/ldo.cc
    src/ldump.cc
    src/lfunc.cc
    src/lgc.cc
    src/linit.cc
    src/liolib.cc
    src/llex.cc
    src/lmathlib.cc
    src/lmem.cc
    src/loadlib.cc
    src/lobject.cc
    src/lopcodes.cc
    src/loslib.cc
    src/lparser.cc
    src/lstate.cc
    src/lstring.cc
    src/lstrlib.cc
    src/ltable.cc
    src/ltablib.cc
    src/ltm.cc
    src/lundump.cc
    src/lvm.cc
    src/lzio.cc
    src/print.cc
)

add_library(lua_llvm_config INTERFACE)

target_include_directories(lua_llvm_config
    INTERFACE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
        ${LLVM_SRC_INCLUDE_DIR}
        ${LLVM_BINARY_DIR}/include
)

target_compile_definitions(lua_llvm_config
    INTERFACE
        ${LLVM_DEFINITIONS}
)

target_link_libraries(lua_llvm_config
    INTERFACE
        LLVMSupport
        LLVMCore
        LLVMIRReader
        LLVMOrcJIT
        LLVMRuntimeDyld
        LLVMMC
        LLVMExecutionEngine
)

add_library(lua_static STATIC ${LUA_SRC})

target_link_libraries(lua_static
    PUBLIC
        lua_llvm_config
)

add_executable(lua src/lua.cc)

target_link_libraries(lua
    PRIVATE
        lua_static
        lua_llvm_config
)

add_executable(luac src/luac.cc)

target_link_libraries(luac
    PRIVATE
        lua_static
        lua_llvm_config
)
