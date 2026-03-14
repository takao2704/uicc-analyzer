#ifndef LOGGER_H
#define LOGGER_H

#include <stddef.h>
#include <stdint.h>

void logger_init(void);
void logger_boot(void);
void logger_eventf(const char *fmt, ...);
void logger_atr(const uint8_t *bytes, size_t len);

#endif
