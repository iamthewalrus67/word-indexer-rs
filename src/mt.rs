use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;
use std::sync::{Condvar, Mutex};

use crate::list_and_read::FileForIndex;

use self::size_limits::DEQUE_SIZE_LIMIT_BYTES;

pub mod size_limits {
    pub const KILOBYTE: usize = 1000;
    pub const MEGABYTE: usize = 1000 * KILOBYTE;
    pub const FILE_SIZE_LIMIT_BYTES: usize = 10 * MEGABYTE;
    pub const DEQUE_SIZE_LIMIT_BYTES: usize = 400 * MEGABYTE;
}

#[derive(Debug)]
pub struct MtDeque<T: RealSize> {
    deque: Mutex<VecDeque<T>>,
    cv_empty: Condvar,
    cv_full: Condvar,
    deque_size_bytes: AtomicUsize,
}

unsafe impl<T: RealSize> Sync for MtDeque<T> {}
unsafe impl<T: RealSize> Send for MtDeque<T> {}

impl<T: RealSize> MtDeque<T> {
    pub fn new() -> Self {
        Self {
            deque: Mutex::new(VecDeque::new()),
            cv_empty: Condvar::new(),
            cv_full: Condvar::new(),
            deque_size_bytes: AtomicUsize::new(0),
        }
    }

    pub fn push_back(&self, elem: T) {
        let elem_real_size = elem.get_real_size();
        let mut guard = self
            .cv_full
            .wait_while(self.deque.lock().unwrap(), |_| {
                self.deque_size_bytes
                    .load(std::sync::atomic::Ordering::Relaxed)
                    + elem_real_size
                    >= DEQUE_SIZE_LIMIT_BYTES
            })
            .unwrap();

        guard.push_back(elem);
        self.deque_size_bytes
            .fetch_add(elem_real_size, std::sync::atomic::Ordering::Relaxed);
        self.cv_empty.notify_one();
    }

    pub fn push_front(&self, elem: T) {
        let elem_real_size = elem.get_real_size();
        let mut guard = self
            .cv_full
            .wait_while(self.deque.lock().unwrap(), |_| {
                self.deque_size_bytes
                    .load(std::sync::atomic::Ordering::Relaxed)
                    + elem_real_size
                    >= DEQUE_SIZE_LIMIT_BYTES
            })
            .unwrap();

        guard.push_front(elem);
        self.deque_size_bytes
            .fetch_add(elem_real_size, std::sync::atomic::Ordering::Relaxed);
        self.cv_empty.notify_one();
    }

    #[allow(dead_code)]
    pub fn pop_back(&self) -> T {
        let mut guard = self
            .cv_empty
            .wait_while(self.deque.lock().unwrap(), |deque| deque.is_empty())
            .unwrap();
        let popped_elem = guard.pop_back().unwrap();
        self.deque_size_bytes.fetch_sub(
            popped_elem.get_real_size(),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.cv_full.notify_one();
        return popped_elem;
    }

    pub fn pop_front(&self) -> T {
        let mut guard = self
            .cv_empty
            .wait_while(self.deque.lock().unwrap(), |deque| deque.is_empty())
            .unwrap();
        let popped_elem = guard.pop_front().unwrap();
        self.deque_size_bytes.fetch_sub(
            popped_elem.get_real_size(),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.cv_full.notify_one();
        return popped_elem;
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        return self.deque.lock().unwrap().len();
    }
}

pub trait RealSize {
    fn get_real_size(&self) -> usize;
}

impl RealSize for Option<String> {
    fn get_real_size(&self) -> usize {
        match self {
            Some(v) => v.len(),
            None => std::mem::size_of::<Option<String>>(),
        }
    }
}

impl RealSize for Option<crate::list_and_read::FileForIndex> {
    fn get_real_size(&self) -> usize {
        match self {
            Some(file_for_index) => match file_for_index {
                FileForIndex::Regular(path) => path.as_os_str().len(),
                FileForIndex::Zip(paths, _) => paths.len() * std::mem::size_of::<String>(),
            },
            None => std::mem::size_of::<Option<crate::list_and_read::FileForIndex>>(),
        }
    }
}

impl RealSize for Option<HashMap<String, usize>> {
    fn get_real_size(&self) -> usize {
        match self {
            Some(map) => {
                map.keys().len() * (std::mem::size_of::<String>() + std::mem::size_of::<usize>())
            }
            None => std::mem::size_of::<Option<HashMap<String, usize>>>(),
        }
    }
}
