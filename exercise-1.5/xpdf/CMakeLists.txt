#========================================================================
#
# cmake-xpdf.txt
#
# CMake script for the Xpdf package.
#
# Modified by Ashutosh Varma <ashutoshvarma11@live.com>
# Copyright 2015 Glyph & Cog, LLC
#
#========================================================================

cmake_minimum_required(VERSION 2.8.8)

project(xpdf)

# Enable overriding of option() variables of subproject
cmake_policy(SET CMP0077 NEW)

include(cmake-config.txt)

add_subdirectory(goo)
add_subdirectory(fofi)
add_subdirectory(splash)
add_subdirectory(xpdf)

if (NOT HAVE_FREETYPE_H)
  message(WARNING "Couldn't find FreeType -- will not build pdftoppm, pdftopng, pdftohtml, or xpdf.")
endif ()

if (NOT PNG_FOUND)
  message(WARNING "Couldn't find libpng -- will not build pdftopng or pdftohtml.")
endif ()

if (NOT MULTITHREADED)
  message(WARNING "Multithreading is disabled -- will not build xpdf.")
endif ()

if (NOT USE_EXCEPTIONS)
  message(WARNING "C++ exceptions are disabled -- will not build xpdf.")
endif ()
