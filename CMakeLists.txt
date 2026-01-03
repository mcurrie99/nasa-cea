cmake_minimum_required(VERSION 3.19)
set(CMAKE_DISABLE_SOURCE_CHANGES ON)
set(CMAKE_DISABLE_IN_SOURCE_BUILD ON)
project(CEA
    VERSION 3.0.0
    LANGUAGES Fortran
)

list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake)

include(GNUInstallDirs)
include(cea_utilities)


#-----------------------------------------------------------------------
# Configuration
#-----------------------------------------------------------------------

# Establish if we are part of a bigger project
if(CMAKE_SOURCE_DIR STREQUAL CMAKE_CURRENT_SOURCE_DIR)
    set(PROJECT_IS_TOP_LEVEL TRUE)
else()
    set(PROJECT_IS_TOP_LEVEL FALSE)
endif()

if(PROJECT_IS_TOP_LEVEL)

     # Default Install Path: Local install in current build dir
    if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
        set(CMAKE_INSTALL_PREFIX "${CMAKE_BINARY_DIR}/install"
            CACHE PATH "CEA installation directory"
            FORCE)
        set(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT FALSE)
        message(STATUS "Setting install path to '${CMAKE_INSTALL_PREFIX}' since none specified.")
    endif()

    # Default Build Type: Standard release build
    if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
        set(CMAKE_BUILD_TYPE Release CACHE STRING "Choose the type of build." FORCE)
        set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS
            "Debug" "Release" "MinSizeRel" "RelWithDebInfo")
        message(STATUS "Setting build type to 'Release' since none specified.")
    endif()

endif()

# Testing
option(CEA_BUILD_TESTING "Build test harness for CEA" ${PROJECT_IS_TOP_LEVEL})
if(CEA_BUILD_TESTING)
    find_package(PFUNIT REQUIRED)
    include(CTest)
endif()

set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# change some build systems on windows
if(WIN32)
    # find and alternative to 'ar' archive utility
    find_program(AR_EXECUTABLE NAMES ar)
    if(AR_EXECUTABLE)
        message(STATUS "Found archive utility: ${AR_EXECUTABLE}")
        set(CMAKE_AR ${AR_EXECUTABLE} CACHE FILEPATH "Path to a program." FORCE)
    else()
        find_program(AR_EXECUTABLE NAMES gcc-ar)
        if(AR_EXECUTABLE)
            message(STATUS "Found archive utility: ${AR_EXECUTABLE}")
            set(CMAKE_AR ${AR_EXECUTABLE} CACHE FILEPATH "Path to a program." FORCE)
        else()
            message(WARNING "Archive utility not found.")
        endif()
    endif()
endif()


#-----------------------------------------------------------------------
# Targets
#-----------------------------------------------------------------------
add_subdirectory(extern/fbasics)
add_subdirectory(source)

# Build the thermodynamic properties database
#configure_file(src/thermo.inp thermo.inp COPYONLY)
#add_custom_target(thermo.lib ALL
#    COMMAND cea thermo >/dev/null
#    BYPRODUCTS thermo.lib thermo.out
#    DEPENDS cea
#)

# Build the transport properties database
#configure_file(src/trans.inp trans.inp COPYONLY)
#add_custom_target(trans.lib ALL
#    COMMAND cea trans >/dev/null
#    BYPRODUCTS trans.lib trans.out
#    DEPENDS cea
#)


#-----------------------------------------------------------------------
# Installation
#-----------------------------------------------------------------------
install(
    FILES data/thermo.lib data/trans.lib
    DESTINATION ${CMAKE_INSTALL_PREFIX}/data
)
install(EXPORT cea-config
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/cea
    NAMESPACE cea::
)
