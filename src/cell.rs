//! Structure that enables mutability to a value through a shared reference.

use std::cell::UnsafeCell;

/// Cell enable interior mutability
#[derive(Debug)]
pub struct Cell<T> {
    // This has to be an UnsafeCell so that it can be mutated through a shared reference.
    // `Cell<T>` is `!Sync` because it contains `UnsafeCell<T>`, which is also `!Sync`.
    value: UnsafeCell<T>,
}

impl<T> Cell<T> {
    /// Create a new cell.
    pub fn new(value: T) -> Self {
        Cell {
            value: UnsafeCell::new(value),
        }
    }

    /// Change the inner value.
    pub fn set(&self, value: T) {
        // SAFETY: The value is not accessed concurrently by multiple threads (!Sync)
        // SAFETY: No reference to the underlying value was given out, set does not invalidate any
        // existing reference
        unsafe { *self.value.get() = value };
    }

    /// Get the inner value.
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        // SAFETY: The value is not accessed concurrently by multiple threads (!Sync)
        // SAFETY: No reference to the inner value was given out (Copy)
        unsafe { *self.value.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        Cell::new(0);
    }
}
