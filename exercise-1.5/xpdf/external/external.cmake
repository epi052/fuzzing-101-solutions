include(cmake/mimick_find.cmake)

# To prevent libpng and freetype2 from exporting targets
# which cause ugly error as zlib does not export
set(SKIP_INSTALL_ALL TRUE)

set(ZLIB_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/external/zlib)
set(PNG_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/external/libpng)
set(FREETYPE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/external/freetype2)

add_subdirectory(${ZLIB_DIRECTORY})
set(ZLIB_LIB zlibstatic)

# PNG requires ZLIB
mimick_find("ZLIB" ${ZLIB_LIB})
add_subdirectory(${PNG_DIRECTORY})
set(PNG_LIB png_static)


# Disbale all the freetype2 optional dependencies
set(CMAKE_DISABLE_FIND_PACKAGE_HarfBuzz TRUE)
set(CMAKE_DISABLE_FIND_PACKAGE_BZip2 TRUE)
set(CMAKE_DISABLE_FIND_PACKAGE_ZLIB TRUE)
set(CMAKE_DISABLE_FIND_PACKAGE_PNG TRUE)
unset_mimick_find("ZLIB")

add_subdirectory(${FREETYPE_DIRECTORY})
set(FREETYPE_LIB freetype)

mimick_find("ZLIB" ${ZLIB_LIB})
mimick_find("PNG" ${PNG_LIB})
mimick_find("FREETYPE" ${FREETYPE_LIB})




