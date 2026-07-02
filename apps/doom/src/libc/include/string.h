#ifndef _STRING_H_
#define _STRING_H_

#include <stddef.h>

#ifndef NULL
#define NULL ((void*)0)
#endif

int memcmp(const void *s1, const void *s2, size_t n);
void* memcpy(void *dest, const void *src, size_t n);
void* memmove(void *dest, const void *src, size_t n);
void* memset(void *dest, int c, size_t n);

size_t strlen(const char *s);
int strcmp(const char *lhs, const char *rhs);
int strncmp(const char *lhs, const char *rhs, size_t count);
int strcasecmp(const char *lhs, const char *rhs);
int strncasecmp(const char *lhs, const char *rhs, size_t count);
char* strncpy(char *dest, const char *src, size_t count);
char* strdup(const char *str1);
char* strchr(const char *str, int ch);
char* strrchr(const char *str, int ch);
char* strstr(const char *str, const char *substr);

#endif