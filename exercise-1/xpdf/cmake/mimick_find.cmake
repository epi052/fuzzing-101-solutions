# Set the {LIB}_FOUND, {LIB}_LIBRARY, {LIB}_INCLUDE_DIRS
# Args:
#   LIB_NAME -  Name of Library, one that cmake find_package use
#               Example:- PNG, ZLIB, Freetype
#   LIB -       Library Target with proper properties    
function(mimick_find LIB_NAME LIB)
    set(${LIB_NAME}_FOUND True PARENT_SCOPE)
    set(${LIB_NAME}_LIBRARY ${LIB} PARENT_SCOPE)
    set(${LIB_NAME}_LIBRARIES ${LIB} PARENT_SCOPE)

    # Get the include dirs
    get_target_property(I_DIRS ${LIB} INCLUDE_DIRECTORIES)
    set(${LIB_NAME}_INCLUDE_DIR ${I_DIRS} PARENT_SCOPE)
    set(${LIB_NAME}_INCLUDE_DIRS ${I_DIRS} PARENT_SCOPE)
endfunction()

function(unset_mimick_find LIB_NAME)
    unset(${LIB_NAME}_FOUND PARENT_SCOPE)
    unset(${LIB_NAME}_LIBRARY PARENT_SCOPE)
    unset(${LIB_NAME}_LIBRARIES PARENT_SCOPE)
    unset(${LIB_NAME}_INCLUDE_DIR PARENT_SCOPE)
    unset(${LIB_NAME}_INCLUDE_DIRS PARENT_SCOPE)
endfunction()
