/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use buf_mutex::BufMutex;

#[test]
fn test() {
    let buffered_atomic = BufMutex::new(3, |global, local| *global += *local);
    {
        let mut shared0 = buffered_atomic.share();
        let mut shared1 = shared0.clone();

        *shared0.as_mut() = 5;
        *shared1.as_mut() = 10;
    }

    assert_eq!(buffered_atomic.get(), 18);
}

#[test]
fn test_get() {
    let buffered_atomic = BufMutex::new(3, |global, local| *global += *local);
    {
        let mut shared = buffered_atomic.share();
        *shared.as_mut() = 5;
        assert_eq!(*shared.as_ref(), 5);
    }
    assert_eq!(buffered_atomic.get(), 8);
}

#[test]
#[should_panic]
fn test_missing_drop() {
    let buffered_atomic = BufMutex::new(3, |global, local| *global += *local);
    let mut _shared = buffered_atomic.share();
    assert_eq!(buffered_atomic.get(), 3);
}

#[test]
fn test_peek_count() {
    let buffered_atomic = BufMutex::new(3, |global, local| *global += *local);
    {
        let mut shared = buffered_atomic.share();
        *shared.as_mut() = 5;
        assert_eq!(buffered_atomic.peek(), 3);
        assert_eq!(buffered_atomic.count(), 1);
        assert_eq!(shared.peek(), 3);
        assert_eq!(shared.count(), 1);
    }
    assert_eq!(buffered_atomic.peek(), 8);
}
