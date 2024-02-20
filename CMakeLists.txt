cmake_minimum_required(VERSION 3.16)

project(CEMCL VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_INCLUDE_CURRENT_DIR ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

configure_file(src/config/config.h.in config.h)
add_compile_options(-mavx2 -mpclmul)

find_package(Qt6 REQUIRED COMPONENTS Widgets)

file(GLOB_RECURSE SOURCE_FILES src/*.cpp src/*/*.cpp)

include_directories(src)

add_executable(CEMCL ${HEADER_FILES} ${SOURCE_FILES} res/resource.qrc)

target_link_libraries(CEMCL PRIVATE Qt6::Widgets)
