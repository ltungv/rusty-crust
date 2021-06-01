use crate::cell::Cell;
use std::ops::Deref;
use std::ptr::NonNull;

struct RcInner<T> {
    value: T,
    refcount: Cell<usize>,
}

/// A refennced count smart pointer that deallocates the inner value once there's no reference
/// pointing to the inner value
pub struct Rc<T> {
    inner: NonNull<RcInner<T>>,
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        // Put the inner value onto the heap and keep a raw pointer to that memory location
        let inner = Box::new(RcInner {
            value,
            refcount: Cell::new(1),
        });
        Self {
            // SAFETY: `Box::into_raw` does not give a null pointer
            inner: unsafe { NonNull::new_unchecked(inner.into_raw()) },
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: self.inner is a raw pointer to a `Box` that is only deallocated when the last
        // `Rc` goes away, dereference to a shared reference here is fine since we are having an
        // `Rc`
        let inner = unsafe { self.inner.as_ref() };
        &inner.value
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        // SAFETY: See implementation for `Deref`
        let inner = unsafe { self.inner.as_ref() };
        inner.refcount.set(inner.refcount.get() + 1);
        Rc { inner: self.inner }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        // SAFETY: See implementation for `Deref`
        let inner = unsafe { self.inner.as_ref() };
        let refcount = inner.refcount.get();
        if refcount == 1 {
            // Drop the raw pointer to `Box` so that after the `Box` is dropped, the pointer can no
            // longer be used
            drop(inner);
            // SAFETY: We are dropping the only `Rc` left, after being dropped, there is no more
            // reference to `T`. Hence, dropping the heap-allocated memory is safe
            unsafe { Box::from_raw(self.inner.as_ptr()) };
        } else {
            inner.refcount.set(refcount - 1);
        }
    }
}
