use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::{Condvar, Mutex};

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
    t_size: usize,
}

unsafe impl<T> Sync for MtDeque<T> {}
unsafe impl<T> Send for MtDeque<T> {}

impl<T> MtDeque<T> {
    pub fn new() -> Self {
        Self {
            deque: Mutex::new(VecDeque::new()),
            cv_empty: Condvar::new(),
            cv_full: Condvar::new(),
            t_size: std::mem::size_of::<T>(),
        }
    }

    pub fn push_back(&self, elem: T) {
        let mut guard = self
            .cv_full
            .wait_while(self.deque.lock().unwrap(), |deque| {
                deque.len() * self.t_size >= DEQUE_SIZE_LIMIT_BYTES
            })
            .unwrap();

        guard.push_back(elem);
        self.cv_empty.notify_one();
    }

    pub fn push_front(&self, elem: T) {
        let mut guard = self
            .cv_full
            .wait_while(self.deque.lock().unwrap(), |deque| {
                deque.len() * self.t_size >= DEQUE_SIZE_LIMIT_BYTES
            })
            .unwrap();

        guard.push_back(elem);
        self.cv_empty.notify_one();
    }

    pub fn pop_back(&self) -> T {
        let mut guard = self
            .cv_empty
            .wait_while(self.deque.lock().unwrap(), |deque| deque.is_empty())
            .unwrap();
        let popped_elem = guard.pop_back().unwrap();
        self.cv_full.notify_one();
        return popped_elem;
    }

    pub fn pop_front(&self) -> T {
        let mut guard = self
            .cv_empty
            .wait_while(self.deque.lock().unwrap(), |deque| deque.is_empty())
            .unwrap();
        let popped_elem = guard.pop_front().unwrap();
        self.cv_full.notify_one();
        return popped_elem;
    }

    pub fn size(&self) -> usize {
        return self.deque.lock().unwrap().len();
    }
}


// pub struct MtPairQueue<T> {
//     pair: Mutex<(T, T)>,
//     deque: Mutex<VecDeque<(T, T)>>,
//     cv_new_pair: Condvar,
// }

// unsafe impl<T> Sync for MtPairQueue<T> {}
// unsafe impl<T> Send for MtPairQueue<T> {}


// impl<T> MtPairQueue<T> {
//     pub fn push(&self, elem: T) {
//         let mut pair_guard = self.pair.lock().unwrap();
//         if pair_guard.0 {
//             pair_guard.0 = elem;
//         } else {
//             pair_guard.1 = Some(elem);
//             {
//                 let mut deque_guard = self.deque.lock().unwrap();
//                 deque_guard.push_back((pair_guard.0.unwrap(), pair_guard.1.unwrap()));
//             }
//             pair_guard.0 = None;
//             pair_guard.1 = None;
//             self.cv_new_pair.notify_one();
//         }
//     }

//     pub fn pop_pair(&self) -> (T, T) {
//         let mut deque_guard= self.deque.lock().unwrap();
//         self.cv_new_pair.wait_while(deque_guard, |deque| {
//             deque.is_empty()
//         });

//         let popped_pair = deque_guard.pop_front().unwrap();
//         return popped_pair;
//     }

//     pub fn get_result(&self) -> T {
//         let a = self.pair.lock().unwrap().0.unwrap();
//         return a;
//     }
// }