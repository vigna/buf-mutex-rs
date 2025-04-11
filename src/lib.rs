/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![doc = include_str!("../README.md")]

use std::fmt::Debug;
use std::sync::Mutex;

/// Wrap a global value into a [`Mutex`], providing [shareable, cloneable copies
/// with a local value](#method.share); the copies will be reduced into the
/// global value when dropped.
///
/// The global value can be observed with [`peek`](BufMutex::peek) if the base
/// type is [`Clone`], whereas [`get`](BufMutex::get) consumes self and returns
/// the global value.
///
/// Each shared copy has a reference to the [`BufMutex`] it was created from, so
/// you cannot call [`get`](BufMutex::get) if there are still shared copies
/// around. For example, this code will not compile:
/// ```compile_fail
/// use buf_mutex::BufMutex;
///
/// let buffered_atomic = BufMutex::new(3, |global, local| *global += *local);
/// let mut _shared = buffered_atomic.share();
/// // drop(_shared); // uncommenting this line would make the code compile
/// assert_eq!(buffered_atomic.get(), 3);
///```
///
/// # Examples
///
/// In this example, we manually spawn processes:
///
/// ```rust
/// use buf_mutex::BufMutex;
/// use std::thread;
///
/// let mut counter = BufMutex::new(5, |global, local| *global += *local);
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
/// You can obtain the same behavior with [Rayon](https://docs.rs/rayon) using
/// methods such as
/// [`for_each_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with)
/// and
/// [`map_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.map_with):
///
/// ```rust
/// use buf_mutex::BufMutex;
/// use rayon::prelude::*;
///
/// let mut counter = BufMutex::new(5, |global, local| *global += *local);
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
/// Note that you have to pass `counter.share()`, which can be cloned. Also,
/// since
/// [`for_each_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with)
/// might perform excessive cloning if jobs are too short, you can use
/// [`with_min_len`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.with_min_len)
/// to reduce the amount of cloning.
#[derive(Debug)]
pub struct BufMutex<T: Debug + Default> {
    global: Mutex<T>,
    reduce: fn(&mut T, &T),
}

impl<T: Debug + Default> BufMutex<T> {
    /// Create a new buffered mutex with a given reduction function.
    ///
    /// The function must reduce the local value (second argument) into the
    /// global value (first argument).
    pub fn new(init: T, reduce: fn(global: &mut T, local: &T)) -> Self {
        BufMutex {
            global: Mutex::new(init),
            reduce,
        }
    }

    /// Create a new shared, cloneable copy of the buffered mutex.
    pub fn share(&self) -> SharedBufMutex<T> {
        SharedBufMutex {
            buf_mutex: self,
            local: T::default(),
        }
    }

    /// Consume self and return the global value.
    ///
    /// Note that you cannot call this method if there are still [shared
    /// copies](#method.share) that have not been dropped.
    ///
    /// If you just need to access the global value without consuming self, and
    /// the base type is [`Clone`], use [`peek`](BufMutex::peek).
    ///
    /// # Panics
    ///
    /// This method will panic if the mutex is poisoned.
    /// [`peek`](BufMutex::peek).
    pub fn get(self) -> T {
        self.global.into_inner().unwrap()
    }
}

impl<T: Clone + Debug + Default> BufMutex<T> {
    /// Return the current global value.
    ///
    /// Note that this method does not guarantee that all shared copies have
    /// been dropped. If you need that guarantee, use [`get`](BufMutex::get).
    ///
    /// # Panics
    ///
    /// This method will panic if the mutex is poisoned.
    pub fn peek(&self) -> T {
        self.global.lock().unwrap().clone()
    }
}

/// A shareable copy of a [`BufMutex`] containing a local value and implementing
/// [`Clone`].
///
/// The local value can be accessed with [`AsRef`] and [`AsMut`]
/// implementations.
///
/// When a [`SharedBufMutex`] is dropped, the local value will be reduced into
/// the global value.
#[derive(Debug)]
pub struct SharedBufMutex<'a, T: Debug + Default> {
    buf_mutex: &'a BufMutex<T>,
    local: T,
}

impl<T: Debug + Default> Clone for SharedBufMutex<'_, T> {
    /// Return a copy sharing the same global value and
    /// with local value initialized to the default value.
    fn clone(&self) -> Self {
        SharedBufMutex {
            buf_mutex: self.buf_mutex,
            local: T::default(),
        }
    }
}

impl<T: Debug + Default> Drop for SharedBufMutex<'_, T> {
    /// Reduce the local value into the global value.
    fn drop(&mut self) {
        let mut lock = self.buf_mutex.global.lock().unwrap();
        (self.buf_mutex.reduce)(&mut *lock, &self.local);
    }
}

impl<T: Clone + Debug + Default> SharedBufMutex<'_, T> {
    /// Return the current global value.
    ///
    /// This method delegates to [`BufMutex::peek`].
    pub fn peek(&self) -> T {
        self.buf_mutex.peek()
    }
}

impl<T: Debug + Default> AsRef<T> for SharedBufMutex<'_, T> {
    /// Return a reference to the local value.
    fn as_ref(&self) -> &T {
        &self.local
    }
}

impl<T: Debug + Default> AsMut<T> for SharedBufMutex<'_, T> {
    /// Return a mutable reference to the local value.
    fn as_mut(&mut self) -> &mut T {
        &mut self.local
    }
}
