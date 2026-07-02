/*
 * Contains functions from 'stdio.h' that contain variadic argument lists and cannot be implemented in rust.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */
 #include <stdio.h>

int vprintf(const char *format, va_list vlist) {
	return vfprintf(stdout, format, vlist);
}

int printf(const char *format, ...) {
	va_list list;

	va_start(list, format);
	const int ret = vprintf(format, list);
	va_end(list);

	return ret;
}

int fprintf(FILE *stream, const char *format, ...) {
    va_list list;

    va_start(list, format);
    const int ret = vfprintf(stream, format, list);
    va_end(list);

    return ret;
}

int snprintf(char *buffer, size_t bufsz, const char *format, ...) {
	va_list list;

	va_start(list, format);
	const int ret = vsnprintf(buffer, bufsz, format, list);
	va_end(list);

	return ret;
}

extern void doomgeneric_Tick();

void doom() {
    while (1) {
        doomgeneric_Tick();
    }
}