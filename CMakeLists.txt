cmake_minimum_required(VERSION 4.3)
project(luma)

set(CMAKE_CXX_STANDARD 20)

file(GLOB_RECURSE COMPILE_SOURCES CONFIGURE_DEPENDS
        src/compile/*.cpp
)

file(GLOB_RECURSE RUN_SOURCES CONFIGURE_DEPENDS
        src/run/*.cpp
)

add_executable(compile ${COMPILE_SOURCES}
        src/compile/parser/token_stream/TokenStream.cpp
        src/compile/parser/token_stream/TokenStream.h
        src/compile/semantic_analysis/SemanticAnalysis.cpp
        src/compile/semantic_analysis/SemanticAnalysis.h
        src/compile/overloaded_visitor/overloaded_visitor.h)
add_executable(run ${RUN_SOURCES})

target_include_directories(compile PRIVATE src)
target_include_directories(run PRIVATE src)

target_compile_options(compile PRIVATE -Wall -Wextra)
target_compile_options(run PRIVATE -Wall -Wextra)