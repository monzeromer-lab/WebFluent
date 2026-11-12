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
    // SAFETY: `block` is at least `HEADER` bytes, aligned to 16, so a
    // `usize` fits at its start.
    unsafe { (block as *mut usize).write(size) };
    // SAFETY: `block` is at least `HEADER` bytes, so one past them is still
    // in the allocation (or its end).
    unsafe { block.add(HEADER) }
}

/// The block a pointer this allocator handed out begins at, and the layout
/// it was made with.
///
/// # Safety
///
/// `ptr` came from `malloc`, `calloc` or `realloc` below and was not freed.
unsafe fn block_of(ptr: *mut u8) -> (*mut u8, Layout) {
    // SAFETY: the caller's pointer is `HEADER` bytes into its block.
    let block = unsafe { ptr.sub(HEADER) };
    // SAFETY: `finish` wrote the size at the block's start.
    let size = unsafe { (block as *const usize).read() };
    (block, layout(size).expect("a size this allocator wrote"))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    let Some(l) = layout(size) else {
        return std::ptr::null_mut();
    };
    // SAFETY: the layout is never zero-sized.
    let block = unsafe { alloc(l) };
    // SAFETY: `block` is null or a fresh block of that layout.
    unsafe { finish(block, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut u8 {
    let Some(total) = count.checked_mul(size) else {
        return std::ptr::null_mut();
    };
    let Some(l) = layout(total) else {
        return std::ptr::null_mut();
    };
    // SAFETY: the layout is never zero-sized.
    let block = unsafe { alloc_zeroed(l) };
    // SAFETY: `block` is null or a fresh block of that layout.
    unsafe { finish(block, total) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: C hands `free` only what `malloc`, `calloc` or `realloc` gave.
    let (block, old) = unsafe { block_of(ptr) };
    // SAFETY: `block` was allocated with exactly that layout.
    unsafe { dealloc(block, old) };
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
    // SAFETY: as `free`.
    let (block, old) = unsafe { block_of(ptr) };
    // SAFETY: `block` was allocated with `old`; the new size is non-zero and
    // keeps the header's alignment.
    let grown = unsafe { grow(block, old, new.size()) };
    // SAFETY: `grown` is null or a block of the new layout.
    unsafe { finish(grown, size) }
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
