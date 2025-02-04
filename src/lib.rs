/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![doc = include_str!("../README.md")]

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// Wrap a global value into a [`Arc`]/[`Mutex`], providing [shareable,
/// cloneable copies with a local value](#method.share); the copies will be
/// reduced into the global value when dropped.
///
/// The global value can be observed with [`peek`](BufMutex::peek), whereas
/// [`get`](BufMutex::get) consumes self and panics if not all copies have been
/// dropped.
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
///         let mut shared = counter.share();
///         s.spawn(move || {
///             *shared.as_mut() += 10;
///         });
///     }
/// });
///
/// // Initial value plus additional values from shared copies
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
///     for_each_with(counter.share(), |shared, i| {
///         *shared.as_mut() += 1;
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
pub struct BufMutex<T: Clone + Debug + Default> {
    global: Arc<Mutex<T>>,
    reduce: fn(T, T) -> T,
}

/// A shareable copy of a [`BufMutex`] containing a local value and implementing
/// [`Clone`].
///
/// The local value can be accessed with [`AsRef`] and [`AsMut`]
/// implementations.
///
/// When a [`SharedBufMutex`] is dropped, the local value will be reduced into
/// the global value.
pub struct SharedBufMutex<T: Clone + Debug + Default> {
    local: T,
    global: Arc<Mutex<T>>,
    reduce: fn(T, T) -> T,
}

impl<T: Clone + Debug + Default> BufMutex<T> {
    /// Create a new buffered mutex with a given reduction function.
    pub fn new(init: T, reduce: fn(T, T) -> T) -> Self {
        BufMutex {
            global: Arc::new(Mutex::new(init)),
            reduce,
        }
    }

    /// Create a new shared, cloneable copy of the buffered mutex.
    pub fn share(&self) -> SharedBufMutex<T> {
        SharedBufMutex {
            global: self.global.clone(),
            local: T::default(),
            reduce: self.reduce,
        }
    }

    /// Consume self and return the global value.
    ///
    /// # Panics
    ///
    /// This method will panic if not all shared copies have been dropped, or if
    /// the mutex is poisoned. If you need just to observe the global value, use
    /// [`peek`](BufMutex::peek).
    pub fn get(self) -> T {
        Arc::into_inner(self.global)
            .expect("Not all shared copies have been dropped")
            .into_inner()
            .unwrap()
    }
}

impl<T: Clone + Debug + Default> BufMutex<T> {
    /// Return the current global value.
    ///
    /// Note that this method does not guarantee that all shared copies have
    /// been dropped. If you need that guarantee, use [`get`](BufMutex::get).
    pub fn peek(&self) -> T {
        self.global.lock().unwrap().clone()
    }
}

impl<T: Clone + Debug + Default> AsRef<T> for SharedBufMutex<T> {
    /// Return a reference to the local value.
    fn as_ref(&self) -> &T {
        &self.local
    }
}

impl<T: Clone + Debug + Default> AsMut<T> for SharedBufMutex<T> {
    /// Return a mutable reference to the local value.
    fn as_mut(&mut self) -> &mut T {
        &mut self.local
    }
}

impl<T: Clone + Debug + Default> Clone for SharedBufMutex<T> {
    /// Return a copy sharing the same global value and
    /// with local value initialized to the default value.
    fn clone(&self) -> Self {
        SharedBufMutex {
            global: self.global.clone(),
            local: T::default(),
            reduce: self.reduce,
        }
    }
}

impl<T: Clone + Debug + Default> Drop for SharedBufMutex<T> {
    /// Reduce the local value into the global value.
    fn drop(&mut self) {
        let mut lock = self.global.lock().unwrap();
        *lock = (self.reduce)(lock.clone(), self.local.clone());
    }
}
