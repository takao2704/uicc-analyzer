cmake_minimum_required(VERSION 3.13)

include($ENV{PICO_SDK_PATH}/external/pico_sdk_import.cmake)

project(sim_modem_passive_analyzer C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

pico_sdk_init()

add_executable(sim_modem_passive_analyzer
    src/main.c
    src/logger.c
)

target_include_directories(sim_modem_passive_analyzer PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}/include
)

target_link_libraries(sim_modem_passive_analyzer
    pico_stdlib
    hardware_gpio
    hardware_timer
)

pico_enable_stdio_uart(sim_modem_passive_analyzer 0)
pico_enable_stdio_usb(sim_modem_passive_analyzer 1)

pico_add_extra_outputs(sim_modem_passive_analyzer)
