use std::cell::UnsafeCell;

pub struct Cell<T> {
    // This has to be an UnsafeCell so that it can be mutated through a shared reference
    value: UnsafeCell<T>,
}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Cell {
            value: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, value: T) {
        // SAFETY: This is not okay, nothing ensures that this value can be mutated
        unsafe { *self.value.get() = value };
    }

    pub fn get(&self) -> T {
        // SAFETY: This is not okay, nothing ensures that this returns the latest value
        unsafe { *self.value.get() }
    }
}
