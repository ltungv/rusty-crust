use std::{marker::PhantomData, ptr::NonNull};

#[derive(Debug)]
struct Node<T> {
    prev: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>,
    data: T,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self {
            prev: None,
            next: None,
            data,
        }
    }
}

/// A doubly-linked list with owned nodes.
///
/// The DoublyLinkedList allows pushing and popping elements at either end in
/// constant time.
///
/// NOTE: It is almost always better to use Vec or VecDeque because array-based
/// containers are generally faster, more memory efficient, and make better use
/// of CPU cache.
#[derive(Debug)]
pub struct DoublyLinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<Box<Node<T>>>,
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            marker: PhantomData,
        }
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> DoublyLinkedList<T> {
    /// Creates an empty DoublyLinkedList.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let list: DoublyLinkedList<u32> = DoublyLinkedList::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            marker: PhantomData,
        }
    }

    /// Moves all elements from other to the end of the list.
    ///
    /// This reuses all the nodes from other and moves them into self. After
    /// this operation, other becomes empty.
    ///
    /// This operation should compute in O(1) time and O(1) memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut list1 = DoublyLinkedList::new();
    /// list1.push_back('a');
    ///
    /// let mut list2 = DoublyLinkedList::new();
    /// list2.push_back('b');
    /// list2.push_back('c');
    ///
    /// list1.append(&mut list2);
    ///
    /// let mut iter = list1.iter();
    /// assert_eq!(iter.next(), Some(&'a'));
    /// assert_eq!(iter.next(), Some(&'b'));
    /// assert_eq!(iter.next(), Some(&'c'));
    /// assert!(iter.next().is_none());
    ///
    /// assert!(list2.is_empty());
    /// ```
    #[allow(unsafe_code)]
    pub fn append(&mut self, other: &mut DoublyLinkedList<T>) {
        match self.tail {
            Some(mut tail) => {
                // SAFETY: Our tail is Some so we know that the pointer is still
                // valid and we can direference the raw pointer to access its
                // data
                unsafe { tail.as_mut().next = other.head };
                if let Some(mut head) = other.head {
                    // SAFETY: Other's head is Some so we know that the pointer
                    // is still valid and we can dereference the raw pointer
                    // to access its data
                    unsafe { head.as_mut().prev = self.tail };
                }
            }
            None => {
                self.head = other.head;
                self.tail = other.head;
            }
        }
        self.len += other.len;
        other.head = None;
        other.tail = None;
        other.len = 0;
    }

    /// Provides a forward iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut list: DoublyLinkedList<u32> = DoublyLinkedList::new();
    ///
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// let mut iter = list.iter();
    /// assert_eq!(iter.next(), Some(&0));
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            it: self.head,
            marker: PhantomData,
        }
    }

    /// Provides a forward iterator with mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut list: DoublyLinkedList<u32> = DoublyLinkedList::new();
    ///
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// for element in list.iter_mut() {
    ///     *element += 10;
    /// }
    ///
    /// let mut iter = list.iter();
    /// assert_eq!(iter.next(), Some(&10));
    /// assert_eq!(iter.next(), Some(&11));
    /// assert_eq!(iter.next(), Some(&12));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            it: self.head,
            marker: PhantomData,
        }
    }

    /// Returns true if the DoublyLinkedList is empty.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut dl = DoublyLinkedList::new();
    /// assert!(dl.is_empty());
    ///
    /// dl.push_front("foo");
    /// assert!(!dl.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the length of the DoublyLinkedList.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut dl = DoublyLinkedList::new();
    ///
    /// dl.push_front(2);
    /// assert_eq!(dl.len(), 1);
    ///
    /// dl.push_front(1);
    /// assert_eq!(dl.len(), 2);
    ///
    /// dl.push_back(3);
    /// assert_eq!(dl.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Removes all elements from the DoublyLinkedList.
    ///
    /// This operation should compute in O(n) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut dl = DoublyLinkedList::new();
    ///
    /// dl.push_front(2);
    /// dl.push_front(1);
    /// assert_eq!(dl.len(), 2);
    /// assert_eq!(dl.front(), Some(&1));
    ///
    /// dl.clear();
    /// assert_eq!(dl.len(), 0);
    /// assert_eq!(dl.front(), None);
    /// ```
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Returns true if the DoublyLinkedList contains an element equal to the
    /// given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut list: DoublyLinkedList<u32> = DoublyLinkedList::new();
    ///
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// assert_eq!(list.contains(&0), true);
    /// assert_eq!(list.contains(&10), false);
    /// ```
    #[allow(unsafe_code)]
    pub fn contains(&self, data: &T) -> bool
    where
        T: PartialEq<T>,
    {
        let mut it = self.head;
        while let Some(node) = it {
            // SAFETY: Current Node is not None so we know the raw pointer
            // inside is still valid
            let node = unsafe { node.as_ref() };
            if *data == node.data {
                return true;
            }
            it = node.next;
        }
        false
    }

    /// Provides a reference to the front element, or None if the list is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut dl = DoublyLinkedList::new();
    /// assert_eq!(dl.front(), None);
    ///
    /// dl.push_front(1);
    /// assert_eq!(dl.front(), Some(&1));
    /// ```
    #[allow(unsafe_code)]
    pub fn front(&self) -> Option<&T> {
        if let Some(head) = self.head {
            // SAFETY: Head is not None so we know the raw pointer inside is
            // still valid
            let data = unsafe { &(*head.as_ptr()).data };
            return Some(data);
        }
        None
    }

    /// Provides a mutable reference to the front element, or None if the list
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut dl = DoublyLinkedList::new();
    /// assert_eq!(dl.front(), None);
    ///
    /// dl.push_front(1);
    /// assert_eq!(dl.front(), Some(&1));
    ///
    /// match dl.front_mut() {
    ///     None => {},
    ///     Some(x) => *x = 5,
    /// }
    /// assert_eq!(dl.front(), Some(&5));
    /// ```
    #[allow(unsafe_code)]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if let Some(head) = self.head {
            // SAFETY: Head is not None so we know the raw pointer inside is
            // still valid
            let data = unsafe { &mut (*head.as_ptr()).data };
            return Some(data);
        }
        None
    }

    /// Provides a reference to the back element, or None if the list is empty.
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut dl = DoublyLinkedList::new();
    /// assert_eq!(dl.back(), None);
    ///
    /// dl.push_back(1);
    /// assert_eq!(dl.back(), Some(&1));
    /// ```
    #[allow(unsafe_code)]
    pub fn back(&self) -> Option<&T> {
        if let Some(tail) = self.tail {
            // SAFETY: Tail is not None so we know the raw pointer inside is
            // still valid
            let data = unsafe { &(*tail.as_ptr()).data };
            return Some(data);
        }
        None
    }

    /// Provides a mutable reference to the back element, or None if the list
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut dl = DoublyLinkedList::new();
    /// assert_eq!(dl.back(), None);
    ///
    /// dl.push_back(1);
    /// assert_eq!(dl.back(), Some(&1));
    ///
    /// match dl.back_mut() {
    ///     None => {},
    ///     Some(x) => *x = 5,
    /// }
    /// assert_eq!(dl.back(), Some(&5));
    /// ```
    #[allow(unsafe_code)]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if let Some(tail) = self.tail {
            // SAFETY: Tail is not None so we know the raw pointer inside is
            // still valid
            let data = unsafe { &mut (*tail.as_ptr()).data };
            return Some(data);
        }
        None
    }

    /// Adds an element first in the list.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut dl = DoublyLinkedList::new();
    ///
    /// dl.push_front(2);
    /// assert_eq!(dl.front().unwrap(), &2);
    ///
    /// dl.push_front(1);
    /// assert_eq!(dl.front().unwrap(), &1);
    /// ```
    #[allow(unsafe_code)]
    pub fn push_front(&mut self, data: T) {
        // Create a new node with `next` points to the current head and `prev`
        // points `None`.  Then allocate the node on the heap and take a raw
        // pointer to it.
        let mut node = Node::new(data);
        node.next = self.head;
        node.prev = None;
        let node = Box::into_raw(Box::new(node));

        match self.head {
            // Empty list => `tail` also points to the new node.
            None => self.tail = Some(unsafe { NonNull::new_unchecked(node) }),
            // Non-empty list => `prev` of current `head` points to the new node.
            Some(mut head) => {
                // SAFETY: Head is not None so we know the raw pointer inside
                // is still valid
                unsafe {
                    head.as_mut().prev = Some(NonNull::new_unchecked(node));
                }
            }
        }

        self.head = Some(unsafe { NonNull::new_unchecked(node) });
        self.len += 1;
    }

    /// Removes the first element and returns it, or None if the list is empty.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut d = DoublyLinkedList::new();
    /// assert_eq!(d.pop_front(), None);
    ///
    /// d.push_front(1);
    /// d.push_front(3);
    /// assert_eq!(d.pop_front(), Some(3));
    /// assert_eq!(d.pop_front(), Some(1));
    /// assert_eq!(d.pop_front(), None);
    /// ```
    #[allow(unsafe_code)]
    pub fn pop_front(&mut self) -> Option<T> {
        // SAFETY: The head of the linked list is still pointing to a Node so
        // we know that its data has not been dropped. So we can dereference
        // the raw pointer and deallocate its data outself.
        self.head.map(|head| unsafe {
            let head = Box::from_raw(head.as_ptr());
            self.head = head.next;
            self.len -= 1;

            match self.head {
                None => self.tail = None,
                Some(mut next) => next.as_mut().prev = None,
            }
            head.data
        })
    }

    /// Appends an element to the back of a list.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut d = DoublyLinkedList::new();
    /// d.push_back(1);
    /// d.push_back(3);
    /// assert_eq!(3, *d.back().unwrap());
    /// ```
    #[allow(unsafe_code)]
    pub fn push_back(&mut self, data: T) {
        // Create a new node with `prev` points to the current tail and `next`
        // points `None`. Then allocate the node on the heap and take a raw
        // pointer to it.
        let mut node = Node::new(data);
        node.next = None;
        node.prev = self.tail;
        let node = Box::into_raw(Box::new(node));

        match self.tail {
            // Empty list => `head` also points to the new node.
            None => self.head = Some(unsafe { NonNull::new_unchecked(node) }),
            // Non-empty list => `next` of current `tail` points to the new
            // node.
            Some(tail) => {
                // SAFETY: Tai is not None so we not that the raw pointer is
                // still valid.
                let tail = unsafe { &mut *tail.as_ptr() };
                tail.next = Some(unsafe { NonNull::new_unchecked(node) });
            }
        }

        self.tail = Some(unsafe { NonNull::new_unchecked(node) });
        self.len += 1;
    }

    /// Removes the last element from a list and returns it, or None if it is
    /// empty.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut d = DoublyLinkedList::new();
    /// assert_eq!(d.pop_back(), None);
    /// d.push_back(1);
    /// d.push_back(3);
    /// assert_eq!(d.pop_back(), Some(3));
    /// ```
    #[allow(unsafe_code)]
    pub fn pop_back(&mut self) -> Option<T> {
        // SAFETY: The tail of the linked list is still pointing to a Node so
        // we know that its data has not been dropped. So we can dereference
        // the raw pointer and deallocate its data outself.
        self.tail.map(|tail| unsafe {
            let tail = Box::from_raw(tail.as_ptr());
            self.tail = tail.prev;
            self.len -= 1;

            match self.tail {
                None => self.head = None,
                Some(mut prev) => prev.as_mut().next = None,
            }
            tail.data
        })
    }

    /// Splits the list into two at the given index. Returns everything after
    /// the given index, including the index.
    ///
    /// This operation should compute in O(n) time.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::DoublyLinkedList;
    ///
    /// let mut d = DoublyLinkedList::new();
    ///
    /// d.push_front(1);
    /// d.push_front(2);
    /// d.push_front(3);
    ///
    /// let mut split = d.split_off(2);
    ///
    /// assert_eq!(split.pop_front(), Some(1));
    /// assert_eq!(split.pop_front(), None);
    /// ```
    #[allow(unsafe_code)]
    pub fn split_off(&mut self, at: usize) -> DoublyLinkedList<T> {
        let len = self.len();
        assert!(at <= len, "Cannot split off at a nonexistent index");

        if at == 0 {
            return std::mem::take(self);
        } else if at == len {
            return Self::new();
        }

        let mut it_idx = 0;
        let mut it_node = self.head;
        loop {
            match it_node {
                None => unreachable!(),
                Some(mut node_ptr) => {
                    let node = unsafe { node_ptr.as_mut() };

                    if it_idx == at {
                        if let Some(mut prev) = node.prev {
                            unsafe { prev.as_mut().next = None };
                        }
                        let ll = DoublyLinkedList {
                            head: Some(unsafe { NonNull::new_unchecked(node) }),
                            tail: self.tail,
                            len: len - at,
                            marker: PhantomData,
                        };
                        self.tail = node.prev;
                        node.prev = None;
                        break ll;
                    }

                    it_idx += 1;
                    it_node = node.next;
                }
            }
        }
    }
}

/// An iterator over the elements of a DoublyLinkedList.
///
/// This struct is created by [`DoublyLinkedList::iter()`]. See its
/// documentation for more.
///
/// [`DoublyLinkedList::iter()`]: crate::containers::DoublyLinkedList#iter;
#[derive(Debug)]
pub struct Iter<'a, T> {
    it: Option<NonNull<Node<T>>>,
    marker: PhantomData<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: Current Node is Some, so we know its raw pointer is
        // still valid
        self.it.map(|node| unsafe {
            let node = &mut *node.as_ptr();
            self.it = node.next;
            &node.data
        })
    }
}

/// A mutable iterator over the elements of a DoublyLinkedList.
///
/// This struct is created by [`DoublyLinkedList::iter_mut()`]. See its
/// documentation for more.
///
/// [`DoublyLinkedList::iter_mut()`]: crate::containers::DoublyLinkedList#iter_mut;
#[derive(Debug)]
pub struct IterMut<'a, T> {
    it: Option<NonNull<Node<T>>>,
    marker: PhantomData<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: Current Node is Some, so we know its raw pointer is still
        // valid
        self.it.map(|node| unsafe {
            let node = &mut *node.as_ptr();
            self.it = node.next;
            &mut node.data
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_front_pop_front() {
        let mut ll = DoublyLinkedList::new();
        assert!(ll.is_empty());
        assert_eq!(ll.len(), 0);

        ll.push_front(0);
        ll.push_front(1);
        ll.push_front(2);

        assert!(!ll.is_empty());
        assert_eq!(ll.len(), 3);

        assert_eq!(ll.pop_front(), Some(2));
        assert_eq!(ll.pop_front(), Some(1));
        assert_eq!(ll.pop_front(), Some(0));
        assert_eq!(ll.pop_front(), None);

        assert!(ll.is_empty());
        assert_eq!(ll.len(), 0);
    }

    #[test]
    fn push_front_pop_back() {
        let mut ll = DoublyLinkedList::new();
        assert!(ll.is_empty());
        assert_eq!(ll.len(), 0);

        ll.push_front(0);
        ll.push_front(1);
        ll.push_front(2);

        assert!(!ll.is_empty());
        assert_eq!(ll.len(), 3);

        assert_eq!(ll.pop_back(), Some(0));
        assert_eq!(ll.pop_back(), Some(1));
        assert_eq!(ll.pop_back(), Some(2));
        assert_eq!(ll.pop_back(), None);

        assert!(ll.is_empty());
        assert_eq!(ll.len(), 0);
    }

    #[test]
    fn push_back_pop_back() {
        let mut ll = DoublyLinkedList::new();
        assert!(ll.is_empty());
        assert_eq!(ll.len(), 0);

        ll.push_back(0);
        ll.push_back(1);
        ll.push_back(2);

        assert!(!ll.is_empty());
        assert_eq!(ll.len(), 3);

        assert_eq!(ll.pop_back(), Some(2));
        assert_eq!(ll.pop_back(), Some(1));
        assert_eq!(ll.pop_back(), Some(0));
        assert_eq!(ll.pop_back(), None);

        assert!(ll.is_empty());
        assert_eq!(ll.len(), 0);
    }

    #[test]
    fn push_back_pop_front() {
        let mut ll = DoublyLinkedList::new();
        assert!(ll.is_empty());
        assert_eq!(ll.len(), 0);

        ll.push_back(0);
        ll.push_back(1);
        ll.push_back(2);

        assert!(!ll.is_empty());
        assert_eq!(ll.len(), 3);

        assert_eq!(ll.pop_front(), Some(0));
        assert_eq!(ll.pop_front(), Some(1));
        assert_eq!(ll.pop_front(), Some(2));
        assert_eq!(ll.pop_front(), None);

        assert!(ll.is_empty());
        assert_eq!(ll.len(), 0);
    }
}
