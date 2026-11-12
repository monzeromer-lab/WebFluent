//! Threads and clocks, and their stand-ins where WebAssembly has none.
//!
//! What the compiler asks of the machine that a browser does not give.
//!
//! In WebAssembly (`wasm32-unknown-unknown`) there are no threads, and
//! `std`'s clocks panic when read. The compiler asks for both in a few
//! places — the checks run side by side, a build says how long it took,
//! `ago(date)` measures against now — and asks here, where a native build
//! gets `std`'s and a WebAssembly one gets a stand-in that means the same.

use std::time::Duration;

/// [`std::thread::scope`] where there are threads. Without them a spawned
/// closure runs at once, on this thread, and its handle hands the result
/// back: the same results, one after another.
#[cfg(not(target_arch = "wasm32"))]
pub use std::thread::scope;

#[cfg(target_arch = "wasm32")]
pub fn scope<T>(body: impl FnOnce(&Scope) -> T) -> T {
    body(&Scope)
}

/// The stand-in for [`std::thread::Scope`].
#[cfg(target_arch = "wasm32")]
pub struct Scope;

#[cfg(target_arch = "wasm32")]
impl Scope {
    pub fn spawn<T>(&self, body: impl FnOnce() -> T) -> Ran<T> {
        Ran(body())
    }
}

/// A closure that has already run.
#[cfg(target_arch = "wasm32")]
pub struct Ran<T>(T);

#[cfg(target_arch = "wasm32")]
impl<T> Ran<T> {
    pub fn join(self) -> std::thread::Result<T> {
        Ok(self.0)
    }
}

/// How long something took. Without a clock it took no time at all.
#[derive(Clone, Copy, Debug)]
pub struct Stopwatch {
    #[cfg(not(target_arch = "wasm32"))]
    started: std::time::Instant,
}

impl Stopwatch {
    pub fn start() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            started: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.started.elapsed()
        }
        #[cfg(target_arch = "wasm32")]
        {
            Duration::ZERO
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static NOW: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
}

/// Tell the compiler what time it is, in seconds since the Unix epoch: a
/// WebAssembly host has no clock the compiler can read, so it hands over
/// its own (`Date.now()`) before a build. A native build reads the
/// system's and ignores this.
pub fn set_unix_now(_seconds: i64) {
    #[cfg(target_arch = "wasm32")]
    NOW.with(|now| now.set(_seconds));
}

/// Seconds since the Unix epoch: the system's clock, or, when the
/// environment names one, `SOURCE_DATE_EPOCH` — the convention for a
/// reproducible build, so what a page says of the time it was built (an
/// `ago(…)` in the static paint) is the same on every run.
pub fn unix_now() -> i64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        fixed_now(std::env::var("SOURCE_DATE_EPOCH").ok().as_deref()).unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        NOW.with(|now| now.get())
    }
}

/// The time a `SOURCE_DATE_EPOCH` value fixes, when it is one: whole
/// seconds since the Unix epoch.
#[cfg_attr(
    target_arch = "wasm32",
    allow(dead_code, reason = "a WebAssembly host hands its clock over instead")
)]
fn fixed_now(epoch: Option<&str>) -> Option<i64> {
    epoch?.trim().parse().ok()
}

/// The process's environment, as [`std::env::vars`] gives it — which panics
/// in WebAssembly, where there is none: there it is empty.
pub fn env_vars() -> Vec<(String, String)> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::vars().collect()
    }
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_date_epoch_fixes_the_time_when_it_is_a_number() {
        assert_eq!(fixed_now(Some("1700000000")), Some(1_700_000_000));
        assert_eq!(fixed_now(Some(" 42 ")), Some(42));
        assert_eq!(fixed_now(Some("yesterday")), None);
        assert_eq!(fixed_now(None), None);
    }
}
