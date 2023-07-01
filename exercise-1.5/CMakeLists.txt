cmake_minimum_required(VERSION 3.16)
project(libxpdf)

set(CMAKE_POSITION_INDEPENDENT_CODE ON)

include(external/external.cmake)

set(XPDF_DIRECTORY "xpdf")

# Disable fontconfig and paper library 
set(NO_FONTCONFIG ON)
set(NO_PAPER ON)
set(SPLASH_CMYK ON)

add_subdirectory(${XPDF_DIRECTORY})

# Simple test to check if static library is properly built
# and can be link to any executable without dependencies other than
# Threads
enable_testing()
find_package(Threads)
add_executable(xpdftest ${XPDF_DIRECTORY}/xpdf/pdfinfo.cc)
get_directory_property(
    XPDF_BUILD_DIR
    DIRECTORY
    ${XPDF_DIRECTORY}
    DEFINITION
    CMAKE_CURRENT_BINARY_DIR
)
target_include_directories(
    xpdftest
    PUBLIC 
    ${XPDF_BUILD_DIR}
    ${XPDF_DIRECTORY}
    ${XPDF_DIRECTORY}/xpdf
    ${XPDF_DIRECTORY}/goo
    ${XPDF_DIRECTORY}/fofi
    ${XPDF_DIRECTORY}/splash
)
target_link_libraries(xpdftest xpdf ${CMAKE_THREADS_LIB_INIT})
add_test(xpdf_test xpdftest ${PROJECT_SOURCE_DIR}/Resources/fourier.pdf)
