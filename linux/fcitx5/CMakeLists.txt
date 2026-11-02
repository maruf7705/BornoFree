cmake_minimum_required(VERSION 3.20)
project(fcitx5-borno VERSION 1.0.1 LANGUAGES CXX)

include(GNUInstallDirs)
include(CTest)

find_package(Fcitx5Core REQUIRED)
find_program(CARGO_EXECUTABLE cargo REQUIRED)

get_filename_component(BORNO_ROOT "${CMAKE_CURRENT_LIST_DIR}/../.." ABSOLUTE)
set(BORNO_FFI_MANIFEST "${BORNO_ROOT}/crates/borno-ffi/Cargo.toml")
set(BORNO_CARGO_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/cargo-target")
set(BORNO_FFI_LIBRARY "${BORNO_CARGO_TARGET_DIR}/release/libborno_ffi.a")

file(GLOB_RECURSE BORNO_RUST_SOURCES CONFIGURE_DEPENDS
    "${BORNO_ROOT}/crates/borno-core/src/*.rs"
    "${BORNO_ROOT}/crates/borno-ffi/src/*.rs")

add_custom_command(
    OUTPUT "${BORNO_FFI_LIBRARY}"
    COMMAND "${CARGO_EXECUTABLE}" build --manifest-path "${BORNO_FFI_MANIFEST}"
            --package borno-ffi --release --target-dir "${BORNO_CARGO_TARGET_DIR}"
    WORKING_DIRECTORY "${BORNO_ROOT}"
    DEPENDS ${BORNO_RUST_SOURCES} "${BORNO_ROOT}/Cargo.toml" "${BORNO_ROOT}/Cargo.lock"
    COMMENT "Building the BornoFree Rust FFI bridge")
add_custom_target(borno_ffi_target DEPENDS "${BORNO_FFI_LIBRARY}")

add_library(borno MODULE src/borno.cpp)
add_dependencies(borno borno_ffi_target)
target_include_directories(borno PRIVATE "${CMAKE_CURRENT_LIST_DIR}/include")
target_compile_features(borno PRIVATE cxx_std_17)
target_link_libraries(borno PRIVATE Fcitx5::Core "${BORNO_FFI_LIBRARY}" dl pthread m)
set_target_properties(borno PROPERTIES PREFIX "")

configure_file(borno-addon.conf.in "${CMAKE_CURRENT_BINARY_DIR}/borno-addon.conf" @ONLY)

install(TARGETS borno DESTINATION "${CMAKE_INSTALL_LIBDIR}/fcitx5")
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/borno-addon.conf"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5/addon" RENAME "borno.conf")
install(FILES "${CMAKE_CURRENT_LIST_DIR}/borno-inputmethod.conf"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod" RENAME "borno.conf")
install(FILES "${CMAKE_CURRENT_LIST_DIR}/borno-phonetic-inputmethod.conf"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod" RENAME "borno-phonetic.conf")
install(FILES "${BORNO_ROOT}/linux/assets/borno.svg"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/scalable/apps" RENAME "borno.svg")
install(PROGRAMS "${BORNO_ROOT}/linux/bin/borno-fcitxctl"
                 "${BORNO_ROOT}/linux/bin/borno-hyprland-setup"
        DESTINATION "${CMAKE_INSTALL_BINDIR}")
install(DIRECTORY "${BORNO_ROOT}/linux/docs/"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/borno-free")

if(BUILD_TESTING)
    add_executable(borno-addon-action-contract tests/ffi_smoke.cpp)
    add_dependencies(borno-addon-action-contract borno_ffi_target)
    target_include_directories(borno-addon-action-contract PRIVATE "${CMAKE_CURRENT_LIST_DIR}/include")
    target_compile_features(borno-addon-action-contract PRIVATE cxx_std_17)
    target_link_libraries(borno-addon-action-contract PRIVATE "${BORNO_FFI_LIBRARY}" dl pthread m)
    add_test(NAME borno-addon-action-contract COMMAND borno-addon-action-contract)
endif()
