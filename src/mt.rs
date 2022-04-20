use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::{Mutex, Condvar, Arc};

use self::size_limits::DEQUE_SIZE_LIMIT_BYTES;

pub mod size_limits {
    pub const KILOBYTE: usize = 1000;
    pub const MEGABYTE: usize = 1000 * KILOBYTE;
    pub const FILE_SIZE_LIMIT_BYTES: usize = 10 * MEGABYTE;
    pub const DEQUE_SIZE_LIMIT_BYTES: usize = 100 * MEGABYTE;
}

#[derive(Debug)]
pub struct MtDeque<T> {
    deque: Mutex<VecDeque<T>>,
    cv_empty: Condvar,
    cv_full: Condvar,
    T_size: usize
}

unsafe impl<T> Sync for MtDeque<T> {}
unsafe impl<T> Send for MtDeque<T> {}


impl<T> MtDeque<T> {
    pub fn new() -> Self {
        Self {
            deque: Mutex::new(VecDeque::new()),
            cv_empty: Condvar::new(),
            cv_full: Condvar::new(),
            T_size: std::mem::size_of::<T>()
        }
    }

    pub fn push_back(&self, elem: T) {
        let mut guard = self.cv_full.wait_while(self.deque.lock().unwrap(), |deque| {
            deque.len() * self.T_size >= DEQUE_SIZE_LIMIT_BYTES
        }).unwrap();

        guard.push_back(elem);
        self.cv_empty.notify_one();
    }

    pub fn push_front(&self, elem: T) {
        let mut guard = self.cv_full.wait_while(self.deque.lock().unwrap(), |deque| {
            deque.len() * self.T_size >= DEQUE_SIZE_LIMIT_BYTES
        }).unwrap();

        guard.push_back(elem);
        self.cv_empty.notify_one();
    }

    pub fn pop_back(&self) -> T {
        let mut guard = self.cv_empty.wait_while(self.deque.lock().unwrap(), |deque| {
            deque.is_empty()
        }).unwrap();
        let popped_elem = guard.pop_back().unwrap();
        self.cv_full.notify_one();
        return popped_elem;
    }
    
    pub fn pop_front(&self) -> T {
        let mut guard = self.cv_empty.wait_while(self.deque.lock().unwrap(), |deque| {
            deque.is_empty()
        }).unwrap();
        let popped_elem = guard.pop_front().unwrap();
        self.cv_full.notify_one();
        return popped_elem;
    }
}