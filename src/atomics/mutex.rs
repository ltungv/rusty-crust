use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

const LOCKED: bool = true;
const UNLOCKED: bool = false;

/// A structure for providing mutually exclusive access to shared data.
#[derive(Debug)]
pub struct Mutex<T> {
    locked: AtomicBool,
    v: UnsafeCell<T>,
}

// SAFETY: We know that mutex is [`Sync`] if the value of type `T` is [`Send`]
unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    /// Wrap the given value in a `Mutex` to provide safe concurrent accesses from multiple threads.
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            v: UnsafeCell::new(t),
        }
    }

    /// Acquire exlusive access and perform an action on a mutable reference to the inner value.
    pub fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        while self
            .locked
            .compare_exchange_weak(UNLOCKED, LOCKED, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) == LOCKED {
                std::hint::spin_loop()
            }

            // compare_exchange_weak might fail even if the value matches that one that we give
            // The behavior of compare_exchange is different on different platform
            // x86: Compare-and-swap
            // ARM: LDREX STREX
            //  - this is a 2-step operation
            //      - LDREX take exclusive access and load the value
            //      - STREX store the value iff the thread still has exclusive access to the value
            //  - compare_exchange: implemetation using loop { LDREX STREX }
            //      - compare_exchange in rust becomes a nested loop on ARM, this leads to
            //      generally less efficient code
            //  - compare_exchange_weak: LDREX STREX
        }

        // With ordering [`Relaxed`], there is no gurantee that the value you receive is in order
        // with the operations performed by other thread.
        //
        // The [`Acquire`] and [`Release`] pair of memory ordering ensures that any operation
        // before one thread releases a memory location is observed by the thread that subsequently
        // acquires the same memory location

        // SAFETY: We are holding a lock
        let rtr = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Release);
        rtr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_seqcst_vs_acqrel() {
        use std::sync::atomic::{AtomicBool, AtomicUsize};

        let x: &'static _ = Box::leak(Box::new(AtomicBool::new(false)));
        let y: &'static _ = Box::leak(Box::new(AtomicBool::new(false)));
        let z: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));

        let _tx = std::thread::spawn(move || {
            x.store(true, Ordering::SeqCst);
        });
        let _ty = std::thread::spawn(move || {
            y.store(true, Ordering::SeqCst);
        });
        let t1 = std::thread::spawn(move || {
            while !x.load(Ordering::SeqCst) {}
            if y.load(Ordering::SeqCst) {
                z.fetch_add(1, Ordering::Relaxed);
            }
        });
        let t2 = std::thread::spawn(move || {
            while !y.load(Ordering::SeqCst) {}
            if x.load(Ordering::SeqCst) {
                z.fetch_add(1, Ordering::Relaxed);
            }
        });

        // What are the possible value for z?
        // - Is 0 possible?
        //   Restrictions:
        //      t1 must run after tx
        //      t2 must run after ty
        //   + This outcome seems impossible, according to the thread schedule
        //   + t1 guarantees to observe the effect of tx but does not guarantee to observe the
        //   effect of ty, similarly t2 guarantees to observe the effect of ty but does not
        //   guarantee to observe the effect of tx.
        //   + This outcome is possible
        //   + Using SeqCst to ensure all threads observer to same sequence of events
        //
        // - Is 1 possible?
        //   YES: tx, t1, ty, t2
        // - Is 2 possible?
        //   YES: tx, ty, t1, t2

        t1.join().unwrap();
        t2.join().unwrap();
        let zz = z.load(Ordering::SeqCst);
        assert_ne!(zz, 0);
    }

    #[test]
    fn concurrent_mutex_add() {
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

        // This will fails when load and store are not performed as a single atomic operation, leaving
        // time for other threads to inverleave between the moment the value was loaded and the moment
        // the new value is stored

        assert_eq!(l.with_lock(|v| *v), N_THREADS * N_ITER);
    }

    #[test]
    #[ignore]
    fn too_relaxed() {
        use std::sync::atomic::AtomicUsize;

        let x: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
        let y: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));

        let t1 = std::thread::spawn(move || {
            let r1 = y.load(Ordering::Relaxed);
            x.store(r1, Ordering::Relaxed);
            r1
        });

        let t2 = std::thread::spawn(move || {
            let r2 = x.load(Ordering::Relaxed);
            y.store(42, Ordering::Relaxed);
            r2
        });

        // We might observe that r1 == r2 == 42, which is very unusual. When specify the memory
        // ordering as `Relaxed`, the compiler and hardware is free the reorganize the sequence of
        // operations as long as the observed modification order is correct for each atomic value.
        // The sequence of operations:
        //
        // ```
        // let r2 = x.load(Ordering::Relaxed);
        // y.store(42, Ordering::Relaxed);
        // ```
        //
        // can be reordered to:
        //
        // ```
        // y.store(42, Ordering::Relaxed);
        // let r2 = x.load(Ordering::Relaxed);
        // ```
        //
        // For the value x and y, there is nothing wrong with their order of modification, but the
        // overrall sequence of executions yields an undesirable outcome.

        let _r1 = t1.join().unwrap();
        let _r2 = t2.join().unwrap();
    }
}
