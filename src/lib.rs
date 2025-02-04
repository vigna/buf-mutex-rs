/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![doc = include_str!("../README.md")]

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// Wrap a value into a [`Mutex`], providing clones with local values
/// that will be reduced into the global value when dropped.
///
/// # Examples
///
/// In this example, we manually spawn processes:
///
/// ```rust
/// use buf_mutex::BufMutex;
/// use std::thread;
///
/// let mut counter = BufMutex::new(5, |old, new| old + new);
/// std::thread::scope(|s| {
///     for i in 0..3 {
///         let mut counter = counter.clone();
///         s.spawn(move || {
///             counter.local += 10;
///             eprintln!("Adding value to {:?}", &counter as *const BufMutex<i32>);
///         });
///     }
/// });
///
/// // Initial value plus additional values from clones
/// assert_eq!(counter.get(), 35);
/// ```
///
/// You can obtain the same behavior with [`rayon`](https://docs.rs/rayon) using
/// methods such as
/// [`for_each_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with)
/// and
/// [`map_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.map_with):
///
/// ```rust
/// use buf_mutex::BufMutex;
/// use rayon::prelude::*;
///
/// let mut counter = BufMutex::new(5, |old, new| old + new);
/// (0..1000000).into_par_iter().
///     with_min_len(1000). // optional, might reduce the amount of cloning
///     for_each_with(counter.clone(), |counter, i| {
///         counter.local += 1;
///     }
/// );
///
/// // Initial value plus additional values from clones
/// assert_eq!(counter.get(), 1_000_005);
/// ```
///
/// Note that you have to pass `counter.clone()` to avoid a move that would make
/// the call to [`get`](BufMutex::get) impossible. Also, since
/// [`for_each_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with)
/// might perform excessive cloning if jobs are too short, you can use
/// [`with_min_len`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.with_min_len)
/// to reduce the amount of cloning.

pub struct BufMutex<T: Copy + Debug + Default> {
    pub local: T,
    global: Arc<Mutex<T>>,
    reduce: fn(T, T) -> T,
}

impl<T: Copy + Debug + Default> BufMutex<T> {
    /// Create a new buffered mutex with a given reduction function.
    pub fn new(init: T, reduce: fn(T, T) -> T) -> Self {
        BufMutex {
            global: Arc::new(Mutex::new(init)),
            local: T::default(),
            reduce,
        }
    }

    //
    pub fn get(&self) -> T {
        (self.reduce)(*self.global.lock().unwrap(), self.local)
    }
}

impl<T: Copy + Debug + Default> Clone for BufMutex<T> {
    fn clone(&self) -> Self {
        BufMutex {
            global: self.global.clone(),
            local: T::default(),
            reduce: self.reduce,
        }
    }
}

impl<T: Copy + Debug + Default> Drop for BufMutex<T> {
    fn drop(&mut self) {
        eprintln!("Dropping {:?}", &*self as *const Self);
        let mut lock = self.global.lock().unwrap();
        eprintln!("Old lock: {:?}", lock);
        *lock = (self.reduce)(*lock, self.local);
        eprintln!("New lock: {:?} {:?}", lock, self.local);
    }
}
