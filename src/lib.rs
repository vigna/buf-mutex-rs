/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![doc = include_str!("../README.md")]

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub struct BufferedAtomic<T: Copy + Debug + Default> {
    pub global: Arc<Mutex<T>>,
    pub local: T,
    reduce: fn(T, T) -> T,
}

impl<T: Copy + Debug + Default> BufferedAtomic<T> {
    pub fn new(init: T, reduce: fn(T, T) -> T) -> Self {
        BufferedAtomic {
            global: Arc::new(Mutex::new(init)),
            local: T::default(),
            reduce,
        }
    }

    pub fn get(&self) -> T {
        self.global.lock().unwrap().to_owned()
    }
}

impl<T: Copy + Debug + Default> Clone for BufferedAtomic<T> {
    fn clone(&self) -> Self {
        BufferedAtomic {
            global: self.global.clone(),
            local: T::default(),
            reduce: self.reduce,
        }
    }
}

impl<T: Copy + Debug + Default> Drop for BufferedAtomic<T> {
    fn drop(&mut self) {
        let mut lock = self.global.lock().unwrap();
        *lock = (self.reduce)(*lock, self.local);
    }
}
