//! An atomically managed intrusive linked list of `Arc` nodes

use std::marker;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicUsize, AtomicBool};

pub struct ArcList<T> {
    list: AtomicUsize,
    _marker: marker::PhantomData<T>,
}

impl<T> ArcList<T> {
    pub fn new() -> ArcList<T> {
        ArcList {
            list: AtomicUsize::new(0),
            _marker: marker::PhantomData,
        }
    }

    /// Pushes the `data` provided onto this list if it's not already enqueued
    /// in this list.
    ///
    /// If `data` is already enqueued in this list then this is a noop,
    /// otherwise, the `data` here is pushed on the end of the list.
    pub fn push(&self, data: &Arc<Node<T>>) {
        if data.enqueued.swap(true, SeqCst) {
            return
        }
        let mut head = self.list.load(SeqCst);
        let node = Arc::into_raw(data.clone()) as usize;
        loop {
            data.next.store(head, SeqCst);
            match self.list.compare_exchange(head, node, SeqCst, SeqCst) {
                Ok(_) => break,
                Err(new_head) => head = new_head,
            }
        }
    }

    /// Atomically empties this list, returning a new owned copy which can be
    /// used to iterate over the entries.
    pub fn take(&self) -> ArcList<T> {
        ArcList {
            list: AtomicUsize::new(self.list.swap(0, SeqCst)),
            _marker: marker::PhantomData,
        }
    }

    /// Removes the head of the list of nodes, returning `None` if this is an
    /// empty list.
    pub fn pop(&mut self) -> Option<Arc<Node<T>>> {
        let head = *self.list.get_mut();
        if head == 0 {
            return None
        }
        let head = unsafe { Arc::from_raw(head as *const Node<T>) };
        *self.list.get_mut() = head.next.load(SeqCst);
        // At this point, the node is out of the list, so store `false` so we
        // can enqueue it again and see further changes.
        assert!(head.enqueued.swap(false, SeqCst));
        Some(head)
    }
}

pub struct Node<T> {
    next: AtomicUsize,
    enqueued: AtomicBool,
    data: T,
}

impl<T> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Node {
            next: AtomicUsize::new(0),
            enqueued: AtomicBool::new(false),
            data: data,
        }
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}
