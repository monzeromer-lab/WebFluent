#pragma once
double pow(double x, double y);
double exp(double x);
double log(double x);
double log10(double x);
float expf(float x);
float logf(float x);
float powf(float x, float y);
#define fabs(x) __builtin_fabs(x)
#define fabsf(x) __builtin_fabsf(x)
#define floor(x) __builtin_floor(x)
#define floorf(x) __builtin_floorf(x)
#define ceil(x) __builtin_ceil(x)
#define ceilf(x) __builtin_ceilf(x)
#define sqrt(x) __builtin_sqrt(x)
#define sqrtf(x) __builtin_sqrtf(x)
#define HUGE_VAL __builtin_huge_val()
