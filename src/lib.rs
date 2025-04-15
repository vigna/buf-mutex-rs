/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![doc = include_str!("../README.md")]

use std::fmt::Debug;
use std::sync::Mutex;

/// An OpenMP-style reducer that wraps a global value into a [`Mutex`],
/// providing [shareable, cloneable copies with a local value](#method.share);
/// the copies will be reduced into the global value when dropped.
///
/// The global value can be observed with [`peek`](Reducer::peek) if the base
/// type is [`Clone`], whereas [`get`](Reducer::get) consumes self and returns
/// the global value.
///
/// For convenience, the global value and the local value have distinct type
/// parameters `G` and `L`, respectively; the second type defaults to the first
/// one.
///
/// Each shared copy has a reference to the [`Reducer`] it was created from, so
/// you cannot call [`get`](Reducer::get) if there are still shared copies
/// around. For example, this code will not compile:
/// ```compile_fail
/// use openmp_reducer::Reducer;
///
/// let reducer = Reducer::<usize>::new(3, |global, local| *global += *local);
/// let mut shared = reducer.share();
/// // drop(shared); // uncommenting this line would make the code compile
/// assert_eq!(reducer.get(), 3);
///```
///
/// # Examples
///
/// In this example, we manually spawn processes:
///
/// ```rust
/// use openmp_reducer::Reducer;
/// use std::thread;
///
/// let mut reducer = Reducer::<usize>::new(5, |global, local| *global += *local);
/// std::thread::scope(|s| {
///     for i in 0..3 {
///         let mut shared = reducer.share();
///         s.spawn(move || {
///             *shared.as_mut() += 10;
///         });
///     }
/// });
///
/// // Initial value plus additional values from shared copies
/// assert_eq!(reducer.get(), 35);
/// ```
///
/// You can obtain the same behavior with [Rayon](https://docs.rs/rayon) using
/// methods such as
/// [`for_each_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with)
/// and
/// [`map_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.map_with):
///
/// ```rust
/// use openmp_reducer::Reducer;
/// use rayon::prelude::*;
///
/// let mut reducer = Reducer::<usize>::new(5, |global, local| *global += *local);
/// (0..1000000).into_par_iter().
///     with_min_len(1000). // optional, might reduce the amount of cloning
///     for_each_with(reducer.share(), |shared, i| {
///         *shared.as_mut() += 1;
///     }
/// );
///
/// // Initial value plus additional values from clones
/// assert_eq!(reducer.get(), 1_000_005);
/// ```
///
/// Note that you have to pass `reducer.share()`, which can be cloned. Also,
/// since
/// [`for_each_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with)
/// might perform excessive cloning if jobs are too short, you can use
/// [`with_min_len`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.with_min_len)
/// to reduce the amount of cloning.
#[derive(Debug)]
pub struct Reducer<G: Debug + Default, L: Debug + Default = G> {
    global: Mutex<G>,
    reduce: fn(&mut G, &L),
}

impl<G: Debug + Default, L: Debug + Default> Reducer<G, L> {
    /// Creates a new reducer with a given reduction function.
    ///
    /// The function must reduce the local value (second argument) into the
    /// global value (first argument). For the result to be deterministic, the
    /// global value must be the same regardless of the order in which the local
    /// values are reduced.
    pub fn new(init: G, reduce: fn(global: &mut G, local: &L)) -> Self {
        Reducer {
            global: Mutex::new(init),
            reduce,
        }
    }

    /// Returns a [`SharedReducer`] referencing this [`Reducer`].
    ///
    /// The [`SharedReducer`] will be initialized with the default value of the
    /// base type.
    pub fn share(&self) -> SharedReducer<G, L> {
        SharedReducer {
            openmp_reducer: self,
            local: L::default(),
        }
    }

    /// Consumes self and return the global value.
    ///
    /// Note that you cannot call this method if there are still [shared
    /// copies](#method.share) that have not been dropped.
    ///
    /// If you just need to access the global value without consuming self, and
    /// the base type is [`Clone`], use [`peek`](Reducer::peek).
    ///
    /// # Panics
    ///
    /// This method will panic if the mutex is poisoned.
    /// [`peek`](Reducer::peek).
    pub fn get(self) -> G {
        self.global.into_inner().unwrap()
    }
}

impl<G: Debug + Default + Clone, L: Debug + Default> Reducer<G, L> {
    /// Returns the current global value.
    ///
    /// Note that this method does not guarantee that all shared copies have
    /// been dropped. If you need that guarantee, use [`get`](Reducer::get).
    ///
    /// # Panics
    ///
    /// This method will panic if the mutex is poisoned.
    pub fn peek(&self) -> G {
        self.global.lock().unwrap().clone()
    }
}

/// A shareable copy of a [`Reducer`] containing a local value and implementing
/// [`Clone`].
///
/// The local value can be accessed with [`AsRef`] and [`AsMut`]
/// implementations.
///
/// When a [`SharedReducer`] is dropped, the local value will be reduced into
/// the global value.
#[derive(Debug)]
pub struct SharedReducer<'a, G: Debug + Default, L: Debug + Default> {
    openmp_reducer: &'a Reducer<G, L>,
    local: L,
}

impl<G: Debug + Default, L: Debug + Default> Clone for SharedReducer<'_, G, L> {
    /// Returns a copy sharing the same global value and
    /// with local value initialized to the default value.
    fn clone(&self) -> Self {
        SharedReducer {
            openmp_reducer: self.openmp_reducer,
            local: L::default(),
        }
    }
}

impl<G: Debug + Default, L: Debug + Default> Drop for SharedReducer<'_, G, L> {
    /// Reduces the local value into the global value.
    fn drop(&mut self) {
        let mut lock = self.openmp_reducer.global.lock().unwrap();
        (self.openmp_reducer.reduce)(&mut *lock, &self.local);
    }
}

impl<G: Debug + Default + Clone, L: Debug + Default> SharedReducer<'_, G, L> {
    /// Returns the current global value.
    ///
    /// This method delegates to [`Reducer::peek`].
    pub fn peek(&self) -> G {
        self.openmp_reducer.peek()
    }
}

impl<G: Debug + Default, L: Debug + Default> AsRef<L> for SharedReducer<'_, G, L> {
    /// Returns a reference to the local value.
    fn as_ref(&self) -> &L {
        &self.local
    }
}

impl<G: Debug + Default, L: Debug + Default> AsMut<L> for SharedReducer<'_, G, L> {
    /// Returns a mutable reference to the local value.
    fn as_mut(&mut self) -> &mut L {
        &mut self.local
    }
}
