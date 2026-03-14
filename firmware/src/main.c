#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "hardware/gpio.h"
#include "pico/stdlib.h"

#include "logger.h"

#define SIM_CLK_PIN 2
#define SIM_RST_PIN 3
#define SIM_IO_PIN 4

#define ATR_MAX_BYTES 32
#define IO_POLL_INTERVAL_US 20
#define ATR_CAPTURE_WINDOW_MS 50

typedef struct {
    uint8_t bytes[ATR_MAX_BYTES];
    uint8_t current_byte;
    uint8_t bit_index;
    size_t length;
    bool armed;
    bool reported;
    absolute_time_t start_time;
} atr_capture_t;

static volatile uint32_t g_clk_edge_count = 0;
static volatile bool g_clk_seen_after_reset = false;
static atr_capture_t g_atr;

static void atr_reset_buffer(void) {
    memset(&g_atr, 0, sizeof(g_atr));
}

static void atr_arm(void) {
    atr_reset_buffer();
    g_atr.armed = true;
    g_atr.start_time = get_absolute_time();
}

static void atr_push_bit(uint8_t bit) {
    if (!g_atr.armed || g_atr.length >= ATR_MAX_BYTES) {
        return;
    }

    if (bit) {
        g_atr.current_byte |= (1u << g_atr.bit_index);
    }

    g_atr.bit_index++;
    if (g_atr.bit_index == 8) {
        g_atr.bytes[g_atr.length++] = g_atr.current_byte;
        g_atr.current_byte = 0;
        g_atr.bit_index = 0;
    }
}

static void handle_rst_change(void) {
    const bool rst_high = gpio_get(SIM_RST_PIN);
    logger_eventf("RST=%s", rst_high ? "HIGH" : "LOW");

    if (rst_high) {
        g_clk_seen_after_reset = false;
        g_clk_edge_count = 0;
        atr_arm();
    } else {
        g_atr.armed = false;
    }
}

static void gpio_irq_callback(uint gpio, uint32_t events) {
    if (gpio == SIM_RST_PIN && (events & (GPIO_IRQ_EDGE_RISE | GPIO_IRQ_EDGE_FALL))) {
        handle_rst_change();
    }

    if (gpio == SIM_CLK_PIN && (events & (GPIO_IRQ_EDGE_RISE | GPIO_IRQ_EDGE_FALL))) {
        g_clk_edge_count++;
        if (!g_clk_seen_after_reset && g_atr.armed) {
            g_clk_seen_after_reset = true;
            logger_eventf("CLK detected");
        }
    }
}

static void setup_gpio_inputs(void) {
    gpio_init(SIM_CLK_PIN);
    gpio_set_dir(SIM_CLK_PIN, GPIO_IN);
    gpio_disable_pulls(SIM_CLK_PIN);

    gpio_init(SIM_RST_PIN);
    gpio_set_dir(SIM_RST_PIN, GPIO_IN);
    gpio_disable_pulls(SIM_RST_PIN);

    gpio_init(SIM_IO_PIN);
    gpio_set_dir(SIM_IO_PIN, GPIO_IN);
    gpio_disable_pulls(SIM_IO_PIN);

    gpio_set_irq_enabled_with_callback(SIM_RST_PIN, GPIO_IRQ_EDGE_RISE | GPIO_IRQ_EDGE_FALL, true,
                                       &gpio_irq_callback);
    gpio_set_irq_enabled(SIM_CLK_PIN, GPIO_IRQ_EDGE_RISE | GPIO_IRQ_EDGE_FALL, true);
}

int main(void) {
    logger_init();
    logger_boot();

    atr_reset_buffer();
    setup_gpio_inputs();

    bool prev_io = gpio_get(SIM_IO_PIN);

    while (true) {
        const bool io_level = gpio_get(SIM_IO_PIN);

        if (g_atr.armed && io_level != prev_io) {
            atr_push_bit((uint8_t)io_level);
        }

        prev_io = io_level;

        if (g_atr.armed && !g_atr.reported && g_atr.length >= 4) {
            logger_atr(g_atr.bytes, g_atr.length);
            g_atr.reported = true;
        }

        if (g_atr.armed) {
            const absolute_time_t deadline = delayed_by_ms(g_atr.start_time, ATR_CAPTURE_WINDOW_MS);
            if (absolute_time_diff_us(get_absolute_time(), deadline) < 0) {
                g_atr.armed = false;
                if (!g_atr.reported && g_atr.length > 0) {
                    logger_atr(g_atr.bytes, g_atr.length);
                }
            }
        }

        sleep_us(IO_POLL_INTERVAL_US);
    }
}
