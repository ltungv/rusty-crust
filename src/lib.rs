use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct Mutex<T> {
    locked: AtomicBool,
    v: UnsafeCell<T>,
}

// SAFETY: We know that mutex is [`Sync`] if the value of type `T` is [`Send`]
unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            v: UnsafeCell::new(t),
        }
    }

    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(UNLOCKED, LOCKED, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            // MESI procotocol
            while self.locked.load(Ordering::Relaxed) == LOCKED {}

            // compare_exchange_weak might fail even if the value matches that one
            // that we give
            // x86: Compare-and-swap
            // ARM: LDREX STREX
            //  - this is a 2-step operation
            //      - LDREX take exclusive access and load the value
            //      - STREX store the value iff the thread still has exclusve access to the value
            //  - compare_exchange: implemetation using loop { LDREX STREX }
            //      - compare_exchange in rust becomes a nested loop on ARM, this
            //      leads to generally less efficient code
            //  - compare_exchange_weak: LDREX STREX
        }

        // SAFETY: We are holding a lock
        let rtr = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        rtr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_add() {
        const N_THREADS: usize = 100;
        const N_ITER: usize = 100;

        let l: &'static _ = Box::leak(Box::new(Mutex::new(0)));
        let handles: Vec<_> = (0..N_THREADS)
            .map(|_| {
                std::thread::spawn(move || {
                    for _ in 0..N_ITER {
                        l.with_lock(|v| *v += 1);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(l.with_lock(|v| *v), N_THREADS * N_ITER);
    }
}
