#ifndef _STDLIB_H_
#define _STDLIB_H_

#include <stddef.h>

#ifndef NULL
#define NULL ((void*)0)
#endif

void* malloc(size_t size);
void* calloc(size_t num, size_t size);
void* realloc(void *ptr, size_t new_size);
void free(void *ptr);

int abs(int n);

double atof (const char *str);
int atoi(const char *str);

int system(const char *command);
void exit(int exit_code);

#endif