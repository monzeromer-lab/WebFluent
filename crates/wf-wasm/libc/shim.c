// The parts of libc libwebp calls that are not the allocator or the maths
// (which `src/libc.rs` gives it): a sort, a search, and a stdio that writes
// nowhere, since there is nowhere to write.
#include <stddef.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

FILE* stderr = NULL;
FILE* stdout = NULL;

int printf(const char* format, ...) { (void)format; return 0; }
int fprintf(FILE* f, const char* format, ...) { (void)f; (void)format; return 0; }
int vfprintf(FILE* f, const char* format, va_list args) { (void)f; (void)format; (void)args; return 0; }
size_t fwrite(const void* data, size_t size, size_t count, FILE* f) {
  (void)data; (void)size; (void)f; return count;
}
FILE* fopen(const char* path, const char* mode) { (void)path; (void)mode; return NULL; }
int fclose(FILE* f) { (void)f; return 0; }

// Only ever asked for a short message; writes the format as it is.
int snprintf(char* out, size_t n, const char* format, ...) {
  size_t len = strlen(format);
  if (n > 0) {
    size_t k = len < n - 1 ? len : n - 1;
    memcpy(out, format, k);
    out[k] = 0;
  }
  return (int)len;
}

void abort(void) { __builtin_trap(); }
void exit(int code) { (void)code; __builtin_trap(); }
int abs(int x) { return x < 0 ? -x : x; }
long labs(long x) { return x < 0 ? -x : x; }
char* getenv(const char* name) { (void)name; return NULL; }

int atoi(const char* s) {
  int sign = 1, n = 0;
  while (*s == ' ') s++;
  if (*s == '-' || *s == '+') sign = (*s++ == '-') ? -1 : 1;
  while (*s >= '0' && *s <= '9') n = n * 10 + (*s++ - '0');
  return sign * n;
}

int strcmp(const char* a, const char* b) {
  while (*a && *a == *b) { a++; b++; }
  return (unsigned char)*a - (unsigned char)*b;
}

int strncmp(const char* a, const char* b, size_t n) {
  for (; n > 0; n--, a++, b++) {
    if (*a != *b || !*a) return (unsigned char)*a - (unsigned char)*b;
  }
  return 0;
}

static void swap(char* a, char* b, size_t size) {
  while (size--) { char t = *a; *a++ = *b; *b++ = t; }
}

static void sift(char* base, size_t root, size_t count, size_t size,
                 int (*cmp)(const void*, const void*)) {
  for (;;) {
    size_t child = 2 * root + 1;
    if (child >= count) return;
    if (child + 1 < count && cmp(base + child * size, base + (child + 1) * size) < 0) child++;
    if (cmp(base + root * size, base + child * size) >= 0) return;
    swap(base + root * size, base + child * size, size);
    root = child;
  }
}

// A heap sort: no recursion, no memory, and libwebp sorts a palette.
void qsort(void* base, size_t count, size_t size, int (*cmp)(const void*, const void*)) {
  char* b = (char*)base;
  if (count < 2) return;
  for (size_t i = count / 2; i-- > 0;) sift(b, i, count, size, cmp);
  for (size_t end = count - 1; end > 0; end--) {
    swap(b, b + end * size, size);
    sift(b, 0, end, size, cmp);
  }
}

void* bsearch(const void* key, const void* base, size_t count, size_t size,
              int (*cmp)(const void*, const void*)) {
  const char* b = (const char*)base;
  size_t lo = 0, hi = count;
  while (lo < hi) {
    size_t mid = lo + (hi - lo) / 2;
    int c = cmp(key, b + mid * size);
    if (c == 0) return (void*)(b + mid * size);
    if (c < 0) hi = mid; else lo = mid + 1;
  }
  return NULL;
}
