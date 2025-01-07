cmake_minimum_required(VERSION 3.5)
project(eggmock)

set(CMAKE_CXX_STANDARD 20)

add_subdirectory(mockturtle)

add_library(eggmock INTERFACE
        examples/mig_rewrite/main.cpp)
target_include_directories(eggmock INTERFACE include)
target_link_libraries(eggmock INTERFACE mockturtle)

add_subdirectory(examples)
