/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use openmp_reducer::Reducer;

#[test]
fn test() {
    let reducer = Reducer::new(3, |global, local| *global += *local);
    {
        let mut shared0 = reducer.share();
        let mut shared1 = shared0.clone();

        *shared0.as_mut() = 5;
        *shared1.as_mut() = 10;
    }

    assert_eq!(reducer.get(), 18);
}

#[test]
fn test_get() {
    let reducer = Reducer::new(3, |global, local| *global += *local);
    {
        let mut shared = reducer.share();
        *shared.as_mut() = 5;
        assert_eq!(*shared.as_ref(), 5);
    }
    assert_eq!(reducer.get(), 8);
}

#[test]
fn test_two_types() {
    let reducer = Reducer::new(3, |global, local| *global += *local);
    {
        let mut shared = reducer.share();
        *shared.as_mut() = 5;
        assert_eq!(*shared.as_ref(), 5);
    }
    assert_eq!(reducer.get(), 8);
}

#[test]
fn test_peek_count() {
    let reducer = Reducer::new(3, |global, local| *global += *local);
    {
        let mut shared = reducer.share();
        *shared.as_mut() = 5;
        assert_eq!(reducer.peek(), 3);
        assert_eq!(shared.peek(), 3);
    }
    assert_eq!(reducer.peek(), 8);
}
