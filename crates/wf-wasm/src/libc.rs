//! What libwebp asks of libc that Rust gives best: the allocator, and the
//! maths. `wasm32-unknown-unknown` has no C library; `libc/shim.c` holds
//! the rest of what libwebp calls.

use std::alloc::{Layout, alloc, alloc_zeroed, dealloc, realloc as grow};

/// Every block is preceded by its size, so `free` knows its layout; 16
/// bytes keep what follows aligned for anything C stores in it.
const HEADER: usize = 16;

fn layout(size: usize) -> Option<Layout> {
    Layout::from_size_align(size.checked_add(HEADER)?, HEADER).ok()
}

unsafe fn finish(block: *mut u8, size: usize) -> *mut u8 {
    if block.is_null() {
        return block;
    }
    // SAFETY: `block` is at least `HEADER` bytes, aligned to 16.
    unsafe {
        (block as *mut usize).write(size);
        block.add(HEADER)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    match layout(size) {
        // SAFETY: the layout is never zero-sized.
        Some(l) => unsafe { finish(alloc(l), size) },
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut u8 {
    let Some(total) = count.checked_mul(size) else {
        return std::ptr::null_mut();
    };
    match layout(total) {
        // SAFETY: the layout is never zero-sized.
        Some(l) => unsafe { finish(alloc_zeroed(l), total) },
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: `ptr` came from `malloc`, `calloc` or `realloc` above, so its
    // size sits `HEADER` bytes before it.
    unsafe {
        let block = ptr.sub(HEADER);
        let size = (block as *const usize).read();
        dealloc(block, layout(size).expect("a size this allocator wrote"));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    if ptr.is_null() {
        // SAFETY: as `malloc`.
        return unsafe { malloc(size) };
    }
    let Some(new) = layout(size) else {
        return std::ptr::null_mut();
    };
    // SAFETY: as `free`; the new size keeps the header's alignment.
    unsafe {
        let block = ptr.sub(HEADER);
        let old = (block as *const usize).read();
        let old = layout(old).expect("a size this allocator wrote");
        finish(grow(block, old, new.size()), size)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

#[unsafe(no_mangle)]
pub extern "C" fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

#[unsafe(no_mangle)]
pub extern "C" fn exp(x: f64) -> f64 {
    libm::exp(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn expf(x: f32) -> f32 {
    libm::expf(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn log(x: f64) -> f64 {
    libm::log(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn logf(x: f32) -> f32 {
    libm::logf(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn log10(x: f64) -> f64 {
    libm::log10(x)
}
