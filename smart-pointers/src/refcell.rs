//! Enabling the borrow checker at runtime

use crate::cell::Cell;
use std::cell::UnsafeCell;

#[derive(Clone, Copy)]
enum RefState {
    Unshared,
    Shared(usize),
    Exclusive,
}

/// Keep tracks of how to the value is borrowed at runtime
pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<RefState>,
}

impl<T> RefCell<T> {
    /// Create a new references counted cell
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: Cell::new(RefState::Unshared),
        }
    }

    /// Borrow the inner value of no exclusive access has been given out
    pub fn borrow(&self) -> Option<&T> {
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Shared(1));
                // SAFETY: No exclusive reference has been given out
                Some(unsafe { &*self.value.get() })
            }
            RefState::Shared(n) => {
                self.state.set(RefState::Shared(n + 1));
                // SAFETY: No exclusive reference has been given out
                Some(unsafe { &*self.value.get() })
            }
            RefState::Exclusive => None,
        }
    }

    /// Take exclusive access to  the inner value
    pub fn borrow_mut(&self) -> Option<&mut T> {
        if let RefState::Unshared = self.state.get() {
            self.state.set(RefState::Exclusive);
            // SAFETY: The value has not been shared through any other reference
            Some(unsafe { &mut *self.value.get() })
        } else {
            None
        }
    }
}
