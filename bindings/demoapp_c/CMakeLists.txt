cmake_minimum_required(VERSION 3.16)
project(demo_c C)

set(CMAKE_C_STANDARD 11)

link_directories(../../target/debug/)

add_executable(demo_c
        ../../bindings/limtrac.h
        main.c)

target_link_libraries(demo_c liblimtrac.so)
