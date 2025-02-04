use buf_mutex::BufMutex;

#[test]
fn test() {
    let buffered_atomic = BufMutex::new(3, |old, new| old + new);
    {
        let mut clone1 = buffered_atomic.clone();
        let mut clone2 = buffered_atomic.clone();

        clone1.local = 5;
        clone2.local = 10;
    }

    assert_eq!(buffered_atomic.get(), 18);
}
