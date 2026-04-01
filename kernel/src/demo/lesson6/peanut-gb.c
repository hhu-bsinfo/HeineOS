/*
 * Contains glue functions for the gb struct
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-04-01
 * License: GPLv3
 */

 #include <stdint.h>

#include "peanut-gb.h"

int gb_size() {
    return sizeof(struct gb_s);
}

uint8_t* gb_get_joypad_ptr(struct gb_s* gb) {
    return &gb->direct.joypad;
}