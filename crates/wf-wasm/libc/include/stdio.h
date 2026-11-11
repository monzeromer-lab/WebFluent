#pragma once
#include <stddef.h>
#include <stdarg.h>
typedef struct FILE FILE;
extern FILE* stderr;
extern FILE* stdout;
int printf(const char* format, ...);
int fprintf(FILE* f, const char* format, ...);
int snprintf(char* out, size_t n, const char* format, ...);
int vfprintf(FILE* f, const char* format, va_list args);
size_t fwrite(const void* data, size_t size, size_t count, FILE* f);
FILE* fopen(const char* path, const char* mode);
int fclose(FILE* f);
