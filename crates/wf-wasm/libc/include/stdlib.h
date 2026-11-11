#pragma once
#include <stddef.h>
void* malloc(size_t size);
void* calloc(size_t count, size_t size);
void* realloc(void* ptr, size_t size);
void free(void* ptr);
void abort(void) __attribute__((noreturn));
void exit(int code) __attribute__((noreturn));
int abs(int x);
long labs(long x);
int atoi(const char* s);
char* getenv(const char* name);
void qsort(void* base, size_t count, size_t size, int (*cmp)(const void*, const void*));
void* bsearch(const void* key, const void* base, size_t count, size_t size,
              int (*cmp)(const void*, const void*));
#define EXIT_SUCCESS 0
#define EXIT_FAILURE 1
