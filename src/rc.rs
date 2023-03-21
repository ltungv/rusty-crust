//! Structure that enables single-threaded access to owned data.

use crate::cell::Cell;
use std::ops::Deref;
use std::ptr::NonNull;

/// The inner representation of `Rc<T>` that gets allocated on the heap.
struct RcInner<T> {
    /// The value referenced by our smart pointer.
    value: T,
    /// The number of references that have been handed out.
    refcount: Cell<usize>,
}

/// A reference-counted smart pointer that deallocates the inner value
/// once there's no reference pointing to the inner value.
#[derive(Debug)]
pub struct Rc<T> {
    inner: NonNull<RcInner<T>>,
}

impl<T> Rc<T> {
    /// Allocate the given value onto the heap and return a reference-counted smart pointer to it.
    pub fn new(value: T) -> Self {
        // Put the inner value onto the heap and keep a raw pointer to that memory location.
        let inner = Box::new(RcInner {
            value,
            refcount: Cell::new(1),
        });
        Self {
            // SAFETY: `Box::into_raw` does not give a null pointer.
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: self.inner is a raw pointer to a `Box` that is deallocated when the last `Rc`
        // goes away, dereference the shared poninter here is fine since we are having an `Rc`.
        let inner = unsafe { self.inner.as_ref() };
        &inner.value
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        // SAFETY: self.inner is a raw pointer to a `Box` that is deallocated when the last `Rc`
        // goes away, dereference the shared poninter here is fine since we are having an `Rc`.
        let inner = unsafe { self.inner.as_ref() };
        inner.refcount.set(inner.refcount.get() + 1);
        Rc { inner: self.inner }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        // SAFETY: self.inner is a raw pointer to a `Box` that is deallocated when the last `Rc`
        // goes away, dereference the shared poninter here is fine since we are having an `Rc`.
        let inner = unsafe { self.inner.as_ref() };
        let refcount = inner.refcount.get();
        if refcount == 1 {
            // SAFETY: We are dropping the only `Rc` left, after being dropped, there is no more
            // reference to `T`. Hence, deallocating the heap memory is safe.
            unsafe { Box::from_raw(self.inner.as_ptr()) };
        } else {
            inner.refcount.set(refcount - 1);
        }
    }
}
