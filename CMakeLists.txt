cmake_minimum_required(VERSION 3.30)
project(eggmock)

set(CMAKE_CXX_STANDARD 20)

add_subdirectory(mockturtle)

add_executable(eggmock main.cpp)
target_link_libraries(eggmock PUBLIC mockturtle)