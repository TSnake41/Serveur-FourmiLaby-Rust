cmake_minimum_required(VERSION 3.10)

project(projetfourmis)

file(GLOB_RECURSE sources
  ${CMAKE_CURRENT_SOURCE_DIR}/*.c
  ${CMAKE_CURRENT_SOURCE_DIR}/*.cpp
)

add_library(projetfourmis ${sources})

install(TARGETS projetfourmis DESTINATION .)