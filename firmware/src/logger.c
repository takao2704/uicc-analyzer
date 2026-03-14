#include "logger.h"

#include <stdarg.h>
#include <stdio.h>

#include "pico/stdlib.h"
#include "pico/time.h"

static absolute_time_t g_boot_time;

static double ms_since_boot(void) {
    const int64_t us = absolute_time_diff_us(g_boot_time, get_absolute_time());
    return (double)us / 1000.0;
}

void logger_init(void) {
    stdio_usb_init();
    sleep_ms(250);
    g_boot_time = get_absolute_time();
}

void logger_boot(void) {
    printf("[0.000 ms] boot\n");
}

void logger_eventf(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);

    printf("[%.3f ms] ", ms_since_boot());
    vprintf(fmt, args);
    printf("\n");

    va_end(args);
}

void logger_atr(const uint8_t *bytes, size_t len) {
    printf("[%.3f ms] ATR:", ms_since_boot());
    for (size_t i = 0; i < len; ++i) {
        printf(" %02X", bytes[i]);
    }
    printf("\n");
}
